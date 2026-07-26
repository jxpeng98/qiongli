use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use std::ffi::OsStr;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

const CLI_RECEIPT_SCHEMA_VERSION: u32 = 1;
const MAX_RECEIPT_BYTES: u64 = 16 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CliInstallState {
    Missing,
    InstalledCurrent,
    UpdateAvailable,
    Unavailable,
    Conflict,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CliPathState {
    Active,
    NotConfigured,
    Shadowed,
    NotObservable,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CliInstallInspection {
    pub(crate) state: CliInstallState,
    pub(crate) installed_version: Option<String>,
    pub(crate) available_version: String,
    pub(crate) target: PathBuf,
    pub(crate) path_state: CliPathState,
    pub(crate) reason_code: &'static str,
    pub(crate) can_install: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct CliInstallPlan {
    home: PathBuf,
    source: PathBuf,
    target: PathBuf,
    receipt_path: PathBuf,
    product_version: String,
    source_sha256: String,
    expected_target: TargetObservation,
    previous_managed: bool,
    plan_sha256: String,
}

impl CliInstallPlan {
    pub(crate) fn target(&self) -> &Path {
        &self.target
    }

    pub(crate) fn plan_sha256(&self) -> &str {
        &self.plan_sha256
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum TargetObservation {
    Missing,
    RegularFile(String),
    Symlink(String),
    Unsupported,
}

impl TargetObservation {
    fn fingerprint(&self) -> String {
        match self {
            Self::Missing => "missing".to_owned(),
            Self::RegularFile(sha256) => format!("file:{sha256}"),
            Self::Symlink(sha256) => format!("symlink:{sha256}"),
            Self::Unsupported => "unsupported".to_owned(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct CliInstallReceiptV1 {
    schema_version: u32,
    product_version: String,
    installed_sha256: String,
    target_name: String,
    retained_backup_name: Option<String>,
}

#[derive(Serialize)]
struct CliInstallPlanDigest<'a> {
    schema_version: u32,
    product_version: &'a str,
    source_sha256: &'a str,
    target_name: &'a str,
    expected_target: String,
    previous_managed: bool,
}

pub(crate) fn bundled_cli_path() -> Option<PathBuf> {
    let executable = std::env::current_exe().ok()?;
    let parent = executable.parent()?;
    Some(parent.join(if cfg!(windows) {
        "qiongli-cli.exe"
    } else {
        "qiongli-cli"
    }))
}

pub(crate) fn inspect_cli_install(
    home: Option<&Path>,
    source: Option<&Path>,
    product_version: &str,
    search_path: Option<&OsStr>,
) -> CliInstallInspection {
    let Some(home) = home else {
        return unavailable_inspection(product_version, "qiongli-cli-home-unavailable");
    };
    let target = cli_target(home);
    let Some(source) = source else {
        return unavailable_inspection_with_target(
            product_version,
            target,
            "qiongli-cli-bundle-unavailable",
        );
    };
    let source_sha256 = match regular_file_sha256(source) {
        Ok(sha256) => sha256,
        Err(code) => {
            return unavailable_inspection_with_target(product_version, target, code);
        }
    };
    let receipt_path = cli_receipt_path(home);
    let receipt = read_receipt(&receipt_path).ok().flatten();
    let target_observation = match observe_target(&target) {
        Ok(observation) => observation,
        Err(code) => {
            return CliInstallInspection {
                state: CliInstallState::Conflict,
                installed_version: None,
                available_version: product_version.to_owned(),
                path_state: observe_path_state(&target, search_path),
                target,
                reason_code: code,
                can_install: false,
            };
        }
    };
    let path_state = observe_path_state(&target, search_path);
    match target_observation {
        TargetObservation::Missing => CliInstallInspection {
            state: CliInstallState::Missing,
            installed_version: None,
            available_version: product_version.to_owned(),
            target,
            path_state,
            reason_code: "qiongli-cli-not-installed",
            can_install: true,
        },
        TargetObservation::RegularFile(ref target_sha256) if *target_sha256 == source_sha256 => {
            CliInstallInspection {
                state: CliInstallState::InstalledCurrent,
                installed_version: Some(
                    receipt
                        .filter(|receipt| receipt.installed_sha256 == *target_sha256)
                        .map_or_else(
                            || product_version.to_owned(),
                            |receipt| receipt.product_version,
                        ),
                ),
                available_version: product_version.to_owned(),
                target,
                path_state,
                reason_code: if path_state == CliPathState::Active {
                    "qiongli-cli-installed-current"
                } else {
                    "qiongli-cli-installed-path-attention"
                },
                can_install: false,
            }
        }
        TargetObservation::Unsupported => CliInstallInspection {
            state: CliInstallState::Conflict,
            installed_version: None,
            available_version: product_version.to_owned(),
            target,
            path_state,
            reason_code: "qiongli-cli-target-type-unsupported",
            can_install: false,
        },
        TargetObservation::RegularFile(_) | TargetObservation::Symlink(_) => CliInstallInspection {
            state: CliInstallState::UpdateAvailable,
            installed_version: receipt.map(|receipt| receipt.product_version),
            available_version: product_version.to_owned(),
            target,
            path_state,
            reason_code: "qiongli-cli-replacement-available",
            can_install: true,
        },
    }
}

pub(crate) fn preview_cli_install(
    home: &Path,
    source: &Path,
    product_version: &str,
) -> Result<CliInstallPlan, &'static str> {
    validate_install_roots(home)?;
    let source_sha256 = regular_file_sha256(source)?;
    let target = cli_target(home);
    validate_target_ancestors(home, &target)?;
    let expected_target = observe_target(&target)?;
    if expected_target == TargetObservation::Unsupported {
        return Err("qiongli-cli-target-type-unsupported");
    }
    let receipt_path = cli_receipt_path(home);
    let previous_managed =
        read_receipt(&receipt_path)?.is_some_and(|receipt| match &expected_target {
            TargetObservation::RegularFile(sha256) => receipt.installed_sha256 == *sha256,
            TargetObservation::Missing
            | TargetObservation::Symlink(_)
            | TargetObservation::Unsupported => false,
        });
    let target_name = target
        .file_name()
        .and_then(OsStr::to_str)
        .ok_or("qiongli-cli-target-invalid")?;
    let digest_payload = CliInstallPlanDigest {
        schema_version: CLI_RECEIPT_SCHEMA_VERSION,
        product_version,
        source_sha256: &source_sha256,
        target_name,
        expected_target: expected_target.fingerprint(),
        previous_managed,
    };
    let encoded =
        serde_json::to_vec(&digest_payload).map_err(|_| "qiongli-cli-plan-serialization-failed")?;
    let plan_sha256 = sha256_bytes(&encoded);
    Ok(CliInstallPlan {
        home: home.to_path_buf(),
        source: source.to_path_buf(),
        target,
        receipt_path,
        product_version: product_version.to_owned(),
        source_sha256,
        expected_target,
        previous_managed,
        plan_sha256,
    })
}

pub(crate) fn apply_cli_install(plan: &CliInstallPlan) -> Result<&'static str, &'static str> {
    validate_install_roots(&plan.home)?;
    validate_target_ancestors(&plan.home, &plan.target)?;
    if regular_file_sha256(&plan.source)? != plan.source_sha256 {
        return Err("qiongli-cli-bundle-changed");
    }
    if observe_target(&plan.target)? != plan.expected_target {
        return Err("qiongli-cli-target-changed");
    }

    let bin_dir = plan.target.parent().ok_or("qiongli-cli-target-invalid")?;
    create_private_directory_chain(&plan.home, bin_dir)?;
    let backup_path = if plan.expected_target == TargetObservation::Missing {
        None
    } else {
        let backup_root = plan
            .receipt_path
            .parent()
            .ok_or("qiongli-cli-receipt-path-invalid")?
            .join("backups");
        create_private_directory_chain(&plan.home, &backup_root)?;
        let suffix = &plan.plan_sha256[..12];
        let backup = backup_root.join(format!("preinstall-{suffix}-qiongli"));
        if fs::symlink_metadata(&backup).is_ok() {
            return Err("qiongli-cli-backup-conflict");
        }
        fs::rename(&plan.target, &backup).map_err(|_| "qiongli-cli-backup-failed")?;
        Some(backup)
    };

    let temporary = bin_dir.join(format!(".qiongli-install-{}", &plan.plan_sha256[..12]));
    if fs::symlink_metadata(&temporary).is_ok() {
        restore_previous_target(&plan.target, backup_path.as_deref());
        return Err("qiongli-cli-temporary-conflict");
    }
    if let Err(code) = copy_executable(&plan.source, &temporary).and_then(|()| {
        fs::rename(&temporary, &plan.target).map_err(|_| "qiongli-cli-commit-failed")
    }) {
        let _ = fs::remove_file(&temporary);
        restore_previous_target(&plan.target, backup_path.as_deref());
        return Err(code);
    }

    let retained_backup_name = if plan.previous_managed {
        None
    } else {
        backup_path
            .as_ref()
            .and_then(|path| path.file_name())
            .and_then(OsStr::to_str)
            .map(str::to_owned)
    };
    let receipt = CliInstallReceiptV1 {
        schema_version: CLI_RECEIPT_SCHEMA_VERSION,
        product_version: plan.product_version.clone(),
        installed_sha256: plan.source_sha256.clone(),
        target_name: plan
            .target
            .file_name()
            .and_then(OsStr::to_str)
            .ok_or("qiongli-cli-target-invalid")?
            .to_owned(),
        retained_backup_name,
    };
    if let Err(code) = write_receipt(&plan.home, &plan.receipt_path, &receipt, &plan.plan_sha256) {
        let _ = fs::remove_file(&plan.target);
        restore_previous_target(&plan.target, backup_path.as_deref());
        return Err(code);
    }
    if plan.previous_managed {
        if let Some(backup) = backup_path {
            let _ = fs::remove_file(backup);
        }
    }
    Ok(if plan.expected_target == TargetObservation::Missing {
        "qiongli-cli-installed"
    } else {
        "qiongli-cli-updated"
    })
}

fn unavailable_inspection(
    product_version: &str,
    reason_code: &'static str,
) -> CliInstallInspection {
    unavailable_inspection_with_target(
        product_version,
        PathBuf::from("<user-home>/.local/bin/qiongli"),
        reason_code,
    )
}

fn unavailable_inspection_with_target(
    product_version: &str,
    target: PathBuf,
    reason_code: &'static str,
) -> CliInstallInspection {
    CliInstallInspection {
        state: CliInstallState::Unavailable,
        installed_version: None,
        available_version: product_version.to_owned(),
        target,
        path_state: CliPathState::NotObservable,
        reason_code,
        can_install: false,
    }
}

fn cli_target(home: &Path) -> PathBuf {
    if cfg!(windows) {
        home.join("AppData/Local/Qiongli/bin/qiongli.exe")
    } else {
        home.join(".local/bin/qiongli")
    }
}

fn cli_receipt_path(home: &Path) -> PathBuf {
    home.join(".qiongli/v2/cli/install-receipt.json")
}

fn validate_install_roots(home: &Path) -> Result<(), &'static str> {
    if !home.is_absolute() {
        return Err("qiongli-cli-home-invalid");
    }
    let metadata = fs::symlink_metadata(home).map_err(|_| "qiongli-cli-home-unavailable")?;
    if !metadata.file_type().is_dir() || metadata.file_type().is_symlink() {
        return Err("qiongli-cli-home-unsafe");
    }
    Ok(())
}

fn validate_target_ancestors(home: &Path, target: &Path) -> Result<(), &'static str> {
    if !target.starts_with(home) {
        return Err("qiongli-cli-target-invalid");
    }
    let mut current = target.parent();
    while let Some(path) = current {
        if path == home {
            break;
        }
        match fs::symlink_metadata(path) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err("qiongli-cli-target-ancestor-symlink");
            }
            Ok(metadata) if !metadata.is_dir() => {
                return Err("qiongli-cli-target-ancestor-invalid");
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return Err("qiongli-cli-target-unavailable"),
        }
        current = path.parent();
    }
    Ok(())
}

fn create_private_directory_chain(home: &Path, target: &Path) -> Result<(), &'static str> {
    if !target.starts_with(home) {
        return Err("qiongli-cli-directory-invalid");
    }
    let relative = target
        .strip_prefix(home)
        .map_err(|_| "qiongli-cli-directory-invalid")?;
    let mut current = home.to_path_buf();
    for component in relative.components() {
        current.push(component.as_os_str());
        match fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
                return Err("qiongli-cli-directory-unsafe");
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                create_private_directory(&current)?;
            }
            Err(_) => return Err("qiongli-cli-directory-unavailable"),
        }
    }
    Ok(())
}

#[cfg(unix)]
fn create_private_directory(path: &Path) -> Result<(), &'static str> {
    use std::os::unix::fs::DirBuilderExt;

    fs::DirBuilder::new()
        .mode(0o700)
        .create(path)
        .map_err(|_| "qiongli-cli-directory-create-failed")
}

