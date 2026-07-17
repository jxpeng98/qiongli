# Qiongli Config Global Settings Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the `CFG-201A` Rust-native global settings service with strict typed config, opaque secret references, redacted diagnostics, revision-safe atomic Unix persistence, and zero-side-effect Windows write refusal.

**Architecture:** Add one focused `qiongli-config` crate to the canonical native workspace. Keep path resolution, typed settings/codec, secret facade, redaction, and persistence in separate modules; expose one `GlobalSettingsStore` which owns `settings.json` replacement and never accepts arbitrary output paths or credential values. Use the standard-library file lock and filesystem APIs, Unix owner/mode checks, complete-document replacement with a synchronized recovery copy, and an internal fault-injection boundary.

**Tech Stack:** Rust 1.97 / edition 2024, `serde`, `serde_json`, `qiongli-content::ProfileId`, `rustix` process identity, `zeroize`, standard-library file locking, Cargo workspace tests, GitHub Actions native matrix.

---

## Approved Input

Implement the approved design in:

- `docs/superpowers/specs/2026-07-14-qiongli-config-global-settings-design.md`
- ADR 0204: `docs/architecture/decisions/0204-versioned-state-and-secret-storage.md`
- accelerated R1 roadmap: `docs/superpowers/roadmaps/2026-07-13-qiongli-2-accelerated-rust-migration-roadmap.md`

Do not edit ADR 0204 or frozen 1.x product/oracle paths. Work only on the
existing rolling branch `feat/2x-native-alpha1` and existing Draft PR #63.

## File Map

| File | Responsibility |
|---|---|
| `packages/qiongli-native/Cargo.toml` | Add the crate and shared pinned dependencies |
| `packages/qiongli-native/Cargo.lock` | Record the exact dependency graph |
| `packages/qiongli-native/crates/qiongli-config/Cargo.toml` | Declare the config crate |
| `packages/qiongli-native/crates/qiongli-config/src/lib.rs` | Re-export only supported service contracts |
| `packages/qiongli-native/crates/qiongli-config/src/error.rs` | Path-free typed reason/stage errors |
| `packages/qiongli-native/crates/qiongli-config/src/path.rs` | Pure v2 config-root resolution and symbolic diagnostics |
| `packages/qiongli-native/crates/qiongli-config/src/secret.rs` | Strict `SecretRef`, unavailable facade, protected secret container |
| `packages/qiongli-native/crates/qiongli-config/src/document.rs` | Public typed settings plus private strict JSON V1 adapter |
| `packages/qiongli-native/crates/qiongli-config/src/redaction.rs` | Provider readiness and redacted config status |
| `packages/qiongli-native/crates/qiongli-config/src/store.rs` | Bounded reads, secure path checks, lock/revision, commit/rollback |
| `packages/qiongli-native/crates/qiongli-config/tests/config_root.rs` | Cross-platform root contract |
| `packages/qiongli-native/crates/qiongli-config/tests/settings_contract.rs` | Secret, model, codec, and redaction contract |
| `packages/qiongli-native/crates/qiongli-config/tests/store_contract.rs` | Read, Unix persistence, conflict, permission, and Windows refusal contract |
| `docs/superpowers/roadmaps/2026-07-13-qiongli-2-accelerated-rust-migration-roadmap.md` | Add the completed checkpoint and exact evidence only after verification |

### Task 1: Scaffold The Crate And Resolve The V2 Root

**Files:**
- Modify: `packages/qiongli-native/Cargo.toml`
- Create: `packages/qiongli-native/crates/qiongli-config/Cargo.toml`
- Create: `packages/qiongli-native/crates/qiongli-config/src/lib.rs`
- Create: `packages/qiongli-native/crates/qiongli-config/src/error.rs`
- Create: `packages/qiongli-native/crates/qiongli-config/src/path.rs`
- Create: `packages/qiongli-native/crates/qiongli-config/tests/config_root.rs`
- Modify: `packages/qiongli-native/Cargo.lock`

- [ ] **Step 1: Add the workspace member, crate manifest, empty public surface, and failing root tests**

Add `crates/qiongli-config` to `workspace.members`. Add these shared dependencies:

```toml
rustix = { version = "1.1.3", features = ["process"] }
zeroize = "1.9.0"
```

Create the crate manifest:

```toml
[package]
name = "qiongli-config"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true
publish.workspace = true
description = "Versioned native Qiongli state and secret-reference boundary"

[dependencies]
qiongli-content = { path = "../qiongli-content" }
rustix.workspace = true
serde.workspace = true
serde_json.workspace = true
zeroize.workspace = true

[lints]
workspace = true
```

