#[cfg(unix)]
use std::fs::OpenOptions;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use qiongli_config::{
    UpdateActiveTransaction, UpdateState, UpdateStateStore, UpdateStreamPreference,
    UpdateTransactionPhase,
};
use qiongli_platform::{
    Architecture, NativeReleaseAuthority, NativeUpdateDisposition, NativeUpdateStream,
    NativeUpdateVerificationContext, OperatingSystem, SignedNativeUpdateManifestV1,
    VerifiedNativeUpdateManifest,
};
use reqwest::StatusCode;
use reqwest::blocking::Client;
use reqwest::header::{ACCEPT, ACCEPT_ENCODING, CONTENT_ENCODING};
use reqwest::redirect::Policy;
use serde::Serialize;
use sha2::{Digest, Sha256};

const STABLE_MANIFEST_ENDPOINT: &str = "https://qiongli.dev/updates/v2/stable/macos-aarch64.json";
const BETA_MANIFEST_ENDPOINT: &str = "https://qiongli.dev/updates/v2/beta/macos-aarch64.json";
const ARCHIVE_HOSTS: &[&str] = &[
    "github.com",
    "objects.githubusercontent.com",
    "release-assets.githubusercontent.com",
];
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const ARCHIVE_REQUEST_TIMEOUT: Duration = Duration::from_secs(10 * 60);
const MAX_ARCHIVE_REDIRECTS: usize = 3;
const ARCHIVE_BUFFER_BYTES: usize = 64 * 1024;
const STAGED_MANIFEST_FILE: &str = "update-manifest.json";
const PARTIAL_ARCHIVE_FILE: &str = ".archive.partial";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UpdateCliCommand {
    Status,
    Channel {
        expected_revision: u64,
        stream: UpdateStreamPreference,
    },
    Check,
    Download {
        expected_revision: u64,
    },
    Cancel {
        expected_revision: u64,
    },
}

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(untagged)]
pub(crate) enum UpdateCliOutput {
    Status(UpdateStatusOutput),
    Channel(UpdateChannelOutput),
    Check(UpdateCheckOutput),
    Download(UpdateDownloadOutput),
    Cancel(UpdateCancelOutput),
}

#[derive(Debug, Eq, PartialEq, Serialize)]
pub(crate) struct UpdateStatusOutput {
    schema_version: u32,
    command: &'static str,
    product_version: &'static str,
    revision: u64,
    selected_stream: UpdateStreamPreference,
    last_accepted_generation: u64,
    active_transaction: &'static str,
    release_authority: &'static str,
    macos_team_id: &'static str,
    manifest_source: &'static str,
    download: &'static str,
    install: &'static str,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
pub(crate) struct UpdateChannelOutput {
    schema_version: u32,
    command: &'static str,
    revision: u64,
    selected_stream: UpdateStreamPreference,
    cleanup_required: bool,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
pub(crate) struct UpdateCheckOutput {
    schema_version: u32,
    command: &'static str,
    status: &'static str,
    selected_stream: UpdateStreamPreference,
    current_version: &'static str,
    target_version: String,
    target_channel: qiongli_platform::ReleaseChannel,
    generation: u64,
    archive_size_bytes: u64,
    archive_sha256: String,
    resource_pack_sha256: String,
    signed_payload_sha256: String,
    release_key_id: String,
    download: &'static str,
    install: &'static str,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
pub(crate) struct UpdateDownloadOutput {
    schema_version: u32,
    command: &'static str,
    revision: u64,
    transaction_id: String,
    target_version: String,
    generation: u64,
    archive_file_name: String,
    archive_size_bytes: u64,
    archive_sha256: String,
    staging: &'static str,
    install: &'static str,
    cleanup_required: bool,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
pub(crate) struct UpdateCancelOutput {
    schema_version: u32,
    command: &'static str,
    revision: u64,
    transaction_id: String,
    staging: &'static str,
    cleanup_required: bool,
}

pub(crate) fn execute(
    command: UpdateCliCommand,
    store: &UpdateStateStore,
    authority: Option<&NativeReleaseAuthority>,
    expected_macos_team_id: Option<&str>,
) -> Result<UpdateCliOutput, &'static str> {
    let now_unix = if matches!(
        command,
        UpdateCliCommand::Check | UpdateCliCommand::Download { .. }
    ) {
        now_unix()?
    } else {
        0
    };
    let runtime = UpdateRuntimeContext {
        os: OperatingSystem::current(),
        arch: Architecture::current(),
        current_version: env!("CARGO_PKG_VERSION"),
        now_unix,
        expected_macos_team_id,
    };
    execute_with_fetchers(
        command,
        store,
        authority,
        &runtime,
        &ReqwestManifestFetcher,
        &ReqwestArchiveFetcher,
    )
}

fn execute_with_fetchers(
    command: UpdateCliCommand,
    store: &UpdateStateStore,
    authority: Option<&NativeReleaseAuthority>,
    runtime: &UpdateRuntimeContext<'_>,
    manifest_fetcher: &impl ManifestFetcher,
    archive_fetcher: &impl ArchiveFetcher,
) -> Result<UpdateCliOutput, &'static str> {
    let loaded = store.load().map_err(|error| error.reason_code())?;
    match command {
        UpdateCliCommand::Status => Ok(UpdateCliOutput::Status(UpdateStatusOutput {
            schema_version: 1,
            command: "update-status",
            product_version: runtime.current_version,
            revision: loaded.revision,
            selected_stream: loaded.state.selected_stream,
            last_accepted_generation: loaded.state.last_accepted_generation,
            active_transaction: if loaded.state.active_transaction.is_some() {
                "present"
            } else {
                "none"
            },
            release_authority: if authority.is_some() {
                "embedded"
            } else {
                "unavailable"
            },
            macos_team_id: if runtime.expected_macos_team_id.is_some() {
                "embedded"
            } else {
                "unavailable"
            },
            manifest_source: "qiongli-managed",
            download: update_download_status(&loaded.state),
            install: "not-started",
        })),
        UpdateCliCommand::Channel {
            expected_revision,
            stream,
        } => {
            if loaded.state.active_transaction.is_some() {
                return Err("native-update-transaction-active");
            }
            let mut state = loaded.state;
            state.selected_stream = stream;
            let outcome = store
                .replace(expected_revision, state)
                .map_err(|error| error.reason_code())?;
            Ok(UpdateCliOutput::Channel(UpdateChannelOutput {
                schema_version: 1,
                command: "update-channel",
                revision: outcome.revision,
                selected_stream: stream,
                cleanup_required: outcome.cleanup_required,
            }))
        }
        UpdateCliCommand::Check => {
            if loaded.state.active_transaction.is_some() {
                return Err("native-update-transaction-active");
            }
            let verified =
                fetch_verified_manifest(&loaded.state, authority, runtime, manifest_fetcher)?;
            let manifest = verified.manifest();
            Ok(UpdateCliOutput::Check(UpdateCheckOutput {
                schema_version: 1,
                command: "update-check",
                status: match verified.disposition() {
                    NativeUpdateDisposition::Current => "current",
                    NativeUpdateDisposition::Available => "update-available",
                },
                selected_stream: loaded.state.selected_stream,
                current_version: runtime.current_version,
                target_version: manifest.artifact.version.clone(),
                target_channel: manifest.artifact.channel,
                generation: manifest.generation,
                archive_size_bytes: manifest.archive_size_bytes,
                archive_sha256: manifest.archive_sha256.clone(),
                resource_pack_sha256: manifest.resource_pack_sha256.clone(),
                signed_payload_sha256: verified.signed_payload_sha256().to_string(),
                release_key_id: verified.release_key_id().to_string(),
                download: "not-started",
                install: "not-started",
            }))
        }
        UpdateCliCommand::Download { expected_revision } => {
            if loaded.revision != expected_revision {
                return Err("revision-conflict");
            }
            if loaded.state.active_transaction.is_some() {
                return Err("native-update-transaction-active");
            }
            let verified =
                fetch_verified_manifest(&loaded.state, authority, runtime, manifest_fetcher)?;
            if verified.disposition() != NativeUpdateDisposition::Available {
                return Err("native-update-not-available");
            }
            download_verified_update(
                store,
                loaded.state,
                expected_revision,
                verified,
                archive_fetcher,
            )
            .map(UpdateCliOutput::Download)
        }
        UpdateCliCommand::Cancel { expected_revision } => {
            cancel_download(store, loaded.state, loaded.revision, expected_revision)
                .map(UpdateCliOutput::Cancel)
        }
    }
}

fn fetch_verified_manifest(
    state: &UpdateState,
    authority: Option<&NativeReleaseAuthority>,
    runtime: &UpdateRuntimeContext<'_>,
    fetcher: &impl ManifestFetcher,
) -> Result<VerifiedNativeUpdateManifest, &'static str> {
    if runtime.os != Some(OperatingSystem::Macos) || runtime.arch != Some(Architecture::Aarch64) {
        return Err("native-update-target-unsupported");
    }
    let authority = authority.ok_or("native-update-release-authority-unavailable")?;
    let team_id = runtime
        .expected_macos_team_id
        .ok_or("native-update-macos-team-id-unavailable")?;
    let endpoint = manifest_endpoint(state.selected_stream);
    let bytes = fetcher.fetch(endpoint)?;
    let signed =
        SignedNativeUpdateManifestV1::from_json(&bytes).map_err(|error| error.reason_code())?;
    let authority_floor = authority.minimum_release_generation().saturating_sub(1);
    let context = NativeUpdateVerificationContext {
        now_unix: runtime.now_unix,
        last_accepted_generation: state.last_accepted_generation.max(authority_floor),
        current_version: runtime.current_version,
        selected_stream: native_stream(state.selected_stream),
        expected_macos_team_id: team_id,
        allowed_download_hosts: ARCHIVE_HOSTS,
        allow_current_version: true,
    };
    signed
        .verify(authority.release_keys(), &context)
        .map_err(|error| error.reason_code())
}

fn update_download_status(state: &UpdateState) -> &'static str {
    match state
        .active_transaction
        .as_ref()
        .map(|transaction| transaction.phase)
    {
        None => "not-started",
        Some(UpdateTransactionPhase::Downloading) => "in-progress",
        Some(UpdateTransactionPhase::Cancelling) => "cancelling",
        Some(_) => "staged",
    }
}