#[cfg(not(unix))]
fn create_private_directory(path: &Path) -> Result<(), &'static str> {
    fs::create_dir(path).map_err(|_| "qiongli-cli-directory-create-failed")
}

fn observe_target(path: &Path) -> Result<TargetObservation, &'static str> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            let target = fs::read_link(path).map_err(|_| "qiongli-cli-target-unreadable")?;
            Ok(TargetObservation::Symlink(sha256_bytes(
                target.as_os_str().as_encoded_bytes(),
            )))
        }
        Ok(metadata) if metadata.is_file() => {
            Ok(TargetObservation::RegularFile(regular_file_sha256(path)?))
        }
        Ok(_) => Ok(TargetObservation::Unsupported),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Ok(TargetObservation::Missing)
        }
        Err(_) => Err("qiongli-cli-target-unavailable"),
    }
}

fn regular_file_sha256(path: &Path) -> Result<String, &'static str> {
    let metadata = fs::symlink_metadata(path).map_err(|_| "qiongli-cli-file-unavailable")?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err("qiongli-cli-file-invalid");
    }
    let mut file = File::open(path).map_err(|_| "qiongli-cli-file-unavailable")?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|_| "qiongli-cli-file-unavailable")?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn sha256_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn read_receipt(path: &Path) -> Result<Option<CliInstallReceiptV1>, &'static str> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err("qiongli-cli-receipt-unavailable"),
    };
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() > MAX_RECEIPT_BYTES
    {
        return Err("qiongli-cli-receipt-invalid");
    }
    let bytes = fs::read(path).map_err(|_| "qiongli-cli-receipt-unavailable")?;
    let receipt: CliInstallReceiptV1 =
        serde_json::from_slice(&bytes).map_err(|_| "qiongli-cli-receipt-invalid")?;
    if receipt.schema_version != CLI_RECEIPT_SCHEMA_VERSION
        || receipt.product_version.is_empty()
        || receipt.product_version.len() > 128
        || receipt.installed_sha256.len() != 64
        || !receipt
            .installed_sha256
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
        || receipt.target_name
            != if cfg!(windows) {
                "qiongli.exe"
            } else {
                "qiongli"
            }
    {
        return Err("qiongli-cli-receipt-invalid");
    }
    Ok(Some(receipt))
}

