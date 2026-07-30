use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};

use qiongli_content::{
    EmbeddedContent, MaterializationReceiptV1, MaterializationTarget, ProfileId,
    remove_materialization, verify_materialization,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const REGISTRY_FILE: &str = "managed-content.json";
const REGISTRY_LOCK_FILE: &str = ".managed-content.lock";
const REGISTRY_DOCUMENT_KIND: &str = "qiongli-managed-content";
const REGISTRY_SCHEMA_VERSION: u32 = 1;
const MAX_REGISTRY_BYTES: u64 = 1024 * 1024;
const MAX_ENTRIES: usize = 128;
const MAX_GENERATION: u64 = 9_007_199_254_740_991;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ManagedContentSurface {
    Skills,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ManagedContentEntryV1 {
    pub(crate) surface: ManagedContentSurface,
    pub(crate) target: String,
    pub(crate) product_version: String,
    pub(crate) profile: ProfileId,
    pub(crate) receipt_sha256: String,
    pub(crate) pack_sha256: String,
    pub(crate) content_root_sha256: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ManagedSkillsEntryState {
    Current,
    UpdateAvailable,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ManagedSkillsEntryObservation {
    pub(crate) target: MaterializationTarget,
    pub(crate) receipt: MaterializationReceiptV1,
    pub(crate) receipt_sha256: String,
    pub(crate) state: ManagedSkillsEntryState,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ManagedContentRegistryV1 {
    document_kind: String,
    schema_version: u32,
    pub(crate) generation: u64,
    pub(crate) entries: Vec<ManagedContentEntryV1>,
}

impl ManagedContentRegistryV1 {
    fn empty() -> Self {
        Self {
            document_kind: REGISTRY_DOCUMENT_KIND.to_string(),
            schema_version: REGISTRY_SCHEMA_VERSION,
            generation: 0,
            entries: Vec::new(),
        }
    }

    pub(crate) fn validate(&self) -> Result<(), &'static str> {
        if self.document_kind != REGISTRY_DOCUMENT_KIND
            || self.schema_version != REGISTRY_SCHEMA_VERSION
            || self.generation > MAX_GENERATION
            || self.entries.len() > MAX_ENTRIES
        {
            return Err("managed-content-registry-invalid");
        }
        for (index, entry) in self.entries.iter().enumerate() {
            validate_entry(entry)?;
            if index > 0 && self.entries[index - 1].target >= entry.target {
                return Err("managed-content-registry-invalid");
            }
        }
        Ok(())
    }
}

pub(crate) fn materialization_receipt_sha256(
    receipt: &MaterializationReceiptV1,
) -> Result<String, &'static str> {
    let bytes =
        serde_json_canonicalizer::to_vec(receipt).map_err(|_| "managed-content-receipt-invalid")?;
    Ok(sha256_hex(&bytes))
}

pub(crate) fn managed_skills_target_id(target: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"QIONGLI-MANAGED-SKILLS-TARGET-V1\0");
    hash_component(&mut hasher, target.as_bytes());
    format!("skills-target-{}", encode_lower_hex(&hasher.finalize()))
}

pub(crate) fn observe_managed_skills_entry(
    content: &EmbeddedContent,
    entry: &ManagedContentEntryV1,
) -> Result<ManagedSkillsEntryObservation, &'static str> {
    let target = qiongli_content::approve_materialization_target(Path::new(&entry.target))
        .map_err(|error| error.reason_code())?;
    let receipt = verify_materialization(&target).map_err(|_| "managed-skills-target-drifted")?;
    let receipt_sha256 =
        materialization_receipt_sha256(&receipt).map_err(|_| "managed-skills-target-drifted")?;
    if receipt.profile != entry.profile
        || receipt.pack_sha256 != entry.pack_sha256
        || receipt.content_root_sha256 != entry.content_root_sha256
        || receipt_sha256 != entry.receipt_sha256
    {
        return Err("managed-skills-target-drifted");
    }
    Ok(ManagedSkillsEntryObservation {
        target,
        receipt,
        receipt_sha256,
        state: if entry.pack_sha256 == content.pack().pack_sha256() {
            ManagedSkillsEntryState::Current
        } else {
            ManagedSkillsEntryState::UpdateAvailable
        },
    })
}

pub(crate) fn apply_managed_materialization(
    state_root: &Path,
    content: &EmbeddedContent,
    target: &MaterializationTarget,
    profile: ProfileId,
) -> Result<MaterializationReceiptV1, &'static str> {
    let previous = verify_materialization(target).ok();
    let receipt = content
        .materialize_profile(profile_name(profile), target)
        .map_err(|error| error.reason_code())?;
    match register_managed_materialization(state_root, target, &receipt) {
        Ok(()) => Ok(receipt),
        Err(code) => {
            match compensate_unregistered_materialization(
                content,
                target,
                &receipt,
                previous.as_ref(),
            ) {
                Ok(()) => Err(code),
                Err(recovery) => Err(recovery),
            }
        }
    }
}

