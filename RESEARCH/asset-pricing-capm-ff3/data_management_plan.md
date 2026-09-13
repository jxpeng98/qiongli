# Data Management Plan

## 1. Data Inventory

| Dataset | Source | Contains PII? | Sensitivity | Access | Notes |
|---|---|---|---|---|---|
| Research factors ZIP | Kenneth French Data Library | No | low; publisher-controlled | public download | Raw archive ignored and not redistributed. |
| 25 portfolios ZIP | Kenneth French Data Library | No | low; publisher-controlled | public download | Raw archive ignored and not redistributed. |
| Derived model outputs | Local deterministic analysis | No | low | repository | Bounded aggregate/model tables only. |

## 2. Collection And Storage

- Collection method: HTTPS download by the analysis entrypoint or manual placement at the documented raw-data directory.
- Storage: `analysis/data/raw/`, excluded from Git.
- Access control: ordinary local workspace permissions; no secrets required.
- Encryption: transport relies on HTTPS; no confidential data at rest.
- Backups: source URLs and digests are committed; raw files can be reacquired only while the pinned bytes remain available.

## 3. De-identification

Not applicable. Inputs are aggregate portfolio and factor series with no personal identifiers.

## 4. Retention And Deletion

- Raw local archives are cacheable and may be deleted after reproduction.
- Derived results, provenance, code, and the dependency lock are retained in version control.
- Digest drift or corrupted raw files triggers deletion/re-download only after explicit source review.

## 5. Documentation

- Variable definitions: `design/variable_spec.md`.
- Provenance: `design/dataset_plan.md` and `analysis/provenance.json`.
- Versioning: Git for code/derived outputs; SHA-256 for source archives.

## 6. Sharing And Reproducibility

- Shared: code, lock, source URLs/digests, scholarly artifacts, and derived model summaries.
- Not shared: raw ZIP/CSV inputs, dependency caches, Host conversations, credentials, or private runtime state.
- Licensing: repository terms apply to original code; upstream data remain under their publisher's terms.
