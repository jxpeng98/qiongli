use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::ffi::OsString;
use std::fs::{self, File, OpenOptions};
use std::io::{Read as _, Write as _};
use std::path::{Component, Path, PathBuf};

use qiongli_platform::{
    Architecture, ArtifactIdentityV1, CapabilityProfile, InstallerKind,
    NativeCommunityAlphaAssetRole, NativeCommunityAlphaAssetV1, NativeCommunityAlphaCandidateSetV1,
    NativeCommunityAlphaEvidenceRole, NativeCommunityAlphaEvidenceV1,
    NativeCommunityAlphaTargetPromotionV1, NativeDistributionClass, NativeDistributionPolicyV1,
    OperatingSystem, ProductId, ReleaseChannel,
};
use sha2::{Digest as _, Sha256};

const TARGET_PROMOTION_FILE: &str = "qiongli-community-alpha-target-promotion.json";
const CANDIDATE_SET_FILE: &str = "qiongli-community-alpha-candidate-set.json";
const ASSETS_DIRECTORY: &str = "assets";
const EVIDENCE_DIRECTORY: &str = "evidence";
const MAX_ASSET_BYTES: u64 = 1024 * 1024 * 1024;
const MAX_EVIDENCE_BYTES: u64 = 256 * 1024;
const MAX_PROMOTION_BYTES: u64 = 256 * 1024;

fn main() {
    if let Err(code) = run() {
        eprintln!("error: {code}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), &'static str> {
    match Command::parse(env::args_os().skip(1))? {
        Command::Target(arguments) => promote_target(&arguments),
        Command::Aggregate(arguments) => aggregate(&arguments),
    }
}

enum Command {
    Target(TargetArguments),
    Aggregate(AggregateArguments),
}

impl Command {
    fn parse(values: impl IntoIterator<Item = OsString>) -> Result<Self, &'static str> {
        let mut values = values.into_iter();
        let command = values
            .next()
            .and_then(|value| value.into_string().ok())
            .ok_or("community-alpha-promotion-usage-invalid")?;
        let options = OptionMap::parse(values)?;
        match command.as_str() {
            "target" => Ok(Self::Target(TargetArguments::parse(options)?)),
            "aggregate" => Ok(Self::Aggregate(AggregateArguments::parse(options)?)),
            _ => Err("community-alpha-promotion-usage-invalid"),
        }
    }
}

struct TargetArguments {
    source_commit: String,
    build_run_url: String,
    primary_asset: PathBuf,
    secondary_asset: Option<PathBuf>,
    desktop_manifest: PathBuf,
    desktop_receipt: PathBuf,
    platform_receipt: Option<PathBuf>,
    acceptance_receipt: Option<PathBuf>,
    output: PathBuf,
}

impl TargetArguments {
    fn parse(mut options: OptionMap) -> Result<Self, &'static str> {
        let arguments = Self {
            source_commit: options.text("--source-commit")?,
            build_run_url: options.text("--build-run-url")?,
            primary_asset: options.path("--primary-asset")?,
            secondary_asset: options.optional_path("--secondary-asset")?,
            desktop_manifest: options.path("--desktop-manifest")?,
            desktop_receipt: options.path("--desktop-receipt")?,
            platform_receipt: options.optional_path("--platform-receipt")?,
            acceptance_receipt: options.optional_path("--acceptance-receipt")?,
            output: options.path("--output")?,
        };
        options.finish()?;
        validate_source(&arguments.source_commit)?;
        validate_output_path(&arguments.output)?;
        Ok(arguments)
    }
}

struct AggregateArguments {
    source_commit: String,
    build_run_url: String,
    macos: PathBuf,
    windows: PathBuf,
    linux: PathBuf,
    output: PathBuf,
}

impl AggregateArguments {
    fn parse(mut options: OptionMap) -> Result<Self, &'static str> {
        let arguments = Self {
            source_commit: options.text("--source-commit")?,
            build_run_url: options.text("--build-run-url")?,
            macos: options.path("--macos")?,
            windows: options.path("--windows")?,
            linux: options.path("--linux")?,
            output: options.path("--output")?,
        };
        options.finish()?;
        validate_source(&arguments.source_commit)?;
        for directory in [&arguments.macos, &arguments.windows, &arguments.linux] {
            validate_input_directory(directory)?;
        }
        validate_output_path(&arguments.output)?;
        Ok(arguments)
    }
}

