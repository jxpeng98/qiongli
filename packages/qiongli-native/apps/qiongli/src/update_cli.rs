use std::io::Read;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use qiongli_config::{UpdateStateStore, UpdateStreamPreference};
use qiongli_platform::{
    Architecture, NativeReleaseAuthority, NativeUpdateDisposition, NativeUpdateStream,
    NativeUpdateVerificationContext, OperatingSystem, SignedNativeUpdateManifestV1,
};
use reqwest::StatusCode;
use reqwest::blocking::Client;
use reqwest::header::{ACCEPT, ACCEPT_ENCODING, CONTENT_ENCODING};
use reqwest::redirect::Policy;
use serde::Serialize;

const STABLE_MANIFEST_ENDPOINT: &str = "https://qiongli.dev/updates/v2/stable/macos-aarch64.json";
const BETA_MANIFEST_ENDPOINT: &str = "https://qiongli.dev/updates/v2/beta/macos-aarch64.json";
const ARCHIVE_HOSTS: &[&str] = &[
    "github.com",
    "objects.githubusercontent.com",
    "release-assets.githubusercontent.com",
];
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UpdateCliCommand {
    Status,
    Channel {
        expected_revision: u64,
        stream: UpdateStreamPreference,
    },
    Check,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(untagged)]
pub(crate) enum UpdateCliOutput {
    Status(UpdateStatusOutput),
    Channel(UpdateChannelOutput),
    Check(UpdateCheckOutput),
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

pub(crate) fn execute(
    command: UpdateCliCommand,
    store: &UpdateStateStore,
    authority: Option<&NativeReleaseAuthority>,
    expected_macos_team_id: Option<&str>,
) -> Result<UpdateCliOutput, &'static str> {
    let now_unix = if matches!(command, UpdateCliCommand::Check) {
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
    execute_with_fetcher(command, store, authority, &runtime, &ReqwestManifestFetcher)
}

fn execute_with_fetcher(
    command: UpdateCliCommand,
    store: &UpdateStateStore,
    authority: Option<&NativeReleaseAuthority>,
    runtime: &UpdateRuntimeContext<'_>,
    fetcher: &impl ManifestFetcher,
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
            download: "not-started",
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
            if runtime.os != Some(OperatingSystem::Macos)
                || runtime.arch != Some(Architecture::Aarch64)
            {
                return Err("native-update-target-unsupported");
            }
            if loaded.state.active_transaction.is_some() {
                return Err("native-update-transaction-active");
            }
            let authority = authority.ok_or("native-update-release-authority-unavailable")?;
            let team_id = runtime
                .expected_macos_team_id
                .ok_or("native-update-macos-team-id-unavailable")?;
            let endpoint = manifest_endpoint(loaded.state.selected_stream);
            let bytes = fetcher.fetch(endpoint)?;
            let signed = SignedNativeUpdateManifestV1::from_json(&bytes)
                .map_err(|error| error.reason_code())?;
            let selected_stream = native_stream(loaded.state.selected_stream);
            let authority_floor = authority.minimum_release_generation().saturating_sub(1);
            let context = NativeUpdateVerificationContext {
                now_unix: runtime.now_unix,
                last_accepted_generation: loaded
                    .state
                    .last_accepted_generation
                    .max(authority_floor),
                current_version: runtime.current_version,
                selected_stream,
                expected_macos_team_id: team_id,
                allowed_download_hosts: ARCHIVE_HOSTS,
                allow_current_version: true,
            };
            let verified = signed
                .verify(authority.release_keys(), &context)
                .map_err(|error| error.reason_code())?;
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
    }
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
        if response.status() != StatusCode::OK {
            return Err("native-update-manifest-response-invalid");
        }
        if response
            .headers()
            .get(CONTENT_ENCODING)
            .is_some_and(|value| value.as_bytes() != b"identity")
        {
            return Err("native-update-manifest-encoding-invalid");
        }
        if response.content_length().is_some_and(|length| {
            length > qiongli_platform::MAX_NATIVE_UPDATE_MANIFEST_BYTES as u64
        }) {
            return Err("native-update-manifest-too-large");
        }
        let mut bytes = Vec::new();
        response
            .take((qiongli_platform::MAX_NATIVE_UPDATE_MANIFEST_BYTES + 1) as u64)
            .read_to_end(&mut bytes)
            .map_err(|_| "native-update-manifest-read-failed")?;
        if bytes.len() > qiongli_platform::MAX_NATIVE_UPDATE_MANIFEST_BYTES {
            return Err("native-update-manifest-too-large");
        }
        Ok(bytes)
    }
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

#[cfg(all(test, unix))]
mod tests {
    use std::path::Path;

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

    struct FixedFetcher(Vec<u8>);

    impl ManifestFetcher for FixedFetcher {
        fn fetch(&self, _endpoint: &str) -> Result<Vec<u8>, &'static str> {
            Ok(self.0.clone())
        }
    }

    #[test]
    fn status_and_channel_are_revision_safe_and_do_not_require_release_authority() {
        let (store, root) = store("status");
        let runtime = runtime();
        let status = execute_with_fetcher(
            UpdateCliCommand::Status,
            &store,
            None,
            &runtime,
            &FixedFetcher(Vec::new()),
        )
        .unwrap();
        let json = serde_json::to_value(status).unwrap();
        assert_eq!(json["revision"], 0);
        assert_eq!(json["selected_stream"], "beta");
        assert_eq!(json["release_authority"], "unavailable");
        assert!(!root.exists());

        let changed = execute_with_fetcher(
            UpdateCliCommand::Channel {
                expected_revision: 0,
                stream: UpdateStreamPreference::Stable,
            },
            &store,
            None,
            &runtime,
            &FixedFetcher(Vec::new()),
        )
        .unwrap();
        let json = serde_json::to_value(changed).unwrap();
        assert_eq!(json["revision"], 1);
        assert_eq!(json["selected_stream"], "stable");
        assert_eq!(
            execute_with_fetcher(
                UpdateCliCommand::Channel {
                    expected_revision: 0,
                    stream: UpdateStreamPreference::Beta,
                },
                &store,
                None,
                &runtime,
                &FixedFetcher(Vec::new()),
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
        let output = execute_with_fetcher(
            UpdateCliCommand::Check,
            &store,
            Some(&authority),
            &runtime(),
            &fetcher,
        )
        .unwrap();
        let json = serde_json::to_value(output).unwrap();
        assert_eq!(json["status"], "update-available");
        assert_eq!(json["target_version"], "2.0.0-alpha.2");
        assert_eq!(json["download"], "not-started");
        assert_eq!(json["install"], "not-started");
        assert!(!root.exists());

        assert_eq!(
            execute_with_fetcher(UpdateCliCommand::Check, &store, None, &runtime(), &fetcher,),
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

        let output = execute_with_fetcher(
            UpdateCliCommand::Check,
            &store,
            Some(&authority),
            &runtime(),
            &fetcher,
        )
        .unwrap();
        let json = serde_json::to_value(output).unwrap();
        assert_eq!(json["status"], "current");
        assert_eq!(json["generation"], 2);
        assert_eq!(store.load().unwrap().revision, 1);
        let _ = std::fs::remove_dir_all(root);
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
            archive_size_bytes: 42_000_000,
            archive_sha256: "1".repeat(64),
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
            execute_with_fetcher(
                UpdateCliCommand::Check,
                &store,
                None,
                &runtime(),
                &FixedFetcher(Vec::new()),
            ),
            Err("native-update-transaction-active")
        );
        let _ = std::fs::remove_dir_all(root);
    }
}
