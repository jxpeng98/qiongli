//! Read public paper bytes into bounded, source-bound excerpts. This does not
//! establish screening eligibility, approve evidence, or write project files.

use std::collections::VecDeque;
use std::io::Read;
use std::net::{IpAddr, SocketAddr, ToSocketAddrs};
use std::sync::{Arc, Mutex, mpsc};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use quick_xml::Reader;
use quick_xml::events::Event;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use url::Url;

const MAX_BYTES: usize = 12 * 1024 * 1024;
const MAX_TEXT: usize = 2 * 1024 * 1024;
const SEGMENT_CHARS: usize = 2_000;
const DEADLINE: Duration = Duration::from_secs(30);

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FulltextRequest {
    pub url: String,
    #[serde(default)]
    offset: usize,
    #[serde(default = "default_limit")]
    limit: usize,
    #[serde(default)]
    refresh: bool,
    #[serde(default)]
    expected_sha256: Option<String>,
    #[serde(default)]
    expected_doi: Option<String>,
}

fn default_limit() -> usize {
    10
}

impl FulltextRequest {
    pub fn from_arguments(value: &Value) -> Result<Self, FulltextError> {
        if ["expected_sha256", "expected_doi"]
            .iter()
            .any(|field| value.get(field).is_some_and(|value| !value.is_string()))
        {
            return Err(failure(
                "invalid-input",
                "Expected digest and DOI must be strings when supplied",
            ));
        }
        let mut request: Self = serde_json::from_value(value.clone())
            .map_err(|_| failure("invalid-input", "Invalid fulltext arguments"))?;
        let url = public_url(&request.url)?;
        request.url = url.to_string();
        if !(1..=50).contains(&request.limit)
            || request.expected_sha256.as_ref().is_some_and(|hash| {
                hash.len() != 64
                    || !hash
                        .bytes()
                        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            })
            || (request.offset > 0 && request.expected_sha256.is_none())
            || request.expected_doi.as_ref().is_some_and(|doi| {
                doi.len() > 512 || !normalize_doi(doi).starts_with("10.") || !doi.contains('/')
            })
        {
            return Err(failure(
                "invalid-input",
                "Use limit 1..50, a DOI, and a lowercase SHA-256; continuation requires expected_sha256",
            ));
        }
        Ok(request)
    }

    pub fn requires_openalex_key(&self) -> bool {
        Url::parse(&self.url).is_ok_and(|url| openalex_content(&url))
    }
}

#[derive(Debug)]
pub struct FulltextError {
    pub code: &'static str,
    pub message: &'static str,
}

impl std::fmt::Display for FulltextError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.message)
    }
}
impl std::error::Error for FulltextError {}

fn failure(code: &'static str, message: &'static str) -> FulltextError {
    FulltextError { code, message }
}

#[derive(Clone, Debug, Serialize)]
pub struct FulltextSegment {
    index: usize,
    anchor: String,
    text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    section: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    page: Option<u32>,
}

#[derive(Clone, Debug, Serialize)]
pub struct FulltextOutput {
    status: &'static str,
    source_url: String,
    resolved_url: String,
    source_sha256: String,
    format: &'static str,
    retrieved_at_unix_seconds: u64,
    cached: bool,
    total_segments: usize,
    segments: Vec<FulltextSegment>,
    #[serde(skip_serializing_if = "Option::is_none")]
    next_offset: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    document_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    document_doi: Option<String>,
    identity_status: &'static str,
    warnings: Vec<String>,
    #[serde(skip)]
    text_bytes: usize,
}

#[derive(Clone, Default)]
pub struct FulltextReader {
    // ponytail: eight session-local documents; reuse an existing artifact owner
    // if durable storage is later needed, rather than adding a research store.
    cache: Arc<Mutex<VecDeque<FulltextOutput>>>,
}

impl FulltextReader {
    pub fn read(
        &self,
        request: &FulltextRequest,
        openalex_key: Option<&str>,
    ) -> Result<FulltextOutput, FulltextError> {
        self.read_with(request, || fetch(&request.url, openalex_key))
    }

