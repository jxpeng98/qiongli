use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::ffi::OsString;
use std::fs::{self, File, OpenOptions};
use std::io::{Read as _, Write as _};
use std::path::{Component, Path, PathBuf};

use ed25519_dalek::{Signer as _, SigningKey};
use qiongli_platform::{
    NativeCommunityAlphaCandidateSetV1, NativeCommunityAlphaIntegrityManifestV1,
    NativeCommunityAlphaPublicationReceiptV1, NativeCommunityAlphaTargetPromotionV1,
    NativeDistributionClass, NativePublicationAuthorizationContext,
    NativePublicationAuthorizationV1, NativeReleaseAuthority, NativeReleaseSignatureV1,
    SignatureAlgorithm, SignedNativeCommunityAlphaIntegrityV1,
    native_community_alpha_integrity_signing_bytes,
};
use serde::Serialize;
use serde_json::{Value, json};
use sha2::{Digest as _, Sha256};
use zeroize::{Zeroize as _, Zeroizing};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const RELEASE_KEY_ID: &str = "community-alpha-release-1";
const LAUNCH_KEY_ID: &str = "community-alpha-launch-1";
const RELEASE_PRIVATE_KEY_ENV: &str = "QIONGLI_ALPHA_RELEASE_PRIVATE_KEY_HEX";
const CANDIDATE_SET_FILE: &str = "qiongli-community-alpha-candidate-set.json";
const AUTHORITY_FILE: &str = "qiongli-native-release-authority.json";
const INTEGRITY_FILE: &str = "qiongli-community-alpha-integrity.json";
const PUBLICATION_RECEIPT_FILE: &str = "qiongli-community-alpha-publication-authorization.json";
const MAX_ASSET_BYTES: u64 = 1024 * 1024 * 1024;
const MAX_JSON_BYTES: u64 = 2 * 1024 * 1024;
const MAX_LOCK_BYTES: u64 = 8 * 1024 * 1024;
const MAX_NOTES_BYTES: u64 = 256 * 1024;

fn main() {
    if let Err(code) = run() {
        eprintln!("error: {code}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), &'static str> {
    match Command::parse(env::args_os().skip(1))? {
        Command::Keygen(arguments) => generate_keys(&arguments),
        Command::Prepare(arguments) => prepare_signed_release(&arguments),
        Command::AuthorizeCandidate(arguments) => authorize_candidate(&arguments),
        Command::Authorize(arguments) => authorize_release(&arguments),
        Command::Verify(arguments) => verify_release(&arguments),
    }
}

enum Command {
    Keygen(KeygenArguments),
    Prepare(PrepareArguments),
    AuthorizeCandidate(AuthorizeCandidateArguments),
    Authorize(AuthorizeArguments),
    Verify(VerifyArguments),
}

impl Command {
    fn parse(values: impl IntoIterator<Item = OsString>) -> Result<Self, &'static str> {
        let mut values = values.into_iter();
        let command = values
            .next()
            .and_then(|value| value.into_string().ok())
            .ok_or("community-alpha-release-usage-invalid")?;
        let options = OptionMap::parse(values)?;
        match command.as_str() {
            "keygen" => Ok(Self::Keygen(KeygenArguments::parse(options)?)),
            "prepare" => Ok(Self::Prepare(PrepareArguments::parse(options)?)),
            "authorize-candidate" => Ok(Self::AuthorizeCandidate(
                AuthorizeCandidateArguments::parse(options)?,
            )),
            "authorize" => Ok(Self::Authorize(AuthorizeArguments::parse(options)?)),
            "verify" => Ok(Self::Verify(VerifyArguments::parse(options)?)),
            _ => Err("community-alpha-release-usage-invalid"),
        }
    }
}

struct KeygenArguments {
    release_private: PathBuf,
    launch_private: PathBuf,
    authority: PathBuf,
}

impl KeygenArguments {
    fn parse(mut options: OptionMap) -> Result<Self, &'static str> {
        let arguments = Self {
            release_private: options.path("--release-private")?,
            launch_private: options.path("--launch-private")?,
            authority: options.path("--authority")?,
        };
        options.finish()?;
        for path in [
            &arguments.release_private,
            &arguments.launch_private,
            &arguments.authority,
        ] {
            validate_new_output_file(path)?;
        }
        Ok(arguments)
    }
}

struct PrepareArguments {
    candidate: PathBuf,
    authority: PathBuf,
    cargo_lock: PathBuf,
    output: PathBuf,
}

impl PrepareArguments {
    fn parse(mut options: OptionMap) -> Result<Self, &'static str> {
        let arguments = Self {
            candidate: options.path("--candidate")?,
            authority: options.path("--authority")?,
            cargo_lock: options.path("--cargo-lock")?,
            output: options.path("--output")?,
        };
        options.finish()?;
        validate_input_directory(&arguments.candidate)?;
        validate_input_file(&arguments.authority, MAX_JSON_BYTES)?;
        validate_input_file(&arguments.cargo_lock, MAX_LOCK_BYTES)?;
        validate_new_output_directory(&arguments.output)?;
        Ok(arguments)
    }
}

struct AuthorizeArguments {
    release_dir: PathBuf,
    authorization_file: PathBuf,
    repository: String,
    environment: String,
    workflow_run_url: String,
    actor: String,
    verified_at_unix: u64,
}

impl AuthorizeArguments {
    fn parse(mut options: OptionMap) -> Result<Self, &'static str> {
        let arguments = Self {
            release_dir: options.path("--release-dir")?,
            authorization_file: options.path("--authorization-file")?,
            repository: options.text("--repository")?,
            environment: options.text("--environment")?,
            workflow_run_url: options.text("--workflow-run-url")?,
            actor: options.text("--actor")?,
            verified_at_unix: options.u64("--verified-at-unix")?,
        };
        options.finish()?;
        validate_input_directory(&arguments.release_dir)?;
        validate_input_file(&arguments.authorization_file, MAX_JSON_BYTES)?;
        if arguments
            .release_dir
            .join(PUBLICATION_RECEIPT_FILE)
            .exists()
        {
            return Err("community-alpha-release-authorization-output-exists");
        }
        Ok(arguments)
    }
}

struct AuthorizeCandidateArguments {
    candidate: PathBuf,
    repository: String,
    environment: String,
    workflow_run_url: String,
    actor: String,
    authorized_at_unix: u64,
    output: PathBuf,
}