fn write_receipt(
    home: &Path,
    path: &Path,
    receipt: &CliInstallReceiptV1,
    token: &str,
) -> Result<(), &'static str> {
    let parent = path.parent().ok_or("qiongli-cli-receipt-path-invalid")?;
    create_private_directory_chain(home, parent)?;
    let temporary = parent.join(format!(".install-receipt-{}.json", &token[..12]));
    let bytes =
        serde_json::to_vec_pretty(receipt).map_err(|_| "qiongli-cli-receipt-write-failed")?;
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options
        .open(&temporary)
        .map_err(|_| "qiongli-cli-receipt-write-failed")?;
    file.write_all(&bytes)
        .and_then(|()| file.sync_all())
        .map_err(|_| "qiongli-cli-receipt-write-failed")?;
    fs::rename(&temporary, path).map_err(|_| "qiongli-cli-receipt-write-failed")
}

fn copy_executable(source: &Path, target: &Path) -> Result<(), &'static str> {
    fs::copy(source, target).map_err(|_| "qiongli-cli-copy-failed")?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(target, fs::Permissions::from_mode(0o755))
            .map_err(|_| "qiongli-cli-permissions-failed")?;
    }
    File::open(target)
        .and_then(|file| file.sync_all())
        .map_err(|_| "qiongli-cli-copy-failed")
}

