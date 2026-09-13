use std::collections::BTreeMap;
use std::fmt::{self, Write as _};
use std::io;

use serde_json::{Value, json};
use thiserror::Error;

use crate::providers::search::LiteratureResult;

const MAX_RECORDS: usize = 1_000;
const MAX_RECORD_INPUT_BYTES: usize = 2 * 1024 * 1024;
const MAX_EXPORT_BYTES: usize = 2 * 1024 * 1024;
const MAX_TEXT_FIELD_BYTES: usize = 16 * 1024;
const MAX_DOI_BYTES: usize = 2 * 1024;
const MAX_PROVIDER_IDS: usize = 16;
const MAX_PROVIDER_ID_BYTES: usize = 128;
const MAX_JSON_DEPTH: usize = 32;
const MAX_JSON_VALUES: usize = 100_000;

const ALLOWED_ARGUMENTS: [&str; 2] = ["records", "formats"];

pub const ALL_FORMAT_NAMES: [&str; 4] = [
    "references.json",
    "references.ris",
    "bibliography.bib",
    "zotero-import-report.md",
];

#[derive(Debug, Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub enum ZoteroFormat {
    CslJson,
    Ris,
    Bibtex,
    ImportReport,
}

impl ZoteroFormat {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CslJson => "references.json",
            Self::Ris => "references.ris",
            Self::Bibtex => "bibliography.bib",
            Self::ImportReport => "zotero-import-report.md",
        }
    }

    fn parse(value: &str) -> Result<Self, ZoteroExportError> {
        match value {
            "references.json" => Ok(Self::CslJson),
            "references.ris" => Ok(Self::Ris),
            "bibliography.bib" => Ok(Self::Bibtex),
            "zotero-import-report.md" => Ok(Self::ImportReport),
            _ => Err(ZoteroExportError::UnsupportedFormat),
        }
    }

    const fn all() -> [Self; 4] {
        [Self::CslJson, Self::Ris, Self::Bibtex, Self::ImportReport]
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Error)]
pub enum ZoteroExportError {
    #[error("Zotero export arguments must be an object")]
    ArgumentsNotObject,
    #[error("Unsupported argument")]
    UnsupportedArgument,
    #[error("records must be an array")]
    RecordsNotArray,
    #[error("records must contain literature results")]
    InvalidRecord,
    #[error("records exceed the record limit")]
    TooManyRecords,
    #[error("records exceed the byte limit")]
    InputTooLarge,
    #[error("records exceed the nesting limit")]
    InputTooDeep,
    #[error("records exceed the value-count limit")]
    InputTooComplex,
    #[error("formats must be an array")]
    FormatsNotArray,
    #[error("formats must contain strings")]
    FormatNotString,
    #[error("Unsupported format")]
    UnsupportedFormat,
    #[error("formats must not contain duplicates")]
    DuplicateFormat,
    #[error("record title must not be empty")]
    EmptyTitle,
    #[error("record text field exceeds the byte limit")]
    TextFieldTooLong,
    #[error("record DOI exceeds the byte limit")]
    DoiTooLong,
    #[error("record provider must not be empty")]
    EmptyProvider,
    #[error("record provider metadata exceeds the limit")]
    ProviderMetadataTooLarge,
    #[error("generated Zotero files exceed the byte limit")]
    OutputTooLarge,
    #[error("generated Zotero files could not be serialized")]
    Serialization,
}

pub struct ZoteroExportRequest {
    records: Vec<LiteratureResult>,
    formats: Vec<ZoteroFormat>,
}

impl ZoteroExportRequest {
    pub fn from_arguments(arguments: &Value) -> Result<Self, ZoteroExportError> {
        let entries = arguments
            .as_object()
            .ok_or(ZoteroExportError::ArgumentsNotObject)?;
        if entries
            .keys()
            .any(|key| !ALLOWED_ARGUMENTS.contains(&key.as_str()))
        {
            return Err(ZoteroExportError::UnsupportedArgument);
        }
        validate_json_shape(arguments)?;
        ensure_json_within_limit(arguments, MAX_RECORD_INPUT_BYTES)?;

        let records_value = entries
            .get("records")
            .cloned()
            .unwrap_or_else(|| Value::Array(Vec::new()));
        let records_array = records_value
            .as_array()
            .ok_or(ZoteroExportError::RecordsNotArray)?;
        if records_array.len() > MAX_RECORDS {
            return Err(ZoteroExportError::TooManyRecords);
        }
        if records_array.iter().any(|record| !record.is_object()) {
            return Err(ZoteroExportError::InvalidRecord);
        }
        let records =
            serde_json::from_value(records_value).map_err(|_| ZoteroExportError::InvalidRecord)?;
        let formats = parse_formats(entries.get("formats"))?;
        Self::validated(records, formats)
    }