impl AuthorizeCandidateArguments {
    fn parse(mut options: OptionMap) -> Result<Self, &'static str> {
        let arguments = Self {
            candidate: options.path("--candidate")?,
            repository: options.text("--repository")?,
            environment: options.text("--environment")?,
            workflow_run_url: options.text("--workflow-run-url")?,
            actor: options.text("--actor")?,
            authorized_at_unix: options.u64("--authorized-at-unix")?,
            output: options.path("--output")?,
        };
        options.finish()?;
        validate_input_directory(&arguments.candidate)?;
        validate_new_output_file(&arguments.output)?;
        Ok(arguments)
    }
}

struct VerifyArguments {
    release_dir: PathBuf,
    require_authorization: bool,
}

impl VerifyArguments {
    fn parse(mut options: OptionMap) -> Result<Self, &'static str> {
        let arguments = Self {
            release_dir: options.path("--release-dir")?,
            require_authorization: options.flag("--require-authorization")?,
        };
        options.finish()?;
        validate_input_directory(&arguments.release_dir)?;
        Ok(arguments)
    }
}

struct OptionMap(BTreeMap<String, Option<OsString>>);

impl OptionMap {
    fn parse(values: impl IntoIterator<Item = OsString>) -> Result<Self, &'static str> {
        let values = values.into_iter().collect::<Vec<_>>();
        let mut options = BTreeMap::new();
        let mut index = 0;
        while index < values.len() {
            let name = values[index]
                .to_str()
                .filter(|value| value.starts_with("--"))
                .ok_or("community-alpha-release-usage-invalid")?
                .to_string();
            let value = values.get(index + 1).and_then(|value| {
                value
                    .to_str()
                    .filter(|value| !value.starts_with("--"))
                    .map(|_| value.clone())
            });
            index += if value.is_some() { 2 } else { 1 };
            if options.insert(name, value).is_some() {
                return Err("community-alpha-release-usage-invalid");
            }
        }
        Ok(Self(options))
    }

    fn text(&mut self, name: &str) -> Result<String, &'static str> {
        self.0
            .remove(name)
            .flatten()
            .and_then(|value| value.into_string().ok())
            .filter(|value| !value.is_empty())
            .ok_or("community-alpha-release-usage-invalid")
    }

    fn path(&mut self, name: &str) -> Result<PathBuf, &'static str> {
        self.0
            .remove(name)
            .flatten()
            .map(PathBuf::from)
            .ok_or("community-alpha-release-usage-invalid")
    }

    fn u64(&mut self, name: &str) -> Result<u64, &'static str> {
        self.text(name)?
            .parse::<u64>()
            .ok()
            .filter(|value| *value > 0)
            .ok_or("community-alpha-release-usage-invalid")
    }

    fn flag(&mut self, name: &str) -> Result<bool, &'static str> {
        match self.0.remove(name) {
            None => Ok(false),
            Some(None) => Ok(true),
            Some(Some(_)) => Err("community-alpha-release-usage-invalid"),
        }
    }

    fn finish(self) -> Result<(), &'static str> {
        if self.0.is_empty() {
            Ok(())
        } else {
            Err("community-alpha-release-usage-invalid")
        }
    }
}

fn generate_keys(arguments: &KeygenArguments) -> Result<(), &'static str> {
    let mut release_secret = Zeroizing::new([0_u8; 32]);
    let mut launch_secret = Zeroizing::new([0_u8; 32]);
    getrandom::fill(&mut *release_secret).map_err(|_| "community-alpha-release-random-failed")?;
    getrandom::fill(&mut *launch_secret).map_err(|_| "community-alpha-release-random-failed")?;
    if release_secret.as_ref() == launch_secret.as_ref() {
        return Err("community-alpha-release-random-failed");
    }
    let release_key = SigningKey::from_bytes(&release_secret);
    let launch_key = SigningKey::from_bytes(&launch_secret);
    let authority_value = json!({
        "channel": "alpha",
        "launch_grant_keys": [{
            "key_id": LAUNCH_KEY_ID,
            "public_key_hex": encode_hex(launch_key.verifying_key().as_bytes())
        }],
        "minimum_launch_grant_generation": 1,
        "minimum_release_generation": 1,
        "release_keys": [{
            "key_id": RELEASE_KEY_ID,
            "maximum_generation_exclusive": null,
            "minimum_generation": 1,
            "public_key_hex": encode_hex(release_key.verifying_key().as_bytes())
        }],
        "schema_version": 1
    });
    let authority_bytes = canonical_json(&authority_value)?;
    NativeReleaseAuthority::from_json(&authority_bytes)
        .map_err(|_| "community-alpha-release-authority-invalid")?;
    write_new_private_file(
        &arguments.release_private,
        encode_hex(release_secret.as_ref()).as_bytes(),
    )?;
    write_new_private_file(
        &arguments.launch_private,
        encode_hex(launch_secret.as_ref()).as_bytes(),
    )?;
    write_new_private_file(&arguments.authority, &authority_bytes)?;
    println!(
        "{}",
        String::from_utf8(canonical_json(&json!({
            "authority_sha256": sha256_hex(&authority_bytes),
            "release_key_id": RELEASE_KEY_ID,
            "status": "generated"
        }))?)
        .map_err(|_| "community-alpha-release-output-invalid")?
    );
    Ok(())
}

