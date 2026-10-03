use qiongli_platform::{
    ArtifactIdentityV1, ReleaseChannel, current_target_native_artifact_identity,
};

pub fn current_native_artifact() -> ArtifactIdentityV1 {
    [
        ReleaseChannel::Alpha,
        ReleaseChannel::Beta,
        ReleaseChannel::Stable,
    ]
    .into_iter()
    .find_map(|channel| {
        current_target_native_artifact_identity(env!("CARGO_PKG_VERSION"), channel).ok()
    })
    .expect("current artifact version must match a supported release channel")
}