pub(crate) fn remove_managed_materialization(
    state_root: &Path,
    content: &EmbeddedContent,
    target: &MaterializationTarget,
    expected_receipt: &MaterializationReceiptV1,
) -> Result<MaterializationReceiptV1, &'static str> {
    let observed = verify_materialization(target).map_err(|error| error.reason_code())?;
    if &observed != expected_receipt {
        return Err("materialization-target-changed");
    }
    let removed = remove_materialization(target).map_err(|error| error.reason_code())?;
    if &removed != expected_receipt {
        return Err("materialization-target-changed");
    }
    match unregister_managed_materialization(state_root, target, expected_receipt) {
        Ok(()) => Ok(removed),
        Err(code) => match restore_managed_materialization(content, target, expected_receipt) {
            Ok(()) => Err(code),
            Err(recovery) => Err(recovery),
        },
    }
}

pub(crate) fn compensate_unregistered_materialization(
    content: &EmbeddedContent,
    target: &MaterializationTarget,
    installed: &MaterializationReceiptV1,
    previous: Option<&MaterializationReceiptV1>,
) -> Result<(), &'static str> {
    match previous {
        Some(previous) => restore_managed_materialization(content, target, previous),
        None => {
            let removed = remove_materialization(target)
                .map_err(|_| "managed-content-registry-recovery-required")?;
            if &removed != installed {
                return Err("managed-content-registry-recovery-required");
            }
            Ok(())
        }
    }
}

pub(crate) fn restore_managed_materialization(
    content: &EmbeddedContent,
    target: &MaterializationTarget,
    receipt: &MaterializationReceiptV1,
) -> Result<(), &'static str> {
    if receipt.pack_sha256 != content.pack().pack_sha256() {
        return Err("managed-content-registry-recovery-required");
    }
    let restored = content
        .materialize_profile(profile_name(receipt.profile), target)
        .map_err(|_| "managed-content-registry-recovery-required")?;
    if &restored != receipt {
        return Err("managed-content-registry-recovery-required");
    }
    Ok(())
}

pub(crate) fn managed_content_registry_path(state_root: &Path) -> PathBuf {
    state_root.join(REGISTRY_FILE)
}

pub(crate) fn managed_content_registry_bytes(
    registry: &ManagedContentRegistryV1,
) -> Result<Vec<u8>, &'static str> {
    registry.validate()?;
    serde_json_canonicalizer::to_vec(registry).map_err(|_| "managed-content-registry-invalid")
}

pub(crate) fn parse_managed_content_registry(
    bytes: &[u8],
) -> Result<ManagedContentRegistryV1, &'static str> {
    if bytes.len() as u64 > MAX_REGISTRY_BYTES {
        return Err("managed-content-registry-invalid");
    }
    let registry: ManagedContentRegistryV1 =
        serde_json::from_slice(bytes).map_err(|_| "managed-content-registry-invalid")?;
    registry.validate()?;
    if managed_content_registry_bytes(&registry)? != bytes {
        return Err("managed-content-registry-invalid");
    }
    Ok(registry)
}

