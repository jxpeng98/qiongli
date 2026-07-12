pub mod manifest;

pub use manifest::{
    CompatibleProduct, LogicalMode, ManifestError, ProfileId, ProfileProjection, ResourceEntry,
    ResourceKind, ResourcePackManifestV1,
};

pub const RESOURCE_PACK_MANIFEST_SCHEMA_V1: &str =
    include_str!("../schemas/resource-pack-manifest-v1.schema.json");