    fn read_with(
        &self,
        request: &FulltextRequest,
        fetch: impl FnOnce() -> Result<(String, Vec<u8>), FulltextError>,
    ) -> Result<FulltextOutput, FulltextError> {
        let mut cached = self
            .cache
            .lock()
            .map_err(|_| failure("fulltext-cache-error", "Session cache is unavailable"))?;
        if request.refresh {
            cached.retain(|doc| doc.source_url != request.url);
        }
        let found = cached
            .iter()
            .find(|doc| doc.source_url == request.url)
            .cloned();
        drop(cached);
        let (mut doc, is_cached) = if let Some(doc) = found {
            (doc, true)
        } else {
            let (resolved_url, bytes) = fetch()?;
            let doc = parse_document(&request.url, &resolved_url, &bytes)?;
            let mut cached = self
                .cache
                .lock()
                .map_err(|_| failure("fulltext-cache-error", "Session cache is unavailable"))?;
            cached.retain(|existing| existing.source_url != request.url);
            if cached.len() >= 8 {
                cached.pop_front();
            }
            cached.push_back(doc.clone());
            (doc, false)
        };
        if request
            .expected_sha256
            .as_ref()
            .is_some_and(|expected| expected != &doc.source_sha256)
        {
            return Err(failure(
                "fulltext-source-changed",
                "Source digest differs; restart reading and review the new source",
            ));
        }
        if let Some(expected) = &request.expected_doi {
            doc.identity_status = match &doc.document_doi {
                Some(actual) if normalize_doi(actual) == normalize_doi(expected) => "matched",
                Some(_) => {
                    return Err(failure(
                        "fulltext-identity-mismatch",
                        "Document DOI does not match the requested paper",
                    ));
                }
                None => {
                    doc.warnings.push("Paper identity is unverified: no structured document DOI; check the title, authors and version against the candidate".into());
                    "unverified"
                }
            };
        }
        if request.offset >= doc.total_segments {
            return Err(failure(
                "invalid-input",
                "offset is beyond the available segments",
            ));
        }
        let end = request
            .offset
            .saturating_add(request.limit)
            .min(doc.total_segments);
        doc.segments = doc.segments[request.offset..end].to_vec();
        doc.next_offset = (end < doc.total_segments).then_some(end);
        doc.cached = is_cached;
        Ok(doc)
    }
}

fn public_url(raw: &str) -> Result<Url, FulltextError> {
    let error = || {
        failure(
            "fulltext-url-blocked",
            "Use a public HTTPS URL without credentials, fragments or a nonstandard port",
        )
    };
    if raw.len() > 4096 || raw.chars().any(|c| c.is_whitespace() || c.is_control()) {
        return Err(error());
    }
    let url = Url::parse(raw).map_err(|_| error())?;
    if url.scheme() != "https"
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
        || url.port_or_known_default() != Some(443)
        || url.host_str().is_none()
        || url.query_pairs().any(|(name, _)| {
            let name = name.to_ascii_lowercase();
            matches!(
                name.as_str(),
                "api_key"
                    | "apikey"
                    | "key"
                    | "token"
                    | "access_token"
                    | "auth"
                    | "authorization"
                    | "signature"
            ) || name.starts_with("x-amz-")
                || name.starts_with("x-goog-")
        })
    {
        return Err(error());
    }
    match url.host() {
        Some(url::Host::Ipv4(ip)) if !public_ip(ip.into()) => return Err(error()),
        Some(url::Host::Ipv6(ip)) if !public_ip(ip.into()) => return Err(error()),
        Some(url::Host::Domain(host))
            if host == "localhost"
                || host.ends_with(".localhost")
                || host.ends_with(".local")
                || !host.contains('.') =>
        {
            return Err(error());
        }
        _ => {}
    }
    Ok(url)
}

fn public_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => {
            let [a, b, c, _] = ip.octets();
            !ip.is_private()
                && !ip.is_loopback()
                && !ip.is_link_local()
                && !ip.is_multicast()
                && a != 0
                && a < 240
                && !(a == 100 && (64..=127).contains(&b))
                && !(a == 192 && ((b == 0 && (c == 0 || c == 2)) || (b == 88 && c == 99)))
                && !(a == 198 && (b == 18 || b == 19 || (b == 51 && c == 100)))
                && !(a == 203 && b == 0 && c == 113)
        }
        IpAddr::V6(ip) => {
            let s = ip.segments();
            // Public global unicast only, excluding special-use and transition ranges.
            s[0] & 0xe000 == 0x2000
                && s[0] != 0x2002
                && !(s[0] == 0x2001 && (s[1] < 0x0200 || s[1] == 0x0db8))
                && !(s[0] == 0x3fff && s[1] < 0x1000)
        }
    }
}

fn resolve(url: &Url, remaining: Duration) -> Result<Vec<SocketAddr>, FulltextError> {
    let host = url
        .host_str()
        .ok_or_else(|| failure("fulltext-url-blocked", "Missing public host"))?
        .trim_matches(['[', ']'])
        .to_string();
    let (sender, receiver) = mpsc::sync_channel(1);
    std::thread::Builder::new()
        .name("fulltext-dns".into())
        .spawn(move || {
            let result = (host.as_str(), 443)
                .to_socket_addrs()
                .map(|addresses| addresses.collect::<Vec<_>>());
            let _ = sender.send(result);
        })
        .map_err(|_| failure("fulltext-network-error", "Cannot start DNS lookup"))?;
    let addresses = receiver
        .recv_timeout(remaining.min(Duration::from_secs(5)))
        .map_err(|_| failure("fulltext-network-error", "DNS lookup timed out"))?
        .map_err(|_| {
            failure(
                "fulltext-network-error",
                "Public host could not be resolved",
            )
        })?;
    if addresses.is_empty() || addresses.iter().any(|address| !public_ip(address.ip())) {
        return Err(failure(
            "fulltext-url-blocked",
            "The source resolves to a nonpublic address",
        ));
    }
    Ok(addresses)
}