fn download_verified_update(
    store: &UpdateStateStore,
    mut state: UpdateState,
    expected_revision: u64,
    verified: VerifiedNativeUpdateManifest,
    archive_fetcher: &impl ArchiveFetcher,
) -> Result<UpdateDownloadOutput, &'static str> {
    let transaction_id = new_transaction_id()?;
    let manifest = verified.manifest();
    let target_version = manifest.artifact.version.clone();
    let generation = manifest.generation;
    let archive_file_name = manifest.archive_file_name.clone();
    let archive_size_bytes = manifest.archive_size_bytes;
    let archive_sha256 = manifest.archive_sha256.clone();
    let signed_manifest = verified
        .signed_manifest()
        .to_canonical_json()
        .map_err(|error| error.reason_code())?;
    state.active_transaction = Some(UpdateActiveTransaction {
        transaction_id: transaction_id.clone(),
        target_version: target_version.clone(),
        phase: UpdateTransactionPhase::Downloading,
    });
    let reservation = store
        .replace(expected_revision, state.clone())
        .map_err(|error| error.reason_code())?;
    if reservation.cleanup_required {
        return Err("native-update-state-cleanup-required");
    }

    let staging = match prepare_transaction_staging(store, &transaction_id, &signed_manifest) {
        Ok(staging) => staging,
        Err(error) => {
            return cleanup_failed_download(
                store,
                state,
                reservation.revision,
                &transaction_id,
                error,
            );
        }
    };
    if let Err(error) = archive_fetcher.fetch(&verified, &staging) {
        return cleanup_failed_download(store, state, reservation.revision, &transaction_id, error);
    }

    state
        .active_transaction
        .as_mut()
        .ok_or("native-update-transaction-missing")?
        .phase = UpdateTransactionPhase::Downloaded;
    let outcome = store
        .replace(reservation.revision, state)
        .map_err(|error| error.reason_code())?;
    Ok(UpdateDownloadOutput {
        schema_version: 1,
        command: "update-download",
        revision: outcome.revision,
        transaction_id,
        target_version,
        generation,
        archive_file_name,
        archive_size_bytes,
        archive_sha256,
        staging: "owner-private",
        install: "not-started",
        cleanup_required: outcome.cleanup_required,
    })
}

fn cleanup_failed_download(
    store: &UpdateStateStore,
    mut state: UpdateState,
    expected_revision: u64,
    transaction_id: &str,
    original_error: &'static str,
) -> Result<UpdateDownloadOutput, &'static str> {
    state
        .active_transaction
        .as_mut()
        .ok_or("native-update-transaction-missing")?
        .phase = UpdateTransactionPhase::Cancelling;
    let cancellation = match store.replace(expected_revision, state.clone()) {
        Ok(outcome) if !outcome.cleanup_required => outcome,
        Ok(_) | Err(_) => return Err("native-update-state-cleanup-required"),
    };
    if discard_transaction_staging(store, transaction_id).is_err() {
        return Err("native-update-staging-cleanup-required");
    }
    state.active_transaction = None;
    match store.replace(cancellation.revision, state) {
        Ok(outcome) if !outcome.cleanup_required => Err(original_error),
        Ok(_) | Err(_) => Err("native-update-state-cleanup-required"),
    }
}

