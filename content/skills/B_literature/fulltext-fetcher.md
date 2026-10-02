---
id: fulltext-fetcher
stage: B_literature
description: "Retrieve and read selected papers with source anchors, version checks, and PRISMA-ready access records."
inputs:
  - type: ScreeningDecisionLog
    description: "Papers requiring full-text retrieval"
  - type: SearchResults
    description: "Optional records with DOI, URL, provider IDs, or OA metadata"
    required: false
outputs:
  - type: FullTextStatus
    artifact: "screening/full_text.md"
  - type: RetrievalManifest
    artifact: "retrieval_manifest.csv"
constraints:
  - "Keep fulltext-retrieval ownership of retrieval records and preserve preview/approval/CAS for project writes"
  - "Record source, version, access limits, and the actual sections read"
  - "Do not bypass paywalls or treat a candidate URL as retrieved evidence"
failure_modes:
  - "No readable full-text tool or authorized source is available"
  - "Only abstract or metadata is available"
  - "Document identity, version, or source digest cannot be confirmed"
tools: [filesystem, fulltext-retrieval]
tags: [literature, fulltext, open-access, retrieval, PRISMA, Zotero]
domain_aware: false
---

# Full-text Fetcher Skill

## Purpose

Retrieve the selected report and read the passages needed for the research
question. `fulltext-retrieval` retains ownership of `retrieval_manifest.csv` and
access status in `screening/full_text.md`; `paper-screener` owns eligibility.
Related tasks: B1 review pipeline and B2 targeted paper reading. A single-paper
request does not require a full review scaffold.

## Provider Ownership Boundary

Use `references/literature-provider-routing.md` for provider and Host search.
In native Lite or Full sessions exposing `qiongli_literature_read_fulltext`, use
that read-only tool for public HTTPS PDF, TEI XML or JATS XML candidates. It
returns source-anchored text segments, not a saved PDF or project artifact.
It does not search, perform OCR, obtain subscription access, or write the manifest.

Older Hosts and the retained `fulltext-retrieval` planning adapter may expose
only locator planning or resolver handoff. State that limit and use an actually
available authorized Host reader, local attachment, or external resolver. Do not
imitate an unavailable tool or describe a stub response as a completed download.
A missing tool does not justify changing the user's installation or configuration.

## Process

### Select and verify the report

Use the existing record ID, citekey, DOI, repository/provider IDs, title and
version. Search `fulltext_candidates` and user-supplied locators; prefer readable
structured full text when available, then a text-bearing PDF. A landing page may
need an actual Host browser/search to locate its public document URL. Select the
source for its identity, version and access, not a universal provider order.
Do not exclude otherwise eligible studies because they lack open full text.

Compare document title, authors/identifiers and version with the selected report;
a preprint, accepted manuscript and published report may differ. Keep conflicting
or unavailable identity evidence explicit. Candidate URLs, OA flags, abstracts
and search summaries are not evidence that the body was retrieved or read.

### Read the relevant body text

Call `qiongli_literature_read_fulltext` with `url` and the known `expected_doi`
when available. Start at `offset: 0`; use `limit` up to 50 segments. Continue with
the returned `next_offset` and `expected_sha256: source_sha256` so subsequent
passages refer to the same bytes; a nonzero offset requires that digest. The
session cache avoids repeat downloads;
`refresh: true` explicitly checks the remote source again. A digest mismatch
requires reconciling the changed version and affected claims before reuse.

`status: readable_text` means text was parsed. Inspect `identity_status`, warnings,
section/page anchors and the returned text before using it. `unverified` does not
establish identity. Follow pagination as needed for the question: methods and
limitations for design claims, results and relevant tables for findings. A first
page of segments does not establish that the whole article was read. Parsed
body text may omit figures, formulae, table structure or scanned pages; inspect
those with an available Host tool when the claim depends on them. Do not invent
OCR, missing values or page numbers.

For every used passage retain record ID/citekey, source URL, source digest,
version and its section/page/segment anchor. Record which passages were actually
read. A `tool_error` means this tool has no readable result; retain its specific
reason. Do not substitute an abstract and label it `full_text`. Continue useful
bounded work at the actual evidence limit or resolve another authorized source.