struct OptionMap(BTreeMap<String, OsString>);

impl OptionMap {
    fn parse(values: impl IntoIterator<Item = OsString>) -> Result<Self, &'static str> {
        let values = values.into_iter().collect::<Vec<_>>();
        if values.len() % 2 != 0 {
            return Err("community-alpha-promotion-usage-invalid");
        }
        let mut options = BTreeMap::new();
        for pair in values.chunks_exact(2) {
            let name = pair[0]
                .to_str()
                .filter(|value| value.starts_with("--"))
                .ok_or("community-alpha-promotion-usage-invalid")?
                .to_string();
            if options.insert(name, pair[1].clone()).is_some() {
                return Err("community-alpha-promotion-usage-invalid");
            }
        }
        Ok(Self(options))
    }

    fn text(&mut self, name: &str) -> Result<String, &'static str> {
        self.0
            .remove(name)
            .and_then(|value| value.into_string().ok())
            .filter(|value| !value.is_empty())
            .ok_or("community-alpha-promotion-usage-invalid")
    }

    fn path(&mut self, name: &str) -> Result<PathBuf, &'static str> {
        self.0
            .remove(name)
            .map(PathBuf::from)
            .ok_or("community-alpha-promotion-usage-invalid")
    }

    fn optional_path(&mut self, name: &str) -> Result<Option<PathBuf>, &'static str> {
        Ok(self.0.remove(name).map(PathBuf::from))
    }

    fn finish(self) -> Result<(), &'static str> {
        if self.0.is_empty() {
            Ok(())
        } else {
            Err("community-alpha-promotion-usage-invalid")
        }
    }
}

fn promote_target(arguments: &TargetArguments) -> Result<(), &'static str> {
    verify_embedded_source(&arguments.source_commit)?;
    let os = OperatingSystem::current().ok_or("community-alpha-promotion-target-unsupported")?;
    let arch = Architecture::current().ok_or("community-alpha-promotion-target-unsupported")?;
    if !matches!(
        (os, arch),
        (OperatingSystem::Macos, Architecture::Aarch64)
            | (OperatingSystem::Windows, Architecture::X86_64)
            | (OperatingSystem::Linux, Architecture::X86_64)
    ) {
        return Err("community-alpha-promotion-target-unsupported");
    }
    let policy = NativeDistributionPolicyV1::for_artifact(
        NativeDistributionClass::CommunityAlpha,
        ArtifactIdentityV1 {
            product: ProductId::Qiongli,
            version: env!("CARGO_PKG_VERSION").to_string(),
            channel: ReleaseChannel::Alpha,
            profile: CapabilityProfile::Lite,
            os,
            arch,
            installer_kind: InstallerKind::NativeInstaller,
        },
    )
    .map_err(|_| "community-alpha-promotion-policy-invalid")?;
    let asset_inputs = target_asset_inputs(arguments, os)?;
    let evidence_inputs = target_evidence_inputs(arguments, os)?;
    let assets = asset_inputs
        .iter()
        .map(|(role, path)| asset_record(*role, path))
        .collect::<Result<Vec<_>, _>>()?;
    let evidence = evidence_inputs
        .iter()
        .map(|(role, path)| evidence_record(*role, path))
        .collect::<Result<Vec<_>, _>>()?;
    let promotion = NativeCommunityAlphaTargetPromotionV1::fresh_target_native(
        &arguments.source_commit,
        &arguments.build_run_url,
        policy,
        assets,
        evidence,
    )
    .map_err(|error| error.reason_code())?;

    create_private_directory(&arguments.output)?;
    let result = (|| {
        let assets_output = arguments.output.join(ASSETS_DIRECTORY);
        let evidence_output = arguments.output.join(EVIDENCE_DIRECTORY);
        create_private_directory(&assets_output)?;
        create_private_directory(&evidence_output)?;
        for (_, source) in &asset_inputs {
            copy_bounded_leaf(source, &assets_output, MAX_ASSET_BYTES)?;
        }
        for (_, source) in &evidence_inputs {
            copy_bounded_leaf(source, &evidence_output, MAX_EVIDENCE_BYTES)?;
        }
        write_new_private_file(
            &arguments.output.join(TARGET_PROMOTION_FILE),
            &promotion
                .to_canonical_json()
                .map_err(|error| error.reason_code())?,
        )?;
        verify_promoted_target_directory(&arguments.output, &promotion)
    })();
    if result.is_err() {
        let _ = fs::remove_dir_all(&arguments.output);
    }
    result?;
    println!(
        "{}",
        String::from_utf8(
            promotion
                .to_canonical_json()
                .map_err(|error| error.reason_code())?
        )
        .map_err(|_| "community-alpha-promotion-output-invalid")?
    );
    Ok(())
}