Create `src/lib.rs` with only crate documentation so this integration test fails
on unresolved imports:

```rust
//! Versioned native Qiongli configuration boundary.
```

Create `tests/config_root.rs` with concrete default, absolute, home-relative,
empty, ordinary-relative, traversal, and redacted-debug assertions:

```rust
use std::ffi::OsStr;
use std::path::Path;

use qiongli_config::{ConfigError, ConfigRootSource, resolve_config_root};

#[test]
#[cfg(unix)]
fn default_root_appends_the_v2_namespace() {
    let root = resolve_config_root(None, Path::new("/users/researcher")).unwrap();
    assert_eq!(root.source(), ConfigRootSource::Default);
    assert_eq!(root.compatibility_root(), Path::new("/users/researcher/.config/qiongli"));
    assert_eq!(root.state_root(), Path::new("/users/researcher/.config/qiongli/v2"));
    assert_eq!(root.symbolic_state_root(), "<user-home>/.config/qiongli/v2");
}

#[test]
#[cfg(unix)]
fn absolute_and_home_relative_overrides_append_exactly_one_namespace() {
    let absolute = resolve_config_root(Some(OsStr::new("/srv/qiongli")), Path::new("/home/u")).unwrap();
    assert_eq!(absolute.state_root(), Path::new("/srv/qiongli/v2"));
    let home = resolve_config_root(Some(OsStr::new("~/state")), Path::new("/home/u")).unwrap();
    assert_eq!(home.state_root(), Path::new("/home/u/state/v2"));
    let already_named = resolve_config_root(Some(OsStr::new("/srv/v2")), Path::new("/home/u")).unwrap();
    assert_eq!(already_named.state_root(), Path::new("/srv/v2/v2"));
}

#[test]
#[cfg(unix)]
fn unsafe_or_ambiguous_roots_fail_closed() {
    for value in ["", "relative", "~/../escape", "/tmp/../escape"] {
        assert_eq!(
            resolve_config_root(Some(OsStr::new(value)), Path::new("/home/u")),
            Err(ConfigError::InvalidConfigHome),
        );
    }
    assert_eq!(
        resolve_config_root(None, Path::new("relative-home")),
        Err(ConfigError::HomeUnavailable),
    );
}

#[test]
#[cfg(unix)]
fn debug_output_never_contains_the_concrete_path() {
    let root = resolve_config_root(Some(OsStr::new("/private/canary-user/qiongli")), Path::new("/home/u")).unwrap();
    let debug = format!("{root:?}");
    assert!(!debug.contains("canary-user"));
    assert!(debug.contains("<configured-root>/v2"));
}
```

- [ ] **Step 2: Run the root test and verify the intended compile failure**

Run:

```bash
cargo test --manifest-path packages/qiongli-native/Cargo.toml -p qiongli-config --test config_root
```

Expected: FAIL with unresolved `qiongli_config` imports for
`resolve_config_root`, `ConfigError`, and `ConfigRootSource`.

- [ ] **Step 3: Implement sanitized errors and the pure resolver**