fn openalex_content(url: &Url) -> bool {
    url.host_str() == Some("content.openalex.org")
        && url
            .path()
            .strip_prefix("/works/W")
            .and_then(|path| {
                path.strip_suffix(".pdf")
                    .or_else(|| path.strip_suffix(".grobid-xml"))
            })
            .is_some_and(|id| !id.is_empty() && id.bytes().all(|b| b.is_ascii_digit()))
}

fn fetch(source: &str, openalex_key: Option<&str>) -> Result<(String, Vec<u8>), FulltextError> {
    let started = Instant::now();
    let mut url = public_url(source)?;
    for _ in 0..=5 {
        let remaining = DEADLINE
            .checked_sub(started.elapsed())
            .ok_or_else(|| failure("fulltext-network-error", "Fulltext download timed out"))?;
        let addresses = resolve(&url, remaining)?;
        let remaining = DEADLINE
            .checked_sub(started.elapsed())
            .ok_or_else(|| failure("fulltext-network-error", "Fulltext download timed out"))?;
        let host = url
            .host_str()
            .ok_or_else(|| failure("fulltext-url-blocked", "Missing public host"))?;
        let client = reqwest::blocking::Client::builder()
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .connect_timeout(remaining.min(Duration::from_secs(5)))
            .timeout(remaining)
            .user_agent("Qiongli/2.1 research fulltext reader")
            .resolve_to_addrs(host, &addresses)
            .build()
            .map_err(|_| {
                failure(
                    "fulltext-network-error",
                    "Cannot create fulltext HTTP client",
                )
            })?;
        let mut request_url = url.clone();
        if openalex_content(&url) {
            let key = openalex_key.filter(|key| !key.is_empty())
                .ok_or_else(|| failure("fulltext-auth-required", "OpenAlex content requires its configured API key; use another public candidate or configure OpenAlex"))?;
            request_url.query_pairs_mut().append_pair("api_key", key);
        }
        let response = client
            .get(request_url)
            .header(
                reqwest::header::ACCEPT,
                "application/pdf, application/xml, text/xml;q=0.9",
            )
            .send()
            .map_err(|_| {
                failure(
                    "fulltext-network-error",
                    "Fulltext request failed or timed out",
                )
            })?;
        if response.status().is_redirection() {
            let location = response
                .headers()
                .get(reqwest::header::LOCATION)
                .and_then(|value| value.to_str().ok())
                .ok_or_else(|| {
                    failure("fulltext-redirect-error", "Redirect has no valid location")
                })?;
            let next = url
                .join(location)
                .map_err(|_| failure("fulltext-redirect-error", "Invalid redirect"))?;
            url = public_url(next.as_str())?;
            continue;
        }
        match response.status().as_u16() {
            401 | 403 => {
                return Err(failure(
                    "fulltext-access-restricted",
                    "The source denied access; try an authorized open version or Host browser",
                ));
            }
            404 | 410 => {
                return Err(failure(
                    "fulltext-not-found",
                    "Fulltext candidate was not found",
                ));
            }
            429 => {
                return Err(failure(
                    "fulltext-rate-limited",
                    "The source rate limit was reached; retry later or use another source",
                ));
            }
            _ if !response.status().is_success() => {
                return Err(failure(
                    "fulltext-http-error",
                    "The source returned an unsuccessful response",
                ));
            }
            _ => {}
        }
        if response
            .content_length()
            .is_some_and(|size| size > MAX_BYTES as u64)
        {
            return Err(failure(
                "fulltext-too-large",
                "Fulltext exceeds the 12 MiB download limit",
            ));
        }
        let gzip = response
            .headers()
            .get(reqwest::header::CONTENT_ENCODING)
            .is_some_and(|value| value == "gzip");
        let bytes = bounded_bytes(response)?;
        let bytes = if gzip || bytes.starts_with(&[0x1f, 0x8b]) {
            bounded_bytes(flate2::read::GzDecoder::new(bytes.as_slice()))?
        } else {
            bytes
        };
        return Ok((url.to_string(), bytes));
    }
    Err(failure("fulltext-redirect-error", "Too many redirects"))
}