fn aggregate(arguments: &AggregateArguments) -> Result<(), &'static str> {
    verify_embedded_source(&arguments.source_commit)?;
    let inputs = [
        (
            &arguments.macos,
            OperatingSystem::Macos,
            Architecture::Aarch64,
        ),
        (
            &arguments.windows,
            OperatingSystem::Windows,
            Architecture::X86_64,
        ),
        (
            &arguments.linux,
            OperatingSystem::Linux,
            Architecture::X86_64,
        ),
    ];
    let mut targets = Vec::with_capacity(inputs.len());
    for (directory, expected_os, expected_arch) in inputs {
        let bytes = read_bounded(&directory.join(TARGET_PROMOTION_FILE), MAX_PROMOTION_BYTES)?;
        let promotion = NativeCommunityAlphaTargetPromotionV1::from_json(&bytes)
            .map_err(|error| error.reason_code())?;
        if promotion.source_commit != arguments.source_commit
            || promotion.build_run_url != arguments.build_run_url
            || promotion.policy.artifact.os != expected_os
            || promotion.policy.artifact.arch != expected_arch
        {
            return Err("community-alpha-promotion-target-mismatch");
        }
        verify_promoted_target_directory(directory, &promotion)?;
        targets.push(promotion);
    }
    let candidate = NativeCommunityAlphaCandidateSetV1::from_fresh_targets(targets)
        .map_err(|error| error.reason_code())?;
    create_private_directory(&arguments.output)?;
    let result = (|| {
        let public_output = arguments.output.join("public");
        let evidence_output = arguments.output.join(EVIDENCE_DIRECTORY);
        create_private_directory(&public_output)?;
        create_private_directory(&evidence_output)?;
        let mut public_names = BTreeSet::new();
        for (index, (directory, _, _)) in inputs.into_iter().enumerate() {
            let target = &candidate.content.targets[index];
            let target_label = target_label(target.policy.artifact.os);
            let target_evidence = evidence_output.join(target_label);
            create_private_directory(&target_evidence)?;
            for asset in &target.assets {
                if !public_names.insert(asset.file.clone()) {
                    return Err("community-alpha-promotion-public-name-conflict");
                }
                copy_verified_file(
                    &directory.join(ASSETS_DIRECTORY).join(&asset.file),
                    &public_output.join(&asset.file),
                    asset.size_bytes,
                    &asset.sha256,
                    MAX_ASSET_BYTES,
                )?;
            }
            for evidence in &target.evidence {
                copy_verified_file(
                    &directory.join(EVIDENCE_DIRECTORY).join(&evidence.file),
                    &target_evidence.join(&evidence.file),
                    evidence.size_bytes,
                    &evidence.sha256,
                    MAX_EVIDENCE_BYTES,
                )?;
            }
            copy_verified_file(
                &directory.join(TARGET_PROMOTION_FILE),
                &target_evidence.join(TARGET_PROMOTION_FILE),
                fs::metadata(directory.join(TARGET_PROMOTION_FILE))
                    .map_err(|_| "community-alpha-promotion-input-invalid")?
                    .len(),
                &sha256_file(&directory.join(TARGET_PROMOTION_FILE), MAX_PROMOTION_BYTES)?.1,
                MAX_PROMOTION_BYTES,
            )?;
        }
        write_new_private_file(
            &arguments.output.join(CANDIDATE_SET_FILE),
            &candidate
                .to_canonical_json()
                .map_err(|error| error.reason_code())?,
        )
    })();
    if result.is_err() {
        let _ = fs::remove_dir_all(&arguments.output);
    }
    result?;
    println!(
        "{}",
        String::from_utf8(
            candidate
                .to_canonical_json()
                .map_err(|error| error.reason_code())?
        )
        .map_err(|_| "community-alpha-promotion-output-invalid")?
    );
    Ok(())
}