fn cancel_download(
    store: &UpdateStateStore,
    mut state: UpdateState,
    observed_revision: u64,
    expected_revision: u64,
) -> Result<UpdateCancelOutput, &'static str> {
    if observed_revision != expected_revision {
        return Err("revision-conflict");
    }
    let transaction = state
        .active_transaction
        .as_ref()
        .ok_or("native-update-transaction-missing")?;
    if !matches!(
        transaction.phase,
        UpdateTransactionPhase::Downloading
            | UpdateTransactionPhase::Downloaded
            | UpdateTransactionPhase::Cancelling
    ) {
        return Err("native-update-transaction-not-cancellable");
    }
    let transaction_id = transaction.transaction_id.clone();
    let cancellation_revision = if transaction.phase == UpdateTransactionPhase::Cancelling {
        observed_revision
    } else {
        state
            .active_transaction
            .as_mut()
            .ok_or("native-update-transaction-missing")?
            .phase = UpdateTransactionPhase::Cancelling;
        let outcome = store
            .replace(expected_revision, state.clone())
            .map_err(|error| error.reason_code())?;
        if outcome.cleanup_required {
            return Err("native-update-state-cleanup-required");
        }
        outcome.revision
    };
    discard_transaction_staging(store, &transaction_id)?;
    state.active_transaction = None;
    let outcome = store
        .replace(cancellation_revision, state)
        .map_err(|error| error.reason_code())?;
    Ok(UpdateCancelOutput {
        schema_version: 1,
        command: "update-cancel",
        revision: outcome.revision,
        transaction_id,
        staging: "removed",
        cleanup_required: outcome.cleanup_required,
    })
}

fn new_transaction_id() -> Result<String, &'static str> {
    let mut bytes = [0_u8; 16];
    getrandom::fill(&mut bytes).map_err(|_| "native-update-transaction-id-unavailable")?;
    Ok(format!("update-{}", encode_lower_hex(&bytes)))
}

fn prepare_transaction_staging(
    store: &UpdateStateStore,
    transaction_id: &str,
    signed_manifest: &[u8],
) -> Result<PathBuf, &'static str> {
    let staging_root = store.staging_root();
    let updates_root = staging_root
        .parent()
        .ok_or("native-update-staging-unavailable")?;
    let state_root = updates_root
        .parent()
        .ok_or("native-update-staging-unavailable")?;
    ensure_private_directory(state_root, false)?;
    ensure_private_directory(updates_root, true)?;
    ensure_private_directory(&staging_root, true)?;
    let transaction_root = staging_root.join(transaction_id);
    create_new_private_directory(&transaction_root)?;
    if let Err(error) = write_new_private_file(
        &transaction_root.join(STAGED_MANIFEST_FILE),
        signed_manifest,
    )
    .and_then(|()| sync_directory(&transaction_root))
    .and_then(|()| sync_directory(&staging_root))
    {
        let _ = fs::remove_dir_all(&transaction_root);
        return Err(error);
    }
    Ok(transaction_root)
}

fn discard_transaction_staging(
    store: &UpdateStateStore,
    transaction_id: &str,
) -> Result<(), &'static str> {
    let staging_root = store.staging_root();
    let metadata = match fs::symlink_metadata(&staging_root) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(_) => return Err("native-update-staging-cleanup-required"),
    };
    validate_private_directory(&metadata)?;
    let transaction_root = staging_root.join(transaction_id);
    let metadata = match fs::symlink_metadata(&transaction_root) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(_) => return Err("native-update-staging-cleanup-required"),
    };
    validate_private_directory(&metadata)?;
    fs::remove_dir_all(&transaction_root).map_err(|_| "native-update-staging-cleanup-required")?;
    sync_directory(&staging_root)
}

fn ensure_private_directory(path: &Path, create: bool) -> Result<(), &'static str> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => validate_private_directory(&metadata),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound && create => {
            create_new_private_directory(path)
        }
        Err(_) => Err("native-update-staging-unavailable"),
    }
}

#[cfg(unix)]
fn create_new_private_directory(path: &Path) -> Result<(), &'static str> {
    use std::os::unix::fs::DirBuilderExt;

    let mut builder = fs::DirBuilder::new();
    builder.mode(0o700);
    builder
        .create(path)
        .map_err(|_| "native-update-staging-unavailable")?;
    let metadata = fs::symlink_metadata(path).map_err(|_| "native-update-staging-unavailable")?;
    validate_private_directory(&metadata)
}

#[cfg(not(unix))]
fn create_new_private_directory(_path: &Path) -> Result<(), &'static str> {
    Err("native-update-target-unsupported")
}

#[cfg(unix)]
fn validate_private_directory(metadata: &fs::Metadata) -> Result<(), &'static str> {
    use std::os::unix::fs::MetadataExt;

    if metadata.file_type().is_symlink()
        || !metadata.is_dir()
        || metadata.uid() != rustix::process::geteuid().as_raw()
        || metadata.mode() & 0o077 != 0
    {
        return Err("native-update-staging-unsafe");
    }
    Ok(())
}

#[cfg(not(unix))]
fn validate_private_directory(_metadata: &fs::Metadata) -> Result<(), &'static str> {
    Err("native-update-target-unsupported")
}

#[cfg(unix)]
fn write_new_private_file(path: &Path, bytes: &[u8]) -> Result<(), &'static str> {
    let mut file = open_new_private_file(path)?;
    file.write_all(bytes)
        .and_then(|()| file.flush())
        .and_then(|()| file.sync_all())
        .map_err(|_| "native-update-staging-write-failed")
}

#[cfg(not(unix))]
fn write_new_private_file(_path: &Path, _bytes: &[u8]) -> Result<(), &'static str> {
    Err("native-update-target-unsupported")
}

#[cfg(unix)]
fn open_new_private_file(path: &Path) -> Result<File, &'static str> {
    use std::os::unix::fs::OpenOptionsExt;

    OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(0o600)
        .open(path)
        .map_err(|_| "native-update-staging-write-failed")
}

#[cfg(not(unix))]
fn open_new_private_file(_path: &Path) -> Result<File, &'static str> {
    Err("native-update-target-unsupported")
}

fn sync_directory(path: &Path) -> Result<(), &'static str> {
    File::open(path)
        .and_then(|file| file.sync_all())
        .map_err(|_| "native-update-staging-sync-failed")
}

struct UpdateRuntimeContext<'a> {
    os: Option<OperatingSystem>,
    arch: Option<Architecture>,
    current_version: &'static str,
    now_unix: u64,
    expected_macos_team_id: Option<&'a str>,
}

trait ManifestFetcher {
    fn fetch(&self, endpoint: &str) -> Result<Vec<u8>, &'static str>;
}

struct ReqwestManifestFetcher;

impl ManifestFetcher for ReqwestManifestFetcher {
    fn fetch(&self, endpoint: &str) -> Result<Vec<u8>, &'static str> {
        let client = Client::builder()
            .connect_timeout(CONNECT_TIMEOUT)
            .timeout(REQUEST_TIMEOUT)
            .redirect(Policy::none())
            .user_agent(concat!("qiongli-native-update/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|_| "native-update-http-client-unavailable")?;
        let response = client
            .get(endpoint)
            .header(ACCEPT, "application/json")
            .header(ACCEPT_ENCODING, "identity")
            .send()
            .map_err(map_reqwest_error)?;
        validate_manifest_response(
            response.status(),
            response.headers().get(CONTENT_ENCODING),
            response.content_length(),
        )?;
        read_manifest_body(response)
    }
}

fn validate_manifest_response(
    status: StatusCode,
    content_encoding: Option<&reqwest::header::HeaderValue>,
    content_length: Option<u64>,
) -> Result<(), &'static str> {
    if status != StatusCode::OK {
        return Err("native-update-manifest-response-invalid");
    }
    if unsupported_content_encoding(content_encoding) {
        return Err("native-update-manifest-encoding-invalid");
    }
    if content_length
        .is_some_and(|length| length > qiongli_platform::MAX_NATIVE_UPDATE_MANIFEST_BYTES as u64)
    {
        return Err("native-update-manifest-too-large");
    }
    Ok(())
}