A `fulltext-url-blocked` error can concern the input URL, a redirect destination,
or a nonpublic DNS answer; read its message before assigning the failure stage.
The reader refuses signed or credential-bearing query parameters, including on
redirects from public publishers. Such a redirect does not establish a paywall
or an invalid original locator. Preserve the failure stage without copying
signed URLs or token values into notes; do not strip parameters or weaken the
guard to force access. Use another supported public representation when useful;
Host reading remains subject to the authorization and provenance limits below.
DNS failure, timeout, or an empty answer is a network failure, not proof that the
publisher denied access or that the paper has no body.

A parser timeout, worker failure or resource limit is a reading failure, not a
paywall or proof that the paper lacks a body. The native reader isolates parsing
with time and Rust-heap limits; it is not an OS memory/security sandbox. Keep the
failure reason and try a different authorized representation or available Host
reader when useful. Do not repeat an unchanged failing input or disable the
boundary to force a result.

A DNS result in a private or special-use range can block a public-looking URL
before HTTP executes. Record that as a transport/access limitation, not a paywall
or unavailable body. A proxy/DNS mapping is one possible cause, not a diagnosis
from the URL alone. Do not whitelist that address, replace DNS, enable a proxy or
weaken redirect checks to force the native read. When public access is otherwise
authorized, an available Host reader may inspect the publisher/repository page;
retain that Host provenance and any unknown byte hash. A permission or publisher
access denial still applies and must not be bypassed on another surface.

### Record access through the existing owners

Use the existing preview/approval/CAS path for project writes. Do not overwrite
search or bibliography artifacts. Minimum `retrieval_manifest.csv` schema:

```csv
record_id,citekey,doi,retrieval_status,version_label,source_provider,retrieved_at,fulltext_path,access_url,license,notes
```

Controlled `retrieval_status` values remain `retrieved_oa`, `retrieved_preprint`,
`abstract_only`, or `not_retrieved:` followed by `paywall`, `embargo`,
`broken_link`, `not_found`, `access_restricted`, `needs_provider`,
`missing_locator`, or `oa_candidate`. Use the closest supported status and put
specific parse/network/identity failures in `notes`; do not invent new enum values.
A readable source with verified report identity can support a retrieved status;
unknown identity stays a candidate pending verification. A session-cache read
leaves `fulltext_path` empty unless an actual authorized local file exists.

Controlled `version_label` values remain `published`, `accepted`, `submitted`,
`abstract_only`, `metadata_only`, or `unknown`. Record the actual source provider
and access URL; preserve supplied license evidence, leaving absent licenses unknown.
The retrieval timestamp and digest identify the observed bytes, not the paper's
publication date. Use `notes` and existing paper notes for format, digest, identity
checks, parser warnings, actual sections read and unresolved limits.

Mirror access status in `screening/full_text.md` without changing inclusion or
exclusion. Preserve the reason for every report not retrieved. Retrieval that
changes an eligibility basis returns to `paper-screener` for reconciliation.

## Output Contract

- `RetrievalManifest`: `RESEARCH/[topic]/retrieval_manifest.csv`.
- `FullTextStatus`: `RESEARCH/[topic]/screening/full_text.md`.
- Distinguish candidate located, source fetched, text parsed, and relevant
  passages read in the existing records; do not add a parallel state machine.
- Apply `references/evidence-ledger-contract.md` to claims drawn from those
  passages and `references/academic-output-rubric.md` to the requested prose.
  Preserve claim/decision IDs and source anchors; expose unsupported gaps.
- Retrieval success alone does not establish a study's validity, eligibility,
  complete table extraction or permission to redistribute its contents.

## Quality Bar

Every sought report has an access result and provenance. Every full-text claim
points to an actually inspected passage in the identified source revision.
Missing access, uncertain identity and parser limits remain visible, with a
concrete next supported action. Zotero writes and reference exports stay with
`reference-manager-bridge` and their existing authorization owners.