fn target_asset_inputs(
    arguments: &TargetArguments,
    os: OperatingSystem,
) -> Result<Vec<(NativeCommunityAlphaAssetRole, PathBuf)>, &'static str> {
    match (os, &arguments.secondary_asset) {
        (OperatingSystem::Macos, Some(secondary)) => Ok(vec![
            (
                NativeCommunityAlphaAssetRole::MacosApplicationZip,
                arguments.primary_asset.clone(),
            ),
            (
                NativeCommunityAlphaAssetRole::MacosInstallerDmg,
                secondary.clone(),
            ),
        ]),
        (OperatingSystem::Windows, None) => Ok(vec![(
            NativeCommunityAlphaAssetRole::WindowsPortableZip,
            arguments.primary_asset.clone(),
        )]),
        (OperatingSystem::Linux, Some(secondary)) => Ok(vec![
            (
                NativeCommunityAlphaAssetRole::LinuxAppimage,
                arguments.primary_asset.clone(),
            ),
            (
                NativeCommunityAlphaAssetRole::LinuxPortableDirectoryZip,
                secondary.clone(),
            ),
        ]),
        _ => Err("community-alpha-promotion-asset-input-invalid"),
    }
}

fn target_evidence_inputs(
    arguments: &TargetArguments,
    os: OperatingSystem,
) -> Result<Vec<(NativeCommunityAlphaEvidenceRole, PathBuf)>, &'static str> {
    let common = vec![
        (
            NativeCommunityAlphaEvidenceRole::DesktopPackageManifest,
            arguments.desktop_manifest.clone(),
        ),
        (
            NativeCommunityAlphaEvidenceRole::DesktopPackageReceipt,
            arguments.desktop_receipt.clone(),
        ),
    ];
    match (
        os,
        &arguments.platform_receipt,
        &arguments.acceptance_receipt,
    ) {
        (OperatingSystem::Macos, Some(platform), Some(acceptance)) => Ok(common
            .into_iter()
            .chain([
                (
                    NativeCommunityAlphaEvidenceRole::MacosSourceAcceptanceReceipt,
                    acceptance.clone(),
                ),
                (
                    NativeCommunityAlphaEvidenceRole::MacosSigningReceipt,
                    platform.clone(),
                ),
            ])
            .collect()),
        (OperatingSystem::Windows, None, None) => Ok(common),
        (OperatingSystem::Linux, Some(platform), None) => Ok(common
            .into_iter()
            .chain([(
                NativeCommunityAlphaEvidenceRole::LinuxAppimageReceipt,
                platform.clone(),
            )])
            .collect()),
        _ => Err("community-alpha-promotion-evidence-input-invalid"),
    }
}

fn asset_record(
    role: NativeCommunityAlphaAssetRole,
    path: &Path,
) -> Result<NativeCommunityAlphaAssetV1, &'static str> {
    let (size, sha256) = sha256_file(path, MAX_ASSET_BYTES)?;
    NativeCommunityAlphaAssetV1::new(role, leaf_name(path)?, size, sha256)
        .map_err(|error| error.reason_code())
}

fn evidence_record(
    role: NativeCommunityAlphaEvidenceRole,
    path: &Path,
) -> Result<NativeCommunityAlphaEvidenceV1, &'static str> {
    let (size, sha256) = sha256_file(path, MAX_EVIDENCE_BYTES)?;
    NativeCommunityAlphaEvidenceV1::new(role, leaf_name(path)?, size, sha256)
        .map_err(|error| error.reason_code())
}