fn read_manifest_body(reader: impl Read) -> Result<Vec<u8>, &'static str> {
    let mut bytes = Vec::new();
    reader
        .take((qiongli_platform::MAX_NATIVE_UPDATE_MANIFEST_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|_| "native-update-manifest-read-failed")?;
    if bytes.len() > qiongli_platform::MAX_NATIVE_UPDATE_MANIFEST_BYTES {
        return Err("native-update-manifest-too-large");
    }
    Ok(bytes)
}

trait ArchiveFetcher {
    fn fetch(
        &self,
        verified: &VerifiedNativeUpdateManifest,
        transaction_root: &Path,
    ) -> Result<(), &'static str>;
}

struct ReqwestArchiveFetcher;

impl ArchiveFetcher for ReqwestArchiveFetcher {
    fn fetch(
        &self,
        verified: &VerifiedNativeUpdateManifest,
        transaction_root: &Path,
    ) -> Result<(), &'static str> {
        let manifest = verified.manifest();
        let source = reqwest::Url::parse(&manifest.archive_url)
            .map_err(|_| "native-update-archive-url-invalid")?;
        if !allowed_archive_transport_url(&source) {
            return Err("native-update-archive-url-invalid");
        }
        let client = Client::builder()
            .connect_timeout(CONNECT_TIMEOUT)
            .timeout(ARCHIVE_REQUEST_TIMEOUT)
            .redirect(Policy::custom(|attempt| {
                if !archive_redirect_allowed(attempt.url(), attempt.previous().len()) {
                    attempt.stop()
                } else {
                    attempt.follow()
                }
            }))
            .user_agent(concat!("qiongli-native-update/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|_| "native-update-http-client-unavailable")?;
        let response = client
            .get(source)
            .header(ACCEPT, "application/octet-stream")
            .header(ACCEPT_ENCODING, "identity")
            .send()
            .map_err(map_archive_reqwest_error)?;
        validate_archive_response(
            response.status(),
            response.url(),
            response.headers().get(CONTENT_ENCODING),
            response.content_length(),
            manifest.archive_size_bytes,
        )?;
        stage_archive_from_reader(
            response,
            transaction_root,
            &manifest.archive_file_name,
            manifest.archive_size_bytes,
            &manifest.archive_sha256,
        )
    }
}

fn validate_archive_response(
    status: StatusCode,
    final_url: &reqwest::Url,
    content_encoding: Option<&reqwest::header::HeaderValue>,
    content_length: Option<u64>,
    expected_size: u64,
) -> Result<(), &'static str> {
    if status != StatusCode::OK || !allowed_archive_transport_url(final_url) {
        return Err("native-update-archive-response-invalid");
    }
    if unsupported_content_encoding(content_encoding) {
        return Err("native-update-archive-encoding-invalid");
    }
    if content_length.is_some_and(|length| length != expected_size) {
        return Err("native-update-archive-size-mismatch");
    }
    Ok(())
}

fn stage_archive_from_reader(
    mut reader: impl Read,
    transaction_root: &Path,
    archive_file_name: &str,
    expected_size: u64,
    expected_sha256: &str,
) -> Result<(), &'static str> {
    if !safe_file_name(archive_file_name) {
        return Err("native-update-archive-name-invalid");
    }
    let partial = transaction_root.join(PARTIAL_ARCHIVE_FILE);
    let destination = transaction_root.join(archive_file_name);
    let result = (|| {
        let mut file = open_new_private_file(&partial)?;
        let mut hasher = Sha256::new();
        let mut total = 0_u64;
        let mut buffer = [0_u8; ARCHIVE_BUFFER_BYTES];
        loop {
            let count = reader
                .read(&mut buffer)
                .map_err(|_| "native-update-archive-read-failed")?;
            if count == 0 {
                break;
            }
            total = total
                .checked_add(count as u64)
                .ok_or("native-update-archive-size-mismatch")?;
            if total > expected_size {
                return Err("native-update-archive-size-mismatch");
            }
            file.write_all(&buffer[..count])
                .map_err(|_| "native-update-staging-write-failed")?;
            hasher.update(&buffer[..count]);
        }
        if total != expected_size {
            return Err("native-update-archive-size-mismatch");
        }
        let observed_sha256 = encode_lower_hex(&hasher.finalize());
        if observed_sha256 != expected_sha256 {
            return Err("native-update-archive-digest-mismatch");
        }
        file.flush()
            .and_then(|()| file.sync_all())
            .map_err(|_| "native-update-staging-write-failed")?;
        drop(file);
        fs::hard_link(&partial, &destination)
            .map_err(|_| "native-update-staging-activate-failed")?;
        if fs::remove_file(&partial).is_err() {
            let _ = fs::remove_file(&destination);
            return Err("native-update-staging-activate-failed");
        }
        sync_directory(transaction_root)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&partial);
        let _ = fs::remove_file(&destination);
    }
    result
}

fn allowed_archive_transport_url(url: &reqwest::Url) -> bool {
    url.scheme() == "https"
        && url.host_str().is_some_and(|host| {
            ARCHIVE_HOSTS
                .iter()
                .any(|allowed| host.eq_ignore_ascii_case(allowed))
        })
        && url.username().is_empty()
        && url.password().is_none()
        && url.port().is_none()
        && url.fragment().is_none()
}

fn archive_redirect_allowed(url: &reqwest::Url, previous_redirects: usize) -> bool {
    previous_redirects < MAX_ARCHIVE_REDIRECTS && allowed_archive_transport_url(url)
}

fn unsupported_content_encoding(value: Option<&reqwest::header::HeaderValue>) -> bool {
    value.is_some_and(|value| {
        value
            .to_str()
            .map_or(true, |value| !value.eq_ignore_ascii_case("identity"))
    })
}

fn safe_file_name(value: &str) -> bool {
    !value.is_empty()
        && value != "."
        && value != ".."
        && !value.bytes().any(|byte| matches!(byte, b'/' | b'\\' | 0))
}

fn encode_lower_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len().saturating_mul(2));
    for byte in bytes {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}

const fn manifest_endpoint(stream: UpdateStreamPreference) -> &'static str {
    match stream {
        UpdateStreamPreference::Stable => STABLE_MANIFEST_ENDPOINT,
        UpdateStreamPreference::Beta => BETA_MANIFEST_ENDPOINT,
    }
}

const fn native_stream(stream: UpdateStreamPreference) -> NativeUpdateStream {
    match stream {
        UpdateStreamPreference::Stable => NativeUpdateStream::Stable,
        UpdateStreamPreference::Beta => NativeUpdateStream::Beta,
    }
}