fn prepare_signed_release(arguments: &PrepareArguments) -> Result<(), &'static str> {
    let candidate_bytes = read_input_file(
        &arguments.candidate.join(CANDIDATE_SET_FILE),
        MAX_JSON_BYTES,
    )?;
    let candidate = NativeCommunityAlphaCandidateSetV1::from_json(&candidate_bytes)
        .map_err(|error| error.reason_code())?;
    if candidate.content.version != VERSION {
        return Err("community-alpha-release-version-mismatch");
    }
    verify_embedded_source(&candidate.content.source_commit)?;
    verify_candidate_directory(&arguments.candidate, &candidate)?;

    let authority_bytes = read_authority_file(&arguments.authority)?;
    let authority = NativeReleaseAuthority::from_json(&authority_bytes)
        .map_err(|_| "community-alpha-release-authority-invalid")?;
    if authority.to_canonical_json().ok().as_deref() != Some(authority_bytes.as_slice())
        || qiongli::embedded_release_authority()
            .map_err(|_| "community-alpha-release-authority-invalid")?
            .and_then(|embedded| embedded.to_canonical_json().ok())
            .as_deref()
            != Some(authority_bytes.as_slice())
    {
        return Err("community-alpha-release-authority-unbound");
    }

    let cargo_lock = read_input_file(&arguments.cargo_lock, MAX_LOCK_BYTES)?;
    let packages = parse_cargo_lock(&cargo_lock)?;
    let release_notes = release_notes(&candidate);
    let sbom = cyclonedx_sbom(&candidate, &packages)?;
    let provenance = provenance_statement(&candidate)?;
    let checksums_file = format!("qiongli-{VERSION}-community-alpha.SHA256SUMS");
    let sbom_file = format!("qiongli-{VERSION}-community-alpha.cdx.json");
    let provenance_file = format!("qiongli-{VERSION}-community-alpha.provenance.json");
    let notes_file = format!("qiongli-{VERSION}-community-alpha.release-notes.md");

    create_private_directory(&arguments.output)?;
    let result = (|| {
        let public_source = arguments.candidate.join("public");
        for asset in candidate
            .content
            .targets
            .iter()
            .flat_map(|target| &target.assets)
        {
            copy_verified_file(
                &public_source.join(&asset.file),
                &arguments.output.join(&asset.file),
                asset.size_bytes,
                &asset.sha256,
                MAX_ASSET_BYTES,
            )?;
        }
        write_new_private_file(&arguments.output.join(CANDIDATE_SET_FILE), &candidate_bytes)?;
        write_new_private_file(&arguments.output.join(AUTHORITY_FILE), &authority_bytes)?;
        write_new_private_file(&arguments.output.join(&sbom_file), &sbom)?;
        write_new_private_file(&arguments.output.join(&provenance_file), &provenance)?;
        write_new_private_file(
            &arguments.output.join(&notes_file),
            release_notes.as_bytes(),
        )?;

        let checksums = checksums_document(
            &arguments.output,
            &candidate,
            [
                CANDIDATE_SET_FILE,
                AUTHORITY_FILE,
                sbom_file.as_str(),
                provenance_file.as_str(),
                notes_file.as_str(),
            ],
        )?;
        write_new_private_file(&arguments.output.join(&checksums_file), &checksums)?;

        let manifest = NativeCommunityAlphaIntegrityManifestV1::from_candidate(
            &candidate,
            1,
            sha256_hex(&candidate_bytes),
            sha256_hex(&authority_bytes),
            sha256_hex(&checksums),
            sha256_hex(&sbom),
            sha256_hex(&provenance),
            sha256_hex(release_notes.as_bytes()),
        )
        .map_err(|error| error.reason_code())?;
        let signing_key = release_signing_key()?;
        let signature = signing_key.sign(
            &native_community_alpha_integrity_signing_bytes(&manifest)
                .map_err(|error| error.reason_code())?,
        );
        let signed = SignedNativeCommunityAlphaIntegrityV1 {
            manifest,
            signature: NativeReleaseSignatureV1 {
                algorithm: SignatureAlgorithm::Ed25519,
                key_id: RELEASE_KEY_ID.to_string(),
                value_hex: encode_hex(&signature.to_bytes()),
            },
        };
        signed
            .verify(&authority)
            .map_err(|error| error.reason_code())?;
        write_new_private_file(
            &arguments.output.join(INTEGRITY_FILE),
            &signed
                .to_canonical_json()
                .map_err(|error| error.reason_code())?,
        )?;
        verify_release_directory(&arguments.output, false).map(|_| ())
    })();
    if result.is_err() {
        let _ = fs::remove_dir_all(&arguments.output);
    }
    result?;
    println!(
        "{}",
        String::from_utf8(canonical_json(&json!({
            "publication_allowed": false,
            "release_set_sha256": candidate.candidate_set_sha256,
            "source_commit": candidate.content.source_commit,
            "status": "signed-integrity-ready"
        }))?)
        .map_err(|_| "community-alpha-release-output-invalid")?
    );
    Ok(())
}

fn authorize_release(arguments: &AuthorizeArguments) -> Result<(), &'static str> {
    let verified = verify_release_directory(&arguments.release_dir, false)?;
    let signed_bytes =
        read_input_file(&arguments.release_dir.join(INTEGRITY_FILE), MAX_JSON_BYTES)?;
    let release_set = &verified.signed().manifest.release_set;
    let authorization_bytes = read_input_file(&arguments.authorization_file, MAX_JSON_BYTES)?;
    let authorization = NativePublicationAuthorizationV1::from_json(&authorization_bytes)
        .map_err(|error| error.reason_code())?;
    let context = NativePublicationAuthorizationContext {
        expected_distribution_class: NativeDistributionClass::CommunityAlpha,
        expected_source_commit: &release_set.source_commit,
        expected_release_set_sha256: &release_set.release_set_sha256,
        expected_repository: &arguments.repository,
        expected_environment: &arguments.environment,
        expected_workflow_run_url: &arguments.workflow_run_url,
        expected_actor: &arguments.actor,
        verified_at_unix: arguments.verified_at_unix,
        max_authorization_age_seconds: 86_400,
    };
    let receipt = NativeCommunityAlphaPublicationReceiptV1::authorize(
        &verified,
        sha256_hex(&signed_bytes),
        authorization,
        &context,
    )
    .map_err(|error| error.reason_code())?;
    write_new_private_file(
        &arguments.release_dir.join(PUBLICATION_RECEIPT_FILE),
        &receipt
            .to_canonical_json()
            .map_err(|error| error.reason_code())?,
    )?;
    verify_release_directory(&arguments.release_dir, true)?;
    println!(
        "{}",
        String::from_utf8(
            receipt
                .to_canonical_json()
                .map_err(|error| error.reason_code())?
        )
        .map_err(|_| "community-alpha-release-output-invalid")?
    );
    Ok(())
}

