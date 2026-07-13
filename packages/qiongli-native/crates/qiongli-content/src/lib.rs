pub mod collector;
pub mod manifest;
pub mod writer;

pub use collector::{
    CollectedResource, CollectorError, CollectorLimits, collect_canonical_sources,
    collect_canonical_sources_with_limits,
};
pub use manifest::{
    CompatibleProduct, JCS_MAX_SAFE_INTEGER, LogicalMode, ManifestError, ProfileId,
    ProfileProjection, RESOURCE_PACK_COMPILER_CONTRACT_VERSION, RESOURCE_PACK_FORMAT_VERSION,
    ResourceEntry, ResourceKind, ResourcePackManifestV1,
};
pub use writer::{
    BuiltResourcePack, RESOURCE_PACK_CONTENT_ROOT_DOMAIN_V1, RESOURCE_PACK_HEADER_LEN,
    RESOURCE_PACK_MAGIC, ResourcePackBuildMetadata, ResourcePackWriterError, build_resource_pack,
};

pub const RESOURCE_PACK_MANIFEST_SCHEMA_V1: &str =
    include_str!("../schemas/resource-pack-manifest-v1.schema.json");