fn bounded_bytes(reader: impl Read) -> Result<Vec<u8>, FulltextError> {
    let mut bytes = Vec::new();
    reader
        .take((MAX_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|_| {
            failure(
                "fulltext-network-error",
                "Cannot read complete fulltext bytes",
            )
        })?;
    if bytes.len() > MAX_BYTES {
        return Err(failure(
            "fulltext-too-large",
            "Fulltext exceeds the 12 MiB decoded byte limit",
        ));
    }
    Ok(bytes)
}

fn normalize_doi(doi: &str) -> String {
    let doi = doi.trim().to_ascii_lowercase();
    doi.strip_prefix("https://doi.org/")
        .or_else(|| doi.strip_prefix("http://doi.org/"))
        .or_else(|| doi.strip_prefix("doi:"))
        .unwrap_or(&doi)
        .trim()
        .to_string()
}

fn parse_document(
    source: &str,
    resolved: &str,
    bytes: &[u8],
) -> Result<FulltextOutput, FulltextError> {
    if bytes.len() > MAX_BYTES {
        return Err(failure(
            "fulltext-too-large",
            "Fulltext exceeds the decoded byte limit",
        ));
    }
    let mut doc = FulltextOutput {
        status: "readable_text", source_url: source.into(), resolved_url: resolved.into(),
        source_sha256: format!("{:x}", Sha256::digest(bytes)), format: "pdf",
        retrieved_at_unix_seconds: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
        cached: false, total_segments: 0, segments: Vec::new(), next_offset: None,
        document_title: None, document_doi: None, identity_status: "not_checked",
        warnings: vec!["Only returned segments have been exposed for inspection; parsing does not establish complete reading or evidence eligibility".into()], text_bytes: 0,
    };
    if bytes.starts_with(b"%PDF-") {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| parse_pdf(bytes, &mut doc)))
            .map_err(|_| {
                failure(
                    "fulltext-parse-error",
                    "PDF parser could not read this document",
                )
            })??;
    } else {
        parse_xml(bytes, &mut doc)?;
    }
    if doc.segments.is_empty() {
        return Err(failure(
            "fulltext-no-readable-body",
            "No readable body text was found; metadata, abstracts and image-only pages are not fulltext",
        ));
    }
    doc.total_segments = doc.segments.len();
    Ok(doc)
}

fn append_text(
    doc: &mut FulltextOutput,
    text: &str,
    section: Option<String>,
    page: Option<u32>,
) -> Result<(), FulltextError> {
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if doc.text_bytes + text.len() > MAX_TEXT
        || doc.segments.len() + text.chars().count().div_ceil(SEGMENT_CHARS) > 10_000
    {
        return Err(failure(
            "fulltext-too-large",
            "Extracted body text exceeds 2 MiB or 10,000 segments",
        ));
    }
    doc.text_bytes += text.len();
    let mut start = 0;
    let mut count = 0;
    for (position, _) in text.char_indices() {
        if count == SEGMENT_CHARS {
            push_segment(doc, &text[start..position], section.clone(), page);
            start = position;
            count = 0;
        }
        count += 1;
    }
    push_segment(doc, &text[start..], section, page);
    Ok(())
}

fn push_segment(doc: &mut FulltextOutput, text: &str, section: Option<String>, page: Option<u32>) {
    if text.trim().is_empty() {
        return;
    }
    let index = doc.segments.len();
    let anchor = match page {
        Some(page) => format!("page={page};segment={index}"),
        None => format!("body;segment={index}"),
    };
    doc.segments.push(FulltextSegment {
        index,
        anchor,
        text: text.into(),
        section,
        page,
    });
}

struct BoundedText {
    bytes: Vec<u8>,
    remaining: usize,
}