fn authorize_candidate(arguments: &AuthorizeCandidateArguments) -> Result<(), &'static str> {
    let candidate_bytes = read_input_file(
        &arguments.candidate.join(CANDIDATE_SET_FILE),
        MAX_JSON_BYTES,
    )?;
    let candidate = NativeCommunityAlphaCandidateSetV1::from_json(&candidate_bytes)
        .map_err(|error| error.reason_code())?;
    if candidate.content.version != VERSION {
        return Err("community-alpha-release-version-mismatch");
    }
    verify_embedded_source(&candidate.content.source_commit)?;
    verify_candidate_directory(&arguments.candidate, &candidate)?;
    let authorization = NativePublicationAuthorizationV1::exact_release_set(
        NativeDistributionClass::CommunityAlpha,
        &candidate.content.source_commit,
        &candidate.candidate_set_sha256,
        &arguments.repository,
        &arguments.environment,
        &arguments.workflow_run_url,
        &arguments.actor,
        arguments.authorized_at_unix,
    )
    .map_err(|error| error.reason_code())?;
    let bytes = authorization
        .to_canonical_json()
        .map_err(|error| error.reason_code())?;
    write_new_private_file(&arguments.output, &bytes)?;
    println!(
        "{}",
        String::from_utf8(bytes).map_err(|_| "community-alpha-release-output-invalid")?
    );
    Ok(())
}

fn verify_release(arguments: &VerifyArguments) -> Result<(), &'static str> {
    let verified =
        verify_release_directory(&arguments.release_dir, arguments.require_authorization)?;
    println!(
        "{}",
        String::from_utf8(canonical_json(&json!({
            "publication_authorization": arguments.require_authorization,
            "release_key_id": verified.signed().signature.key_id,
            "release_set_sha256": verified.signed().manifest.release_set.release_set_sha256,
            "source_commit": verified.signed().manifest.release_set.source_commit,
            "status": "verified"
        }))?)
        .map_err(|_| "community-alpha-release-output-invalid")?
    );
    Ok(())
}

fn verify_release_directory(
    directory: &Path,
    require_authorization: bool,
) -> Result<qiongli_platform::VerifiedNativeCommunityAlphaIntegrity, &'static str> {
    let signed_bytes = read_input_file(&directory.join(INTEGRITY_FILE), MAX_JSON_BYTES)?;
    let signed = SignedNativeCommunityAlphaIntegrityV1::from_json(&signed_bytes)
        .map_err(|error| error.reason_code())?;
    let authority_bytes = read_authority_file(&directory.join(AUTHORITY_FILE))?;
    let authority = NativeReleaseAuthority::from_json(&authority_bytes)
        .map_err(|_| "community-alpha-release-authority-invalid")?;
    let verified = signed
        .verify(&authority)
        .map_err(|error| error.reason_code())?;
    let manifest = &signed.manifest;
    verify_digest(
        &directory.join(CANDIDATE_SET_FILE),
        MAX_JSON_BYTES,
        &manifest.candidate_set_file_sha256,
    )?;
    let candidate_bytes = read_input_file(&directory.join(CANDIDATE_SET_FILE), MAX_JSON_BYTES)?;
    let candidate = NativeCommunityAlphaCandidateSetV1::from_json(&candidate_bytes)
        .map_err(|error| error.reason_code())?;
    if candidate.candidate_set_sha256 != manifest.release_set.release_set_sha256
        || candidate.content.source_commit != manifest.release_set.source_commit
        || candidate.content.version != manifest.release_set.version
    {
        return Err("community-alpha-release-candidate-mismatch");
    }
    verify_digest(
        &directory.join(AUTHORITY_FILE),
        MAX_JSON_BYTES,
        &manifest.authority_sha256,
    )?;
    verify_digest(
        &directory.join(&manifest.checksums_file),
        MAX_JSON_BYTES,
        &manifest.checksums_sha256,
    )?;
    verify_digest(
        &directory.join(&manifest.sbom_file),
        MAX_JSON_BYTES,
        &manifest.sbom_sha256,
    )?;
    verify_digest(
        &directory.join(&manifest.provenance_file),
        MAX_JSON_BYTES,
        &manifest.provenance_sha256,
    )?;
    verify_digest(
        &directory.join(&manifest.release_notes_file),
        MAX_NOTES_BYTES,
        &manifest.release_notes_sha256,
    )?;
    for asset in &manifest.assets {
        verify_file(
            &directory.join(&asset.file),
            asset.size_bytes,
            &asset.sha256,
            MAX_ASSET_BYTES,
        )?;
    }
    verify_checksums(directory, &candidate, manifest)?;
    verify_sbom(directory, &candidate, manifest)?;
    verify_provenance(directory, &candidate, manifest)?;
    verify_release_notes(directory, &candidate, manifest)?;
    let mut expected = manifest
        .assets
        .iter()
        .map(|asset| asset.file.clone())
        .collect::<BTreeSet<_>>();
    expected.extend([
        CANDIDATE_SET_FILE.to_string(),
        AUTHORITY_FILE.to_string(),
        manifest.checksums_file.clone(),
        manifest.sbom_file.clone(),
        manifest.provenance_file.clone(),
        manifest.release_notes_file.clone(),
        INTEGRITY_FILE.to_string(),
    ]);
    if require_authorization {
        expected.insert(PUBLICATION_RECEIPT_FILE.to_string());
        let receipt_bytes =
            read_input_file(&directory.join(PUBLICATION_RECEIPT_FILE), MAX_JSON_BYTES)?;
        let receipt = NativeCommunityAlphaPublicationReceiptV1::from_json(&receipt_bytes)
            .map_err(|error| error.reason_code())?;
        if receipt.source_commit != manifest.release_set.source_commit
            || receipt.release_set_sha256 != manifest.release_set.release_set_sha256
            || receipt.integrity_sha256 != sha256_hex(&signed_bytes)
            || receipt.tag != format!("v{}", manifest.release_set.version)
        {
            return Err("community-alpha-release-authorization-mismatch");
        }
    }
    if exact_entry_names(directory)? != expected {
        return Err("community-alpha-release-directory-drift");
    }
    Ok(verified)
}

