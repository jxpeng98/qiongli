mod activation;
mod candidate_install;
mod candidate_source;
mod claude;
mod claude_bundle;
mod codex;
mod codex_bundle;
mod desktop_package;
mod error;
mod grant;
mod identity;
mod native_archive;
mod native_artifact;
mod native_install;
mod native_release;
mod plan;
mod release_authority;
mod release_candidate;
mod transaction;

pub use activation::{
    CLIENT_ACTIVATION_SCHEMA_VERSION, ClientActivationCommit, ClientActivationCoordinator,
    ClientActivationDiscoveryV1, ClientActivationDisposition, ClientActivationEffect,
    ClientActivationError, ClientActivationHandle, ClientActivationLifecycleCommit,
    ClientActivationLifecycleDisposition, ClientActivationPreview, ClientActivationState,
    ClientActivationTarget, ClientActivationVerification, discover_client_activation,
    preview_client_activation,
};
pub use candidate_install::{
    NATIVE_CANDIDATE_MANAGED_ROOT_SYMBOLIC_PATH, NativeCandidateLocalInstallCommit,
    NativeCandidateLocalInstallError, NativeCandidateLocalRemoveCommit,
    NativeCandidateLocalVerification, NativeCandidateRegistrationCommit,
    NativeCandidateRegistrationLifecycleCommit, NativeCandidateRegistrationVerification,
    apply_native_release_candidate_local, discover_native_candidate_managed_root,
    prepare_native_candidate_managed_root, remove_native_release_candidate_local,
    verify_native_release_candidate_local,
};
pub use candidate_source::{
    NativeCandidatePluginSourceCommit, NativeCandidatePluginSourceDisposition,
    NativeCandidatePluginSourceError, NativeCandidatePluginSourceTarget,
    NativeCandidatePluginSourceVerification, discover_native_candidate_plugin_source_target,
    materialize_native_candidate_plugin_source, prepare_native_candidate_plugin_source_target,
    remove_native_candidate_plugin_source, verify_native_candidate_plugin_source,
};
pub use claude::{
    CLAUDE_ADAPTER_SCHEMA_VERSION, CLAUDE_MARKETPLACE_SYMBOLIC_PATH,
    CLAUDE_PLUGIN_SOURCE_MARKETPLACE_PATH, CLAUDE_PLUGIN_SOURCE_SYMBOLIC_PATH,
    CLAUDE_REGISTRATION_RECEIPT_SCHEMA_VERSION, CLAUDE_REGISTRATION_STATE_SCHEMA_VERSION,
    CLAUDE_SKILLS_PLUGIN_SYMBOLIC_PATH, ClaudeAdapterError, ClaudeDiscoverySummaryV1,
    ClaudeMarketplaceState, ClaudeRegistrationCommit, ClaudeRegistrationDisposition,
    ClaudeRegistrationEffect, ClaudeRegistrationExecutor, ClaudeRegistrationLifecycleCommit,
    ClaudeRegistrationLifecycleDisposition, ClaudeRegistrationLifecycleKind,
    ClaudeRegistrationLifecycleReceiptV1, ClaudeRegistrationPreview, ClaudeRegistrationReceiptV1,
    ClaudeRegistrationState, ClaudeRegistrationStateV1, ClaudeRegistrationVerification,
    ClaudeSkillsPluginState, ClaudeSourceState, ClaudeUserTarget, discover_claude_user,
    discover_claude_user_with_config, preview_claude_registration,
};
pub use claude_bundle::{
    CLAUDE_PLUGIN_BUNDLE_RECEIPT_FILE, CLAUDE_PLUGIN_BUNDLE_RECEIPT_SCHEMA_VERSION,
    ClaudePluginBundleEntryV1, ClaudePluginBundleError, ClaudePluginBundleKind,
    ClaudePluginBundleReceiptV1, ClaudePluginBundleTarget, VerifiedClaudePluginBundle,
    approve_claude_plugin_bundle_target, compose_claude_plugin_bundle, remove_claude_plugin_bundle,
    verify_claude_plugin_bundle,
};
pub use codex::{
    CODEX_ADAPTER_SCHEMA_VERSION, CODEX_MARKETPLACE_SYMBOLIC_PATH,
    CODEX_PLUGIN_SOURCE_MARKETPLACE_PATH, CODEX_PLUGIN_SOURCE_SYMBOLIC_PATH,
    CODEX_REGISTRATION_RECEIPT_SCHEMA_VERSION, CODEX_REGISTRATION_STATE_SCHEMA_VERSION,
    CodexAdapterError, CodexDiscoverySummaryV1, CodexMarketplaceState, CodexRegistrationCommit,
    CodexRegistrationDisposition, CodexRegistrationEffect, CodexRegistrationExecutor,
    CodexRegistrationLifecycleCommit, CodexRegistrationLifecycleDisposition,
    CodexRegistrationLifecycleKind, CodexRegistrationLifecycleReceiptV1, CodexRegistrationPreview,
    CodexRegistrationReceiptV1, CodexRegistrationState, CodexRegistrationStateV1,
    CodexRegistrationVerification, CodexSourceState, CodexUserTarget, discover_codex_user,
    preview_codex_registration,
};
pub use codex_bundle::{
    CODEX_PLUGIN_BUNDLE_RECEIPT_FILE, CODEX_PLUGIN_BUNDLE_RECEIPT_SCHEMA_VERSION,
    CodexPluginBundleEntryV1, CodexPluginBundleError, CodexPluginBundleKind,
    CodexPluginBundleReceiptV1, CodexPluginBundleTarget, VerifiedCodexPluginBundle,
    approve_codex_plugin_bundle_target, compose_codex_plugin_bundle, remove_codex_plugin_bundle,
    verify_codex_plugin_bundle,
};
pub use desktop_package::{
    DESKTOP_PACKAGE_MANIFEST_FILE, DESKTOP_PACKAGE_MANIFEST_SCHEMA_VERSION,
    DesktopApplicationMetadataV1, DesktopPackageEntryV1, DesktopPackageError, DesktopPackageInput,
    DesktopPackageKind, DesktopPackageManifestV1, DesktopPackageRecordType, DesktopPackageStatus,
    VerifiedDesktopPackage, compose_desktop_package, desktop_package_file_name,
    verify_desktop_package,
};
pub use error::PlatformError;
pub use grant::{
    GrantMode, GrantSignatureV1, GrantVerificationContext, IntegrationScope, LaunchGrantV1,
    SignatureAlgorithm, SignedLaunchGrantV1, TrustedPublicKey, VerifiedLaunchGrant,
    launch_grant_signing_bytes,
};
pub use identity::{
    Architecture, ArtifactIdentityV1, CapabilityProfile, InstallerKind, OperatingSystem, ProductId,
    ReleaseChannel,
};
pub use native_archive::{
    NATIVE_PORTABLE_ARCHIVE_EXTENSION, NativePortableArchiveError, NativePortableArchiveTarget,
    VerifiedNativePortableArchive, approve_native_portable_archive_target,
    compose_native_portable_archive, extract_native_portable_archive,
    native_portable_archive_file_name, verify_native_portable_archive,
};
pub use native_artifact::{
    NATIVE_ARTIFACT_MANIFEST_FILE, NATIVE_ARTIFACT_MANIFEST_SCHEMA_VERSION,
    NativeArtifactContentV1, NativeArtifactEntryV1, NativeArtifactError, NativeArtifactManifestV1,
    NativeArtifactRecordType, NativeArtifactStatus, NativeArtifactTarget, VerifiedNativeArtifact,
    approve_native_artifact_target, compose_native_artifact,
    current_target_native_artifact_identity, native_artifact_binary_path, native_artifact_id,
    verify_native_artifact,
};
pub use native_install::{
    ManagedNativePayloadExecutor, NATIVE_PAYLOAD_INSTALL_JOURNAL_SCHEMA_VERSION,
    NATIVE_PAYLOAD_INSTALL_RECEIPT_SCHEMA_VERSION, NATIVE_PAYLOAD_INSTALL_STATE_SCHEMA_VERSION,
    NativePayloadInstallCommit, NativePayloadInstallReceiptV1, NativePayloadInstallStateV1,
    NativePayloadInstallVerification, NativePayloadLifecycleCommit,
    NativePayloadLifecycleReceiptV1, NativePayloadOperationReceiptV1, native_payload_install_id,
    preview_native_payload_install,
};
pub use native_release::{
    MAX_NATIVE_RELEASE_ENVELOPE_BYTES, NATIVE_RELEASE_ENVELOPE_SCHEMA_VERSION,
    NativeReleaseEnvelopeV1, NativeReleaseError, NativeReleaseSignatureV1,
    NativeReleaseVerificationContext, SignedNativeReleaseEnvelopeV1, TrustedReleasePublicKey,
    VerifiedNativeReleaseEnvelope, build_native_release_envelope,
    native_release_envelope_signing_bytes,
};
pub use plan::{
    AllowedRootV1, ApprovalRequirement, HostAction, InstallActionV1, InstallOperationV1,
    InstallPlanDraftV1, InstallPlanMetadataV1, InstallPlanV1, InstallScope, LocalSurface,
    LocalTargetFamily, OwnershipMarkerV1, PlanStateV1, SymbolicRoot, TargetDescriptorV1,
    VerifiedInstallPlan, observed_plan_state_sha256,
};
pub use release_authority::{
    MAX_NATIVE_RELEASE_AUTHORITY_BYTES, NATIVE_RELEASE_AUTHORITY_SCHEMA_VERSION,
    NativeReleaseAuthority, NativeReleaseAuthorityError,
};
pub use release_candidate::{
    MAX_NATIVE_RELEASE_CANDIDATE_BYTES, MAX_NATIVE_RELEASE_NOTES_BYTES,
    NATIVE_RELEASE_CANDIDATE_SCHEMA_VERSION, NativeClientPluginGrantV1,
    NativeReleaseCandidateError, NativeReleaseCandidateStatus, NativeReleaseCandidateV1,
    NativeReleaseCandidateVerificationContext, NativeReleaseNotesV1,
    SignedNativeReleaseCandidateV1, VerifiedNativeReleaseCandidate, build_native_release_candidate,
    native_release_candidate_file_name, native_release_candidate_signing_bytes,
    native_release_notes_file_name,
};
pub use transaction::{
    ApprovedInstallPlan, ApprovedManagedRoot, INSTALL_JOURNAL_SCHEMA_VERSION,
    INSTALL_RECEIPT_SCHEMA_VERSION, InstallCommit, InstallDisposition, InstallLifecycleKind,
    InstallLifecycleReceiptV1, InstallReceiptV1, InstallVerification, LifecycleCommit,
    LifecycleDisposition, MANAGED_INSTALL_STATE_SCHEMA_VERSION, ManagedInstallStateV1,
    ManagedOperationReceiptV1, ManagedResourceExecutor, TransactionError, approve_install_plan,
    approve_managed_root,
};

pub const ARTIFACT_IDENTITY_SCHEMA_VERSION: u32 = 1;
pub const LAUNCH_GRANT_SCHEMA_VERSION: u32 = 1;
pub const INSTALL_PLAN_SCHEMA_VERSION: u32 = 1;