fn verify_promoted_target_directory(
    directory: &Path,
    promotion: &NativeCommunityAlphaTargetPromotionV1,
) -> Result<(), &'static str> {
    let root_entries = exact_entry_names(directory)?;
    if root_entries
        != BTreeSet::from([
            ASSETS_DIRECTORY.to_string(),
            EVIDENCE_DIRECTORY.to_string(),
            TARGET_PROMOTION_FILE.to_string(),
        ])
    {
        return Err("community-alpha-promotion-directory-drift");
    }
    let assets = directory.join(ASSETS_DIRECTORY);
    let evidence = directory.join(EVIDENCE_DIRECTORY);
    validate_input_directory(&assets)?;
    validate_input_directory(&evidence)?;
    let expected_assets = promotion
        .assets
        .iter()
        .map(|item| item.file.clone())
        .collect::<BTreeSet<_>>();
    let expected_evidence = promotion
        .evidence
        .iter()
        .map(|item| item.file.clone())
        .collect::<BTreeSet<_>>();
    if exact_entry_names(&assets)? != expected_assets
        || exact_entry_names(&evidence)? != expected_evidence
    {
        return Err("community-alpha-promotion-directory-drift");
    }
    for asset in &promotion.assets {
        verify_file(
            &assets.join(&asset.file),
            asset.size_bytes,
            &asset.sha256,
            MAX_ASSET_BYTES,
        )?;
    }
    for item in &promotion.evidence {
        verify_file(
            &evidence.join(&item.file),
            item.size_bytes,
            &item.sha256,
            MAX_EVIDENCE_BYTES,
        )?;
    }
    Ok(())
}

fn verify_embedded_source(source_commit: &str) -> Result<(), &'static str> {
    if qiongli::embedded_source_commit() != Some(source_commit) {
        return Err("community-alpha-promotion-source-unbound");
    }
    Ok(())
}

fn validate_source(source_commit: &str) -> Result<(), &'static str> {
    if !matches!(source_commit.len(), 40 | 64)
        || !source_commit
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    {
        return Err("community-alpha-promotion-source-invalid");
    }
    Ok(())
}

fn validate_input_directory(path: &Path) -> Result<(), &'static str> {
    if !valid_absolute_path(path) {
        return Err("community-alpha-promotion-input-invalid");
    }
    let metadata =
        fs::symlink_metadata(path).map_err(|_| "community-alpha-promotion-input-invalid")?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err("community-alpha-promotion-input-invalid");
    }
    Ok(())
}

fn validate_output_path(path: &Path) -> Result<(), &'static str> {
    if !valid_absolute_path(path) || path.exists() {
        return Err("community-alpha-promotion-output-invalid");
    }
    let parent = path
        .parent()
        .ok_or("community-alpha-promotion-output-invalid")?;
    validate_input_directory(parent)?;
    let output_parent =
        fs::canonicalize(parent).map_err(|_| "community-alpha-promotion-output-invalid")?;
    let checkout = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(4)
        .and_then(|path| fs::canonicalize(path).ok())
        .ok_or("community-alpha-promotion-output-invalid")?;
    if output_parent.starts_with(checkout) {
        return Err("community-alpha-promotion-output-invalid");
    }
    Ok(())
}

fn valid_absolute_path(path: &Path) -> bool {
    path.is_absolute()
        && !path
            .components()
            .any(|component| matches!(component, Component::CurDir | Component::ParentDir))
}

fn leaf_name(path: &Path) -> Result<String, &'static str> {
    path.file_name()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .ok_or("community-alpha-promotion-file-name-invalid")
}

fn exact_entry_names(directory: &Path) -> Result<BTreeSet<String>, &'static str> {
    fs::read_dir(directory)
        .map_err(|_| "community-alpha-promotion-directory-invalid")?
        .map(|entry| {
            entry
                .map_err(|_| "community-alpha-promotion-directory-invalid")?
                .file_name()
                .into_string()
                .map_err(|_| "community-alpha-promotion-directory-invalid")
        })
        .collect()
}