pub(crate) fn load_managed_content_registry(
    state_root: &Path,
) -> Result<ManagedContentRegistryV1, &'static str> {
    validate_state_root_path(state_root)?;
    let path = managed_content_registry_path(state_root);
    let metadata = match fs::symlink_metadata(&path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(ManagedContentRegistryV1::empty());
        }
        Err(_) => return Err("managed-content-registry-unavailable"),
    };
    validate_private_file(&metadata)?;
    if metadata.len() > MAX_REGISTRY_BYTES {
        return Err("managed-content-registry-invalid");
    }
    let file = open_private_file_for_read(&path)?;
    let mut bytes = Vec::with_capacity(usize::try_from(metadata.len()).unwrap_or(0));
    file.take(MAX_REGISTRY_BYTES.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|_| "managed-content-registry-unavailable")?;
    if bytes.len() as u64 > MAX_REGISTRY_BYTES {
        return Err("managed-content-registry-invalid");
    }
    parse_managed_content_registry(&bytes)
}

pub(crate) fn register_managed_materialization(
    state_root: &Path,
    target: &MaterializationTarget,
    receipt: &MaterializationReceiptV1,
) -> Result<(), &'static str> {
    let target_path = target
        .path()
        .to_str()
        .ok_or("managed-content-target-invalid")?
        .to_string();
    let entry = ManagedContentEntryV1 {
        surface: ManagedContentSurface::Skills,
        target: target_path,
        product_version: env!("CARGO_PKG_VERSION").to_string(),
        profile: receipt.profile,
        receipt_sha256: materialization_receipt_sha256(receipt)?,
        pack_sha256: receipt.pack_sha256.clone(),
        content_root_sha256: receipt.content_root_sha256.clone(),
    };
    mutate_registry(state_root, |registry| {
        match registry
            .entries
            .binary_search_by(|candidate| candidate.target.cmp(&entry.target))
        {
            Ok(index) => registry.entries[index] = entry,
            Err(index) => registry.entries.insert(index, entry),
        }
        Ok(())
    })
}

pub(crate) fn unregister_managed_materialization(
    state_root: &Path,
    target: &MaterializationTarget,
    expected_receipt: &MaterializationReceiptV1,
) -> Result<(), &'static str> {
    let target_path = target
        .path()
        .to_str()
        .ok_or("managed-content-target-invalid")?;
    let expected_digest = materialization_receipt_sha256(expected_receipt)?;
    mutate_registry(state_root, |registry| {
        let index = registry
            .entries
            .binary_search_by(|candidate| candidate.target.as_str().cmp(target_path))
            .map_err(|_| "managed-content-registry-entry-missing")?;
        if registry.entries[index].receipt_sha256 != expected_digest {
            return Err("managed-content-registry-entry-changed");
        }
        registry.entries.remove(index);
        Ok(())
    })
}

pub(crate) fn detach_managed_materialization(
    state_root: &Path,
    target: &MaterializationTarget,
    expected_receipt_sha256: &str,
) -> Result<(), &'static str> {
    if !valid_sha256(expected_receipt_sha256) {
        return Err("managed-content-registry-entry-invalid");
    }
    let target_path = target
        .path()
        .to_str()
        .ok_or("managed-content-target-invalid")?;
    mutate_registry(state_root, |registry| {
        let index = registry
            .entries
            .binary_search_by(|candidate| candidate.target.as_str().cmp(target_path))
            .map_err(|_| "managed-content-registry-entry-missing")?;
        if registry.entries[index].receipt_sha256 != expected_receipt_sha256 {
            return Err("managed-content-registry-entry-changed");
        }
        registry.entries.remove(index);
        Ok(())
    })
}

fn mutate_registry(
    state_root: &Path,
    mutation: impl FnOnce(&mut ManagedContentRegistryV1) -> Result<(), &'static str>,
) -> Result<(), &'static str> {
    prepare_private_state_root(state_root)?;
    let _lock = acquire_lock(state_root)?;
    let mut registry = load_managed_content_registry(state_root)?;
    mutation(&mut registry)?;
    registry.generation = registry
        .generation
        .checked_add(1)
        .filter(|generation| *generation <= MAX_GENERATION)
        .ok_or("managed-content-registry-invalid")?;
    registry.validate()?;
    persist_registry(state_root, &registry)
}

