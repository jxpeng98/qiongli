use ed25519_dalek::{Signature, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{ArtifactIdentityV1, LAUNCH_GRANT_SCHEMA_VERSION, PlatformError};

const MAX_LAUNCH_GRANT_BYTES: usize = 64 * 1024;
const MAX_KEY_ID_BYTES: usize = 64;
const ED25519_PUBLIC_KEY_BYTES: usize = 32;
const ED25519_SIGNATURE_BYTES: usize = 64;
const JCS_MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;
const SIGNING_DOMAIN: &[u8] = b"QIONGLI-LAUNCH-GRANT-V1\0";

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum GrantMode {
    Cli,
    LiteMcp,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum IntegrationScope {
    CodexLocal,
    ClaudeCodeLocal,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SignatureAlgorithm {
    Ed25519,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LaunchGrantV1 {
    pub schema_version: u32,
    pub generation: u64,
    pub artifact: ArtifactIdentityV1,
    pub binary_sha256: String,
    pub resource_pack_sha256: String,
    pub allowed_modes: Vec<GrantMode>,
    pub integration_scopes: Vec<IntegrationScope>,
    pub not_before_unix: u64,
    pub expires_at_unix: u64,
}

impl LaunchGrantV1 {
    pub fn validate(&self) -> Result<(), PlatformError> {
        if self.schema_version != LAUNCH_GRANT_SCHEMA_VERSION {
            return Err(PlatformError::InvalidLaunchGrantSchema);
        }
        self.artifact.validate_lite()?;
        if self.generation == 0 || self.generation > JCS_MAX_SAFE_INTEGER {
            return Err(PlatformError::InvalidLaunchGrant);
        }
        validate_digest(&self.binary_sha256)?;
        validate_digest(&self.resource_pack_sha256)?;
        if !is_sorted_unique(&self.allowed_modes)
            || self.allowed_modes.is_empty()
            || self.allowed_modes.len() > 2
            || self
                .allowed_modes
                .binary_search(&GrantMode::LiteMcp)
                .is_err()
        {
            return Err(PlatformError::InvalidLaunchGrant);
        }
        if !is_sorted_unique(&self.integration_scopes)
            || self.integration_scopes.is_empty()
            || self.integration_scopes.len() > 2
        {
            return Err(PlatformError::InvalidLaunchGrant);
        }
        if self.not_before_unix > JCS_MAX_SAFE_INTEGER
            || self.expires_at_unix > JCS_MAX_SAFE_INTEGER
            || self.not_before_unix >= self.expires_at_unix
        {
            return Err(PlatformError::InvalidLaunchGrant);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GrantSignatureV1 {
    pub algorithm: SignatureAlgorithm,
    pub key_id: String,
    pub value_hex: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SignedLaunchGrantV1 {
    pub grant: LaunchGrantV1,
    pub signature: GrantSignatureV1,
}

impl SignedLaunchGrantV1 {
    pub fn from_json(input: &[u8]) -> Result<Self, PlatformError> {
        if input.len() > MAX_LAUNCH_GRANT_BYTES {
            return Err(PlatformError::LaunchGrantTooLarge);
        }
        let signed = serde_json::from_slice::<Self>(input)
            .map_err(|_| PlatformError::InvalidLaunchGrantJson)?;
        signed.validate_structure()?;
        Ok(signed)
    }

    pub fn to_canonical_json(&self) -> Result<Vec<u8>, PlatformError> {
        self.validate_structure()?;
        serde_json_canonicalizer::to_vec(self)
            .map_err(|_| PlatformError::CanonicalSerializationFailed)
    }

    pub fn verify(
        &self,
        trusted_keys: &[TrustedPublicKey],
        context: &GrantVerificationContext<'_>,
    ) -> Result<VerifiedLaunchGrant, PlatformError> {
        self.validate_structure()?;
        validate_trusted_keys(trusted_keys)?;
        let trusted_key = trusted_keys
            .iter()
            .find(|key| key.key_id == self.signature.key_id)
            .ok_or(PlatformError::LaunchGrantKeyUntrusted)?;
        let signature_bytes =
            decode_fixed_hex::<ED25519_SIGNATURE_BYTES>(&self.signature.value_hex)
                .ok_or(PlatformError::InvalidLaunchGrant)?;
        let signed_bytes = launch_grant_signing_bytes(&self.grant)?;
        let verifying_key = VerifyingKey::from_bytes(&trusted_key.public_key)
            .map_err(|_| PlatformError::LaunchGrantSignatureInvalid)?;
        let signature = Signature::from_bytes(&signature_bytes);
        verifying_key
            .verify_strict(&signed_bytes, &signature)
            .map_err(|_| PlatformError::LaunchGrantSignatureInvalid)?;

        if context.now_unix < self.grant.not_before_unix {
            return Err(PlatformError::LaunchGrantNotYetValid);
        }
        if context.now_unix >= self.grant.expires_at_unix {
            return Err(PlatformError::LaunchGrantExpired);
        }
        if self.grant.generation < context.minimum_generation {
            return Err(PlatformError::LaunchGrantReplayed);
        }
        if &self.grant.artifact != context.expected_artifact {
            return Err(PlatformError::LaunchGrantArtifactMismatch);
        }
        validate_digest(context.binary_sha256)?;
        if self.grant.binary_sha256 != context.binary_sha256 {
            return Err(PlatformError::LaunchGrantBinaryMismatch);
        }
        validate_digest(context.resource_pack_sha256)?;
        if self.grant.resource_pack_sha256 != context.resource_pack_sha256 {
            return Err(PlatformError::LaunchGrantContentMismatch);
        }
        if self
            .grant
            .allowed_modes
            .binary_search(&context.requested_mode)
            .is_err()
        {
            return Err(PlatformError::LaunchGrantModeUnavailable);
        }
        if self
            .grant
            .integration_scopes
            .binary_search(&context.requested_scope)
            .is_err()
        {
            return Err(PlatformError::LaunchGrantScopeUnavailable);
        }

        Ok(VerifiedLaunchGrant {
            signed: self.clone(),
            signed_payload_sha256: sha256_hex(&signed_bytes),
            authorized_mode: context.requested_mode,
            authorized_scope: context.requested_scope,
            verified_at_unix: context.now_unix,
        })
    }

    pub(crate) fn validate_structure(&self) -> Result<(), PlatformError> {
        self.grant.validate()?;
        if self.signature.algorithm != SignatureAlgorithm::Ed25519
            || !valid_identifier(&self.signature.key_id, MAX_KEY_ID_BYTES)
            || !is_lower_hex(
                &self.signature.value_hex,
                ED25519_SIGNATURE_BYTES.saturating_mul(2),
            )
        {
            return Err(PlatformError::InvalidLaunchGrant);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustedPublicKey {
    key_id: String,
    public_key: [u8; ED25519_PUBLIC_KEY_BYTES],
}

impl TrustedPublicKey {
    pub fn new(
        key_id: impl Into<String>,
        public_key: [u8; ED25519_PUBLIC_KEY_BYTES],
    ) -> Result<Self, PlatformError> {
        let key_id = key_id.into();
        if !valid_identifier(&key_id, MAX_KEY_ID_BYTES) {
            return Err(PlatformError::InvalidLaunchGrant);
        }
        Ok(Self { key_id, public_key })
    }

    #[must_use]
    pub fn key_id(&self) -> &str {
        &self.key_id
    }
}

#[derive(Clone, Copy, Debug)]
pub struct GrantVerificationContext<'a> {
    pub now_unix: u64,
    pub minimum_generation: u64,
    pub expected_artifact: &'a ArtifactIdentityV1,
    pub binary_sha256: &'a str,
    pub resource_pack_sha256: &'a str,
    pub requested_mode: GrantMode,
    pub requested_scope: IntegrationScope,
}

#[derive(Clone, Debug)]
pub struct VerifiedLaunchGrant {
    signed: SignedLaunchGrantV1,
    signed_payload_sha256: String,
    authorized_mode: GrantMode,
    authorized_scope: IntegrationScope,
    verified_at_unix: u64,
}

impl VerifiedLaunchGrant {
    #[must_use]
    pub fn grant(&self) -> &LaunchGrantV1 {
        &self.signed.grant
    }

    #[must_use]
    pub fn signed_grant(&self) -> &SignedLaunchGrantV1 {
        &self.signed
    }

    #[must_use]
    pub fn signed_payload_sha256(&self) -> &str {
        &self.signed_payload_sha256
    }

    #[must_use]
    pub const fn authorized_mode(&self) -> GrantMode {
        self.authorized_mode
    }

    #[must_use]
    pub const fn authorized_scope(&self) -> IntegrationScope {
        self.authorized_scope
    }

    #[must_use]
    pub const fn verified_at_unix(&self) -> u64 {
        self.verified_at_unix
    }
}

pub fn launch_grant_signing_bytes(grant: &LaunchGrantV1) -> Result<Vec<u8>, PlatformError> {
    grant.validate()?;
    let canonical = serde_json_canonicalizer::to_vec(grant)
        .map_err(|_| PlatformError::CanonicalSerializationFailed)?;
    let mut output = Vec::with_capacity(SIGNING_DOMAIN.len().saturating_add(canonical.len()));
    output.extend_from_slice(SIGNING_DOMAIN);
    output.extend_from_slice(&canonical);
    Ok(output)
}

pub(crate) fn validate_digest(value: &str) -> Result<(), PlatformError> {
    if is_lower_hex(value, 64) {
        Ok(())
    } else {
        Err(PlatformError::InvalidLaunchGrant)
    }
}

pub(crate) fn is_lower_hex(value: &str, expected_len: usize) -> bool {
    value.len() == expected_len
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

pub(crate) fn valid_identifier(value: &str, maximum_bytes: usize) -> bool {
    !value.is_empty()
        && value.len() <= maximum_bytes
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_')
        })
}

pub(crate) fn is_sorted_unique<T: Ord>(values: &[T]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}

pub(crate) fn sha256_hex(input: &[u8]) -> String {
    let digest = Sha256::digest(input);
    let mut output = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write;
        write!(&mut output, "{byte:02x}").expect("writing to String cannot fail");
    }
    output
}

fn validate_trusted_keys(trusted_keys: &[TrustedPublicKey]) -> Result<(), PlatformError> {
    if trusted_keys.len() > 16 {
        return Err(PlatformError::InvalidLaunchGrant);
    }
    for (index, key) in trusted_keys.iter().enumerate() {
        if trusted_keys[..index]
            .iter()
            .any(|prior| prior.key_id == key.key_id)
        {
            return Err(PlatformError::InvalidLaunchGrant);
        }
    }
    Ok(())
}

fn decode_fixed_hex<const N: usize>(value: &str) -> Option<[u8; N]> {
    if !is_lower_hex(value, N.checked_mul(2)?) {
        return None;
    }
    let mut output = [0_u8; N];
    for (index, pair) in value.as_bytes().chunks_exact(2).enumerate() {
        output[index] = (hex_value(pair[0])? << 4) | hex_value(pair[1])?;
    }
    Some(output)
}

const fn hex_value(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    use ed25519_dalek::{Signer, SigningKey};

    use super::*;
    use crate::{
        Architecture, CapabilityProfile, InstallerKind, OperatingSystem, ProductId, ReleaseChannel,
    };

    const BINARY_DIGEST: &str = "1111111111111111111111111111111111111111111111111111111111111111";
    const PACK_DIGEST: &str = "2222222222222222222222222222222222222222222222222222222222222222";
    static NEXT_KEY_ID: AtomicU64 = AtomicU64::new(0);

    fn artifact() -> ArtifactIdentityV1 {
        ArtifactIdentityV1 {
            product: ProductId::Qiongli,
            version: "2.0.0-alpha.1".to_string(),
            channel: ReleaseChannel::Alpha,
            profile: CapabilityProfile::Lite,
            os: OperatingSystem::Linux,
            arch: Architecture::X86_64,
            installer_kind: InstallerKind::PortableArchive,
        }
    }

    fn signed_fixture() -> (SignedLaunchGrantV1, TrustedPublicKey) {
        signed_fixture_with(
            vec![GrantMode::Cli, GrantMode::LiteMcp],
            vec![
                IntegrationScope::CodexLocal,
                IntegrationScope::ClaudeCodeLocal,
            ],
        )
    }

    fn signed_fixture_with(
        allowed_modes: Vec<GrantMode>,
        integration_scopes: Vec<IntegrationScope>,
    ) -> (SignedLaunchGrantV1, TrustedPublicKey) {
        let grant = LaunchGrantV1 {
            schema_version: LAUNCH_GRANT_SCHEMA_VERSION,
            generation: 7,
            artifact: artifact(),
            binary_sha256: BINARY_DIGEST.to_string(),
            resource_pack_sha256: PACK_DIGEST.to_string(),
            allowed_modes,
            integration_scopes,
            not_before_unix: 1_700_000_000,
            expires_at_unix: 1_800_000_000,
        };
        let key_pair = temporary_test_signing_key();
        let signing_bytes = launch_grant_signing_bytes(&grant).unwrap();
        let signature = key_pair.sign(&signing_bytes);
        let public_key = key_pair.verifying_key().to_bytes();
        (
            SignedLaunchGrantV1 {
                grant,
                signature: GrantSignatureV1 {
                    algorithm: SignatureAlgorithm::Ed25519,
                    key_id: "test-key-1".to_string(),
                    value_hex: encode_hex(&signature.to_bytes()),
                },
            },
            TrustedPublicKey::new("test-key-1", public_key).unwrap(),
        )
    }

    fn context(artifact: &ArtifactIdentityV1) -> GrantVerificationContext<'_> {
        GrantVerificationContext {
            now_unix: 1_750_000_000,
            minimum_generation: 7,
            expected_artifact: artifact,
            binary_sha256: BINARY_DIGEST,
            resource_pack_sha256: PACK_DIGEST,
            requested_mode: GrantMode::LiteMcp,
            requested_scope: IntegrationScope::CodexLocal,
        }
    }

    #[test]
    fn valid_signature_creates_verified_token() {
        let (signed, key) = signed_fixture();
        let expected = artifact();
        let verified = signed.verify(&[key], &context(&expected)).unwrap();
        assert_eq!(verified.grant().generation, 7);
        assert_eq!(verified.signed_payload_sha256().len(), 64);
    }

    #[test]
    fn tampering_and_untrusted_keys_fail_closed() {
        let (mut signed, key) = signed_fixture();
        let expected = artifact();
        signed.grant.generation = 8;
        assert_eq!(
            signed.verify(&[key], &context(&expected)).unwrap_err(),
            PlatformError::LaunchGrantSignatureInvalid
        );

        let (signed, _) = signed_fixture();
        assert_eq!(
            signed.verify(&[], &context(&expected)).unwrap_err(),
            PlatformError::LaunchGrantKeyUntrusted
        );

        let (signed, key) = signed_fixture();
        assert_eq!(
            signed
                .verify(&[key.clone(), key], &context(&expected))
                .unwrap_err(),
            PlatformError::InvalidLaunchGrant
        );
    }

    #[test]
    fn time_generation_identity_digest_mode_and_scope_are_bound() {
        let (signed, key) = signed_fixture();
        let expected = artifact();
        let mut check = context(&expected);

        check.now_unix = signed.grant.not_before_unix - 1;
        assert_eq!(
            signed
                .verify(std::slice::from_ref(&key), &check)
                .unwrap_err(),
            PlatformError::LaunchGrantNotYetValid
        );
        check.now_unix = signed.grant.expires_at_unix;
        assert_eq!(
            signed
                .verify(std::slice::from_ref(&key), &check)
                .unwrap_err(),
            PlatformError::LaunchGrantExpired
        );
        check = context(&expected);
        check.minimum_generation = 8;
        assert_eq!(
            signed
                .verify(std::slice::from_ref(&key), &check)
                .unwrap_err(),
            PlatformError::LaunchGrantReplayed
        );

        let mut other_artifact = expected.clone();
        other_artifact.installer_kind = InstallerKind::PluginBundle;
        check = context(&other_artifact);
        assert_eq!(
            signed
                .verify(std::slice::from_ref(&key), &check)
                .unwrap_err(),
            PlatformError::LaunchGrantArtifactMismatch
        );

        check = context(&expected);
        check.binary_sha256 = PACK_DIGEST;
        assert_eq!(
            signed
                .verify(std::slice::from_ref(&key), &check)
                .unwrap_err(),
            PlatformError::LaunchGrantBinaryMismatch
        );
        check = context(&expected);
        check.resource_pack_sha256 = BINARY_DIGEST;
        assert_eq!(
            signed
                .verify(std::slice::from_ref(&key), &check)
                .unwrap_err(),
            PlatformError::LaunchGrantContentMismatch
        );

        let (signed, key) = signed_fixture_with(
            vec![GrantMode::LiteMcp],
            vec![
                IntegrationScope::CodexLocal,
                IntegrationScope::ClaudeCodeLocal,
            ],
        );
        check = context(&expected);
        check.requested_mode = GrantMode::Cli;
        assert_eq!(
            signed
                .verify(std::slice::from_ref(&key), &check)
                .unwrap_err(),
            PlatformError::LaunchGrantModeUnavailable
        );

        let (signed, key) = signed_fixture_with(
            vec![GrantMode::Cli, GrantMode::LiteMcp],
            vec![IntegrationScope::CodexLocal],
        );
        check = context(&expected);
        check.requested_scope = IntegrationScope::ClaudeCodeLocal;
        assert_eq!(
            signed
                .verify(std::slice::from_ref(&key), &check)
                .unwrap_err(),
            PlatformError::LaunchGrantScopeUnavailable
        );
    }

    #[test]
    fn strict_bounded_json_rejects_unknown_and_oversized_input() {
        let (signed, _) = signed_fixture();
        let mut value = serde_json::to_value(&signed).unwrap();
        value
            .as_object_mut()
            .unwrap()
            .insert("private-canary".to_string(), serde_json::json!(true));
        assert_eq!(
            SignedLaunchGrantV1::from_json(&serde_json::to_vec(&value).unwrap()).unwrap_err(),
            PlatformError::InvalidLaunchGrantJson
        );
        assert_eq!(
            SignedLaunchGrantV1::from_json(&vec![b' '; MAX_LAUNCH_GRANT_BYTES + 1]).unwrap_err(),
            PlatformError::LaunchGrantTooLarge
        );
    }

    #[test]
    fn grant_lists_must_be_sorted_unique_and_lite_capable() {
        let (mut signed, _) = signed_fixture();
        signed.grant.allowed_modes = vec![GrantMode::LiteMcp, GrantMode::Cli];
        assert_eq!(
            signed.validate_structure(),
            Err(PlatformError::InvalidLaunchGrant)
        );
        signed.grant.allowed_modes = vec![GrantMode::Cli];
        assert_eq!(
            signed.validate_structure(),
            Err(PlatformError::InvalidLaunchGrant)
        );
    }

    fn encode_hex(input: &[u8]) -> String {
        let mut output = String::with_capacity(input.len() * 2);
        for byte in input {
            use std::fmt::Write;
            write!(&mut output, "{byte:02x}").unwrap();
        }
        output
    }

    fn temporary_test_signing_key() -> SigningKey {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("test clock must follow the Unix epoch")
            .as_nanos();
        let sequence = NEXT_KEY_ID.fetch_add(1, Ordering::Relaxed);
        let mut hasher = Sha256::new();
        hasher.update(b"qiongli-r3a-ephemeral-test-key");
        hasher.update(nonce.to_le_bytes());
        hasher.update(std::process::id().to_le_bytes());
        hasher.update(sequence.to_le_bytes());
        SigningKey::from_bytes(&hasher.finalize().into())
    }
}