    pub fn from_records(records: Vec<LiteratureResult>) -> Result<Self, ZoteroExportError> {
        Self::validated(records, ZoteroFormat::all().to_vec())
    }

    fn validated(
        records: Vec<LiteratureResult>,
        formats: Vec<ZoteroFormat>,
    ) -> Result<Self, ZoteroExportError> {
        validate_records(&records)?;
        ensure_json_within_limit(&records, MAX_RECORD_INPUT_BYTES)?;
        Ok(Self { records, formats })
    }
}

pub fn export_import_files(
    records: Vec<LiteratureResult>,
) -> Result<BTreeMap<String, String>, ZoteroExportError> {
    export_selected_import_files(ZoteroExportRequest::from_records(records)?)
}

pub fn export_selected_import_files(
    request: ZoteroExportRequest,
) -> Result<BTreeMap<String, String>, ZoteroExportError> {
    let mut files = BTreeMap::new();
    let mut remaining = MAX_EXPORT_BYTES;
    for format in request.formats {
        let content = match format {
            ZoteroFormat::CslJson => to_csl_json(&request.records, remaining)?,
            ZoteroFormat::Ris => to_ris(&request.records, remaining)?,
            ZoteroFormat::Bibtex => to_bibtex(&request.records, remaining)?,
            ZoteroFormat::ImportReport => to_report(&request.records, remaining)?,
        };
        remaining = remaining
            .checked_sub(content.len())
            .ok_or(ZoteroExportError::OutputTooLarge)?;
        files.insert(format.as_str().to_owned(), content);
    }
    Ok(files)
}

fn parse_formats(value: Option<&Value>) -> Result<Vec<ZoteroFormat>, ZoteroExportError> {
    let Some(value) = value else {
        return Ok(ZoteroFormat::all().to_vec());
    };
    let values = value.as_array().ok_or(ZoteroExportError::FormatsNotArray)?;
    if values.is_empty() {
        return Ok(ZoteroFormat::all().to_vec());
    }
    let mut formats = Vec::with_capacity(values.len());
    for value in values {
        let value = value.as_str().ok_or(ZoteroExportError::FormatNotString)?;
        let format = ZoteroFormat::parse(value)?;
        if formats.contains(&format) {
            return Err(ZoteroExportError::DuplicateFormat);
        }
        formats.push(format);
    }
    Ok(formats)
}

fn validate_records(records: &[LiteratureResult]) -> Result<(), ZoteroExportError> {
    if records.len() > MAX_RECORDS {
        return Err(ZoteroExportError::TooManyRecords);
    }
    for record in records {
        if record.title.trim().is_empty() {
            return Err(ZoteroExportError::EmptyTitle);
        }
        if record.title.len() > MAX_TEXT_FIELD_BYTES
            || record
                .venue
                .as_ref()
                .is_some_and(|venue| venue.len() > MAX_TEXT_FIELD_BYTES)
        {
            return Err(ZoteroExportError::TextFieldTooLong);
        }
        if record
            .doi
            .as_ref()
            .is_some_and(|doi| doi.len() > MAX_DOI_BYTES)
        {
            return Err(ZoteroExportError::DoiTooLong);
        }
        if record.provider.trim().is_empty() {
            return Err(ZoteroExportError::EmptyProvider);
        }
        if record.provider.len() > MAX_PROVIDER_ID_BYTES
            || record.providers.len() > MAX_PROVIDER_IDS
            || record.providers.iter().any(|provider| {
                provider.trim().is_empty() || provider.len() > MAX_PROVIDER_ID_BYTES
            })
        {
            return Err(ZoteroExportError::ProviderMetadataTooLarge);
        }
    }
    Ok(())
}

