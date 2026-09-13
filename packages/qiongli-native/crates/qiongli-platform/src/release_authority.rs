use std::fmt::{self, Debug, Display, Formatter};

use ed25519_dalek::VerifyingKey;
use serde::{Deserialize, Serialize};

use crate::{
    Architecture, ArtifactIdentityV1, CapabilityProfile, InstallerKind, NativeReleaseError,
    OperatingSystem, ProductId, ReleaseChannel, TrustedPublicKey, TrustedReleasePublicKey,
};

pub const NATIVE_RELEASE_AUTHORITY_SCHEMA_VERSION: u32 = 1;
pub const MAX_NATIVE_RELEASE_AUTHORITY_BYTES: usize = 64 * 1024;

const MAX_TRUSTED_KEYS_PER_ROLE: usize = 16;
const PUBLIC_KEY_BYTES: usize = 32;
const PUBLIC_KEY_HEX_BYTES: usize = PUBLIC_KEY_BYTES * 2;
const JCS_MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct NativeReleaseAuthorityDocumentV1 {
    schema_version: u32,
    channel: ReleaseChannel,
    minimum_release_generation: u64,
    minimum_launch_grant_generation: u64,
    release_keys: Vec<ReleaseKeyDocumentV1>,
    launch_grant_keys: Vec<LaunchGrantKeyDocumentV1>,
}

#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct ReleaseKeyDocumentV1 {
    key_id: String,
    public_key_hex: String,
    minimum_generation: u64,
    maximum_generation_exclusive: Option<u64>,
}

#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct LaunchGrantKeyDocumentV1 {
    key_id: String,
    public_key_hex: String,
}

#[derive(Clone)]
pub struct NativeReleaseAuthority {
    document: NativeReleaseAuthorityDocumentV1,
    release_keys: Vec<TrustedReleasePublicKey>,
    launch_grant_keys: Vec<TrustedPublicKey>,
}

impl NativeReleaseAuthority {
    pub fn from_json(input: &[u8]) -> Result<Self, NativeReleaseAuthorityError> {
        if input.len() > MAX_NATIVE_RELEASE_AUTHORITY_BYTES {
            return Err(NativeReleaseAuthorityError::InputTooLarge);
        }
        let document = serde_json::from_slice::<NativeReleaseAuthorityDocumentV1>(input)
            .map_err(|_| NativeReleaseAuthorityError::InvalidJson)?;
        let authority = Self::from_document(document)?;
        if authority.to_canonical_json()?.as_slice() != input {
            return Err(NativeReleaseAuthorityError::NonCanonicalJson);
        }
        Ok(authority)
    }

    pub fn to_canonical_json(&self) -> Result<Vec<u8>, NativeReleaseAuthorityError> {
        let bytes = serde_json_canonicalizer::to_vec(&self.document)
            .map_err(|_| NativeReleaseAuthorityError::CanonicalSerializationFailed)?;
        if bytes.len() > MAX_NATIVE_RELEASE_AUTHORITY_BYTES {
            return Err(NativeReleaseAuthorityError::InputTooLarge);
        }
        Ok(bytes)
    }

    #[must_use]
    pub const fn channel(&self) -> ReleaseChannel {
        self.document.channel
    }

    #[must_use]
    pub const fn minimum_release_generation(&self) -> u64 {
        self.document.minimum_release_generation
    }

    #[must_use]
    pub const fn minimum_launch_grant_generation(&self) -> u64 {
        self.document.minimum_launch_grant_generation
    }

    #[must_use]
    pub fn release_keys(&self) -> &[TrustedReleasePublicKey] {
        &self.release_keys
    }

    #[must_use]
    pub fn launch_grant_keys(&self) -> &[TrustedPublicKey] {
        &self.launch_grant_keys
    }

    pub fn validate_product_version(
        &self,
        version: &str,
    ) -> Result<(), NativeReleaseAuthorityError> {
        ArtifactIdentityV1 {
            product: ProductId::Qiongli,
            version: version.to_string(),
            channel: self.document.channel,
            profile: CapabilityProfile::Lite,
            os: OperatingSystem::Linux,
            arch: Architecture::X86_64,
            installer_kind: InstallerKind::PortableArchive,
        }
        .validate_lite()
        .map_err(|_| NativeReleaseAuthorityError::ProductVersionMismatch)
    }