fn sha256_file(path: &Path, limit: u64) -> Result<(u64, String), &'static str> {
    let metadata =
        fs::symlink_metadata(path).map_err(|_| "community-alpha-promotion-input-invalid")?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() == 0
        || metadata.len() > limit
    {
        return Err("community-alpha-promotion-input-invalid");
    }
    let file = File::open(path).map_err(|_| "community-alpha-promotion-input-invalid")?;
    let mut hasher = Sha256::new();
    let mut limited = file.take(limit.saturating_add(1));
    let mut writer = HashWriter(&mut hasher);
    let copied = std::io::copy(&mut limited, &mut writer)
        .map_err(|_| "community-alpha-promotion-input-invalid")?;
    if copied != metadata.len() || copied > limit {
        return Err("community-alpha-promotion-input-invalid");
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

fn copy_bounded_leaf(source: &Path, output: &Path, limit: u64) -> Result<(), &'static str> {
    let (size, sha256) = sha256_file(source, limit)?;
    copy_verified_file(
        source,
        &output.join(leaf_name(source)?),
        size,
        &sha256,
        limit,
    )
}

fn copy_verified_file(
    source: &Path,
    destination: &Path,
    expected_size: u64,
    expected_sha256: &str,
    limit: u64,
) -> Result<(), &'static str> {
    verify_file(source, expected_size, expected_sha256, limit)?;
    let input = File::open(source).map_err(|_| "community-alpha-promotion-copy-failed")?;
    let mut output = create_new_private_file(destination)?;
    let copied = std::io::copy(&mut input.take(limit.saturating_add(1)), &mut output)
        .map_err(|_| "community-alpha-promotion-copy-failed")?;
    output
        .sync_all()
        .map_err(|_| "community-alpha-promotion-copy-failed")?;
    drop(output);
    if copied != expected_size {
        return Err("community-alpha-promotion-copy-failed");
    }
    verify_file(destination, expected_size, expected_sha256, limit)
}

fn verify_file(
    path: &Path,
    expected_size: u64,
    expected_sha256: &str,
    limit: u64,
) -> Result<(), &'static str> {
    let (size, sha256) = sha256_file(path, limit)?;
    if size != expected_size || sha256 != expected_sha256 {
        return Err("community-alpha-promotion-file-drift");
    }
    Ok(())
}

fn create_private_directory(path: &Path) -> Result<(), &'static str> {
    fs::create_dir(path).map_err(|_| "community-alpha-promotion-output-create-failed")?;
    set_private_directory_mode(path)
}

fn write_new_private_file(path: &Path, bytes: &[u8]) -> Result<(), &'static str> {
    let mut file = create_new_private_file(path)?;
    file.write_all(bytes)
        .map_err(|_| "community-alpha-promotion-output-write-failed")?;
    file.sync_all()
        .map_err(|_| "community-alpha-promotion-output-write-failed")
}

fn create_new_private_file(path: &Path) -> Result<File, &'static str> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    set_private_file_options(&mut options);
    options
        .open(path)
        .map_err(|_| "community-alpha-promotion-output-create-failed")
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
        .map_err(|_| "community-alpha-promotion-output-create-failed")
}

#[cfg(not(unix))]
fn set_private_directory_mode(_path: &Path) -> Result<(), &'static str> {
    Ok(())
}

fn read_bounded(path: &Path, limit: u64) -> Result<Vec<u8>, &'static str> {
    let metadata =
        fs::symlink_metadata(path).map_err(|_| "community-alpha-promotion-input-invalid")?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() == 0
        || metadata.len() > limit
    {
        return Err("community-alpha-promotion-input-invalid");
    }
    let mut bytes = Vec::with_capacity(
        usize::try_from(metadata.len()).map_err(|_| "community-alpha-promotion-input-invalid")?,
    );
    File::open(path)
        .map_err(|_| "community-alpha-promotion-input-invalid")?
        .take(limit.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|_| "community-alpha-promotion-input-invalid")?;
    if bytes.len() as u64 != metadata.len() {
        return Err("community-alpha-promotion-input-invalid");
    }
    Ok(bytes)
}

fn target_label(os: OperatingSystem) -> &'static str {
    match os {
        OperatingSystem::Macos => "macos-aarch64",
        OperatingSystem::Windows => "windows-x86-64",
        OperatingSystem::Linux => "linux-x86-64",
    }
}

fn encode_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut value = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        value.push(HEX[(byte >> 4) as usize] as char);
        value.push(HEX[(byte & 0x0f) as usize] as char);
    }
    value
}