fn verify_candidate_directory(
    directory: &Path,
    candidate: &NativeCommunityAlphaCandidateSetV1,
) -> Result<(), &'static str> {
    if exact_entry_names(directory)?
        != BTreeSet::from([
            CANDIDATE_SET_FILE.to_string(),
            "evidence".to_string(),
            "public".to_string(),
        ])
    {
        return Err("community-alpha-release-candidate-directory-drift");
    }
    let public = directory.join("public");
    validate_input_directory(&public)?;
    let expected = candidate
        .content
        .targets
        .iter()
        .flat_map(|target| &target.assets)
        .map(|asset| asset.file.clone())
        .collect::<BTreeSet<_>>();
    if exact_entry_names(&public)? != expected {
        return Err("community-alpha-release-candidate-directory-drift");
    }
    for asset in candidate
        .content
        .targets
        .iter()
        .flat_map(|target| &target.assets)
    {
        verify_file(
            &public.join(&asset.file),
            asset.size_bytes,
            &asset.sha256,
            MAX_ASSET_BYTES,
        )?;
    }
    let evidence = directory.join("evidence");
    validate_input_directory(&evidence)?;
    let labels = ["macos-aarch64", "windows-x86-64", "linux-x86-64"];
    if exact_entry_names(&evidence)?
        != labels
            .iter()
            .map(ToString::to_string)
            .collect::<BTreeSet<_>>()
    {
        return Err("community-alpha-release-candidate-directory-drift");
    }
    for (target, label) in candidate.content.targets.iter().zip(labels) {
        let target_evidence = evidence.join(label);
        validate_input_directory(&target_evidence)?;
        let mut expected = target
            .evidence
            .iter()
            .map(|item| item.file.clone())
            .collect::<BTreeSet<_>>();
        expected.insert("qiongli-community-alpha-target-promotion.json".to_string());
        if exact_entry_names(&target_evidence)? != expected {
            return Err("community-alpha-release-candidate-directory-drift");
        }
        for item in &target.evidence {
            verify_file(
                &target_evidence.join(&item.file),
                item.size_bytes,
                &item.sha256,
                MAX_JSON_BYTES,
            )?;
        }
        let promotion_bytes = read_input_file(
            &target_evidence.join("qiongli-community-alpha-target-promotion.json"),
            MAX_JSON_BYTES,
        )?;
        let promotion = NativeCommunityAlphaTargetPromotionV1::from_json(&promotion_bytes)
            .map_err(|error| error.reason_code())?;
        if &promotion != target {
            return Err("community-alpha-release-candidate-evidence-mismatch");
        }
    }
    Ok(())
}

fn release_signing_key() -> Result<SigningKey, &'static str> {
    let mut secret = Zeroizing::new(
        env::var(RELEASE_PRIVATE_KEY_ENV)
            .map_err(|_| "community-alpha-release-private-key-missing")?,
    );
    let mut bytes = Zeroizing::new(
        decode_fixed_hex::<32>(&secret).ok_or("community-alpha-release-private-key-invalid")?,
    );
    secret.zeroize();
    let signing = SigningKey::from_bytes(&bytes);
    bytes.zeroize();
    Ok(signing)
}

#[derive(Clone, Debug)]
struct LockedPackage {
    name: String,
    version: String,
    source: Option<String>,
    checksum: Option<String>,
}

fn parse_cargo_lock(bytes: &[u8]) -> Result<Vec<LockedPackage>, &'static str> {
    let text = std::str::from_utf8(bytes).map_err(|_| "community-alpha-release-lock-invalid")?;
    let mut packages = Vec::new();
    let mut current = BTreeMap::<String, String>::new();
    let flush = |current: &mut BTreeMap<String, String>,
                 packages: &mut Vec<LockedPackage>|
     -> Result<(), &'static str> {
        if current.is_empty() {
            return Ok(());
        }
        let name = current
            .remove("name")
            .ok_or("community-alpha-release-lock-invalid")?;
        let version = current
            .remove("version")
            .ok_or("community-alpha-release-lock-invalid")?;
        packages.push(LockedPackage {
            name,
            version,
            source: current.remove("source"),
            checksum: current.remove("checksum"),
        });
        current.clear();
        Ok(())
    };
    let mut in_package = false;
    for line in text.lines() {
        let line = line.trim();
        if line == "[[package]]" {
            if in_package {
                flush(&mut current, &mut packages)?;
            }
            in_package = true;
            continue;
        }
        if !in_package || line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') {
            flush(&mut current, &mut packages)?;
            in_package = false;
            continue;
        }
        if let Some((key, value)) = line.split_once(" = ")
            && matches!(key, "name" | "version" | "source" | "checksum")
        {
            current.insert(key.to_string(), parse_quoted(value)?);
        }
    }
    if in_package {
        flush(&mut current, &mut packages)?;
    }
    packages.sort_by(|left, right| {
        (&left.name, &left.version, &left.source).cmp(&(&right.name, &right.version, &right.source))
    });
    if packages.is_empty()
        || packages.windows(2).any(|pair| {
            (&pair[0].name, &pair[0].version, &pair[0].source)
                == (&pair[1].name, &pair[1].version, &pair[1].source)
        })
    {
        return Err("community-alpha-release-lock-invalid");
    }
    Ok(packages)
}

fn parse_quoted(value: &str) -> Result<String, &'static str> {
    serde_json::from_str::<String>(value).map_err(|_| "community-alpha-release-lock-invalid")
}

fn cyclonedx_sbom(
    candidate: &NativeCommunityAlphaCandidateSetV1,
    packages: &[LockedPackage],
) -> Result<Vec<u8>, &'static str> {
    let components = packages
        .iter()
        .map(|package| {
            let purl = format!(
                "pkg:cargo/{}@{}",
                percent_encode(&package.name),
                percent_encode(&package.version)
            );
            let mut component = json!({
                "bom-ref": purl,
                "name": package.name,
                "purl": purl,
                "type": "library",
                "version": package.version
            });
            if let Some(checksum) = &package.checksum {
                component["hashes"] = json!([{"alg": "SHA-256", "content": checksum}]);
            }
            if let Some(source) = &package.source {
                component["externalReferences"] = json!([{
                    "type": "distribution",
                    "url": source
                }]);
            }
            component
        })
        .collect::<Vec<_>>();
    canonical_json(&json!({
        "bomFormat": "CycloneDX",
        "components": components,
        "metadata": {
            "component": {
                "bom-ref": format!("pkg:generic/qiongli@{VERSION}?download_url=https://github.com/jxpeng98/qiongli/releases/tag/v{VERSION}"),
                "name": "qiongli",
                "type": "application",
                "version": VERSION
            },
            "properties": [
                {"name": "qiongli:distribution-class", "value": "community-alpha"},
                {"name": "qiongli:release-set-sha256", "value": candidate.candidate_set_sha256},
                {"name": "qiongli:source-commit", "value": candidate.content.source_commit}
            ]
        },
        "serialNumber": format!("urn:uuid:{}", deterministic_uuid(candidate.candidate_set_sha256.as_bytes())),
        "specVersion": "1.6",
        "version": 1
    }))
}