    fn from_document(
        document: NativeReleaseAuthorityDocumentV1,
    ) -> Result<Self, NativeReleaseAuthorityError> {
        if document.schema_version != NATIVE_RELEASE_AUTHORITY_SCHEMA_VERSION {
            return Err(NativeReleaseAuthorityError::UnsupportedSchema);
        }
        if !valid_generation(document.minimum_release_generation)
            || !valid_generation(document.minimum_launch_grant_generation)
            || document.release_keys.is_empty()
            || document.release_keys.len() > MAX_TRUSTED_KEYS_PER_ROLE
            || document.launch_grant_keys.is_empty()
            || document.launch_grant_keys.len() > MAX_TRUSTED_KEYS_PER_ROLE
            || !sorted_unique_by_key_id(&document.release_keys, |key| &key.key_id)
            || !sorted_unique_by_key_id(&document.launch_grant_keys, |key| &key.key_id)
        {
            return Err(NativeReleaseAuthorityError::InvalidAuthority);
        }

        let mut release_keys = Vec::with_capacity(document.release_keys.len());
        for key in &document.release_keys {
            if key
                .maximum_generation_exclusive
                .is_some_and(|maximum| maximum <= document.minimum_release_generation)
            {
                return Err(NativeReleaseAuthorityError::InvalidReleaseKey);
            }
            let public_key = decode_public_key(&key.public_key_hex)
                .ok_or(NativeReleaseAuthorityError::InvalidReleaseKey)?;
            VerifyingKey::from_bytes(&public_key)
                .map_err(|_| NativeReleaseAuthorityError::InvalidReleaseKey)?;
            let trusted = TrustedReleasePublicKey::new(
                key.key_id.clone(),
                public_key,
                key.minimum_generation,
                key.maximum_generation_exclusive,
            )
            .map_err(map_release_key_error)?;
            release_keys.push(trusted);
        }
        if !document.release_keys.iter().any(|key| {
            document.minimum_release_generation >= key.minimum_generation
                && key
                    .maximum_generation_exclusive
                    .is_none_or(|maximum| document.minimum_release_generation < maximum)
        }) {
            return Err(NativeReleaseAuthorityError::InvalidReleaseKey);
        }

        let mut launch_grant_keys = Vec::with_capacity(document.launch_grant_keys.len());
        for key in &document.launch_grant_keys {
            let public_key = decode_public_key(&key.public_key_hex)
                .ok_or(NativeReleaseAuthorityError::InvalidLaunchGrantKey)?;
            VerifyingKey::from_bytes(&public_key)
                .map_err(|_| NativeReleaseAuthorityError::InvalidLaunchGrantKey)?;
            let trusted = TrustedPublicKey::new(key.key_id.clone(), public_key)
                .map_err(|_| NativeReleaseAuthorityError::InvalidLaunchGrantKey)?;
            launch_grant_keys.push(trusted);
        }

        if document.release_keys.iter().any(|release| {
            document.launch_grant_keys.iter().any(|launch| {
                release.key_id == launch.key_id || release.public_key_hex == launch.public_key_hex
            })
        }) {
            return Err(NativeReleaseAuthorityError::KeyRolesOverlap);
        }

        Ok(Self {
            document,
            release_keys,
            launch_grant_keys,
        })
    }
}

