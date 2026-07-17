use crate::loader::{LoadedResource, LoadedResourcePack, ResourcePackLoaderError};
use crate::manifest::ProfileProjection;
use crate::materializer::{
    MaterializationError, MaterializationReceiptV1, MaterializationTarget, materialize_profile,
};

#[derive(Debug)]
pub struct EmbeddedContent {
    pack: LoadedResourcePack<'static>,
}

impl EmbeddedContent {
    pub fn load(
        core_bytes: &'static [u8],
        expected_pack_sha256: &str,
    ) -> Result<Self, ResourcePackLoaderError> {
        let pack = crate::loader::load_resource_pack(core_bytes, expected_pack_sha256)?;
        Ok(Self { pack })
    }

    #[must_use]
    pub fn pack(&self) -> &LoadedResourcePack<'static> {
        &self.pack
    }

    #[must_use]
    pub fn profiles(&self) -> &[ProfileProjection] {
        &self.pack.manifest().profiles
    }

    pub fn read_profile_resource<'pack>(
        &'pack self,
        profile: &str,
        path: &str,
    ) -> Result<Option<LoadedResource<'pack, 'static>>, ResourcePackLoaderError> {
        self.pack.resource_for_profile(profile, path)
    }

    pub fn materialize_profile(
        &self,
        profile: &str,
        target: &MaterializationTarget,
    ) -> Result<MaterializationReceiptV1, MaterializationError> {
        materialize_profile(&self.pack, profile, target)
    }
}