fn persist_registry(
    state_root: &Path,
    registry: &ManagedContentRegistryV1,
) -> Result<(), &'static str> {
    let bytes = managed_content_registry_bytes(registry)?;
    if bytes.len() as u64 > MAX_REGISTRY_BYTES {
        return Err("managed-content-registry-invalid");
    }
    let mut nonce = [0_u8; 12];
    getrandom::fill(&mut nonce).map_err(|_| "managed-content-registry-unavailable")?;
    let staging = state_root.join(format!(
        ".managed-content.stage-{}",
        encode_lower_hex(&nonce)
    ));
    write_new_private_file(&staging, &bytes)?;
    let live = managed_content_registry_path(state_root);
    if let Err(error) = fs::rename(&staging, &live) {
        let _ = fs::remove_file(&staging);
        return Err(if error.kind() == std::io::ErrorKind::PermissionDenied {
            "managed-content-registry-read-only"
        } else {
            "managed-content-registry-unavailable"
        });
    }
    sync_directory(state_root)?;
    let loaded = load_managed_content_registry(state_root)?;
    if &loaded != registry {
        return Err("managed-content-registry-recovery-required");
    }
    Ok(())
}

fn prepare_private_state_root(state_root: &Path) -> Result<(), &'static str> {
    validate_state_root_path(state_root)?;
    if !state_root.exists() {
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            let mut builder = fs::DirBuilder::new();
            builder.recursive(true).mode(0o700);
            builder
                .create(state_root)
                .map_err(|_| "managed-content-registry-unavailable")?;
        }
        #[cfg(not(unix))]
        fs::create_dir_all(state_root).map_err(|_| "managed-content-registry-unavailable")?;
    }
    let metadata =
        fs::symlink_metadata(state_root).map_err(|_| "managed-content-registry-unavailable")?;
    validate_private_directory(&metadata)
}

fn acquire_lock(state_root: &Path) -> Result<File, &'static str> {
    #[cfg(unix)]
    use std::os::unix::fs::OpenOptionsExt;

    let path = state_root.join(REGISTRY_LOCK_FILE);
    let mut options = OpenOptions::new();
    options.create(true).read(true).write(true).truncate(false);
    #[cfg(unix)]
    options
        .mode(0o600)
        .custom_flags(rustix::fs::OFlags::NOFOLLOW.bits() as i32);
    let file = options
        .open(&path)
        .map_err(|_| "managed-content-registry-unavailable")?;
    validate_private_file(
        &file
            .metadata()
            .map_err(|_| "managed-content-registry-unavailable")?,
    )?;
    file.try_lock()
        .map_err(|_| "managed-content-registry-busy")?;
    Ok(file)
}

fn write_new_private_file(path: &Path, bytes: &[u8]) -> Result<(), &'static str> {
    #[cfg(unix)]
    use std::os::unix::fs::OpenOptionsExt;

    let mut options = OpenOptions::new();
    options.create_new(true).write(true);
    #[cfg(unix)]
    options.mode(0o600);
    let mut file = options
        .open(path)
        .map_err(|_| "managed-content-registry-unavailable")?;
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(|_| "managed-content-registry-unavailable")
}

fn open_private_file_for_read(path: &Path) -> Result<File, &'static str> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;

        let file = OpenOptions::new()
            .read(true)
            .custom_flags(rustix::fs::OFlags::NOFOLLOW.bits() as i32)
            .open(path)
            .map_err(|_| "managed-content-registry-unavailable")?;
        validate_private_file(
            &file
                .metadata()
                .map_err(|_| "managed-content-registry-unavailable")?,
        )?;
        Ok(file)
    }
    #[cfg(not(unix))]
    {
        let file = File::open(path).map_err(|_| "managed-content-registry-unavailable")?;
        validate_private_file(
            &file
                .metadata()
                .map_err(|_| "managed-content-registry-unavailable")?,
        )?;
        Ok(file)
    }
}

fn validate_entry(entry: &ManagedContentEntryV1) -> Result<(), &'static str> {
    let path = Path::new(&entry.target);
    if !path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, Component::CurDir | Component::ParentDir))
        || semver::Version::parse(&entry.product_version)
            .ok()
            .is_none_or(|version| version.major < 2 || !version.build.is_empty())
        || !valid_sha256(&entry.receipt_sha256)
        || !valid_sha256(&entry.pack_sha256)
        || !valid_sha256(&entry.content_root_sha256)
    {
        return Err("managed-content-registry-invalid");
    }
    Ok(())
}

fn validate_state_root_path(path: &Path) -> Result<(), &'static str> {
    if !path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, Component::CurDir | Component::ParentDir))
    {
        return Err("managed-content-registry-unavailable");
    }
    Ok(())
}