fn now_unix() -> Result<u64, &'static str> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|_| "native-update-clock-invalid")
}

fn map_reqwest_error(error: reqwest::Error) -> &'static str {
    if error.is_timeout() {
        "native-update-manifest-timeout"
    } else {
        "native-update-manifest-fetch-failed"
    }
}

fn map_archive_reqwest_error(error: reqwest::Error) -> &'static str {
    if error.is_timeout() {
        "native-update-archive-timeout"
    } else {
        "native-update-archive-fetch-failed"
    }
}

#[cfg(all(test, unix))]
mod tests {
    use std::io;
    use std::path::Path;
    use std::sync::{Arc, Barrier};

    use ed25519_dalek::{Signer, SigningKey};
    use qiongli_config::{UpdateState, resolve_config_root};
    use qiongli_platform::{
        ArtifactIdentityV1, CapabilityProfile, InstallerKind, NativeReleaseSignatureV1,
        NativeUpdateManifestV1, ProductId, ReleaseChannel, SignatureAlgorithm,
        native_update_manifest_signing_bytes,
    };
    use serde_json::json;

    use super::*;

    const NOW: u64 = 1_750_000_000;
    const TEAM_ID: &str = "ABC123DEFG";
    const ARCHIVE_BYTES: &[u8] = b"qiongli-signed-archive-fixture";

    struct FixedFetcher(Vec<u8>);