fn provenance_statement(
    candidate: &NativeCommunityAlphaCandidateSetV1,
) -> Result<Vec<u8>, &'static str> {
    let subjects = candidate
        .content
        .targets
        .iter()
        .flat_map(|target| &target.assets)
        .map(|asset| json!({"digest": {"sha256": asset.sha256}, "name": asset.file}))
        .collect::<Vec<_>>();
    canonical_json(&json!({
        "_type": "https://in-toto.io/Statement/v1",
        "predicate": {
            "buildDefinition": {
                "buildType": "https://github.com/jxpeng98/qiongli/blob/2.x/docs/architecture/decisions/0208-community-alpha-distribution-boundary.md",
                "externalParameters": {
                    "candidate_set_sha256": candidate.candidate_set_sha256,
                    "distribution_class": "community-alpha",
                    "source_commit": candidate.content.source_commit,
                    "version": candidate.content.version
                },
                "internalParameters": {
                    "raw_ci_artifact_reused": false,
                    "target_native_build": true
                },
                "resolvedDependencies": [{
                    "digest": {"gitCommit": candidate.content.source_commit},
                    "uri": "git+https://github.com/jxpeng98/qiongli@refs/heads/2.x"
                }]
            },
            "runDetails": {
                "builder": {"id": candidate.content.build_run_url},
                "metadata": {"invocationId": candidate.content.build_run_url}
            }
        },
        "predicateType": "https://slsa.dev/provenance/v1",
        "subject": subjects
    }))
}

fn release_notes(candidate: &NativeCommunityAlphaCandidateSetV1) -> String {
    format!(
        "# Qiongli {VERSION} Community Alpha\n\n\
         **Distribution class:** `community-alpha — not platform-trusted`  \n\
         **Source:** `{source}`  \n\
         **Release set:** `{release_set}`\n\n\
         Qiongli 2 Community Alpha provides a dependency-free native research workspace. The CLI,\n\
         desktop manager, embedded Skills, Lite and Full MCP surfaces, Academic Graph, receipt-backed\n\
         Codex/Claude Code integration, and the Stable/Beta self-update engine run without a\n\
         user-installed Rust, Python, Node.js, Cargo, npm, or pip.\n\n\
         ## Downloads and platform warnings\n\n\
         - **macOS arm64:** use the DMG for first installation. The app is ad-hoc signed, not\n\
           Developer ID signed or notarized. Gatekeeper may require the per-app **Open Anyway** flow.\n\
         - **Windows x86_64:** extract the complete ZIP before launch. It is not Authenticode signed;\n\
           SmartScreen, Smart App Control, antivirus, or enterprise policy may warn or block it.\n\
         - **Linux x86_64:** use the AppImage for the desktop app, or the AppDir ZIP for the full CLI.\n\
           The host must provide the facilities required by the advertised AppImage.\n\n\
         Do not disable global operating-system security controls or import a self-signed Windows root.\n\n\
         ## 下载与平台提示\n\n\
         - **macOS arm64：**首次安装使用 DMG。本版本采用 ad-hoc 签名，没有 Developer ID\n\
           签名或公证；Gatekeeper 可能要求对该 App 使用“仍要打开”。\n\
         - **Windows x86_64：**请先完整解压 ZIP。本版本没有 Authenticode 签名，可能被\n\
           SmartScreen、Smart App Control、杀毒软件或企业策略警告或阻止。\n\
         - **Linux x86_64：**AppImage 用于桌面应用；需要完整 CLI 时使用 AppDir ZIP。\n\
           系统必须具备 AppImage 所声明的运行条件。\n\n\
         请勿关闭全局安全机制，也不要导入自签名 Windows 根证书。\n\n\
         ## Community Alpha scope\n\n\
         - Academic Graph and Full MCP provide bounded, project-owned read and workflow surfaces;\n\
           arbitrary-directory inference, unrestricted Full MCP mutation, and cloud execution are not claimed.\n\
           The only Full MCP project write is previewed, digest-bound, and explicitly approved.\n\
         - Automatic updates remain conditional on separately published, signed update metadata and\n\
           target-native acceptance. Source builds remain inspect-only for client-owned mutations.\n\
         - Claude Desktop, Codex Desktop, ChatGPT Marketplace bypass, cloud execution, and public\n\
           Marketplace distribution are not included.\n\n\
         Verify downloads with the SHA-256 inventory, CycloneDX SBOM, SLSA provenance, public\n\
         authority, and signed Community Alpha integrity record attached to this pre-release.\n",
        source = candidate.content.source_commit,
        release_set = candidate.candidate_set_sha256
    )
}

fn checksums_document<const N: usize>(
    directory: &Path,
    candidate: &NativeCommunityAlphaCandidateSetV1,
    metadata_files: [&str; N],
) -> Result<Vec<u8>, &'static str> {
    let mut files = candidate
        .content
        .targets
        .iter()
        .flat_map(|target| target.assets.iter().map(|asset| asset.file.as_str()))
        .chain(metadata_files)
        .collect::<Vec<_>>();
    files.sort_unstable();
    if files.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err("community-alpha-release-checksums-invalid");
    }
    let mut output = String::new();
    for file in files {
        let digest = sha256_file(&directory.join(file), MAX_ASSET_BYTES)?.1;
        output.push_str(&digest);
        output.push_str("  ");
        output.push_str(file);
        output.push('\n');
    }
    Ok(output.into_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_release_metadata_names_follow_the_workspace_version() {
        assert_eq!(VERSION, env!("CARGO_PKG_VERSION"));
        for file in [
            format!("qiongli-{VERSION}-community-alpha.SHA256SUMS"),
            format!("qiongli-{VERSION}-community-alpha.cdx.json"),
            format!("qiongli-{VERSION}-community-alpha.provenance.json"),
            format!("qiongli-{VERSION}-community-alpha.release-notes.md"),
        ] {
            assert!(file.contains(VERSION));
        }
    }
}

