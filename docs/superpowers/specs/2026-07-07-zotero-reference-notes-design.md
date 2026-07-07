# Zotero Reference Notes Design

## Goal

When Qiongli writes references to local Zotero, it should also be able to add
structured reading notes for those references. Notes should preserve reading
context without overwriting abstracts, metadata, or user-curated Zotero notes.

## Design

Use Zotero child notes attached to the reference item. Do not write reading notes
into `abstractNote` or `extra`:

- `abstractNote` remains the paper abstract.
- `extra` remains compact Qiongli provenance metadata.
- Child notes hold screening and reading context.

The MCPB accepts per-record note fields and maps them onto the Zotero item
payload as `qiongli_notes`. Supported inputs are:

- `reading_note`
- `reading_notes`
- `notes`
- structured `note` objects with fields such as `summary`, `key_findings`,
  `limitations`, `evidence_limit`, `review_status`, `screening_decision`, and
  `source_anchor`

For now, Qiongli writes only notes explicitly provided by the record payload.
It does not generate synthetic reading notes when no note is available; that
avoids polluting Zotero with low-value boilerplate.

## Companion Behavior

`POST /qiongli/upsertItems` should:

- Report planned note writes during dry runs without mutating Zotero.
- Create child notes for newly created items.
- Create child notes for updated or unchanged duplicate items when notes are
  present.
- Return per-record note write status and child note key when available.
- Keep collection behavior independent from note behavior.

The companion runtime gets one narrow API: `createChildNote(parentItemKey,
note)`. The Zotero bootstrap implementation owns the Zotero-specific details of
creating a `note` item, setting the parent item id, setting note content, and
saving it.

## Note Content

Notes are normalized to simple HTML:

- plain text becomes escaped paragraphs;
- structured objects become compact sections;
- arrays become bullet lists;
- the note starts with `Qiongli Reading Note` so generated notes can be
  recognized later.

This feature does not dedupe existing Zotero notes yet. It appends a new child
note when the caller provides one.

## Testing

- Companion unit tests cover dry-run planned notes and real child-note creation.
- Bootstrap VM tests cover creating a child `note` item attached to the parent.
- MCPB tests cover schema exposure and mapping `reading_note` into
  `qiongli_notes`.