impl Debug for NativeReleaseAuthority {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("NativeReleaseAuthority")
            .field("channel", &self.document.channel)
            .field(
                "minimum_release_generation",
                &self.document.minimum_release_generation,
            )
            .field(
                "minimum_launch_grant_generation",
                &self.document.minimum_launch_grant_generation,
            )
            .field(
                "release_key_ids",
                &self
                    .document
                    .release_keys
                    .iter()
                    .map(|key| key.key_id.as_str())
                    .collect::<Vec<_>>(),
            )
            .field(
                "launch_grant_key_ids",
                &self
                    .document
                    .launch_grant_keys
                    .iter()
                    .map(|key| key.key_id.as_str())
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}

fn valid_generation(value: u64) -> bool {
    value > 0 && value <= JCS_MAX_SAFE_INTEGER
}

fn sorted_unique_by_key_id<T>(values: &[T], key_id: impl Fn(&T) -> &str) -> bool {
    values
        .windows(2)
        .all(|pair| key_id(&pair[0]) < key_id(&pair[1]))
}

fn decode_public_key(value: &str) -> Option<[u8; PUBLIC_KEY_BYTES]> {
    if value.len() != PUBLIC_KEY_HEX_BYTES
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return None;
    }
    let mut output = [0_u8; PUBLIC_KEY_BYTES];
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

fn map_release_key_error(_error: NativeReleaseError) -> NativeReleaseAuthorityError {
    NativeReleaseAuthorityError::InvalidReleaseKey
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeReleaseAuthorityError {
    InputTooLarge,
    InvalidJson,
    NonCanonicalJson,
    UnsupportedSchema,
    InvalidAuthority,
    InvalidReleaseKey,
    InvalidLaunchGrantKey,
    KeyRolesOverlap,
    ProductVersionMismatch,
    CanonicalSerializationFailed,
}

impl NativeReleaseAuthorityError {
    #[must_use]
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::InputTooLarge => "native-release-authority-too-large",
            Self::InvalidJson => "native-release-authority-json-invalid",
            Self::NonCanonicalJson => "native-release-authority-json-noncanonical",
            Self::UnsupportedSchema => "native-release-authority-schema-unsupported",
            Self::InvalidAuthority => "native-release-authority-invalid",
            Self::InvalidReleaseKey => "native-release-authority-release-key-invalid",
            Self::InvalidLaunchGrantKey => "native-release-authority-launch-key-invalid",
            Self::KeyRolesOverlap => "native-release-authority-key-roles-overlap",
            Self::ProductVersionMismatch => "native-release-authority-product-version-mismatch",
            Self::CanonicalSerializationFailed => {
                "native-release-authority-canonicalization-failed"
            }
        }
    }
}

impl Display for NativeReleaseAuthorityError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.reason_code())
    }
}

impl std::error::Error for NativeReleaseAuthorityError {}

#[cfg(test)]
mod tests {
    use ed25519_dalek::SigningKey;
    use serde_json::{Value, json};

    use super::*;

    fn encode_hex<const N: usize>(bytes: &[u8; N]) -> String {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut output = String::with_capacity(N * 2);
        for byte in bytes {
            output.push(char::from(HEX[usize::from(byte >> 4)]));
            output.push(char::from(HEX[usize::from(byte & 0x0f)]));
        }
        output
    }

    fn authority_value() -> Value {
        let release = SigningKey::from_bytes(&[81_u8; 32]);
        let launch = SigningKey::from_bytes(&[82_u8; 32]);
        json!({
            "schema_version": 1,
            "channel": "alpha",
            "minimum_release_generation": 7,
            "minimum_launch_grant_generation": 5,
            "release_keys": [{
                "key_id": "release-alpha-1",
                "public_key_hex": encode_hex(&release.verifying_key().to_bytes()),
                "minimum_generation": 7,
                "maximum_generation_exclusive": 12
            }],
            "launch_grant_keys": [{
                "key_id": "launch-alpha-1",
                "public_key_hex": encode_hex(&launch.verifying_key().to_bytes())
            }]
        })
    }

    fn canonical(value: &Value) -> Vec<u8> {
        serde_json_canonicalizer::to_vec(value).unwrap()
    }

    #[test]
    fn authority_is_canonical_bounded_and_role_separated() {
        let bytes = canonical(&authority_value());
        let authority = NativeReleaseAuthority::from_json(&bytes).unwrap();
        assert_eq!(authority.to_canonical_json().unwrap(), bytes);
        assert_eq!(authority.channel(), ReleaseChannel::Alpha);
        assert_eq!(authority.minimum_release_generation(), 7);
        assert_eq!(authority.minimum_launch_grant_generation(), 5);
        assert!(authority.validate_product_version("2.0.0-alpha.1").is_ok());
        assert_eq!(
            authority.validate_product_version("2.0.0-beta.1"),
            Err(NativeReleaseAuthorityError::ProductVersionMismatch)
        );
        assert_eq!(authority.release_keys()[0].key_id(), "release-alpha-1");
        assert_eq!(authority.launch_grant_keys()[0].key_id(), "launch-alpha-1");

        let debug = format!("{authority:?}");
        assert!(debug.contains("release-alpha-1"));
        assert!(!debug.contains("public_key"));

        let mut noncanonical = bytes;
        noncanonical.push(b'\n');
        assert_eq!(
            NativeReleaseAuthority::from_json(&noncanonical).unwrap_err(),
            NativeReleaseAuthorityError::NonCanonicalJson
        );
        assert_eq!(
            NativeReleaseAuthority::from_json(&vec![b' '; 65 * 1024]).unwrap_err(),
            NativeReleaseAuthorityError::InputTooLarge
        );
    }