fn verify_checksums(
    directory: &Path,
    candidate: &NativeCommunityAlphaCandidateSetV1,
    manifest: &NativeCommunityAlphaIntegrityManifestV1,
) -> Result<(), &'static str> {
    let expected = checksums_document(
        directory,
        candidate,
        [
            CANDIDATE_SET_FILE,
            AUTHORITY_FILE,
            manifest.sbom_file.as_str(),
            manifest.provenance_file.as_str(),
            manifest.release_notes_file.as_str(),
        ],
    )?;
    let actual = read_input_file(&directory.join(&manifest.checksums_file), MAX_JSON_BYTES)?;
    if actual != expected {
        return Err("community-alpha-release-checksums-invalid");
    }
    Ok(())
}

fn verify_sbom(
    directory: &Path,
    candidate: &NativeCommunityAlphaCandidateSetV1,
    manifest: &NativeCommunityAlphaIntegrityManifestV1,
) -> Result<(), &'static str> {
    let bytes = read_input_file(&directory.join(&manifest.sbom_file), MAX_JSON_BYTES)?;
    let value: Value = parse_canonical_json(&bytes)?;
    if value["bomFormat"] != "CycloneDX"
        || value["specVersion"] != "1.6"
        || value["metadata"]["properties"]
            .as_array()
            .is_none_or(|properties| {
                !properties.iter().any(|property| {
                    property["name"] == "qiongli:release-set-sha256"
                        && property["value"] == candidate.candidate_set_sha256
                })
            })
    {
        return Err("community-alpha-release-sbom-invalid");
    }
    Ok(())
}

fn verify_provenance(
    directory: &Path,
    candidate: &NativeCommunityAlphaCandidateSetV1,
    manifest: &NativeCommunityAlphaIntegrityManifestV1,
) -> Result<(), &'static str> {
    let bytes = read_input_file(&directory.join(&manifest.provenance_file), MAX_JSON_BYTES)?;
    let value: Value = parse_canonical_json(&bytes)?;
    if value["_type"] != "https://in-toto.io/Statement/v1"
        || value["predicateType"] != "https://slsa.dev/provenance/v1"
        || value["predicate"]["buildDefinition"]["externalParameters"]["candidate_set_sha256"]
            != candidate.candidate_set_sha256
        || value["subject"].as_array().map(Vec::len) != Some(5)
    {
        return Err("community-alpha-release-provenance-invalid");
    }
    Ok(())
}

fn verify_release_notes(
    directory: &Path,
    candidate: &NativeCommunityAlphaCandidateSetV1,
    manifest: &NativeCommunityAlphaIntegrityManifestV1,
) -> Result<(), &'static str> {
    let bytes = read_input_file(
        &directory.join(&manifest.release_notes_file),
        MAX_NOTES_BYTES,
    )?;
    let text = std::str::from_utf8(&bytes).map_err(|_| "community-alpha-release-notes-invalid")?;
    for required in [
        "community-alpha — not platform-trusted",
        "Open Anyway",
        "Smart App Control",
        "AppImage",
        "Rust, Python, Node.js",
        "下载与平台提示",
        &candidate.content.source_commit,
        &candidate.candidate_set_sha256,
    ] {
        if !text.contains(required) {
            return Err("community-alpha-release-notes-invalid");
        }
    }
    Ok(())
}

fn percent_encode(value: &str) -> String {
    let mut output = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            output.push(byte as char);
        } else {
            output.push('%');
            output.push_str(&format!("{byte:02X}"));
        }
    }
    output
}

fn deterministic_uuid(input: &[u8]) -> String {
    let digest = Sha256::digest(input);
    let mut bytes = [0_u8; 16];
    bytes.copy_from_slice(&digest[..16]);
    bytes[6] = (bytes[6] & 0x0f) | 0x50;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    format!(
        "{}-{}-{}-{}-{}",
        encode_hex(&bytes[0..4]),
        encode_hex(&bytes[4..6]),
        encode_hex(&bytes[6..8]),
        encode_hex(&bytes[8..10]),
        encode_hex(&bytes[10..16])
    )
}

fn verify_embedded_source(source_commit: &str) -> Result<(), &'static str> {
    if qiongli::embedded_source_commit() != Some(source_commit) {
        return Err("community-alpha-release-source-unbound");
    }
    Ok(())
}

fn validate_input_directory(path: &Path) -> Result<(), &'static str> {
    if !valid_absolute_path(path) {
        return Err("community-alpha-release-input-invalid");
    }
    let metadata =
        fs::symlink_metadata(path).map_err(|_| "community-alpha-release-input-invalid")?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err("community-alpha-release-input-invalid");
    }
    Ok(())
}

fn validate_input_file(path: &Path, limit: u64) -> Result<(), &'static str> {
    if !valid_absolute_path(path) {
        return Err("community-alpha-release-input-invalid");
    }
    let metadata =
        fs::symlink_metadata(path).map_err(|_| "community-alpha-release-input-invalid")?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() == 0
        || metadata.len() > limit
    {
        return Err("community-alpha-release-input-invalid");
    }
    Ok(())
}

fn validate_new_output_file(path: &Path) -> Result<(), &'static str> {
    if !valid_absolute_path(path) || path.exists() || !outside_checkout(path) {
        return Err("community-alpha-release-output-invalid");
    }
    validate_input_directory(
        path.parent()
            .ok_or("community-alpha-release-output-invalid")?,
    )
}

fn validate_new_output_directory(path: &Path) -> Result<(), &'static str> {
    if !valid_absolute_path(path) || path.exists() || !outside_checkout(path) {
        return Err("community-alpha-release-output-invalid");
    }
    validate_input_directory(
        path.parent()
            .ok_or("community-alpha-release-output-invalid")?,
    )
}

fn valid_absolute_path(path: &Path) -> bool {
    path.is_absolute()
        && !path
            .components()
            .any(|component| matches!(component, Component::CurDir | Component::ParentDir))
}

fn outside_checkout(path: &Path) -> bool {
    let Some(parent) = path.parent() else {
        return false;
    };
    let Some(parent) = fs::canonicalize(parent).ok() else {
        return false;
    };
    let Some(checkout) = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(4)
        .and_then(|path| fs::canonicalize(path).ok())
    else {
        return false;
    };
    !parent.starts_with(checkout)
}