fn to_csl_json(records: &[LiteratureResult], limit: usize) -> Result<String, ZoteroExportError> {
    let csl_records = records
        .iter()
        .map(|record| {
            json!({
                "type": "article-journal",
                "title": record.title,
                "DOI": record.doi,
                "container-title": record.venue,
                "issued": record.year.map(|year| json!({"date-parts": [[year]]})),
                "source": "qiongli-runtime",
                "qiongli_provider": record.provider,
                "qiongli_providers": record.providers
            })
        })
        .collect::<Vec<_>>();
    let mut output = LimitedBytes::new(limit);
    match serde_json::to_writer_pretty(&mut output, &csl_records) {
        Ok(()) => output.into_string(),
        Err(_) if output.exceeded => Err(ZoteroExportError::OutputTooLarge),
        Err(_) => Err(ZoteroExportError::Serialization),
    }
}

fn to_ris(records: &[LiteratureResult], limit: usize) -> Result<String, ZoteroExportError> {
    let mut output = LimitedText::new(limit);
    for (index, record) in records.iter().enumerate() {
        writeln!(output, "TY  - JOUR").map_err(|_| ZoteroExportError::OutputTooLarge)?;
        write_ris_field(&mut output, "TI", &record.title)?;
        write_ris_field(
            &mut output,
            "PY",
            &record.year.map(|year| year.to_string()).unwrap_or_default(),
        )?;
        write_ris_field(
            &mut output,
            "JO",
            record.venue.as_deref().unwrap_or_default(),
        )?;
        write_ris_field(&mut output, "DO", record.doi.as_deref().unwrap_or_default())?;
        writeln!(output, "ER  -").map_err(|_| ZoteroExportError::OutputTooLarge)?;
        if index + 1 < records.len() {
            writeln!(output).map_err(|_| ZoteroExportError::OutputTooLarge)?;
        }
    }
    Ok(output.into_string())
}

fn write_ris_field(
    output: &mut LimitedText,
    name: &str,
    value: &str,
) -> Result<(), ZoteroExportError> {
    let value = fold_single_line(value);
    writeln!(output, "{name}  - {value}").map_err(|_| ZoteroExportError::OutputTooLarge)
}

fn to_bibtex(records: &[LiteratureResult], limit: usize) -> Result<String, ZoteroExportError> {
    let mut output = LimitedText::new(limit);
    for (index, record) in records.iter().enumerate() {
        writeln!(output, "@article{{qiongli{},", index + 1)
            .map_err(|_| ZoteroExportError::OutputTooLarge)?;
        write_bibtex_field(&mut output, "title", &record.title)?;
        write_bibtex_field(
            &mut output,
            "year",
            &record.year.map(|year| year.to_string()).unwrap_or_default(),
        )?;
        write_bibtex_field(
            &mut output,
            "journal",
            record.venue.as_deref().unwrap_or_default(),
        )?;
        write_bibtex_field(
            &mut output,
            "doi",
            record.doi.as_deref().unwrap_or_default(),
        )?;
        writeln!(output, "}}")
            .and_then(|()| writeln!(output))
            .map_err(|_| ZoteroExportError::OutputTooLarge)?;
    }
    Ok(output.into_string())
}

fn write_bibtex_field(
    output: &mut LimitedText,
    name: &str,
    value: &str,
) -> Result<(), ZoteroExportError> {
    let value = escape_bibtex(value, output.remaining())?;
    writeln!(output, "  {name} = {{{value}}},").map_err(|_| ZoteroExportError::OutputTooLarge)
}

fn to_report(records: &[LiteratureResult], limit: usize) -> Result<String, ZoteroExportError> {
    let mut output = LimitedText::new(limit);
    write!(
        output,
        "# Zotero Import Report\n\nRecords: {}\n\nGenerated by qiongli-runtime.\n",
        records.len()
    )
    .map_err(|_| ZoteroExportError::OutputTooLarge)?;
    Ok(output.into_string())
}

fn fold_single_line(value: &str) -> String {
    value
        .trim()
        .chars()
        .map(|character| {
            if character.is_control() || (character.is_whitespace() && character != ' ') {
                ' '
            } else {
                character
            }
        })
        .collect()
}