Define `ConfigError` without a path or raw `io::Error` payload:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PersistenceStage {
    Inspect,
    CreateStore,
    AcquireLock,
    ReadCurrent,
    WriteStaging,
    SyncStaging,
    CreateRecovery,
    Activate,
    SyncDirectory,
    Rollback,
    Cleanup,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConfigError {
    InvalidConfigHome,
    HomeUnavailable,
    InvalidDocumentKind,
    UnsupportedSchema { observed: Option<u64> },
    InvalidDocument,
    DocumentTooLarge,
    UnsafeManagedPath,
    InsecurePermissions,
    LockBusy,
    RevisionConflict { observed: u64 },
    RevisionExhausted,
    PersistenceFailed { stage: PersistenceStage, kind: std::io::ErrorKind },
    RecoveryRequired,
    UnsupportedPlatformSecurity,
}
```

Give every variant a stable `reason_code()` and implement `Display` using only
that code, stage, safe `ErrorKind`, and observed revision.

Define the root API exactly as:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConfigRootSource { Default, Override }

#[derive(Clone, Eq, PartialEq)]
pub struct ConfigRoot {
    compatibility_root: PathBuf,
    state_root: PathBuf,
    source: ConfigRootSource,
}

pub fn resolve_config_root(
    configured: Option<&OsStr>,
    platform_home: &Path,
) -> Result<ConfigRoot, ConfigError>;
```

Reject raw `.` and `..` path segments before `Path::components` can normalize
them. Accept non-UTF-8 only for already-absolute paths on Unix. Implement a
custom `Debug` using `symbolic_state_root()` and re-export the supported types
from `lib.rs`.

- [ ] **Step 4: Run and pass the focused root contract**

Run the command from Step 2.

Expected: 4 tests PASS on Unix; platform-gated Windows separator and Unix
non-UTF-8 cases also pass when compiled on their target.

- [ ] **Step 5: Format, inspect the diff, and commit the root boundary**

Run:

```bash
cargo fmt --manifest-path packages/qiongli-native/Cargo.toml --all
cargo test --manifest-path packages/qiongli-native/Cargo.toml -p qiongli-config --test config_root
git diff --check
```

Commit only the Task 1 files:

```bash
git add packages/qiongli-native/Cargo.toml packages/qiongli-native/Cargo.lock packages/qiongli-native/crates/qiongli-config
git commit -m "feat(config): resolve versioned global state root"
```

### Task 2: Add Opaque Secret And Typed Settings Contracts

**Files:**
- Create: `packages/qiongli-native/crates/qiongli-config/src/secret.rs`
- Create: `packages/qiongli-native/crates/qiongli-config/src/document.rs`
- Create: `packages/qiongli-native/crates/qiongli-config/tests/settings_contract.rs`
- Modify: `packages/qiongli-native/crates/qiongli-config/src/lib.rs`

- [ ] **Step 1: Write failing tests for secret references, unavailable storage, defaults, private email, and provider readiness**

The test must assert these exact behaviors:

```rust
use qiongli_config::{
    EmailAddress, GlobalSettings, ProviderReadiness, SecretRef, SecretStore,
    SecretStoreError, SecretStoreStatus, UnavailableSecretStore,
};
use qiongli_content::ProfileId;

#[test]
fn secret_reference_is_strict_and_redacted() {
    let reference = SecretRef::parse("qsr1_0123456789abcdef0123456789abcdef").unwrap();
    assert_eq!(format!("{reference:?}"), "<redacted-secret-ref>");
    assert_eq!(format!("{reference}"), "<redacted-secret-ref>");
    for invalid in [
        "secret-key",
        "qsr1_0123456789ABCDEF0123456789ABCDEF",
        "qsr1_0123",
        " qsr1_0123456789abcdef0123456789abcdef",
    ] {
        assert!(SecretRef::parse(invalid).is_err());
    }
}

#[test]
fn unavailable_secret_store_has_no_fallback() {
    let store = UnavailableSecretStore;
    let reference = SecretRef::parse("qsr1_0123456789abcdef0123456789abcdef").unwrap();
    assert_eq!(store.status(), SecretStoreStatus::Unavailable);
    assert!(matches!(store.resolve(&reference), Err(SecretStoreError::Unavailable)));
}

#[test]
fn defaults_enable_only_arxiv_and_select_marketplace_lite() {
    let settings = GlobalSettings::default();
    assert_eq!(settings.default_profile, ProfileId::MarketplaceLite);
    assert!(!settings.providers.openalex.enabled);
    assert!(!settings.providers.semantic_scholar.enabled);
    assert!(!settings.providers.crossref.enabled);
    assert!(!settings.providers.pubmed.enabled);
    assert!(settings.providers.arxiv.enabled);
}

#[test]
fn private_email_is_normalized_but_never_debugged() {
    let email = EmailAddress::parse("  researcher@example.org  ").unwrap();
    assert_eq!(email.as_str(), "researcher@example.org");
    assert_eq!(format!("{email:?}"), "<redacted-email>");
    assert!(EmailAddress::parse("").is_err());
    assert!(EmailAddress::parse("bad\nmail@example.org").is_err());
}

#[test]
fn enabled_provider_readiness_is_typed() {
    let mut settings = GlobalSettings::default();
    settings.providers.openalex.enabled = true;
    assert_eq!(settings.providers.openalex.readiness(), ProviderReadiness::NeedsSecret);
    settings.providers.openalex.api_key_ref = Some(
        SecretRef::parse("qsr1_0123456789abcdef0123456789abcdef").unwrap(),
    );
    assert_eq!(settings.providers.openalex.readiness(), ProviderReadiness::Ready);
    settings.providers.crossref.enabled = true;
    assert_eq!(settings.providers.crossref.readiness(), ProviderReadiness::NeedsPublicSetting);
}
```

- [ ] **Step 2: Run the settings test and verify it fails on missing types**

Run:

```bash
cargo test --manifest-path packages/qiongli-native/Cargo.toml -p qiongli-config --test settings_contract
```

Expected: FAIL with unresolved secret and settings types.

- [ ] **Step 3: Implement the secret facade**

Implement:

```rust
pub const MAX_SECRET_VALUE_BYTES: usize = 16 * 1024;

#[derive(Clone, Eq, PartialEq)]
pub struct SecretRef(String);

pub enum SecretStoreStatus { Unavailable }
pub enum SecretStoreError { Unavailable }

pub struct SecretValue {
    bytes: zeroize::Zeroizing<Vec<u8>>,
}

pub trait SecretStore: Send + Sync {
    fn status(&self) -> SecretStoreStatus;
    fn resolve(&self, secret_ref: &SecretRef) -> Result<SecretValue, SecretStoreError>;
}

pub struct UnavailableSecretStore;
```

`SecretRef::parse` accepts exactly `qsr1_` plus 32 lower-hex characters.
Expose its raw value only as `pub(crate) fn as_raw(&self) -> &str`. Implement
redacted `Debug` and `Display`. Keep `SecretValue` construction crate-private;
its public byte accessor is allowed for a future bounded provider operation,
but this slice constructs no value.

- [ ] **Step 4: Implement the typed settings model**

Create public non-Serde types for `EmailAddress`, `GlobalSettings`,
`ProviderSettings`, `OpenAlexSettings`, `SemanticScholarSettings`,
`CrossrefSettings`, `PubmedSettings`, and `ArxivSettings`. Use public typed
fields so later service intents can build a complete replacement without a
JSON patch API. Implement:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProviderReadiness {
    Disabled,
    Ready,
    NeedsSecret,
    NeedsPublicSetting,
}
```

Default to `ProfileId::MarketplaceLite`, disabled provider settings with null
private values, and enabled arXiv. `EmailAddress::parse` trims, enforces 1..=320
Unicode scalar values, rejects controls, and uses redacted `Debug`.

- [ ] **Step 5: Run focused tests, format, and commit the typed boundary**

Run:

```bash
cargo fmt --manifest-path packages/qiongli-native/Cargo.toml --all
cargo test --manifest-path packages/qiongli-native/Cargo.toml -p qiongli-config --test settings_contract
git diff --check
```

Expected: all tests in `settings_contract` PASS.

Commit:

```bash
git add packages/qiongli-native/crates/qiongli-config
git commit -m "feat(config): define typed settings and secret refs"
```

### Task 3: Implement The Strict Global Settings V1 Codec

**Files:**
- Modify: `packages/qiongli-native/crates/qiongli-config/src/document.rs`
- Modify: `packages/qiongli-native/crates/qiongli-config/src/lib.rs`

- [ ] **Step 1: Add failing exact-shape, round-trip, bounds, future-schema, duplicate-key, and unknown-field tests**

Add unit tests inside `document.rs` which call the crate-private
`encode_global_settings` and `decode_global_settings` adapters and assert:

```rust
#[test]
fn document_round_trip_has_the_exact_v1_envelope() {
    let bytes = encode_global_settings(&GlobalSettings::default(), 1).unwrap();
    let text = std::str::from_utf8(&bytes).unwrap();
    assert!(text.ends_with("\n"));
    assert!(text.contains("\"document_kind\": \"qiongli-global-settings\""));
    assert!(text.contains("\"schema_version\": 1"));
    assert!(text.contains("\"revision\": 1"));
    assert_eq!(decode_global_settings(&bytes).unwrap().revision, 1);
}

#[test]
fn ambiguous_or_future_documents_fail_closed() {
    let duplicate = br#"{"document_kind":"qiongli-global-settings","schema_version":1,"revision":1,"revision":2}"#;
    assert_eq!(decode_global_settings(duplicate), Err(ConfigError::InvalidDocument));
    let future = br#"{"document_kind":"qiongli-global-settings","schema_version":2}"#;
    assert_eq!(
        decode_global_settings(future),
        Err(ConfigError::UnsupportedSchema { observed: Some(2) }),
    );
    let unknown = br#"{"document_kind":"qiongli-global-settings","schema_version":1,"revision":1,"unknown":true}"#;
    assert_eq!(decode_global_settings(unknown), Err(ConfigError::InvalidDocument));
}

#[test]
fn document_size_and_revision_are_bounded() {
    assert_eq!(
        decode_global_settings(&vec![b' '; MAX_GLOBAL_SETTINGS_BYTES + 1]),
        Err(ConfigError::DocumentTooLarge),
    );
    assert_eq!(
        encode_global_settings(&GlobalSettings::default(), 0),
        Err(ConfigError::InvalidDocument),
    );
    assert_eq!(
        encode_global_settings(&GlobalSettings::default(), MAX_GLOBAL_SETTINGS_REVISION + 1),
        Err(ConfigError::RevisionExhausted),
    );
}
```

Also persist a valid `api_key_ref` and email, assert the private decoder restores
them, and assert a credential canary not assigned to a typed field never
appears in bytes or debug output.

- [ ] **Step 2: Run the codec tests and verify unresolved functions fail**

Run the Task 2 focused test command.

Expected: FAIL for missing codec functions and constants.

- [ ] **Step 3: Implement duplicate-safe parsing and the private Serde adapter**

Export:

```rust
pub const GLOBAL_SETTINGS_DOCUMENT_KIND: &str = "qiongli-global-settings";
pub const GLOBAL_SETTINGS_SCHEMA_VERSION: u64 = 1;
pub const MAX_GLOBAL_SETTINGS_BYTES: usize = 64 * 1024;
pub const MAX_GLOBAL_SETTINGS_REVISION: u64 = 9_007_199_254_740_991;

#[derive(Clone, Eq, PartialEq)]
pub struct LoadedGlobalSettings {
    pub revision: u64,
    pub settings: GlobalSettings,
}

pub(crate) fn encode_global_settings(
    settings: &GlobalSettings,
    revision: u64,
) -> Result<Vec<u8>, ConfigError>;

pub(crate) fn decode_global_settings(bytes: &[u8]) -> Result<LoadedGlobalSettings, ConfigError>;
```

Keep the serializable `GlobalSettingsDocumentV1` and five provider document
structs private with `#[serde(deny_unknown_fields)]`. Parse first through a
recursive custom Serde visitor that inserts object keys into a set and returns
an error on any duplicate at any nesting depth. Inspect kind and schema before
deserializing the full object so wrong kind and future schema retain their
specific error codes. Require every provider and every shown field. Serialize
pretty JSON plus exactly one trailing newline, decode the result again, and
require equality before returning bytes.

- [ ] **Step 4: Run and pass all model/codec tests**

Run:

```bash
cargo test --manifest-path packages/qiongli-native/Cargo.toml -p qiongli-config --test settings_contract
cargo test --manifest-path packages/qiongli-native/Cargo.toml -p qiongli-config --lib
```

Expected: all tests PASS, including nested duplicate keys and all unknown-field
fixtures.

- [ ] **Step 5: Format and commit the codec**

```bash
cargo fmt --manifest-path packages/qiongli-native/Cargo.toml --all
git diff --check
git add packages/qiongli-native/crates/qiongli-config
git commit -m "feat(config): add strict global settings codec"
```

### Task 4: Add Bounded Read And Redacted Status

**Files:**
- Create: `packages/qiongli-native/crates/qiongli-config/src/redaction.rs`
- Create: `packages/qiongli-native/crates/qiongli-config/src/store.rs`
- Modify: `packages/qiongli-native/crates/qiongli-config/src/lib.rs`
- Create: `packages/qiongli-native/crates/qiongli-config/tests/store_contract.rs`

- [ ] **Step 1: Write failing tests for missing state, bounded reads, unsafe paths, and diagnostic canaries**

Use a unique owner-only test root under
`packages/qiongli-native/target/qiongli-config-tests/`. Assert:

```rust
#[test]
fn missing_state_returns_revision_zero_without_writing() {
    let fixture = Fixture::new("missing");
    let store = fixture.store();
    let loaded = store.load().unwrap();
    assert_eq!(loaded.revision, 0);
    assert_eq!(loaded.settings, GlobalSettings::default());
    assert!(!fixture.compatibility_root().exists());
}

#[test]
fn status_redacts_paths_emails_and_secret_refs() {
    let fixture = Fixture::new("redaction");
    fixture.write_valid_document_with_private_canaries();
    let status = fixture.store().status();
    let debug = format!("{status:?}");
    let json = serde_json::to_string(&status).unwrap();
    for canary in ["private-user", "researcher@example.org", "qsr1_0123456789abcdef0123456789abcdef"] {
        assert!(!debug.contains(canary));
        assert!(!json.contains(canary));
    }
}
```

Add tests for oversize input, wrong kind, future schema, malformed JSON,
symlinked `v2`, symlinked settings, non-regular settings, and Unix mode `0644`.
Every invalid live file must remain byte-identical after `load` and `status`.

- [ ] **Step 2: Run the store test and verify missing store/status types fail**

```bash
cargo test --manifest-path packages/qiongli-native/Cargo.toml -p qiongli-config --test store_contract
```

Expected: FAIL for missing `GlobalSettingsStore` and redacted status contracts.

- [ ] **Step 3: Implement no-follow managed-path checks and bounded read**

Define:

```rust
pub struct GlobalSettingsStore {
    root: ConfigRoot,
    lock_timeout: Duration,
}

impl GlobalSettingsStore {
    pub fn new(root: ConfigRoot) -> Self;
    pub fn load(&self) -> Result<LoadedGlobalSettings, ConfigError>;
    pub fn status(&self) -> RedactedConfigStatus;
}
```

`load` must not create anything. Treat a missing `v2` directory or missing
`settings.json` as the virtual default. Walk every existing selected-root
component with `symlink_metadata`; reject symlinks and Windows reparse points.
Require `v2` to be a directory and settings to be one regular file. On Unix,
require `v2` and settings to match `rustix::process::geteuid()` and have no
group/other bits. Read at most 64 KiB plus one sentinel byte and pass bytes to
the strict codec. Convert every I/O error to a safe stage and `ErrorKind`.

- [ ] **Step 4: Implement serializable redacted status**

Expose only these safe concepts:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConfigState {
    Missing,
    Ready,
    Invalid,
    FutureSchema,
    Insecure,
    RecoveryRequired,
    WriteUnsupported,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct RedactedProviderStatus {
    pub enabled: bool,
    pub readiness: ProviderReadiness,
    pub secret_ref_present: bool,
}
```

`RedactedConfigStatus` contains symbolic root, state, optional revision/profile,
five redacted provider statuses, `secure-store-not-implemented`, and a boolean
cleanup marker. It contains no raw `PathBuf`, email, `SecretRef`, artifact name,
or raw I/O message. Detect cleanup state only by scanning strict Qiongli-owned
recovery-name prefixes; never return the matching entry name. On Windows, a
valid or missing readable document reports `WriteUnsupported` until
`CFG-201B`, while retaining safe revision/profile/provider fields when present.

- [ ] **Step 5: Run focused read/status tests and commit**

```bash
cargo fmt --manifest-path packages/qiongli-native/Cargo.toml --all
cargo test --manifest-path packages/qiongli-native/Cargo.toml -p qiongli-config --test store_contract
cargo test --manifest-path packages/qiongli-native/Cargo.toml -p qiongli-config
git diff --check
git add packages/qiongli-native/crates/qiongli-config
git commit -m "feat(config): load and redact global settings"
```

Expected: all config tests PASS and canary values are absent from all diagnostic
representations.

### Task 5: Implement Revision-Safe Atomic Unix Replacement

**Files:**
- Modify: `packages/qiongli-native/crates/qiongli-config/src/store.rs`
- Modify: `packages/qiongli-native/crates/qiongli-config/tests/store_contract.rs`

- [ ] **Step 1: Add failing create, update, conflict, mode, lock, and concurrency tests**

On Unix assert:

```rust
#[test]
fn unix_replace_commits_owner_only_monotonic_documents() {
    let fixture = Fixture::new("replace");
    let store = fixture.store();
    let first = store.replace(0, GlobalSettings::default()).unwrap();
    assert_eq!(first.revision, 1);
    assert_eq!(mode(fixture.state_root()), 0o700);
    assert_eq!(mode(fixture.settings_path()), 0o600);
    let mut next = GlobalSettings::default();
    next.default_profile = ProfileId::Full;
    let second = store.replace(1, next.clone()).unwrap();
    assert_eq!(second.revision, 2);
    assert_eq!(store.load().unwrap().settings, next);
}

#[test]
fn stale_revision_never_changes_live_bytes() {
    let fixture = Fixture::new("conflict");
    let store = fixture.store();
    store.replace(0, GlobalSettings::default()).unwrap();
    let before = std::fs::read(fixture.settings_path()).unwrap();
    assert_eq!(
        store.replace(0, GlobalSettings::default()),
        Err(ConfigError::RevisionConflict { observed: 1 }),
    );
    assert_eq!(std::fs::read(fixture.settings_path()).unwrap(), before);
}
```

Add one held-lock timeout test using a short test policy, one two-thread test
where exactly one replacement wins, and one rejection test each for insecure
existing `v2`, settings, and lock modes.

- [ ] **Step 2: Run the Unix persistence tests and verify `replace` is missing**

Run:

```bash
cargo test --manifest-path packages/qiongli-native/Cargo.toml -p qiongli-config --test store_contract unix_
```

Expected: FAIL because `replace` and `CommitOutcome` do not exist.

- [ ] **Step 3: Add bounded lock acquisition and expected-revision handling**

Expose:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CommitOutcome {
    pub revision: u64,
    pub cleanup_required: bool,
}

impl GlobalSettingsStore {
    pub fn replace(
        &self,
        expected_revision: u64,
        replacement: GlobalSettings,
    ) -> Result<CommitOutcome, ConfigError>;
}
```

On Unix, create the compatibility parent without following an existing link,
create `v2` with `DirBuilderExt::mode(0o700)`, verify owner/mode, and create or
open `.settings.lock` as `0600`. Acquire `File::try_lock()` in a bounded loop;
sleep only until the configured deadline. Once locked, re-read the current
document, compare revision, and use checked increment bounded by
`MAX_GLOBAL_SETTINGS_REVISION`.

- [ ] **Step 4: Add synchronized staging, recovery, activation, and cleanup**

Generate transaction-unique names inside `v2` using process ID, system time,
and an atomic counter. Create all transaction files with `create_new(true)` and
Unix mode `0600`.

The exact normal sequence is:

```text
encode next complete document
write staging bytes
flush and sync staging
if live state exists, write identical synchronized recovery bytes
sync v2 directory
rename staging over settings.json
sync v2 directory
verify committed revision and bytes
remove recovery
sync v2 directory
return CommitOutcome
```

Before activation, any error removes only the transaction's staging/recovery
files and leaves live bytes unchanged. After activation, any error before the
durable commit point restores recovery over `settings.json`, or removes the new
file for a first-write rollback, synchronizes the directory, and verifies the
old bytes/revision. Cleanup failure after the durable commit returns the new
revision with `cleanup_required: true` and retains only an owner-only recovery
artifact.

- [ ] **Step 5: Run and pass the persistence and concurrency tests**

```bash
cargo fmt --manifest-path packages/qiongli-native/Cargo.toml --all
cargo test --manifest-path packages/qiongli-native/Cargo.toml -p qiongli-config --test store_contract -- --test-threads=1
```

Expected: initial write, replacement, conflict, lock timeout, concurrency,
permissions, and byte-equivalence tests PASS.

- [ ] **Step 6: Commit the Unix transaction boundary**

```bash
git diff --check
git add packages/qiongli-native/crates/qiongli-config
git commit -m "feat(config): persist settings atomically on unix"
```

### Task 6: Prove Failure Recovery And Windows Fail-Closed Behavior

**Files:**
- Modify: `packages/qiongli-native/crates/qiongli-config/src/store.rs`
- Modify: `packages/qiongli-native/crates/qiongli-config/tests/store_contract.rs`

- [ ] **Step 1: Add an internal deterministic persistence fault boundary and failing unit tests**

Define internal stages:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FaultPoint {
    AfterLock,
    AfterCurrentRead,
    AfterStagingSync,
    AfterRecoverySync,
    AfterActivation,
    BeforeCommitDirectorySync,
    DuringCleanup,
    DuringRollback,
}
```

Production uses a no-op implementation. The test implementation owns a set of
fault points so it can inject either one stage or the paired rollback case.
Unit tests construct a store with one fault point and assert every single fault
before/during commit restores the exact prior bytes and revision. A paired
`AfterActivation + DuringRollback` test must return
`ConfigError::RecoveryRequired`, and a `DuringCleanup` test must return the
committed revision with `cleanup_required: true`.

- [ ] **Step 2: Run the fault tests and verify at least the first injected stage fails**

```bash
cargo test --manifest-path packages/qiongli-native/Cargo.toml -p qiongli-config store::tests::fault_ -- --nocapture
```

Expected before the hook is connected: FAIL because the injected stage does not
change control flow.

- [ ] **Step 3: Route every persistence stage through the hook and pass recovery tests**

Call the hook only at the named deterministic boundaries. Map injected failures
to `PersistenceFailed` with a safe stage and `ErrorKind::Other`. Preserve the
normal commit point: a cleanup-only fault is a successful commit with a cleanup
flag; a rollback fault is never reported as ordinary success or ordinary
failure.

- [ ] **Step 4: Add and pass Windows no-mutation replacement coverage**

Under `#[cfg(windows)]`, `replace` must immediately return:

```rust
Err(ConfigError::UnsupportedPlatformSecurity)
```

It must do so before calling `exists`, `create_dir_all`, opening a lock, or
reading live state. The Windows test snapshots the parent entry list before and
after replacement and requires exact equality. Keep root/model/codec/bounded
read tests enabled on Windows.

- [ ] **Step 5: Run the complete crate gate and commit failure semantics**

```bash
cargo fmt --manifest-path packages/qiongli-native/Cargo.toml --all
cargo check --manifest-path packages/qiongli-native/Cargo.toml -p qiongli-config --all-targets --all-features
cargo clippy --manifest-path packages/qiongli-native/Cargo.toml -p qiongli-config --all-targets --all-features -- -D warnings
cargo test --manifest-path packages/qiongli-native/Cargo.toml -p qiongli-config --all-targets --all-features
git diff --check
git add packages/qiongli-native/crates/qiongli-config
git commit -m "test(config): prove rollback and fail-closed writes"
```

Expected: all config tests PASS and Clippy emits no warning.

### Task 7: Run The Native Gate, Record Truthful Evidence, And Update The Rolling PR

**Files:**
- Modify: `docs/superpowers/roadmaps/2026-07-13-qiongli-2-accelerated-rust-migration-roadmap.md`
- Modify: Draft PR #63 body after the evidence exists

- [ ] **Step 1: Run the exact local native gate**

Run from the repository root:

```bash
./scripts/check_2x_native_change_boundary.sh --base-ref origin/2.x
cargo fmt --manifest-path packages/qiongli-native/Cargo.toml --all -- --check
cargo check --manifest-path packages/qiongli-native/Cargo.toml --workspace --all-targets --all-features --locked
cargo clippy --manifest-path packages/qiongli-native/Cargo.toml --workspace --all-targets --all-features --locked -- -D warnings
cargo test --manifest-path packages/qiongli-native/Cargo.toml --workspace --all-targets --all-features --locked
```

Expected: boundary PASS; format/check/Clippy PASS; all native tests PASS.

- [ ] **Step 2: Perform a private-data and dependency audit**

Run:

```bash
rg -n "researcher@example\.org|qsr1_0123456789abcdef0123456789abcdef|canary-user" packages/qiongli-native/crates/qiongli-config --glob '!tests/**'
rg -n "Command::new|std::process|python|node|plaintext|api_key\s*:" packages/qiongli-native/crates/qiongli-config/src
```

Expected: no production private canary, process launch, Python/Node invocation,
plaintext fallback, or raw API-key field. Legitimate documentation phrases are
reviewed manually and do not represent executable fallbacks.

- [ ] **Step 3: Update the roadmap receipt with only observed facts**

Add a short `CFG-201A` checkpoint under R1 containing:

- the implementation commit range;
- local test count and exact commands;
- macOS/Linux atomic owner-only write support;
- Windows read/validation plus explicit unsupported write status;
- unavailable secret-store facade and absence of a real keychain backend; and
- the next batch `CFG-201B`, without claiming CLI/UI/MCP wiring.

Do not report GitHub results until the exact implementation head completes.

- [ ] **Step 4: Commit the roadmap receipt and push the same rolling branch**

```bash
git add docs/superpowers/roadmaps/2026-07-13-qiongli-2-accelerated-rust-migration-roadmap.md
git commit -m "docs(roadmap): record global config checkpoint"
git push origin feat/2x-native-alpha1
```

- [ ] **Step 5: Wait for and verify exact-head GitHub Actions**

Use the run whose `headSha` equals `git rev-parse HEAD`:

```bash
head_sha=$(git rev-parse HEAD)
run_id=$(gh run list --branch feat/2x-native-alpha1 --workflow "Native CI" --limit 10 --json databaseId,headSha --jq ".[] | select(.headSha == \"$head_sha\") | .databaseId" | head -1)
gh run view "$run_id" --json headSha,status,conclusion,jobs,url
```

Expected: `Native 2.x change boundary` and Linux/macOS/Windows Rust jobs all
conclude `success` on the exact head. If any job fails, inspect its log, fix the
owned defect on the same branch, rerun the full local gate, commit, push, and
verify the new exact head.

- [ ] **Step 6: Update Draft PR #63 ledger without unsupported claims**

Record the checkpoint commits, exact run URL and timings, current supported
capabilities, next `CFG-201B` batch, and these explicit nonclaims:

- no Windows config persistence yet;
- no real credential backend or vault;
- no config CLI/UI/MCP integration;
- no project-state or 1.x migration behavior.

Keep the PR Draft. Do not create another PR or branch.

## Completion Definition

The plan is complete when every checkbox above is satisfied, the working tree
is clean, local and remote rolling-branch heads match, Draft PR #63 describes
the actual implementation, and exact-head GitHub native checks are green on
Linux, macOS, and Windows.