fn exact_entry_names(directory: &Path) -> Result<BTreeSet<String>, &'static str> {
    fs::read_dir(directory)
        .map_err(|_| "community-alpha-release-directory-invalid")?
        .map(|entry| {
            entry
                .map_err(|_| "community-alpha-release-directory-invalid")?
                .file_name()
                .into_string()
                .map_err(|_| "community-alpha-release-directory-invalid")
        })
        .collect()
}

fn verify_digest(path: &Path, limit: u64, expected: &str) -> Result<(), &'static str> {
    let (_, actual) = sha256_file(path, limit)?;
    if actual != expected {
        return Err("community-alpha-release-file-drift");
    }
    Ok(())
}

fn verify_file(
    path: &Path,
    expected_size: u64,
    expected_sha256: &str,
    limit: u64,
) -> Result<(), &'static str> {
    let (size, sha256) = sha256_file(path, limit)?;
    if size != expected_size || sha256 != expected_sha256 {
        return Err("community-alpha-release-file-drift");
    }
    Ok(())
}

fn copy_verified_file(
    source: &Path,
    destination: &Path,
    expected_size: u64,
    expected_sha256: &str,
    limit: u64,
) -> Result<(), &'static str> {
    verify_file(source, expected_size, expected_sha256, limit)?;
    let input = File::open(source).map_err(|_| "community-alpha-release-copy-failed")?;
    let mut output = create_new_private_file(destination)?;
    let copied = std::io::copy(&mut input.take(limit.saturating_add(1)), &mut output)
        .map_err(|_| "community-alpha-release-copy-failed")?;
    output
        .sync_all()
        .map_err(|_| "community-alpha-release-copy-failed")?;
    drop(output);
    if copied != expected_size {
        return Err("community-alpha-release-copy-failed");
    }
    verify_file(destination, expected_size, expected_sha256, limit)
}

fn sha256_file(path: &Path, limit: u64) -> Result<(u64, String), &'static str> {
    validate_input_file(path, limit)?;
    let metadata = fs::metadata(path).map_err(|_| "community-alpha-release-input-invalid")?;
    let file = File::open(path).map_err(|_| "community-alpha-release-input-invalid")?;
    let mut hasher = Sha256::new();
    let copied = std::io::copy(
        &mut file.take(limit.saturating_add(1)),
        &mut HashWriter(&mut hasher),
    )
    .map_err(|_| "community-alpha-release-input-invalid")?;
    if copied != metadata.len() || copied > limit {
        return Err("community-alpha-release-input-invalid");
    }
    Ok((copied, encode_hex(&hasher.finalize())))
}

struct HashWriter<'a>(&'a mut Sha256);

impl std::io::Write for HashWriter<'_> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.update(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn read_input_file(path: &Path, limit: u64) -> Result<Vec<u8>, &'static str> {
    validate_input_file(path, limit)?;
    let metadata = fs::metadata(path).map_err(|_| "community-alpha-release-input-invalid")?;
    let mut bytes = Vec::with_capacity(
        usize::try_from(metadata.len()).map_err(|_| "community-alpha-release-input-invalid")?,
    );
    File::open(path)
        .map_err(|_| "community-alpha-release-input-invalid")?
        .take(limit.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|_| "community-alpha-release-input-invalid")?;
    if bytes.len() as u64 != metadata.len() {
        return Err("community-alpha-release-input-invalid");
    }
    Ok(bytes)
}

fn read_authority_file(path: &Path) -> Result<Vec<u8>, &'static str> {
    let mut bytes = read_input_file(path, MAX_JSON_BYTES)?;
    if bytes.last() == Some(&b'\n') {
        bytes.pop();
        if bytes.last() == Some(&b'\r') {
            bytes.pop();
        }
    }
    if bytes.is_empty() {
        return Err("community-alpha-release-authority-invalid");
    }
    Ok(bytes)
}

fn parse_canonical_json<T: serde::de::DeserializeOwned + Serialize>(
    bytes: &[u8],
) -> Result<T, &'static str> {
    let value =
        serde_json::from_slice(bytes).map_err(|_| "community-alpha-release-json-invalid")?;
    if canonical_json(&value)?.as_slice() != bytes {
        return Err("community-alpha-release-json-noncanonical");
    }
    Ok(value)
}

fn canonical_json<T: Serialize>(value: &T) -> Result<Vec<u8>, &'static str> {
    serde_json_canonicalizer::to_vec(value)
        .map_err(|_| "community-alpha-release-json-serialization-failed")
}

fn sha256_hex(bytes: &[u8]) -> String {
    encode_hex(&Sha256::digest(bytes))
}

fn decode_fixed_hex<const N: usize>(value: &str) -> Option<[u8; N]> {
    if value.len() != N.saturating_mul(2)
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    {
        return None;
    }
    let mut output = [0_u8; N];
    for (index, pair) in value.as_bytes().chunks_exact(2).enumerate() {
        output[index] = (hex_value(pair[0])? << 4) | hex_value(pair[1])?;
    }
    Some(output)
}

const fn hex_value(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        _ => None,
    }
}

fn encode_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len().saturating_mul(2));
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}

fn create_private_directory(path: &Path) -> Result<(), &'static str> {
    fs::create_dir(path).map_err(|_| "community-alpha-release-output-create-failed")?;
    set_private_directory_mode(path)
}

fn write_new_private_file(path: &Path, bytes: &[u8]) -> Result<(), &'static str> {
    let mut file = create_new_private_file(path)?;
    file.write_all(bytes)
        .map_err(|_| "community-alpha-release-output-write-failed")?;
    file.sync_all()
        .map_err(|_| "community-alpha-release-output-write-failed")
}

fn create_new_private_file(path: &Path) -> Result<File, &'static str> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    set_private_file_options(&mut options);
    options
        .open(path)
        .map_err(|_| "community-alpha-release-output-create-failed")
}

#[cfg(unix)]
fn set_private_file_options(options: &mut OpenOptions) {
    use std::os::unix::fs::OpenOptionsExt as _;
    options.mode(0o600);
}

#[cfg(not(unix))]
fn set_private_file_options(_options: &mut OpenOptions) {}

#[cfg(unix)]
fn set_private_directory_mode(path: &Path) -> Result<(), &'static str> {
    use std::os::unix::fs::PermissionsExt as _;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
        .map_err(|_| "community-alpha-release-output-create-failed")
}

#[cfg(not(unix))]
fn set_private_directory_mode(_path: &Path) -> Result<(), &'static str> {
    Ok(())
}
