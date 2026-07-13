pub mod collector;
pub mod loader;
pub mod manifest;
pub mod materializer;
pub mod writer;

pub use collector::{
    CollectedResource, CollectorError, CollectorLimits, collect_canonical_sources,
    collect_canonical_sources_with_limits,
};
pub use loader::{
    LoadedResource, LoadedResourcePack, ResourcePackLimits, ResourcePackLoaderError,
    load_resource_pack, load_resource_pack_with_limits,
};
pub use manifest::{
    CompatibleProduct, JCS_MAX_SAFE_INTEGER, LogicalMode, ManifestError, ProfileId,
    ProfileProjection, RESOURCE_PACK_COMPILER_CONTRACT_VERSION, RESOURCE_PACK_FORMAT_VERSION,
    ResourceEntry, ResourceKind, ResourcePackManifestV1,
};
pub use materializer::{
    MATERIALIZATION_RECEIPT_FILE, MATERIALIZATION_RECEIPT_VERSION, MaterializationAuthorization,
    MaterializationError, MaterializationReceiptV1, MaterializationTarget, MaterializedEntry,
    approve_materialization_target, materialize_profile, temporary_materialization_target,
};
pub use writer::{
    BuiltResourcePack, RESOURCE_PACK_CONTENT_ROOT_DOMAIN_V1, RESOURCE_PACK_HEADER_LEN,
    RESOURCE_PACK_MAGIC, ResourcePackBuildMetadata, ResourcePackWriterError, build_resource_pack,
};

pub const RESOURCE_PACK_MANIFEST_SCHEMA_V1: &str =
    include_str!("../schemas/resource-pack-manifest-v1.schema.json");