fn escape_bibtex(value: &str, limit: usize) -> Result<String, ZoteroExportError> {
    let mut escaped = LimitedText::new(limit);
    for character in fold_single_line(value).chars() {
        let replacement = match character {
            '\\' => "\\textbackslash{}",
            '{' => "\\{",
            '}' => "\\}",
            '#' => "\\#",
            '%' => "\\%",
            '&' => "\\&",
            '_' => "\\_",
            '$' => "\\$",
            '^' => "\\textasciicircum{}",
            '~' => "\\textasciitilde{}",
            _ => {
                escaped
                    .write_char(character)
                    .map_err(|_| ZoteroExportError::OutputTooLarge)?;
                continue;
            }
        };
        escaped
            .write_str(replacement)
            .map_err(|_| ZoteroExportError::OutputTooLarge)?;
    }
    Ok(escaped.into_string())
}

fn ensure_json_within_limit<T: serde::Serialize + ?Sized>(
    value: &T,
    limit: usize,
) -> Result<(), ZoteroExportError> {
    let mut output = LimitedBytes::new(limit);
    match serde_json::to_writer(&mut output, value) {
        Ok(()) => Ok(()),
        Err(_) if output.exceeded => Err(ZoteroExportError::InputTooLarge),
        Err(_) => Err(ZoteroExportError::Serialization),
    }
}

fn validate_json_shape(root: &Value) -> Result<(), ZoteroExportError> {
    let mut stack = vec![(root, 0_usize)];
    let mut values = 0_usize;
    while let Some((value, depth)) = stack.pop() {
        values = values.saturating_add(1);
        if values > MAX_JSON_VALUES {
            return Err(ZoteroExportError::InputTooComplex);
        }
        match value {
            Value::Array(items) => {
                if depth >= MAX_JSON_DEPTH && !items.is_empty() {
                    return Err(ZoteroExportError::InputTooDeep);
                }
                if values
                    .saturating_add(stack.len())
                    .saturating_add(items.len())
                    > MAX_JSON_VALUES
                {
                    return Err(ZoteroExportError::InputTooComplex);
                }
                stack.extend(items.iter().map(|item| (item, depth + 1)));
            }
            Value::Object(entries) => {
                if depth >= MAX_JSON_DEPTH && !entries.is_empty() {
                    return Err(ZoteroExportError::InputTooDeep);
                }
                if values
                    .saturating_add(stack.len())
                    .saturating_add(entries.len())
                    > MAX_JSON_VALUES
                {
                    return Err(ZoteroExportError::InputTooComplex);
                }
                stack.extend(entries.values().map(|item| (item, depth + 1)));
            }
            _ => {}
        }
    }
    Ok(())
}

struct LimitedText {
    value: String,
    remaining: usize,
}

impl LimitedText {
    fn new(limit: usize) -> Self {
        Self {
            value: String::with_capacity(limit.min(16 * 1024)),
            remaining: limit,
        }
    }

    const fn remaining(&self) -> usize {
        self.remaining
    }

    fn into_string(self) -> String {
        self.value
    }
}

impl fmt::Write for LimitedText {
    fn write_str(&mut self, value: &str) -> fmt::Result {
        if value.len() > self.remaining {
            return Err(fmt::Error);
        }
        self.value.push_str(value);
        self.remaining -= value.len();
        Ok(())
    }
}

struct LimitedBytes {
    value: Vec<u8>,
    remaining: usize,
    exceeded: bool,
}

impl LimitedBytes {
    fn new(limit: usize) -> Self {
        Self {
            value: Vec::with_capacity(limit.min(16 * 1024)),
            remaining: limit,
            exceeded: false,
        }
    }

    fn into_string(self) -> Result<String, ZoteroExportError> {
        String::from_utf8(self.value).map_err(|_| ZoteroExportError::Serialization)
    }
}

impl io::Write for LimitedBytes {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        if buffer.len() > self.remaining {
            self.exceeded = true;
            return Err(io::Error::new(
                io::ErrorKind::WriteZero,
                "serialized Zotero limit exceeded",
            ));
        }
        self.value.extend_from_slice(buffer);
        self.remaining -= buffer.len();
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(title: &str) -> LiteratureResult {
        LiteratureResult {
            title: title.to_owned(),
            doi: Some("10.1234/example".to_owned()),
            year: Some(2025),
            venue: Some("Journal of Tests".to_owned()),
            provider: "openalex".to_owned(),
            providers: vec!["openalex".to_owned()],
        }
    }