    #[test]
    fn authority_rejects_unknown_unsorted_stale_and_overlapping_keys() {
        let mut unknown = authority_value();
        unknown["unknown"] = Value::Bool(true);
        assert_eq!(
            NativeReleaseAuthority::from_json(&canonical(&unknown)).unwrap_err(),
            NativeReleaseAuthorityError::InvalidJson
        );

        let release_two = SigningKey::from_bytes(&[83_u8; 32]);
        let mut unsorted = authority_value();
        unsorted["release_keys"] = json!([
            {
                "key_id": "release-z",
                "public_key_hex": encode_hex(&release_two.verifying_key().to_bytes()),
                "minimum_generation": 7,
                "maximum_generation_exclusive": null
            },
            unsorted["release_keys"][0].clone()
        ]);
        assert_eq!(
            NativeReleaseAuthority::from_json(&canonical(&unsorted)).unwrap_err(),
            NativeReleaseAuthorityError::InvalidAuthority
        );

        let mut stale = authority_value();
        stale["release_keys"][0]["maximum_generation_exclusive"] = json!(7);
        assert_eq!(
            NativeReleaseAuthority::from_json(&canonical(&stale)).unwrap_err(),
            NativeReleaseAuthorityError::InvalidReleaseKey
        );

        let mut overlap = authority_value();
        overlap["launch_grant_keys"][0]["key_id"] = json!("release-alpha-1");
        assert_eq!(
            NativeReleaseAuthority::from_json(&canonical(&overlap)).unwrap_err(),
            NativeReleaseAuthorityError::KeyRolesOverlap
        );

        let mut overlapping_bytes = authority_value();
        let release_public_key = overlapping_bytes["release_keys"][0]["public_key_hex"].clone();
        overlapping_bytes["launch_grant_keys"][0]["public_key_hex"] = release_public_key;
        assert_eq!(
            NativeReleaseAuthority::from_json(&canonical(&overlapping_bytes)).unwrap_err(),
            NativeReleaseAuthorityError::KeyRolesOverlap
        );

        let mut invalid_release_key = authority_value();
        invalid_release_key["release_keys"][0]["public_key_hex"] = json!("x".repeat(64));
        assert_eq!(
            NativeReleaseAuthority::from_json(&canonical(&invalid_release_key)).unwrap_err(),
            NativeReleaseAuthorityError::InvalidReleaseKey
        );

        let mut invalid_launch_key = authority_value();
        invalid_launch_key["launch_grant_keys"][0]["public_key_hex"] = json!("f".repeat(63));
        assert_eq!(
            NativeReleaseAuthority::from_json(&canonical(&invalid_launch_key)).unwrap_err(),
            NativeReleaseAuthorityError::InvalidLaunchGrantKey
        );

        let mut too_many = authority_value();
        too_many["launch_grant_keys"] = Value::Array(
            (0_u8..17)
                .map(|index| {
                    let key = SigningKey::from_bytes(&[index.saturating_add(1); 32]);
                    json!({
                        "key_id": format!("launch-{index:02}"),
                        "public_key_hex": encode_hex(&key.verifying_key().to_bytes())
                    })
                })
                .collect(),
        );
        assert_eq!(
            NativeReleaseAuthority::from_json(&canonical(&too_many)).unwrap_err(),
            NativeReleaseAuthorityError::InvalidAuthority
        );
    }

    #[test]
    fn authority_errors_are_fixed_reason_codes() {
        for error in [
            NativeReleaseAuthorityError::InvalidAuthority,
            NativeReleaseAuthorityError::InvalidReleaseKey,
            NativeReleaseAuthorityError::InvalidLaunchGrantKey,
            NativeReleaseAuthorityError::KeyRolesOverlap,
        ] {
            assert_eq!(error.to_string(), error.reason_code());
            assert!(!error.to_string().contains('/'));
            assert!(!error.to_string().contains('\\'));
        }
    }
}