    impl ManifestFetcher for FixedFetcher {
        fn fetch(&self, _endpoint: &str) -> Result<Vec<u8>, &'static str> {
            Ok(self.0.clone())
        }
    }

    struct NoopArchiveFetcher;

    impl ArchiveFetcher for NoopArchiveFetcher {
        fn fetch(
            &self,
            _verified: &VerifiedNativeUpdateManifest,
            _transaction_root: &Path,
        ) -> Result<(), &'static str> {
            Err("unexpected-archive-fetch")
        }
    }

    struct FixedArchiveFetcher(Vec<u8>);

    impl ArchiveFetcher for FixedArchiveFetcher {
        fn fetch(
            &self,
            verified: &VerifiedNativeUpdateManifest,
            transaction_root: &Path,
        ) -> Result<(), &'static str> {
            let manifest = verified.manifest();
            stage_archive_from_reader(
                self.0.as_slice(),
                transaction_root,
                &manifest.archive_file_name,
                manifest.archive_size_bytes,
                &manifest.archive_sha256,
            )
        }
    }

    struct ErrorManifestFetcher(&'static str);

    impl ManifestFetcher for ErrorManifestFetcher {
        fn fetch(&self, _endpoint: &str) -> Result<Vec<u8>, &'static str> {
            Err(self.0)
        }
    }

    struct ErrorArchiveFetcher(&'static str);

    impl ArchiveFetcher for ErrorArchiveFetcher {
        fn fetch(
            &self,
            _verified: &VerifiedNativeUpdateManifest,
            _transaction_root: &Path,
        ) -> Result<(), &'static str> {
            Err(self.0)
        }
    }

    struct InterruptingReader {
        emitted: bool,
    }

    impl Read for InterruptingReader {
        fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
            if self.emitted {
                return Err(io::Error::new(
                    io::ErrorKind::ConnectionReset,
                    "fixture stream interrupted",
                ));
            }
            self.emitted = true;
            let count = buffer.len().min(8);
            buffer[..count].copy_from_slice(&ARCHIVE_BYTES[..count]);
            Ok(count)
        }
    }

    struct InterruptingArchiveFetcher;

    impl ArchiveFetcher for InterruptingArchiveFetcher {
        fn fetch(
            &self,
            verified: &VerifiedNativeUpdateManifest,
            transaction_root: &Path,
        ) -> Result<(), &'static str> {
            let manifest = verified.manifest();
            stage_archive_from_reader(
                InterruptingReader { emitted: false },
                transaction_root,
                &manifest.archive_file_name,
                manifest.archive_size_bytes,
                &manifest.archive_sha256,
            )
        }
    }

    struct BarrierManifestFetcher {
        bytes: Vec<u8>,
        barrier: Arc<Barrier>,
    }

    impl ManifestFetcher for BarrierManifestFetcher {
        fn fetch(&self, _endpoint: &str) -> Result<Vec<u8>, &'static str> {
            self.barrier.wait();
            Ok(self.bytes.clone())
        }
    }

    struct BlockingArchiveFetcher {
        entered: Arc<Barrier>,
        release: Arc<Barrier>,
    }

    impl ArchiveFetcher for BlockingArchiveFetcher {
        fn fetch(
            &self,
            verified: &VerifiedNativeUpdateManifest,
            transaction_root: &Path,
        ) -> Result<(), &'static str> {
            self.entered.wait();
            self.release.wait();
            let manifest = verified.manifest();
            stage_archive_from_reader(
                ARCHIVE_BYTES,
                transaction_root,
                &manifest.archive_file_name,
                manifest.archive_size_bytes,
                &manifest.archive_sha256,
            )
        }
    }

    struct ResponseFixture {
        status: StatusCode,
        final_url: reqwest::Url,
        content_encoding: Option<reqwest::header::HeaderValue>,
        content_length: Option<u64>,
    }

    impl ResponseFixture {
        fn valid() -> Self {
            Self {
                status: StatusCode::OK,
                final_url: reqwest::Url::parse("https://github.com/qiongli.zip").unwrap(),
                content_encoding: Some(reqwest::header::HeaderValue::from_static("identity")),
                content_length: None,
            }
        }

        fn validate_manifest(&self) -> Result<(), &'static str> {
            validate_manifest_response(
                self.status,
                self.content_encoding.as_ref(),
                self.content_length,
            )
        }

        fn validate_archive(&self, expected_size: u64) -> Result<(), &'static str> {
            validate_archive_response(
                self.status,
                &self.final_url,
                self.content_encoding.as_ref(),
                self.content_length,
                expected_size,
            )
        }
    }

    #[test]
    fn status_and_channel_are_revision_safe_and_do_not_require_release_authority() {
        let (store, root) = store("status");
        let runtime = runtime();
        let status = execute_with_fetchers(
            UpdateCliCommand::Status,
            &store,
            None,
            &runtime,
            &FixedFetcher(Vec::new()),
            &NoopArchiveFetcher,
        )
        .unwrap();
        let json = serde_json::to_value(status).unwrap();
        assert_eq!(json["revision"], 0);
        assert_eq!(json["selected_stream"], "beta");
        assert_eq!(json["release_authority"], "unavailable");
        assert!(!root.exists());

        let changed = execute_with_fetchers(
            UpdateCliCommand::Channel {
                expected_revision: 0,
                stream: UpdateStreamPreference::Stable,
            },
            &store,
            None,
            &runtime,
            &FixedFetcher(Vec::new()),
            &NoopArchiveFetcher,
        )
        .unwrap();
        let json = serde_json::to_value(changed).unwrap();
        assert_eq!(json["revision"], 1);
        assert_eq!(json["selected_stream"], "stable");
        assert_eq!(
            execute_with_fetchers(
                UpdateCliCommand::Channel {
                    expected_revision: 0,
                    stream: UpdateStreamPreference::Beta,
                },
                &store,
                None,
                &runtime,
                &FixedFetcher(Vec::new()),
                &NoopArchiveFetcher,
            ),
            Err("revision-conflict")
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn check_verifies_the_managed_manifest_without_writing_or_downloading() {
        let release_key = SigningKey::from_bytes(&[91_u8; 32]);
        let authority = authority(&release_key);
        let signed = signed_manifest(&release_key, "2.0.0-alpha.2");
        let fetcher = FixedFetcher(signed.to_canonical_json().unwrap());
        let (store, root) = store("check");
        let output = execute_with_fetchers(
            UpdateCliCommand::Check,
            &store,
            Some(&authority),
            &runtime(),
            &fetcher,
            &NoopArchiveFetcher,
        )
        .unwrap();
        let json = serde_json::to_value(output).unwrap();
        assert_eq!(json["status"], "update-available");
        assert_eq!(json["target_version"], "2.0.0-alpha.2");
        assert_eq!(json["download"], "not-started");
        assert_eq!(json["install"], "not-started");
        assert!(!root.exists());

        assert_eq!(
            execute_with_fetchers(
                UpdateCliCommand::Check,
                &store,
                None,
                &runtime(),
                &fetcher,
                &NoopArchiveFetcher,
            ),
            Err("native-update-release-authority-unavailable")
        );
    }

    #[test]
    fn check_reports_current_for_the_last_accepted_generation_without_mutation() {
        let release_key = SigningKey::from_bytes(&[91_u8; 32]);
        let authority = authority(&release_key);
        let signed = signed_manifest(&release_key, "2.0.0-alpha.1");
        let fetcher = FixedFetcher(signed.to_canonical_json().unwrap());
        let (store, root) = store("current");
        let mut state = UpdateState::initial(UpdateStreamPreference::Beta);
        state.last_accepted_generation = 2;
        store.replace(0, state).unwrap();

        let output = execute_with_fetchers(
            UpdateCliCommand::Check,
            &store,
            Some(&authority),
            &runtime(),
            &fetcher,
            &NoopArchiveFetcher,
        )
        .unwrap();
        let json = serde_json::to_value(output).unwrap();
        assert_eq!(json["status"], "current");
        assert_eq!(json["generation"], 2);
        assert_eq!(store.load().unwrap().revision, 1);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn download_stages_exact_private_bytes_and_cancel_removes_the_transaction() {
        use std::os::unix::fs::PermissionsExt;

        let release_key = SigningKey::from_bytes(&[91_u8; 32]);
        let authority = authority(&release_key);
        let signed = signed_manifest(&release_key, "2.0.0-alpha.2");
        let archive_file_name = signed.manifest.archive_file_name.clone();
        let manifest_fetcher = FixedFetcher(signed.to_canonical_json().unwrap());
        let archive_fetcher = FixedArchiveFetcher(ARCHIVE_BYTES.to_vec());
        let (store, root) = store("download");

        let output = execute_with_fetchers(
            UpdateCliCommand::Download {
                expected_revision: 0,
            },
            &store,
            Some(&authority),
            &runtime(),
            &manifest_fetcher,
            &archive_fetcher,
        )
        .unwrap();
        let json = serde_json::to_value(output).unwrap();
        assert_eq!(json["command"], "update-download");
        assert_eq!(json["revision"], 2);
        assert_eq!(json["install"], "not-started");
        let transaction_id = json["transaction_id"].as_str().unwrap();
        let transaction_root = root.join("v2/updates/staging").join(transaction_id);
        let archive = transaction_root.join(&archive_file_name);
        assert_eq!(std::fs::read(&archive).unwrap(), ARCHIVE_BYTES);
        assert_eq!(
            std::fs::metadata(&archive).unwrap().permissions().mode() & 0o777,
            0o600
        );
        assert_eq!(
            std::fs::metadata(&transaction_root)
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o700
        );
        let staged_manifest = transaction_root.join(STAGED_MANIFEST_FILE);
        assert!(staged_manifest.is_file());
        assert_eq!(
            std::fs::metadata(staged_manifest)
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
        assert_eq!(
            store
                .load()
                .unwrap()
                .state
                .active_transaction
                .unwrap()
                .phase,
            UpdateTransactionPhase::Downloaded
        );

        assert_eq!(
            execute_with_fetchers(
                UpdateCliCommand::Download {
                    expected_revision: 0,
                },
                &store,
                Some(&authority),
                &runtime(),
                &manifest_fetcher,
                &archive_fetcher,
            ),
            Err("revision-conflict")
        );
        let cancelled = execute_with_fetchers(
            UpdateCliCommand::Cancel {
                expected_revision: 2,
            },
            &store,
            None,
            &runtime(),
            &FixedFetcher(Vec::new()),
            &NoopArchiveFetcher,
        )
        .unwrap();
        let json = serde_json::to_value(cancelled).unwrap();
        assert_eq!(json["revision"], 4);
        assert_eq!(json["staging"], "removed");
        assert!(!transaction_root.exists());
        assert!(store.load().unwrap().state.active_transaction.is_none());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn corrupt_incomplete_or_oversized_download_is_removed_and_clears_active_state() {
        let release_key = SigningKey::from_bytes(&[91_u8; 32]);
        let authority = authority(&release_key);
        let signed = signed_manifest(&release_key, "2.0.0-alpha.2");
        let manifest_fetcher = FixedFetcher(signed.to_canonical_json().unwrap());

        for (name, bytes, expected_error) in [
            (
                "corrupt",
                b"qiongli-signed-archive-fixturf".to_vec(),
                "native-update-archive-digest-mismatch",
            ),
            (
                "oversized",
                [ARCHIVE_BYTES, b"unexpected"].concat(),
                "native-update-archive-size-mismatch",
            ),
            (
                "incomplete",
                ARCHIVE_BYTES[..ARCHIVE_BYTES.len() - 1].to_vec(),
                "native-update-archive-size-mismatch",
            ),
        ] {
            let (store, root) = store(name);
            assert_eq!(
                execute_with_fetchers(
                    UpdateCliCommand::Download {
                        expected_revision: 0,
                    },
                    &store,
                    Some(&authority),
                    &runtime(),
                    &manifest_fetcher,
                    &FixedArchiveFetcher(bytes),
                ),
                Err(expected_error)
            );
            let loaded = store.load().unwrap();
            assert_eq!(loaded.revision, 3);
            assert!(loaded.state.active_transaction.is_none());
            let staging = root.join("v2/updates/staging");
            assert_eq!(std::fs::read_dir(staging).unwrap().count(), 0);
            let _ = std::fs::remove_dir_all(root);
        }
    }

    #[test]
    fn response_and_stream_fixtures_cover_the_transport_fault_matrix() {
        let valid = ResponseFixture::valid();
        assert_eq!(valid.validate_manifest(), Ok(()));
        assert_eq!(valid.validate_archive(ARCHIVE_BYTES.len() as u64), Ok(()));
        assert_eq!(read_manifest_body(ARCHIVE_BYTES).unwrap(), ARCHIVE_BYTES);

        let mut fixture = ResponseFixture::valid();
        fixture.status = StatusCode::FOUND;
        assert_eq!(
            fixture.validate_manifest(),
            Err("native-update-manifest-response-invalid")
        );
        assert_eq!(
            fixture.validate_archive(ARCHIVE_BYTES.len() as u64),
            Err("native-update-archive-response-invalid")
        );

        let mut fixture = ResponseFixture::valid();
        fixture.content_encoding = Some(reqwest::header::HeaderValue::from_static("gzip"));
        assert_eq!(
            fixture.validate_manifest(),
            Err("native-update-manifest-encoding-invalid")
        );
        assert_eq!(
            fixture.validate_archive(ARCHIVE_BYTES.len() as u64),
            Err("native-update-archive-encoding-invalid")
        );

        let mut fixture = ResponseFixture::valid();
        fixture.content_length =
            Some(u64::try_from(qiongli_platform::MAX_NATIVE_UPDATE_MANIFEST_BYTES).unwrap() + 1);
        assert_eq!(
            fixture.validate_manifest(),
            Err("native-update-manifest-too-large")
        );
        fixture.content_length = Some(ARCHIVE_BYTES.len() as u64 + 1);
        assert_eq!(
            fixture.validate_archive(ARCHIVE_BYTES.len() as u64),
            Err("native-update-archive-size-mismatch")
        );

        let oversized = vec![0_u8; qiongli_platform::MAX_NATIVE_UPDATE_MANIFEST_BYTES + 1];
        assert_eq!(
            read_manifest_body(oversized.as_slice()),
            Err("native-update-manifest-too-large")
        );

        let mut fixture = ResponseFixture::valid();
        fixture.final_url = reqwest::Url::parse("https://example.com/qiongli.zip").unwrap();
        assert_eq!(
            fixture.validate_archive(ARCHIVE_BYTES.len() as u64),
            Err("native-update-archive-response-invalid")
        );
    }

    #[test]
    fn offline_refused_timeout_and_interrupted_fetches_are_fixed_and_redacted() {
        let release_key = SigningKey::from_bytes(&[91_u8; 32]);
        let authority = authority(&release_key);

        for (name, error) in [
            ("manifest-offline", "native-update-manifest-fetch-failed"),
            ("manifest-refused", "native-update-manifest-fetch-failed"),
            ("manifest-timeout", "native-update-manifest-timeout"),
        ] {
            let (store, root) = store(name);
            let observed = execute_with_fetchers(
                UpdateCliCommand::Check,
                &store,
                Some(&authority),
                &runtime(),
                &ErrorManifestFetcher(error),
                &NoopArchiveFetcher,
            )
            .unwrap_err();
            assert_redacted_error(observed, error, &root);
            assert!(!root.exists());
        }

        for (name, error) in [
            ("archive-offline", "native-update-archive-fetch-failed"),
            ("archive-refused", "native-update-archive-fetch-failed"),
            ("archive-timeout", "native-update-archive-timeout"),
        ] {
            assert_archive_failure(
                name,
                &authority,
                &release_key,
                &ErrorArchiveFetcher(error),
                error,
            );
        }
        assert_archive_failure(
            "archive-interrupted",
            &authority,
            &release_key,
            &InterruptingArchiveFetcher,
            "native-update-archive-read-failed",
        );
    }

    #[test]
    fn archive_transport_redirect_policy_rejects_downgrade_and_unknown_hosts() {
        for rejected in [
            "http://github.com/qiongli.zip",
            "https://example.com/qiongli.zip",
            "https://user@github.com/qiongli.zip",
            "https://github.com/qiongli.zip#fragment",
        ] {
            assert!(!allowed_archive_transport_url(
                &reqwest::Url::parse(rejected).unwrap()
            ));
        }
        assert!(allowed_archive_transport_url(
            &reqwest::Url::parse(
                "https://release-assets.githubusercontent.com/qiongli.zip?token=ephemeral"
            )
            .unwrap()
        ));
        let allowed = reqwest::Url::parse(
            "https://release-assets.githubusercontent.com/qiongli.zip?token=ephemeral",
        )
        .unwrap();
        assert!(archive_redirect_allowed(
            &allowed,
            MAX_ARCHIVE_REDIRECTS - 1
        ));
        assert!(!archive_redirect_allowed(&allowed, MAX_ARCHIVE_REDIRECTS));
        assert!(!archive_redirect_allowed(
            &reqwest::Url::parse("https://example.com/qiongli.zip").unwrap(),
            0
        ));
    }

    #[test]
    fn concurrent_downloads_allow_only_one_expected_revision_reservation() {
        let release_key = SigningKey::from_bytes(&[91_u8; 32]);
        let authority = authority(&release_key);
        let manifest = signed_manifest(&release_key, "2.0.0-alpha.2")
            .to_canonical_json()
            .unwrap();
        let (store, root) = store("concurrent-reservation");
        let barrier = Arc::new(Barrier::new(2));
        let mut handles = Vec::new();

        for _ in 0..2 {
            let store = store.clone();
            let authority = authority.clone();
            let bytes = manifest.clone();
            let barrier = Arc::clone(&barrier);
            handles.push(std::thread::spawn(move || {
                execute_with_fetchers(
                    UpdateCliCommand::Download {
                        expected_revision: 0,
                    },
                    &store,
                    Some(&authority),
                    &runtime(),
                    &BarrierManifestFetcher { bytes, barrier },
                    &FixedArchiveFetcher(ARCHIVE_BYTES.to_vec()),
                )
            }));
        }

        let results: Vec<_> = handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect();
        assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
        assert_eq!(
            results
                .iter()
                .filter(|result| matches!(result, Err("revision-conflict")))
                .count(),
            1
        );
        let loaded = store.load().unwrap();
        assert_eq!(loaded.revision, 2);
        assert_eq!(
            loaded.state.active_transaction.unwrap().phase,
            UpdateTransactionPhase::Downloaded
        );
        execute_with_fetchers(
            UpdateCliCommand::Cancel {
                expected_revision: 2,
            },
            &store,
            None,
            &runtime(),
            &FixedFetcher(Vec::new()),
            &NoopArchiveFetcher,
        )
        .unwrap();
        assert_eq!(
            std::fs::read_dir(root.join("v2/updates/staging"))
                .unwrap()
                .count(),
            0
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn concurrent_cancel_wins_without_leaving_download_bytes() {
        let release_key = SigningKey::from_bytes(&[91_u8; 32]);
        let authority = authority(&release_key);
        let manifest = signed_manifest(&release_key, "2.0.0-alpha.2")
            .to_canonical_json()
            .unwrap();
        let (store, root) = store("concurrent-cancel");
        let entered = Arc::new(Barrier::new(2));
        let release = Arc::new(Barrier::new(2));
        let download_store = store.clone();
        let download_entered = Arc::clone(&entered);
        let download_release = Arc::clone(&release);
        let handle = std::thread::spawn(move || {
            execute_with_fetchers(
                UpdateCliCommand::Download {
                    expected_revision: 0,
                },
                &download_store,
                Some(&authority),
                &runtime(),
                &FixedFetcher(manifest),
                &BlockingArchiveFetcher {
                    entered: download_entered,
                    release: download_release,
                },
            )
        });

        entered.wait();
        let reserved = store.load().unwrap();
        assert_eq!(reserved.revision, 1);
        assert_eq!(
            reserved.state.active_transaction.unwrap().phase,
            UpdateTransactionPhase::Downloading
        );
        let cancelled = execute_with_fetchers(
            UpdateCliCommand::Cancel {
                expected_revision: 1,
            },
            &store,
            None,
            &runtime(),
            &FixedFetcher(Vec::new()),
            &NoopArchiveFetcher,
        )
        .unwrap();
        assert_eq!(serde_json::to_value(cancelled).unwrap()["revision"], 3);
        release.wait();
        assert_eq!(
            handle.join().unwrap(),
            Err("native-update-state-cleanup-required")
        );

        let loaded = store.load().unwrap();
        assert_eq!(loaded.revision, 3);
        assert!(loaded.state.active_transaction.is_none());
        assert_eq!(
            std::fs::read_dir(root.join("v2/updates/staging"))
                .unwrap()
                .count(),
            0
        );
        let _ = std::fs::remove_dir_all(root);
    }

    fn assert_archive_failure(
        name: &str,
        authority: &NativeReleaseAuthority,
        release_key: &SigningKey,
        archive_fetcher: &impl ArchiveFetcher,
        expected_error: &'static str,
    ) {
        let signed = signed_manifest(release_key, "2.0.0-alpha.2");
        let (store, root) = store(name);
        let observed = execute_with_fetchers(
            UpdateCliCommand::Download {
                expected_revision: 0,
            },
            &store,
            Some(authority),
            &runtime(),
            &FixedFetcher(signed.to_canonical_json().unwrap()),
            archive_fetcher,
        )
        .unwrap_err();
        assert_redacted_error(observed, expected_error, &root);
        let loaded = store.load().unwrap();
        assert_eq!(loaded.revision, 3);
        assert!(loaded.state.active_transaction.is_none());
        assert_eq!(
            std::fs::read_dir(root.join("v2/updates/staging"))
                .unwrap()
                .count(),
            0
        );
        let _ = std::fs::remove_dir_all(root);
    }

    fn assert_redacted_error(observed: &'static str, expected: &'static str, root: &Path) {
        assert_eq!(observed, expected);
        assert!(!observed.contains(root.to_string_lossy().as_ref()));
    }

    fn runtime() -> UpdateRuntimeContext<'static> {
        UpdateRuntimeContext {
            os: Some(OperatingSystem::Macos),
            arch: Some(Architecture::Aarch64),
            current_version: "2.0.0-alpha.1",
            now_unix: NOW,
            expected_macos_team_id: Some(TEAM_ID),
        }
    }

    fn store(name: &str) -> (UpdateStateStore, std::path::PathBuf) {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .unwrap()
            .join("target/qiongli-update-cli-tests")
            .join(format!("{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let config = resolve_config_root(Some(root.as_os_str()), Path::new("/tmp")).unwrap();
        (
            UpdateStateStore::new(config, UpdateStreamPreference::Beta),
            root,
        )
    }

    fn authority(release_key: &SigningKey) -> NativeReleaseAuthority {
        let launch_key = SigningKey::from_bytes(&[92_u8; 32]);
        let value = json!({
            "schema_version": 1,
            "channel": "alpha",
            "minimum_release_generation": 1,
            "minimum_launch_grant_generation": 1,
            "release_keys": [{
                "key_id": "release-test-key",
                "public_key_hex": encode_hex(&release_key.verifying_key().to_bytes()),
                "minimum_generation": 1,
                "maximum_generation_exclusive": null
            }],
            "launch_grant_keys": [{
                "key_id": "launch-test-key",
                "public_key_hex": encode_hex(&launch_key.verifying_key().to_bytes())
            }]
        });
        NativeReleaseAuthority::from_json(&serde_json_canonicalizer::to_vec(&value).unwrap())
            .unwrap()
    }

    fn signed_manifest(release_key: &SigningKey, version: &str) -> SignedNativeUpdateManifestV1 {
        let archive_file_name =
            format!("qiongli-desktop-{version}-macos-aarch64.signed-notarized.app.zip");
        let manifest = NativeUpdateManifestV1 {
            schema_version: 1,
            stream: NativeUpdateStream::Beta,
            generation: 2,
            artifact: ArtifactIdentityV1 {
                product: ProductId::Qiongli,
                version: version.to_string(),
                channel: ReleaseChannel::Alpha,
                profile: CapabilityProfile::Lite,
                os: OperatingSystem::Macos,
                arch: Architecture::Aarch64,
                installer_kind: InstallerKind::NativeInstaller,
            },
            source_commit: "a".repeat(40),
            minimum_updater_version: "2.0.0-alpha.1".to_string(),
            archive_url: format!(
                "https://github.com/jxpeng98/qiongli/releases/download/v{version}/{archive_file_name}"
            ),
            archive_file_name,
            archive_size_bytes: ARCHIVE_BYTES.len() as u64,
            archive_sha256: encode_lower_hex(&Sha256::digest(ARCHIVE_BYTES)),
            desktop_manifest_sha256: "2".repeat(64),
            signing_receipt_sha256: "3".repeat(64),
            resource_pack_sha256: "4".repeat(64),
            macos_team_id: TEAM_ID.to_string(),
            published_at_unix: NOW - 120,
            not_before_unix: NOW - 60,
            expires_at_unix: NOW + 3_600,
        };
        let signature = release_key.sign(&native_update_manifest_signing_bytes(&manifest).unwrap());
        SignedNativeUpdateManifestV1 {
            manifest,
            signature: NativeReleaseSignatureV1 {
                algorithm: SignatureAlgorithm::Ed25519,
                key_id: "release-test-key".to_string(),
                value_hex: encode_hex(&signature.to_bytes()),
            },
        }
    }

    fn encode_hex<const N: usize>(bytes: &[u8; N]) -> String {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut output = String::with_capacity(N * 2);
        for byte in bytes {
            output.push(char::from(HEX[usize::from(byte >> 4)]));
            output.push(char::from(HEX[usize::from(byte & 0x0f)]));
        }
        output
    }

    #[test]
    fn active_transaction_blocks_channel_and_check() {
        let (store, root) = store("active");
        let mut state = UpdateState::initial(UpdateStreamPreference::Beta);
        state.active_transaction = Some(qiongli_config::UpdateActiveTransaction {
            transaction_id: "update-transaction-1".to_string(),
            target_version: "2.0.0-alpha.2".to_string(),
            phase: qiongli_config::UpdateTransactionPhase::Downloaded,
        });
        store.replace(0, state).unwrap();
        assert_eq!(
            execute_with_fetchers(
                UpdateCliCommand::Check,
                &store,
                None,
                &runtime(),
                &FixedFetcher(Vec::new()),
                &NoopArchiveFetcher,
            ),
            Err("native-update-transaction-active")
        );
        let _ = std::fs::remove_dir_all(root);
    }
}
