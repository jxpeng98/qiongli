pub mod collector;
pub mod manifest;

pub use collector::{
    CollectedResource, CollectorError, CollectorLimits, collect_canonical_sources,
    collect_canonical_sources_with_limits,
};
pub use manifest::{
    CompatibleProduct, LogicalMode, ManifestError, ProfileId, ProfileProjection, ResourceEntry,
    ResourceKind, ResourcePackManifestV1,
};

pub const RESOURCE_PACK_MANIFEST_SCHEMA_V1: &str =
    include_str!("../schemas/resource-pack-manifest-v1.schema.json");