    #[test]
    fn exports_all_formats_deterministically() {
        let files = export_import_files(vec![record("A Test Paper")]).unwrap();
        assert_eq!(files.len(), 4);
        assert!(files["references.json"].contains("qiongli-runtime"));
        assert!(files["references.ris"].contains("TY  - JOUR"));
        assert!(files["bibliography.bib"].contains("@article{qiongli1"));
        assert!(files["zotero-import-report.md"].contains("Records: 1"));
    }

    #[test]
    fn honors_unique_selected_formats_and_rejects_invalid_selection() {
        let request = ZoteroExportRequest::from_arguments(&json!({
            "records": [],
            "formats": ["references.ris"]
        }))
        .unwrap();
        let files = export_selected_import_files(request).unwrap();
        assert_eq!(files.keys().collect::<Vec<_>>(), vec!["references.ris"]);

        let duplicate = ZoteroExportRequest::from_arguments(&json!({
            "formats": ["references.ris", "references.ris"]
        }))
        .err()
        .unwrap();
        assert_eq!(duplicate, ZoteroExportError::DuplicateFormat);

        let unsupported = ZoteroExportRequest::from_arguments(&json!({
            "formats": ["attacker-controlled-format"]
        }))
        .err()
        .unwrap();
        assert_eq!(unsupported, ZoteroExportError::UnsupportedFormat);
    }

    #[test]
    fn folds_ris_lines_and_escapes_bibtex_syntax() {
        let mut unsafe_record = record("Safe\nER  - injected {title} \\ value & more");
        unsafe_record.venue = Some("Venue\r\nTY  - BOOK".to_owned());
        let files = export_import_files(vec![unsafe_record]).unwrap();
        let ris = &files["references.ris"];
        assert_eq!(ris.matches("TY  - JOUR").count(), 1);
        assert!(!ris.contains("\nER  - injected"));
        assert!(!ris.contains("\nTY  - BOOK"));

        let bibtex = &files["bibliography.bib"];
        assert!(bibtex.contains("\\{title\\}"));
        assert!(bibtex.contains("\\textbackslash{}"));
        assert!(bibtex.contains("\\&"));
    }

    #[test]
    fn rejects_record_and_output_bounds() {
        let too_many = ZoteroExportRequest::from_arguments(&json!({
            "records": vec![json!({
                "title": "Paper",
                "provider": "openalex",
                "providers": ["openalex"]
            }); MAX_RECORDS + 1]
        }))
        .err()
        .unwrap();
        assert_eq!(too_many, ZoteroExportError::TooManyRecords);

        let empty_title = ZoteroExportRequest::from_records(vec![record("  ")])
            .err()
            .unwrap();
        assert_eq!(empty_title, ZoteroExportError::EmptyTitle);

        let long_title =
            ZoteroExportRequest::from_records(vec![record(&"x".repeat(MAX_TEXT_FIELD_BYTES + 1))])
                .err()
                .unwrap();
        assert_eq!(long_title, ZoteroExportError::TextFieldTooLong);

        let records = vec![record(&"\\".repeat(MAX_TEXT_FIELD_BYTES)); 50];
        let request = ZoteroExportRequest::validated(records, vec![ZoteroFormat::Bibtex]).unwrap();
        let error = export_selected_import_files(request).err().unwrap();
        assert_eq!(error, ZoteroExportError::OutputTooLarge);
    }

    #[test]
    fn rejects_deep_ignored_record_metadata_before_deserialization() {
        let mut nested = json!({"leaf": true});
        for _ in 0..=MAX_JSON_DEPTH {
            nested = json!({"nested": nested});
        }
        let error = ZoteroExportRequest::from_arguments(&json!({
            "records": [{
                "title": "Paper",
                "provider": "openalex",
                "providers": ["openalex"],
                "ignored": nested
            }]
        }))
        .err()
        .unwrap();
        assert_eq!(error, ZoteroExportError::InputTooDeep);
    }
}