fn validate_private_directory(metadata: &fs::Metadata) -> Result<(), &'static str> {
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err("managed-content-registry-unavailable");
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if metadata.uid() != rustix::process::geteuid().as_raw() || metadata.mode() & 0o077 != 0 {
            return Err("managed-content-registry-permissions-invalid");
        }
    }
    Ok(())
}

fn validate_private_file(metadata: &fs::Metadata) -> Result<(), &'static str> {
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err("managed-content-registry-invalid");
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if metadata.uid() != rustix::process::geteuid().as_raw() || metadata.mode() & 0o077 != 0 {
            return Err("managed-content-registry-permissions-invalid");
        }
    }
    Ok(())
}

#[cfg(unix)]
fn sync_directory(path: &Path) -> Result<(), &'static str> {
    File::open(path)
        .and_then(|directory| directory.sync_all())
        .map_err(|_| "managed-content-registry-unavailable")
}

#[cfg(not(unix))]
fn sync_directory(_path: &Path) -> Result<(), &'static str> {
    Ok(())
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

const fn profile_name(profile: ProfileId) -> &'static str {
    match profile {
        ProfileId::SkillOnly => "skill-only",
        ProfileId::MarketplaceLite => "marketplace-lite",
        ProfileId::Full => "full",
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    encode_lower_hex(&Sha256::digest(bytes))
}

fn hash_component(hasher: &mut Sha256, bytes: &[u8]) {
    hasher.update(u64::try_from(bytes.len()).unwrap_or(u64::MAX).to_be_bytes());
    hasher.update(bytes);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_registry_is_stable() {
        let registry = ManagedContentRegistryV1::empty();
        registry.validate().unwrap();
        assert!(registry.entries.is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn registry_tracks_only_explicit_receipt_bound_materializations() {
        use std::os::unix::fs::PermissionsExt;

        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .unwrap()
            .join("target/qiongli-managed-content-tests")
            .join(format!("{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
        let state_root = root.join("config/v2");
        let destination = root.join("skills");
        let target = qiongli_content::approve_materialization_target(&destination).unwrap();
        let content = crate::embedded_content().unwrap();
        let receipt = content.materialize_profile("skill-only", &target).unwrap();

        register_managed_materialization(&state_root, &target, &receipt).unwrap();
        let registry = load_managed_content_registry(&state_root).unwrap();
        assert_eq!(registry.entries.len(), 1);
        assert_eq!(registry.entries[0].target, destination.to_str().unwrap());
        assert_eq!(
            registry.entries[0].receipt_sha256,
            materialization_receipt_sha256(&receipt).unwrap()
        );
        let observation = observe_managed_skills_entry(&content, &registry.entries[0]).unwrap();
        assert_eq!(observation.state, ManagedSkillsEntryState::Current);
        assert_eq!(observation.receipt, receipt);

        fs::write(destination.join(".qiongli-managed.json"), b"{}").unwrap();
        assert_eq!(
            observe_managed_skills_entry(&content, &registry.entries[0]).unwrap_err(),
            "managed-skills-target-drifted"
        );
        let drifted_receipt = fs::read(destination.join(".qiongli-managed.json")).unwrap();
        let retained_canary = destination.join("retained-user-change.txt");
        fs::write(&retained_canary, b"user-owned-after-drift").unwrap();
        detach_managed_materialization(&state_root, &target, &registry.entries[0].receipt_sha256)
            .unwrap();
        assert!(
            load_managed_content_registry(&state_root)
                .unwrap()
                .entries
                .is_empty()
        );
        assert_eq!(
            fs::read(destination.join(".qiongli-managed.json")).unwrap(),
            drifted_receipt
        );
        assert_eq!(
            fs::read(&retained_canary).unwrap(),
            b"user-owned-after-drift"
        );

        register_managed_materialization(&state_root, &target, &receipt).unwrap();
        fs::remove_dir_all(&destination).unwrap();
        let restored = content.materialize_profile("skill-only", &target).unwrap();
        assert_eq!(restored, receipt);

        unregister_managed_materialization(&state_root, &target, &receipt).unwrap();
        assert!(
            load_managed_content_registry(&state_root)
                .unwrap()
                .entries
                .is_empty()
        );
        let _ = fs::remove_dir_all(root);
    }
}
