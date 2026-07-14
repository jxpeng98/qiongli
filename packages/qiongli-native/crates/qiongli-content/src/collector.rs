use std::collections::{BTreeMap, HashMap};
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::fs::{self, Metadata};
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use same_file::Handle;
use unicode_normalization::UnicodeNormalization;

use crate::manifest::{LogicalMode, ResourceKind};

const DEFAULT_MAX_ENTRIES: usize = 4_096;
const DEFAULT_MAX_ENTRY_BYTES: u64 = 16 * 1024 * 1024;
const DEFAULT_MAX_TOTAL_BYTES: u64 = 128 * 1024 * 1024;
const DEFAULT_MAX_PATH_DEPTH: usize = 32;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CollectorLimits {
    pub max_entries: usize,
    pub max_entry_bytes: u64,
    pub max_total_bytes: u64,
    pub max_path_depth: usize,
}

impl Default for CollectorLimits {
    fn default() -> Self {
        Self {
            max_entries: DEFAULT_MAX_ENTRIES,
            max_entry_bytes: DEFAULT_MAX_ENTRY_BYTES,
            max_total_bytes: DEFAULT_MAX_TOTAL_BYTES,
            max_path_depth: DEFAULT_MAX_PATH_DEPTH,
        }
    }
}

impl CollectorLimits {
    fn validate(self) -> Result<(), CollectorError> {
        if self.max_entries == 0 {
            return Err(CollectorError::InvalidLimits("max_entries"));
        }
        if self.max_entry_bytes == 0 {
            return Err(CollectorError::InvalidLimits("max_entry_bytes"));
        }
        if self.max_total_bytes == 0 {
            return Err(CollectorError::InvalidLimits("max_total_bytes"));
        }
        if self.max_path_depth == 0 {
            return Err(CollectorError::InvalidLimits("max_path_depth"));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CollectedResource {
    pub path: String,
    pub resource_kind: ResourceKind,
    pub mode: LogicalMode,
    bytes: Box<[u8]>,
}

impl CollectedResource {
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    #[must_use]
    pub fn size_bytes(&self) -> u64 {
        self.bytes.len() as u64
    }

    #[must_use]
    pub fn into_bytes(self) -> Box<[u8]> {
        self.bytes
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CollectorError {
    InvalidLimits(&'static str),
    MissingSource {
        path: PathBuf,
    },
    InvalidRoot {
        path: PathBuf,
    },
    InvalidPath {
        path: String,
        reason: &'static str,
    },
    NonUtf8Path {
        path: PathBuf,
    },
    LinkNotAllowed {
        path: String,
    },
    HardLinkNotAllowed {
        path: String,
    },
    HardLinkAlias {
        first: String,
        second: String,
    },
    UnsupportedFileType {
        path: String,
    },
    PathCollision {
        first: String,
        second: String,
    },
    EntryLimitExceeded {
        limit: usize,
    },
    FileTooLarge {
        path: String,
        size_bytes: u64,
        limit: u64,
    },
    TotalSizeExceeded {
        size_bytes: u64,
        limit: u64,
    },
    SourceChanged {
        path: String,
        expected_size: u64,
        actual_size: u64,
    },
    Io {
        operation: &'static str,
        path: PathBuf,
        kind: io::ErrorKind,
    },
}

impl CollectorError {
    fn io(operation: &'static str, path: &Path, error: &io::Error) -> Self {
        if error.kind() == io::ErrorKind::NotFound {
            Self::MissingSource {
                path: path.to_path_buf(),
            }
        } else {
            Self::Io {
                operation,
                path: path.to_path_buf(),
                kind: error.kind(),
            }
        }
    }
}

impl Display for CollectorError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLimits(field) => {
                write!(
                    formatter,
                    "collector limit {field} must be greater than zero"
                )
            }
            Self::MissingSource { path } => {
                write!(formatter, "canonical source is missing: {}", path.display())
            }
            Self::InvalidRoot { path } => {
                write!(
                    formatter,
                    "canonical content root is invalid: {}",
                    path.display()
                )
            }
            Self::InvalidPath { path, reason } => {
                write!(
                    formatter,
                    "canonical source path {path:?} is invalid: {reason}"
                )
            }
            Self::NonUtf8Path { path } => {
                write!(
                    formatter,
                    "canonical source path is not UTF-8: {}",
                    path.display()
                )
            }
            Self::LinkNotAllowed { path } => {
                write!(
                    formatter,
                    "links and reparse points are not allowed: {path}"
                )
            }
            Self::HardLinkNotAllowed { path } => {
                write!(
                    formatter,
                    "hard-linked canonical source is not allowed: {path}"
                )
            }
            Self::HardLinkAlias { first, second } => {
                write!(
                    formatter,
                    "canonical sources are hard-link aliases: {first} and {second}"
                )
            }
            Self::UnsupportedFileType { path } => {
                write!(
                    formatter,
                    "canonical source is not a regular file or directory: {path}"
                )
            }
            Self::PathCollision { first, second } => {
                write!(
                    formatter,
                    "canonical source paths collide portably: {first} and {second}"
                )
            }
            Self::EntryLimitExceeded { limit } => {
                write!(
                    formatter,
                    "canonical source exceeds the {limit}-entry limit"
                )
            }
            Self::FileTooLarge {
                path,
                size_bytes,
                limit,
            } => write!(
                formatter,
                "canonical source {path} is {size_bytes} bytes, above the {limit}-byte limit"
            ),
            Self::TotalSizeExceeded { size_bytes, limit } => write!(
                formatter,
                "canonical source total is {size_bytes} bytes, above the {limit}-byte limit"
            ),
            Self::SourceChanged {
                path,
                expected_size,
                actual_size,
            } => write!(
                formatter,
                "canonical source {path} changed while being read ({expected_size} to {actual_size} bytes)"
            ),
            Self::Io {
                operation,
                path,
                kind,
            } => write!(
                formatter,
                "could not {operation} canonical source {}: {kind:?}",
                path.display()
            ),
        }
    }
}

impl Error for CollectorError {}

#[derive(Clone, Copy)]
enum SourceMatch {
    Directory,
    File,
}

#[derive(Clone, Copy)]
struct SourceSpec {
    path: &'static str,
    resource_kind: ResourceKind,
    source_match: SourceMatch,
}

const CANONICAL_SOURCES: [SourceSpec; 13] = [
    SourceSpec {
        path: ".codex-plugin",
        resource_kind: ResourceKind::Workflow,
        source_match: SourceMatch::Directory,
    },
    SourceSpec {
        path: "distribution",
        resource_kind: ResourceKind::TargetMetadata,
        source_match: SourceMatch::Directory,
    },
    SourceSpec {
        path: "mcp-contracts",
        resource_kind: ResourceKind::McpContract,
        source_match: SourceMatch::Directory,
    },
    SourceSpec {
        path: "roles",
        resource_kind: ResourceKind::Role,
        source_match: SourceMatch::Directory,
    },
    SourceSpec {
        path: "schemas",
        resource_kind: ResourceKind::Schema,
        source_match: SourceMatch::Directory,
    },
    SourceSpec {
        path: "skills",
        resource_kind: ResourceKind::Skill,
        source_match: SourceMatch::Directory,
    },
    SourceSpec {
        path: "skills-core.md",
        resource_kind: ResourceKind::SkillSummary,
        source_match: SourceMatch::File,
    },
    SourceSpec {
        path: "skills-summary.md",
        resource_kind: ResourceKind::SkillSummary,
        source_match: SourceMatch::File,
    },
    SourceSpec {
        path: "standards",
        resource_kind: ResourceKind::Standard,
        source_match: SourceMatch::Directory,
    },
    SourceSpec {
        path: "subjects",
        resource_kind: ResourceKind::Subject,
        source_match: SourceMatch::Directory,
    },
    SourceSpec {
        path: "templates",
        resource_kind: ResourceKind::Template,
        source_match: SourceMatch::Directory,
    },
    SourceSpec {
        path: "venue-profiles",
        resource_kind: ResourceKind::VenueProfile,
        source_match: SourceMatch::Directory,
    },
    SourceSpec {
        path: "workflow",
        resource_kind: ResourceKind::Workflow,
        source_match: SourceMatch::Directory,
    },
];

pub(crate) fn expected_resource_kind(path: &str) -> Option<ResourceKind> {
    CANONICAL_SOURCES.iter().find_map(|source| {
        let matches = match source.source_match {
            SourceMatch::Directory => path
                .strip_prefix(source.path)
                .is_some_and(|suffix| suffix.starts_with('/')),
            SourceMatch::File => path == source.path,
        };
        matches.then_some(source.resource_kind)
    })
}

struct CollectionState {
    limits: CollectorLimits,
    resources: Vec<CollectedResource>,
    portable_paths: BTreeMap<String, String>,
    file_handles: HashMap<Handle, String>,
    total_bytes: u64,
}

impl CollectionState {
    fn new(limits: CollectorLimits) -> Self {
        Self {
            limits,
            resources: Vec::new(),
            portable_paths: BTreeMap::new(),
            file_handles: HashMap::new(),
            total_bytes: 0,
        }
    }

    fn register_node(&mut self, path: &str) -> Result<(), CollectorError> {
        let key = portable_path_key(path);
        if let Some(previous) = self.portable_paths.get(&key) {
            return Err(CollectorError::PathCollision {
                first: previous.clone(),
                second: path.to_string(),
            });
        }
        self.portable_paths.insert(key, path.to_string());
        Ok(())
    }

    fn collect_file(
        &mut self,
        filesystem_path: &Path,
        path_segments: &[String],
        resource_kind: ResourceKind,
    ) -> Result<(), CollectorError> {
        if self.resources.len() >= self.limits.max_entries {
            return Err(CollectorError::EntryLimitExceeded {
                limit: self.limits.max_entries,
            });
        }

        let pack_path = normalize_pack_path(path_segments, self.limits.max_path_depth)?;
        self.register_node(&pack_path)?;
        let metadata = checked_metadata(filesystem_path, &pack_path)?;
        if !metadata.is_file() {
            return Err(CollectorError::UnsupportedFileType { path: pack_path });
        }
        if has_multiple_hard_links(&metadata) {
            return Err(CollectorError::HardLinkNotAllowed { path: pack_path });
        }
        if metadata.len() > self.limits.max_entry_bytes {
            return Err(CollectorError::FileTooLarge {
                path: pack_path,
                size_bytes: metadata.len(),
                limit: self.limits.max_entry_bytes,
            });
        }

        let mut handle = Handle::from_path(filesystem_path)
            .map_err(|error| CollectorError::io("open", filesystem_path, &error))?;
        if let Some(previous) = self.file_handles.get(&handle) {
            return Err(CollectorError::HardLinkAlias {
                first: previous.clone(),
                second: pack_path,
            });
        }

        let read_limit = self.limits.max_entry_bytes.saturating_add(1);
        let mut bytes = Vec::new();
        handle
            .as_file_mut()
            .take(read_limit)
            .read_to_end(&mut bytes)
            .map_err(|error| CollectorError::io("read", filesystem_path, &error))?;
        let actual_size = bytes.len() as u64;
        if actual_size > self.limits.max_entry_bytes {
            return Err(CollectorError::FileTooLarge {
                path: pack_path,
                size_bytes: actual_size,
                limit: self.limits.max_entry_bytes,
            });
        }

        let final_size = handle
            .as_file()
            .metadata()
            .map_err(|error| CollectorError::io("inspect", filesystem_path, &error))?
            .len();
        if metadata.len() != actual_size || final_size != actual_size {
            return Err(CollectorError::SourceChanged {
                path: pack_path,
                expected_size: metadata.len(),
                actual_size,
            });
        }

        let total_bytes = self.total_bytes.saturating_add(actual_size);
        if total_bytes > self.limits.max_total_bytes {
            return Err(CollectorError::TotalSizeExceeded {
                size_bytes: total_bytes,
                limit: self.limits.max_total_bytes,
            });
        }

        self.total_bytes = total_bytes;
        self.file_handles.insert(handle, pack_path.clone());
        self.resources.push(CollectedResource {
            path: pack_path,
            resource_kind,
            mode: LogicalMode::Regular,
            bytes: bytes.into_boxed_slice(),
        });
        Ok(())
    }
}

pub fn collect_canonical_sources(
    content_root: impl AsRef<Path>,
) -> Result<Vec<CollectedResource>, CollectorError> {
    collect_canonical_sources_with_limits(content_root, CollectorLimits::default())
}

pub fn collect_canonical_sources_with_limits(
    content_root: impl AsRef<Path>,
    limits: CollectorLimits,
) -> Result<Vec<CollectedResource>, CollectorError> {
    limits.validate()?;
    let content_root = content_root.as_ref();
    let root_metadata = fs::symlink_metadata(content_root)
        .map_err(|error| CollectorError::io("inspect", content_root, &error))?;
    if root_metadata.file_type().is_symlink() || is_reparse_point(&root_metadata) {
        return Err(CollectorError::LinkNotAllowed {
            path: "<content-root>".to_string(),
        });
    }
    if !root_metadata.is_dir() {
        return Err(CollectorError::InvalidRoot {
            path: content_root.to_path_buf(),
        });
    }

    let mut state = CollectionState::new(limits);
    for source in CANONICAL_SOURCES {
        let path_segments = source
            .path
            .split('/')
            .map(str::to_string)
            .collect::<Vec<_>>();
        let filesystem_path = content_root.join(source.path);
        match source.source_match {
            SourceMatch::Directory => collect_directory(
                &mut state,
                &filesystem_path,
                &path_segments,
                source.resource_kind,
            )?,
            SourceMatch::File => {
                state.collect_file(&filesystem_path, &path_segments, source.resource_kind)?
            }
        }
    }

    state
        .resources
        .sort_by(|left, right| left.path.as_bytes().cmp(right.path.as_bytes()));
    Ok(state.resources)
}

fn collect_directory(
    state: &mut CollectionState,
    filesystem_path: &Path,
    path_segments: &[String],
    resource_kind: ResourceKind,
) -> Result<(), CollectorError> {
    let pack_path = normalize_pack_path(path_segments, state.limits.max_path_depth)?;
    state.register_node(&pack_path)?;
    let metadata = checked_metadata(filesystem_path, &pack_path)?;
    if !metadata.is_dir() {
        return Err(CollectorError::UnsupportedFileType { path: pack_path });
    }

    let mut children = fs::read_dir(filesystem_path)
        .map_err(|error| CollectorError::io("list", filesystem_path, &error))?
        .map(|entry| {
            let entry =
                entry.map_err(|error| CollectorError::io("list", filesystem_path, &error))?;
            let child_path = entry.path();
            let name =
                entry
                    .file_name()
                    .into_string()
                    .map_err(|_| CollectorError::NonUtf8Path {
                        path: child_path.clone(),
                    })?;
            Ok((name, child_path))
        })
        .collect::<Result<Vec<_>, CollectorError>>()?;
    children.sort_by(|left, right| left.0.as_bytes().cmp(right.0.as_bytes()));

    for (name, child_path) in children {
        let mut child_segments = path_segments.to_vec();
        child_segments.push(name);
        let child_pack_path = normalize_pack_path(&child_segments, state.limits.max_path_depth)?;
        let child_metadata = checked_metadata(&child_path, &child_pack_path)?;
        if child_metadata.is_dir() {
            collect_directory(state, &child_path, &child_segments, resource_kind)?;
        } else if child_metadata.is_file() {
            state.collect_file(&child_path, &child_segments, resource_kind)?;
        } else {
            return Err(CollectorError::UnsupportedFileType {
                path: child_pack_path,
            });
        }
    }
    Ok(())
}

fn checked_metadata(path: &Path, pack_path: &str) -> Result<Metadata, CollectorError> {
    let metadata =
        fs::symlink_metadata(path).map_err(|error| CollectorError::io("inspect", path, &error))?;
    if metadata.file_type().is_symlink() || is_reparse_point(&metadata) {
        return Err(CollectorError::LinkNotAllowed {
            path: pack_path.to_string(),
        });
    }
    Ok(metadata)
}

fn normalize_pack_path(
    path_segments: &[String],
    max_depth: usize,
) -> Result<String, CollectorError> {
    let raw_path = path_segments.join("/");
    if path_segments.is_empty() {
        return Err(CollectorError::InvalidPath {
            path: raw_path,
            reason: "path must not be empty",
        });
    }
    if path_segments.len() > max_depth {
        return Err(CollectorError::InvalidPath {
            path: raw_path,
            reason: "path exceeds the configured depth limit",
        });
    }

    let mut normalized_segments = Vec::with_capacity(path_segments.len());
    for component in path_segments {
        let normalized = component.nfc().collect::<String>();
        validate_path_component(&normalized, &raw_path)?;
        normalized_segments.push(normalized);
    }
    Ok(normalized_segments.join("/"))
}

fn validate_path_component(component: &str, raw_path: &str) -> Result<(), CollectorError> {
    if component.is_empty() || component == "." || component == ".." {
        return Err(CollectorError::InvalidPath {
            path: raw_path.to_string(),
            reason: "path contains an empty or traversal component",
        });
    }
    if component.contains('/') || component.contains('\\') {
        return Err(CollectorError::InvalidPath {
            path: raw_path.to_string(),
            reason: "path contains a separator inside a component",
        });
    }
    if component.ends_with('.') || component.ends_with(' ') {
        return Err(CollectorError::InvalidPath {
            path: raw_path.to_string(),
            reason: "path component ends with a dot or space",
        });
    }
    if component.chars().any(|character| {
        character.is_control() || matches!(character, ':' | '*' | '?' | '"' | '<' | '>' | '|')
    }) {
        return Err(CollectorError::InvalidPath {
            path: raw_path.to_string(),
            reason: "path contains a non-portable character",
        });
    }
    if is_windows_device_name(component) {
        return Err(CollectorError::InvalidPath {
            path: raw_path.to_string(),
            reason: "path contains a reserved Windows device name",
        });
    }
    Ok(())
}

fn is_windows_device_name(component: &str) -> bool {
    let base = component.split('.').next().unwrap_or(component);
    let upper = base.to_ascii_uppercase();
    matches!(upper.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || (upper.len() == 4
            && matches!(&upper[..3], "COM" | "LPT")
            && matches!(upper.as_bytes()[3], b'1'..=b'9'))
}

fn portable_path_key(path: &str) -> String {
    path.chars().flat_map(char::to_lowercase).nfc().collect()
}

#[cfg(unix)]
fn has_multiple_hard_links(metadata: &Metadata) -> bool {
    use std::os::unix::fs::MetadataExt;

    metadata.nlink() > 1
}

#[cfg(not(unix))]
fn has_multiple_hard_links(_metadata: &Metadata) -> bool {
    false
}

#[cfg(windows)]
fn is_reparse_point(metadata: &Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;

    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0400;
    metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

#[cfg(not(windows))]
fn is_reparse_point(_metadata: &Metadata) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn segments(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_string()).collect()
    }

    #[test]
    fn path_validation_rejects_traversal_and_non_portable_components() {
        for candidate in [
            segments(&["..", "escape.md"]),
            segments(&["skills", ""]),
            segments(&["skills", "nested\\escape.md"]),
            segments(&["skills", "CON.txt"]),
            segments(&["skills", "trailing."]),
        ] {
            assert!(normalize_pack_path(&candidate, 32).is_err());
        }
    }

    #[test]
    fn unicode_and_case_collisions_fail_closed() {
        let limits = CollectorLimits::default();
        let mut state = CollectionState::new(limits);
        let first = normalize_pack_path(&segments(&["skills", "CAFÉ.md"]), 32)
            .expect("first path must normalize");
        let second = normalize_pack_path(&segments(&["SKILLS", "cafe\u{301}.md"]), 32)
            .expect("second path must normalize");
        state
            .register_node(&first)
            .expect("first path must register");
        assert!(matches!(
            state.register_node(&second),
            Err(CollectorError::PathCollision { .. })
        ));
    }

    #[test]
    fn path_depth_is_bounded() {
        let error =
            normalize_pack_path(&segments(&["a", "b", "c"]), 2).expect_err("deep path must fail");
        assert!(matches!(error, CollectorError::InvalidPath { .. }));
    }
}