impl std::io::Write for BoundedText {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if bytes.len() > self.remaining {
            return Err(std::io::Error::other("PDF text limit exceeded"));
        }
        self.remaining -= bytes.len();
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn parse_pdf(bytes: &[u8], doc: &mut FulltextOutput) -> Result<(), FulltextError> {
    // Download and emitted-text caps are not a PDF parser sandbox: the library
    // still owns internal stream/font decompression. Do not promise a hard
    // process memory/time budget or execute embedded PDF resources.
    let pdf = pdf_extract::Document::load_mem(bytes).map_err(|_| {
        failure(
            "fulltext-parse-error",
            "Cannot parse PDF; encrypted or malformed files require a different source",
        )
    })?;
    if pdf.is_encrypted() {
        return Err(failure(
            "fulltext-access-restricted",
            "Encrypted PDFs are not supported",
        ));
    }
    let pages = pdf.get_pages();
    if pages.len() > 300 {
        return Err(failure("fulltext-too-large", "PDF exceeds 300 pages"));
    }
    doc.warnings.push("PDF reading order, tables and formulas may be incomplete; no OCR is performed. Inspect the original for layout-sensitive claims".into());
    for page in pages.keys() {
        let mut text = BoundedText {
            bytes: Vec::new(),
            remaining: MAX_TEXT - doc.text_bytes,
        };
        let mut output = pdf_extract::PlainTextOutput::new(&mut text as &mut dyn std::io::Write);
        pdf_extract::output_doc_page(&pdf, &mut output, *page).map_err(|_| {
            failure(
                "fulltext-parse-error",
                "A PDF page could not be extracted; use another source or Host PDF reader",
            )
        })?;
        let text = String::from_utf8(text.bytes)
            .map_err(|_| failure("fulltext-parse-error", "Invalid PDF text encoding"))?;
        if text.trim().is_empty() {
            doc.warnings.push(format!(
                "Page {page} contains no extractable text; it may require OCR"
            ));
        }
        append_text(doc, &text, None, Some(*page))?;
    }
    Ok(())
}

// Capture body blocks with their inline text; metadata fields are deliberately
// confined to the article header, never mined from references or body mentions.
fn parse_xml(bytes: &[u8], doc: &mut FulltextOutput) -> Result<(), FulltextError> {
    let mut reader = Reader::from_reader(bytes);
    let mut buffer = Vec::new();
    let mut stack = Vec::<String>::new();
    let mut sections = Vec::<(usize, String)>::new();
    let mut capture = None::<(usize, String, String)>;
    let mut recognized = false;
    let mut paper_depth = None;
    let mut saw_root = false;
    let mut has_body_text = false;
    loop {
        let event = reader
            .read_event_into(&mut buffer)
            .map_err(|_| failure("fulltext-parse-error", "Malformed XML document"))?;
        match event {
            Event::Start(event) => {
                let name =
                    String::from_utf8_lossy(event.local_name().as_ref()).to_ascii_lowercase();
                if matches!(
                    name.as_str(),
                    "p" | "td" | "th" | "tr" | "label" | "title" | "list-item"
                ) && let Some((_, _, text)) = &mut capture
                {
                    text.push(' ');
                }
                if stack.is_empty() {
                    if saw_root {
                        return Err(failure("fulltext-parse-error", "Multiple XML roots"));
                    }
                    saw_root = true;
                    if !matches!(name.as_str(), "tei" | "article" | "pmc-articleset" | "html") {
                        return Err(failure(
                            "fulltext-unsupported-format",
                            "Use PDF, TEI or JATS body content; read publisher HTML with an available Host browser",
                        ));
                    }
                }
                if name == "sub-article" {
                    return Err(failure(
                        "fulltext-unsupported-format",
                        "Sub-articles need separate document selection",
                    ));
                }
                if name == "tei" || name == "article" {
                    let supported_position = if name == "article" {
                        stack.is_empty() || stack.as_slice() == ["pmc-articleset"]
                    } else {
                        stack.is_empty() || stack.as_slice() == ["html", "body"]
                    };
                    if !supported_position || recognized {
                        return Err(failure(
                            "fulltext-unsupported-format",
                            "Expected one TEI or JATS paper; HTML and multi-paper XML require a different reader",
                        ));
                    }
                    doc.format = if name == "tei" { "tei_xml" } else { "jats_xml" };
                    recognized = true;
                    paper_depth = Some(stack.len());
                }
                stack.push(name.clone());
                if stack.len() > 128 {
                    return Err(failure("fulltext-too-large", "XML nesting limit exceeded"));
                }
                let paper = paper_depth
                    .and_then(|depth| stack.get(depth..))
                    .unwrap_or(&[]);
                let starts = |prefix: &[&str]| {
                    paper.len() >= prefix.len()
                        && paper
                            .iter()
                            .zip(prefix)
                            .all(|(part, expected)| part == expected)
                };
                let body = !paper
                    .iter()
                    .any(|s| matches!(s.as_str(), "abstract" | "back" | "front"))
                    && (starts(&["article", "body"]) || starts(&["tei", "text"]));
                let article_header = starts(&["article", "front", "article-meta"]);
                let tei_title = starts(&["tei", "teiheader", "filedesc", "titlestmt"]);
                let tei_source =
                    starts(&["tei", "teiheader", "filedesc", "sourcedesc", "biblstruct"])
                        && (paper.len() == 6 || (paper.len() == 7 && paper[5] == "analytic"));
                if body && matches!(name.as_str(), "sec" | "div") {
                    sections.push((stack.len(), String::new()));
                }
                if capture.is_none() {
                    let kind = if body && matches!(name.as_str(), "head" | "title") {
                        Some("section")
                    } else if body
                        && matches!(
                            name.as_str(),
                            "p" | "ab"
                                | "formula"
                                | "table"
                                | "fig"
                                | "quote"
                                | "item"
                                | "list-item"
                        )
                    {
                        Some("body")
                    } else if !body
                        && ((name == "article-title"
                            && article_header
                            && paper.len() == 5
                            && paper[3] == "title-group")
                            || (name == "title" && tei_title))
                    {
                        Some("title")
                    } else if !body
                        && ((name == "article-id" && article_header && paper.len() == 4)
                            || (name == "idno" && tei_source))
                    {
                        let mut doi = false;
                        for attr in event.attributes() {
                            let attr = attr.map_err(|_| {
                                failure("fulltext-parse-error", "Malformed XML attributes")
                            })?;
                            if matches!(attr.key.as_ref(), b"pub-id-type" | b"type")
                                && attr.value.eq_ignore_ascii_case(b"doi")
                            {
                                doi = true;
                            }
                        }
                        doi.then_some("doi")
                    } else {
                        None
                    };
                    if let Some(kind) = kind {
                        capture = Some((stack.len(), kind.into(), String::new()));
                    }
                }
            }
            Event::Text(event) => {
                if let Some((_, _, text)) = &mut capture {
                    text.push_str(
                        &event.unescape().map_err(|_| {
                            failure("fulltext-parse-error", "Unsupported XML entity")
                        })?,
                    );
                    if text.len() > MAX_TEXT {
                        return Err(failure(
                            "fulltext-too-large",
                            "XML text block exceeds the extraction limit",
                        ));
                    }
                }
            }
            Event::CData(event) => {
                if let Some((_, _, text)) = &mut capture {
                    text.push_str(
                        std::str::from_utf8(event.as_ref())
                            .map_err(|_| failure("fulltext-parse-error", "XML must use UTF-8"))?,
                    );
                    if text.len() > MAX_TEXT {
                        return Err(failure(
                            "fulltext-too-large",
                            "XML text block exceeds the extraction limit",
                        ));
                    }
                }
            }
            Event::Empty(event) => {
                if matches!(event.local_name().as_ref(), b"br" | b"lb" | b"break")
                    && let Some((_, _, text)) = &mut capture
                {
                    text.push(' ');
                }
            }
            Event::End(_) => {
                if capture
                    .as_ref()
                    .is_some_and(|(depth, _, _)| *depth == stack.len())
                {
                    let (_, kind, text) = capture.take().expect("checked capture");
                    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
                    match kind.as_str() {
                        "section" => {
                            if let Some((_, title)) = sections.last_mut() {
                                *title = text.chars().take(512).collect();
                            }
                            append_text(
                                doc,
                                &text,
                                (!text.is_empty()).then(|| text.chars().take(512).collect()),
                                None,
                            )?;
                        }
                        "body" => {
                            let section = sections
                                .iter()
                                .map(|(_, title)| title.as_str())
                                .filter(|title| !title.is_empty())
                                .collect::<Vec<_>>()
                                .join(" / ")
                                .chars()
                                .take(1024)
                                .collect::<String>();
                            if !text.is_empty() {
                                has_body_text = true;
                            }
                            append_text(
                                doc,
                                &text,
                                (!section.is_empty()).then_some(section),
                                None,
                            )?;
                        }
                        "title" if doc.document_title.is_none() && !text.is_empty() => {
                            doc.document_title = Some(text.chars().take(1024).collect())
                        }
                        "doi" if text.len() <= 512 && normalize_doi(&text).starts_with("10.") => {
                            let doi = normalize_doi(&text);
                            if doc
                                .document_doi
                                .as_ref()
                                .is_some_and(|existing| existing != &doi)
                            {
                                return Err(failure(
                                    "fulltext-identity-mismatch",
                                    "Conflicting DOI fields in the document header",
                                ));
                            }
                            doc.document_doi = Some(doi);
                        }
                        _ => {}
                    }
                }
                sections.retain(|(depth, _)| *depth < stack.len());
                stack.pop();
            }
            Event::DocType(event) if event.as_ref().contains(&b'[') => {
                return Err(failure(
                    "fulltext-parse-error",
                    "XML internal entities are not supported",
                ));
            }
            Event::Eof => break,
            _ => {}
        }
        buffer.clear();
    }
    if !stack.is_empty() {
        return Err(failure("fulltext-parse-error", "Incomplete XML document"));
    }
    if !recognized {
        return Err(failure(
            "fulltext-unsupported-format",
            "No TEI or JATS article; use an available Host reader for publisher HTML",
        ));
    }
    if !has_body_text {
        return Err(failure(
            "fulltext-no-readable-body",
            "XML contains no body paragraphs; headings, metadata and abstracts do not establish fulltext",
        ));
    }
    doc.warnings.push("XML extraction includes body text only; external resources are not loaded and figure, table or formula layout may need inspection in the original".into());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const URL: &str = "https://example.org/paper.xml";
    const JATS: &[u8] = br#"<?xml version="1.0"?><!DOCTYPE article SYSTEM "https://example.org/unfetched.dtd"><article><front><article-meta><title-group><article-title>Test paper</article-title></title-group><article-id pub-id-type="doi">10.1234/test</article-id><abstract><p>Abstract only summary.</p></abstract></article-meta></front><body><sec><title>Methods</title><p>Actual <italic>body</italic> evidence.</p><table-wrap><table><tr><td>10</td><td>20</td></tr></table></table-wrap></sec></body><back><ref-list><ref><p>Not body evidence</p></ref></ref-list></back></article>"#;

    #[test]
    fn public_source_boundary_and_continuation_reject_unsafe_inputs() {
        for url in [
            "http://example.org/a.pdf",
            "file:///tmp/paper.pdf",
            "https://127.0.0.1/a",
            "https://2130706433/a",
            "https://[::1]/a",
            "https://[::ffff:127.0.0.1]/a",
            "https://user:secret@example.org/a",
            "https://example.org:8443/a",
            "https://example.org/a?api_key=secret",
            "https://example.org/a?X-Amz-Signature=secret",
            "https://localhost/a",
            "https://printer.local/a",
        ] {
            assert!(
                FulltextRequest::from_arguments(&json!({"url": url})).is_err(),
                "{url}"
            );
        }
        for ip in [
            "10.0.0.1",
            "169.254.169.254",
            "100.64.0.1",
            "192.0.0.8",
            "198.18.0.1",
            "224.0.0.1",
            "240.0.0.1",
            "2001:db8::1",
            "2002:7f00:1::",
            "fc00::1",
        ] {
            assert!(!public_ip(ip.parse().unwrap()), "{ip}");
        }
        assert!(public_ip("8.8.8.8".parse().unwrap()));
        assert!(public_ip("2606:4700:4700::1111".parse().unwrap()));
        for arguments in [
            json!({"url": URL,"offset": 1}),
            json!({"url": URL,"limit": 0}),
            json!({"url": URL,"limit": 51}),
            json!({"url": URL,"expected_sha256":"abc"}),
            json!({"url": URL,"output_path":"/tmp/write"}),
        ] {
            assert!(FulltextRequest::from_arguments(&arguments).is_err());
        }
        let redirect = Url::parse(URL)
            .unwrap()
            .join("//127.0.0.1/private")
            .unwrap();
        assert!(public_url(redirect.as_str()).is_err());
        assert!(openalex_content(
            &Url::parse("https://content.openalex.org/works/W123.pdf").unwrap()
        ));
        assert!(!openalex_content(
            &Url::parse("https://content.openalex.org.evil.org/works/W123.pdf").unwrap()
        ));
        assert!(!openalex_content(
            &Url::parse("https://content.openalex.org/arbitrary").unwrap()
        ));
    }

    #[test]
    fn body_parsing_preserves_identity_sections_and_separates_cells() {
        let doc = parse_document(URL, URL, JATS).unwrap();
        assert_eq!(doc.document_doi.as_deref(), Some("10.1234/test"));
        assert_eq!(doc.document_title.as_deref(), Some("Test paper"));
        assert_eq!(doc.format, "jats_xml");
        assert_eq!(doc.segments[1].section.as_deref(), Some("Methods"));
        let texts = doc
            .segments
            .iter()
            .map(|part| part.text.as_str())
            .collect::<Vec<_>>()
            .join("|");
        assert!(texts.contains("Actual body evidence."));
        assert!(texts.contains("10 20"));
        assert!(!texts.contains("Abstract only") && !texts.contains("Not body"));
        for xml in [
            "<TEI><teiHeader><fileDesc><titleStmt><title>TEI paper</title></titleStmt></fileDesc></teiHeader><text><body><div><head>Results</head><p>Observed result.</p></div></body></text></TEI>",
            "<html><body><tei><text><div><head>Results</head><p>Observed result.</p></div></text></tei></body></html>",
        ] {
            let doc = parse_document(URL, URL, xml.as_bytes()).unwrap();
            assert_eq!(doc.format, "tei_xml");
            assert_eq!(doc.segments[1].section.as_deref(), Some("Results"));
        }
    }

    #[test]
    fn metadata_html_multiple_articles_entities_and_incomplete_xml_are_not_fulltext() {
        for xml in [
            "<article><front><article-meta><abstract><p>Summary</p></abstract></article-meta></front></article>",
            "<article><body><sec><title>Methods</title></sec></body></article>",
            "<html><body><article><p>Paywall page</p></article></body></html>",
            "<pmc-articleset><article><body><p>First paper</p></body></article><article><body><p>Another paper</p></body></article></pmc-articleset>",
            "<article><front><article-meta><article-id pub-id-type='doi'>10.1234/parent</article-id></article-meta></front><body><p>First paper</p></body><sub-article><front-stub/><body><p>Another paper</p></body></sub-article></article>",
            "<!DOCTYPE article [<!ENTITY private SYSTEM 'file:///etc/passwd'>]><article><body><p>&private;</p></body></article>",
            "<article><body><p>Truncated body</p>",
        ] {
            assert!(parse_document(URL, URL, xml.as_bytes()).is_err(), "{xml}");
        }
        let wrapped = b"<pmc-articleset><article-meta><article-id pub-id-type='doi'>10.1234/wrong</article-id></article-meta><article><body><p>Actual paper.</p></body></article></pmc-articleset>";
        assert!(
            parse_document(URL, URL, wrapped)
                .unwrap()
                .document_doi
                .is_none()
        );
        let heading = "h".repeat(20_000);
        let xml = format!(
            "<article><body><sec><title>{heading}</title><p>Body.</p></sec></body></article>"
        );
        let mut doc = parse_document(URL, URL, xml.as_bytes()).unwrap();
        assert!(doc.segments.iter().all(|segment| {
            segment
                .section
                .as_ref()
                .is_none_or(|section| section.chars().count() <= 512)
        }));
        doc.segments.resize(9_999, doc.segments[0].clone());
        assert_eq!(
            append_text(&mut doc, &"x".repeat(2_001), None, None)
                .unwrap_err()
                .code,
            "fulltext-too-large"
        );
    }

    #[test]
    fn cache_pagination_refresh_and_expected_identity_are_source_bound() {
        let reader = FulltextReader::default();
        let request = FulltextRequest::from_arguments(
            &json!({"url":URL, "limit":1, "expected_doi":"https://doi.org/10.1234/TEST"}),
        )
        .unwrap();
        let first = reader
            .read_with(&request, || Ok((URL.into(), JATS.to_vec())))
            .unwrap();
        assert_eq!(first.identity_status, "matched");
        assert_eq!(first.next_offset, Some(1));
        assert!(!first.cached);
        let next = FulltextRequest::from_arguments(
            &json!({"url":URL,"offset":1,"expected_sha256":first.source_sha256}),
        )
        .unwrap();
        let second = reader
            .read_with(&next, || panic!("cached continuation must not fetch"))
            .unwrap();
        assert!(second.cached);
        assert_eq!(second.segments[0].index, 1);
        let mismatch =
            FulltextRequest::from_arguments(&json!({"url":URL,"expected_doi":"10.1234/wrong"}))
                .unwrap();
        assert_eq!(
            reader.read_with(&mismatch, || panic!()).unwrap_err().code,
            "fulltext-identity-mismatch"
        );
        let changed = FulltextRequest::from_arguments(
            &json!({"url":URL,"refresh":true,"expected_sha256":first.source_sha256}),
        )
        .unwrap();
        assert_eq!(
            reader
                .read_with(&changed, || Ok((
                    URL.into(),
                    b"<article><body><p>Changed bytes.</p></body></article>".to_vec()
                )))
                .unwrap_err()
                .code,
            "fulltext-source-changed"
        );
        let unknown =
            FulltextRequest::from_arguments(&json!({"url":URL,"expected_doi":"10.1234/test"}))
                .unwrap();
        assert_eq!(
            reader
                .read_with(&unknown, || panic!())
                .unwrap()
                .identity_status,
            "unverified"
        );
        assert!(
            reader
                .read_with(&changed, || Err(failure("fulltext-not-found", "gone")))
                .is_err()
        );
        assert_eq!(
            reader
                .read_with(&request, || Err(failure("fulltext-not-found", "gone")))
                .unwrap_err()
                .code,
            "fulltext-not-found"
        );
    }

    #[test]
    fn pdf_pages_have_stable_anchors_and_emitted_text_is_bounded() {
        let objects = [
            "<< /Type /Catalog /Pages 2 0 R >>".to_string(),
            "<< /Type /Pages /Kids [3 0 R 4 0 R] /Count 2 >>".into(),
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << /Font << /F1 5 0 R >> >> /Contents 6 0 R >>".into(),
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << /Font << /F1 5 0 R >> >> /Contents 7 0 R >>".into(),
            "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>".into(),
            pdf_stream("BT /F1 12 Tf 72 720 Td (Methods page one) Tj ET"),
            pdf_stream("BT /F1 12 Tf 72 720 Td (Results page two) Tj ET"),
        ];
        let mut pdf = String::from("%PDF-1.4\n");
        let mut offsets = Vec::new();
        for (i, object) in objects.iter().enumerate() {
            offsets.push(pdf.len());
            pdf.push_str(&format!("{} 0 obj\n{object}\nendobj\n", i + 1));
        }
        let xref = pdf.len();
        pdf.push_str("xref\n0 8\n0000000000 65535 f \n");
        for offset in offsets {
            pdf.push_str(&format!("{offset:010} 00000 n \n"));
        }
        pdf.push_str(&format!(
            "trailer\n<< /Size 8 /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n"
        ));
        let doc = parse_document(URL, URL, pdf.as_bytes()).unwrap();
        assert_eq!(doc.segments.len(), 2);
        assert_eq!(doc.segments[1].page, Some(2));
        assert!(doc.segments[1].anchor.starts_with("page=2;"));
        assert!(doc.segments[0].text.contains("Methods page one"));
        let mut bounded = BoundedText {
            bytes: Vec::new(),
            remaining: 2,
        };
        assert!(std::io::Write::write_all(&mut bounded, b"123").is_err());
        assert!(bounded.bytes.is_empty());
        assert!(bounded_bytes(std::io::repeat(0).take((MAX_BYTES + 1) as u64)).is_err());
    }

    fn pdf_stream(text: &str) -> String {
        format!("<< /Length {} >>\nstream\n{text}\nendstream", text.len())
    }
}