fn restore_previous_target(target: &Path, backup: Option<&Path>) {
    if let Some(backup) = backup {
        let _ = fs::rename(backup, target);
    }
}

fn observe_path_state(target: &Path, search_path: Option<&OsStr>) -> CliPathState {
    let Some(search_path) = search_path else {
        return CliPathState::NotObservable;
    };
    let Some(target_parent) = target.parent() else {
        return CliPathState::NotObservable;
    };
    let directories = std::env::split_paths(search_path).collect::<Vec<_>>();
    if !directories.iter().any(|path| path == target_parent) {
        return CliPathState::NotConfigured;
    }
    for directory in directories {
        let candidate = directory.join(if cfg!(windows) {
            "qiongli.exe"
        } else {
            "qiongli"
        });
        if is_executable_file(&candidate) {
            return if candidate == target {
                CliPathState::Active
            } else {
                CliPathState::Shadowed
            };
        }
    }
    CliPathState::Active
}

#[cfg(unix)]
fn is_executable_file(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;

    fs::metadata(path)
        .is_ok_and(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
}

#[cfg(not(unix))]
fn is_executable_file(path: &Path) -> bool {
    fs::metadata(path).is_ok_and(|metadata| metadata.is_file())
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::time::{SystemTime, UNIX_EPOCH};

    fn test_root(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "qiongli-cli-install-{name}-{}-{unique}",
            std::process::id()
        ));
        fs::create_dir_all(&root).unwrap();
        root
    }

    fn write_source(root: &Path, contents: &[u8]) -> PathBuf {
        let source = root.join(if cfg!(windows) {
            "qiongli-cli.exe"
        } else {
            "qiongli-cli"
        });
        fs::write(&source, contents).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&source, fs::Permissions::from_mode(0o755)).unwrap();
        }
        source
    }

    #[test]
    fn installs_bundled_cli_and_reports_current_version() {
        let root = test_root("install");
        let home = root.join("home");
        fs::create_dir(&home).unwrap();
        let source = write_source(&root, b"native-cli-v2");
        let plan = preview_cli_install(&home, &source, "2.0.0-alpha.2").unwrap();

        assert_eq!(apply_cli_install(&plan), Ok("qiongli-cli-installed"));
        let search_path = std::env::join_paths([home.join(".local/bin")]).unwrap();
        let inspection = inspect_cli_install(
            Some(&home),
            Some(&source),
            "2.0.0-alpha.2",
            Some(&search_path),
        );
        assert_eq!(inspection.state, CliInstallState::InstalledCurrent);
        assert_eq!(
            inspection.installed_version.as_deref(),
            Some("2.0.0-alpha.2")
        );
        assert_eq!(inspection.path_state, CliPathState::Active);
        assert_eq!(
            fs::read(home.join(".local/bin/qiongli")).unwrap(),
            b"native-cli-v2"
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn preserves_unmanaged_cli_before_replacement() {
        let root = test_root("backup");
        let home = root.join("home");
        fs::create_dir(&home).unwrap();
        fs::create_dir_all(home.join(".local/bin")).unwrap();
        fs::write(home.join(".local/bin/qiongli"), b"legacy-cli").unwrap();
        let source = write_source(&root, b"native-cli-v2");
        let plan = preview_cli_install(&home, &source, "2.0.0-alpha.2").unwrap();

        assert_eq!(apply_cli_install(&plan), Ok("qiongli-cli-updated"));
        assert_eq!(
            fs::read(home.join(".local/bin/qiongli")).unwrap(),
            b"native-cli-v2"
        );
        let backups = fs::read_dir(home.join(".qiongli/v2/cli/backups"))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(backups.len(), 1);
        assert_eq!(fs::read(backups[0].path()).unwrap(), b"legacy-cli");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn refuses_target_changed_after_preview() {
        let root = test_root("drift");
        let home = root.join("home");
        fs::create_dir(&home).unwrap();
        let source = write_source(&root, b"native-cli-v2");
        let plan = preview_cli_install(&home, &source, "2.0.0-alpha.2").unwrap();
        fs::create_dir_all(home.join(".local/bin")).unwrap();
        fs::write(home.join(".local/bin/qiongli"), b"concurrent-change").unwrap();

        assert_eq!(apply_cli_install(&plan), Err("qiongli-cli-target-changed"));
        assert_eq!(
            fs::read(home.join(".local/bin/qiongli")).unwrap(),
            b"concurrent-change"
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn reports_shadowing_by_an_earlier_path_entry() {
        let root = test_root("shadowed");
        let home = root.join("home");
        fs::create_dir(&home).unwrap();
        let source = write_source(&root, b"native-cli-v2");
        let plan = preview_cli_install(&home, &source, "2.0.0-alpha.2").unwrap();
        apply_cli_install(&plan).unwrap();
        let earlier = root.join("earlier");
        fs::create_dir(&earlier).unwrap();
        let other = earlier.join(if cfg!(windows) {
            "qiongli.exe"
        } else {
            "qiongli"
        });
        fs::write(&other, b"old-cli").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&other, fs::Permissions::from_mode(0o755)).unwrap();
        }
        let search_path = std::env::join_paths([earlier, home.join(".local/bin")]).unwrap();

        let inspection = inspect_cli_install(
            Some(&home),
            Some(&source),
            "2.0.0-alpha.2",
            Some(&search_path),
        );
        assert_eq!(inspection.state, CliInstallState::InstalledCurrent);
        assert_eq!(inspection.path_state, CliPathState::Shadowed);
        assert_eq!(
            inspection.reason_code,
            "qiongli-cli-installed-path-attention"
        );
        let _ = fs::remove_dir_all(root);
    }
}
