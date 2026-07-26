use std::collections::BTreeMap;
use std::fmt::{self, Debug, Formatter};
use std::fs;
use std::path::{Path, PathBuf};

use qiongli_config::ConfigRoot;

use crate::ProjectError;
use crate::capture_delivery::{
    CaptureDeliveryAcknowledgementV1, CaptureDeliveryEnvelopeV1, CaptureDeliveryReason,
    CaptureDeliveryRecordV1, CaptureDeliveryState, DeliveryAcknowledgementId, DeliveryEnvelopeId,
    MAX_DELIVERY_ACKNOWLEDGEMENT_BYTES, MAX_DELIVERY_ENVELOPE_BYTES, MAX_DELIVERY_RECORD_BYTES,
};
use crate::model::valid_lower_hex;
use crate::storage::{
    acquire_lock, atomic_write, ensure_private_directory_beneath, prepare_private_state_directory,
    project_metadata_if_exists, read_bounded_project_file, remove_private_state_file, sha256_bytes,
    validate_private_directory,
};

const CAPTURE_DELIVERY_DIRECTORY: &str = "capture-delivery";
const CAPTURE_DELIVERY_STORAGE_VERSION: &str = "v1";
const CAPTURE_DELIVERY_ENVELOPES_DIRECTORY: &str = "envelopes";
const CAPTURE_DELIVERY_RECORDS_DIRECTORY: &str = "records";
const CAPTURE_DELIVERY_ACKNOWLEDGEMENTS_DIRECTORY: &str = "acknowledgements";
const CAPTURE_DELIVERY_LOCK_FILE: &str = ".ledger.lock";
const MAX_CAPTURE_DELIVERY_ENTRIES: usize = 1_024;

#[derive(Clone)]
pub(crate) struct CaptureDeliveryStore {
    config_root: ConfigRoot,
}

#[derive(Clone, Eq, PartialEq)]
pub(crate) struct StoredCaptureDelivery {
    pub(crate) envelope: CaptureDeliveryEnvelopeV1,
    pub(crate) envelope_sha256: String,
    pub(crate) record: CaptureDeliveryRecordV1,
    pub(crate) record_sha256: String,
    pub(crate) acknowledgement: Option<CaptureDeliveryAcknowledgementV1>,
    pub(crate) acknowledgement_sha256: Option<String>,
}

impl Debug for StoredCaptureDelivery {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("StoredCaptureDelivery")
            .field("envelope", &self.envelope)
            .field("envelope_sha256", &self.envelope_sha256)
            .field("record", &self.record)
            .field("record_sha256", &self.record_sha256)
            .field("acknowledgement", &self.acknowledgement)
            .field("acknowledgement_sha256", &self.acknowledgement_sha256)
            .finish()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CaptureDeliveryLedgerSnapshot {
    pub(crate) entries: Vec<StoredCaptureDelivery>,
}

#[derive(Clone)]
struct StoredAcknowledgement {
    document: CaptureDeliveryAcknowledgementV1,
    sha256: String,
}

struct LedgerPaths {
    state_root: PathBuf,
    root: PathBuf,
    envelopes: PathBuf,
    records: PathBuf,
    acknowledgements: PathBuf,
}

impl CaptureDeliveryStore {
    pub(crate) const fn new(config_root: ConfigRoot) -> Self {
        Self { config_root }
    }

    pub(crate) fn enqueue(
        &self,
        envelope: &CaptureDeliveryEnvelopeV1,
    ) -> Result<StoredCaptureDelivery, ProjectError> {
        envelope.validate()?;
        let queued = CaptureDeliveryRecordV1::queued(envelope, envelope.created_at_unix)?;
        let paths = self.prepare()?;
        let _lock = acquire_lock(&paths.root.join(CAPTURE_DELIVERY_LOCK_FILE))?;
        cleanup_owned_staging_files(&paths)?;

        let envelope_bytes = envelope.to_canonical_json()?;
        match read_envelope(&paths, &envelope.envelope_id)? {
            Some((existing, existing_bytes, _))
                if existing == *envelope && existing_bytes == envelope_bytes => {}
            Some(_) => return Err(ProjectError::DeliveryIdentityConflict),
            None => {
                atomic_write(
                    &paths.envelopes,
                    &envelope_file_name(&envelope.envelope_id),
                    &envelope_bytes,
                    true,
                )?;
            }
        }

        if read_record(&paths, &envelope.envelope_id)?.is_none() {
            let queued_bytes = queued.to_canonical_json()?;
            atomic_write(
                &paths.records,
                &record_file_name(&envelope.envelope_id),
                &queued_bytes,
                true,
            )?;
        }

        let snapshot = rebuild_locked(&paths)?;
        snapshot
            .entries
            .into_iter()
            .find(|entry| entry.envelope.envelope_id == envelope.envelope_id)
            .ok_or(ProjectError::RecoveryRequired)
    }

    pub(crate) fn read(
        &self,
        envelope_id: &DeliveryEnvelopeId,
    ) -> Result<Option<StoredCaptureDelivery>, ProjectError> {
        envelope_id.validate()?;
        let snapshot = self.rebuild()?;
        Ok(snapshot
            .entries
            .into_iter()
            .find(|entry| &entry.envelope.envelope_id == envelope_id))
    }

    pub(crate) fn rebuild(&self) -> Result<CaptureDeliveryLedgerSnapshot, ProjectError> {
        let paths = self.prepare()?;
        let _lock = acquire_lock(&paths.root.join(CAPTURE_DELIVERY_LOCK_FILE))?;
        rebuild_locked(&paths)
    }

    pub(crate) fn replace_record(
        &self,
        envelope_id: &DeliveryEnvelopeId,
        expected_generation: u64,
        expected_record_sha256: &str,
        next_record: &CaptureDeliveryRecordV1,
    ) -> Result<StoredCaptureDelivery, ProjectError> {
        envelope_id.validate()?;
        next_record.validate()?;
        if !valid_lower_hex(expected_record_sha256, 64) {
            return Err(ProjectError::InvalidDeliveryDocument);
        }
        let paths = self.prepare()?;
        let _lock = acquire_lock(&paths.root.join(CAPTURE_DELIVERY_LOCK_FILE))?;
        let snapshot = rebuild_locked(&paths)?;
        let current = snapshot
            .entries
            .iter()
            .find(|entry| &entry.envelope.envelope_id == envelope_id)
            .ok_or(ProjectError::CaptureNotFound)?;
        commit_next_record_locked(
            &paths,
            current,
            expected_generation,
            expected_record_sha256,
            next_record,
        )?;
        let snapshot = rebuild_locked(&paths)?;
        snapshot
            .entries
            .into_iter()
            .find(|entry| &entry.envelope.envelope_id == envelope_id)
            .ok_or(ProjectError::RecoveryRequired)
    }

    pub(crate) fn acknowledge(
        &self,
        acknowledgement: &CaptureDeliveryAcknowledgementV1,
        expected_generation: u64,
        expected_record_sha256: &str,
    ) -> Result<StoredCaptureDelivery, ProjectError> {
        acknowledgement.validate()?;
        if !valid_lower_hex(expected_record_sha256, 64) {
            return Err(ProjectError::InvalidDeliveryDocument);
        }
        let paths = self.prepare()?;
        let _lock = acquire_lock(&paths.root.join(CAPTURE_DELIVERY_LOCK_FILE))?;
        let snapshot = rebuild_locked(&paths)?;
        let current = snapshot
            .entries
            .iter()
            .find(|entry| entry.envelope.envelope_id == acknowledgement.envelope_id)
            .ok_or(ProjectError::CaptureNotFound)?;
        acknowledgement.validate_for_envelope(&current.envelope)?;

        if let Some(existing) = current.acknowledgement.as_ref() {
            let previous = previous_record(&current.record)?;
            let previous_sha256 = sha256_bytes(&previous.to_canonical_json()?);
            if existing == acknowledgement
                && previous.generation == expected_generation
                && previous_sha256 == expected_record_sha256
            {
                return Ok(current.clone());
            }
            if existing == acknowledgement {
                return Err(ProjectError::RevisionConflict);
            }
            return Err(ProjectError::DeliveryAcknowledgementConflict);
        }
        if current.record.state != CaptureDeliveryState::Delivered {
            return Err(ProjectError::InvalidDeliveryTransition);
        }
        if current.record.generation != expected_generation
            || current.record_sha256 != expected_record_sha256
        {
            return Err(ProjectError::RevisionConflict);
        }

        let acknowledgement_bytes = acknowledgement.to_canonical_json()?;
        match read_acknowledgement(&paths, &acknowledgement.acknowledgement_id)? {
            Some((existing, existing_bytes, _))
                if existing == *acknowledgement && existing_bytes == acknowledgement_bytes => {}
            Some(_) => return Err(ProjectError::DeliveryAcknowledgementConflict),
            None => atomic_write(
                &paths.acknowledgements,
                &acknowledgement_file_name(&acknowledgement.acknowledgement_id),
                &acknowledgement_bytes,
                true,
            )?,
        }

        let next = current.record.transition(
            CaptureDeliveryState::Acknowledged,
            acknowledgement.acknowledged_at_unix,
            CaptureDeliveryReason::DeliveryAcknowledged,
            Some(acknowledgement.acknowledgement_id.clone()),
        )?;
        commit_next_record_locked(
            &paths,
            current,
            current.record.generation,
            &current.record_sha256,
            &next,
        )?;
        let snapshot = rebuild_locked(&paths)?;
        snapshot
            .entries
            .into_iter()
            .find(|entry| entry.envelope.envelope_id == acknowledgement.envelope_id)
            .ok_or(ProjectError::RecoveryRequired)
    }

    fn prepare(&self) -> Result<LedgerPaths, ProjectError> {
        let root = prepare_private_state_directory(
            &self.config_root,
            &[CAPTURE_DELIVERY_DIRECTORY, CAPTURE_DELIVERY_STORAGE_VERSION],
        )?;
        let state_root = self.config_root.state_root().to_path_buf();
        let envelopes = root.join(CAPTURE_DELIVERY_ENVELOPES_DIRECTORY);
        let records = root.join(CAPTURE_DELIVERY_RECORDS_DIRECTORY);
        let acknowledgements = root.join(CAPTURE_DELIVERY_ACKNOWLEDGEMENTS_DIRECTORY);
        for directory in [&envelopes, &records, &acknowledgements] {
            ensure_private_directory_beneath(&state_root, directory)?;
        }
        Ok(LedgerPaths {
            state_root,
            root,
            envelopes,
            records,
            acknowledgements,
        })
    }
}

fn rebuild_locked(paths: &LedgerPaths) -> Result<CaptureDeliveryLedgerSnapshot, ProjectError> {
    cleanup_owned_staging_files(paths)?;
    let envelope_ids = list_envelope_ids(paths, &paths.envelopes)?;
    let record_ids = list_envelope_ids(paths, &paths.records)?;
    if record_ids
        .iter()
        .any(|record_id| envelope_ids.binary_search(record_id).is_err())
    {
        return Err(ProjectError::RecoveryRequired);
    }

    let mut acknowledgements = BTreeMap::new();
    for acknowledgement_id in list_acknowledgement_ids(paths)? {
        let (document, _, sha256) = read_acknowledgement(paths, &acknowledgement_id)?
            .ok_or(ProjectError::RecoveryRequired)?;
        let envelope_id = document.envelope_id.clone();
        if acknowledgements
            .insert(envelope_id, StoredAcknowledgement { document, sha256 })
            .is_some()
        {
            return Err(ProjectError::DeliveryAcknowledgementConflict);
        }
    }

    let mut entries = Vec::with_capacity(envelope_ids.len());
    for envelope_id in envelope_ids {
        let (envelope, envelope_bytes, envelope_sha256) =
            read_envelope(paths, &envelope_id)?.ok_or(ProjectError::RecoveryRequired)?;
        let (mut record, _, mut record_sha256) = match read_record(paths, &envelope_id)? {
            Some(record) => record,
            None => {
                let queued = CaptureDeliveryRecordV1::queued(&envelope, envelope.created_at_unix)?;
                let queued_bytes = queued.to_canonical_json()?;
                atomic_write(
                    &paths.records,
                    &record_file_name(&envelope_id),
                    &queued_bytes,
                    true,
                )?;
                (queued, queued_bytes.clone(), sha256_bytes(&queued_bytes))
            }
        };
        validate_record_binding(&envelope, &envelope_sha256, &record)?;

        let acknowledgement = acknowledgements.remove(&envelope_id);
        if let Some(stored) = acknowledgement.as_ref() {
            stored.document.validate_for_envelope(&envelope)?;
            match record.state {
                CaptureDeliveryState::Delivered => {
                    let recovered = record.transition(
                        CaptureDeliveryState::Acknowledged,
                        stored.document.acknowledged_at_unix,
                        CaptureDeliveryReason::DeliveryAcknowledged,
                        Some(stored.document.acknowledgement_id.clone()),
                    )?;
                    let recovered_bytes = recovered.to_canonical_json()?;
                    atomic_write(
                        &paths.records,
                        &record_file_name(&envelope_id),
                        &recovered_bytes,
                        true,
                    )?;
                    record = recovered;
                    record_sha256 = sha256_bytes(&recovered_bytes);
                }
                CaptureDeliveryState::Acknowledged => {
                    let transition = record
                        .transitions
                        .last()
                        .ok_or(ProjectError::RecoveryRequired)?;
                    if transition.acknowledgement_id.as_ref()
                        != Some(&stored.document.acknowledgement_id)
                        || transition.transitioned_at_unix != stored.document.acknowledged_at_unix
                    {
                        return Err(ProjectError::DeliveryAcknowledgementConflict);
                    }
                }
                _ => return Err(ProjectError::DeliveryAcknowledgementConflict),
            }
        } else if record.state == CaptureDeliveryState::Acknowledged {
            return Err(ProjectError::RecoveryRequired);
        }
        validate_record_binding(&envelope, &envelope_sha256, &record)?;

        entries.push(StoredCaptureDelivery {
            envelope,
            envelope_sha256: sha256_bytes(&envelope_bytes),
            record,
            record_sha256,
            acknowledgement: acknowledgement
                .as_ref()
                .map(|stored| stored.document.clone()),
            acknowledgement_sha256: acknowledgement.map(|stored| stored.sha256),
        });
    }
    if !acknowledgements.is_empty() {
        return Err(ProjectError::DeliveryAcknowledgementConflict);
    }
    Ok(CaptureDeliveryLedgerSnapshot { entries })
}

fn commit_next_record_locked(
    paths: &LedgerPaths,
    current: &StoredCaptureDelivery,
    expected_generation: u64,
    expected_record_sha256: &str,
    next_record: &CaptureDeliveryRecordV1,
) -> Result<(), ProjectError> {
    next_record.validate()?;
    if current.envelope.envelope_id != next_record.envelope_id {
        return Err(ProjectError::DeliveryIdentityConflict);
    }
    let previous = previous_record(next_record)?;
    let previous_bytes = previous.to_canonical_json()?;
    let previous_sha256 = sha256_bytes(&previous_bytes);
    if previous.generation != expected_generation
        || previous_sha256 != expected_record_sha256
        || previous != current.record
        || current.record_sha256 != expected_record_sha256
    {
        if current.record == *next_record
            && previous.generation == expected_generation
            && previous_sha256 == expected_record_sha256
        {
            return Ok(());
        }
        return Err(ProjectError::RevisionConflict);
    }
    validate_record_binding(&current.envelope, &current.envelope_sha256, next_record)?;
    let next_bytes = next_record.to_canonical_json()?;
    atomic_write(
        &paths.records,
        &record_file_name(&current.envelope.envelope_id),
        &next_bytes,
        true,
    )?;
    let (committed, committed_bytes, committed_sha256) =
        read_record(paths, &current.envelope.envelope_id)?.ok_or(ProjectError::RecoveryRequired)?;
    if committed != *next_record
        || committed_bytes != next_bytes
        || committed_sha256 != sha256_bytes(&next_bytes)
    {
        return Err(ProjectError::RecoveryRequired);
    }
    Ok(())
}

fn previous_record(
    next_record: &CaptureDeliveryRecordV1,
) -> Result<CaptureDeliveryRecordV1, ProjectError> {
    next_record.previous()
}

fn validate_record_binding(
    envelope: &CaptureDeliveryEnvelopeV1,
    envelope_sha256: &str,
    record: &CaptureDeliveryRecordV1,
) -> Result<(), ProjectError> {
    envelope.validate()?;
    record.validate()?;
    if record.envelope_id != envelope.envelope_id
        || record.envelope_sha256 != envelope_sha256
        || record.created_at_unix != envelope.created_at_unix
        || record
            .transitions
            .first()
            .is_none_or(|transition| transition.transitioned_at_unix != envelope.created_at_unix)
    {
        return Err(ProjectError::DeliveryIdentityConflict);
    }
    Ok(())
}

fn read_envelope(
    paths: &LedgerPaths,
    envelope_id: &DeliveryEnvelopeId,
) -> Result<Option<(CaptureDeliveryEnvelopeV1, Vec<u8>, String)>, ProjectError> {
    let path = paths.envelopes.join(envelope_file_name(envelope_id));
    let Some(bytes) = read_private_document(paths, &path, MAX_DELIVERY_ENVELOPE_BYTES)? else {
        return Ok(None);
    };
    let document = CaptureDeliveryEnvelopeV1::from_json_slice(&bytes)?;
    if document.envelope_id != *envelope_id {
        return Err(ProjectError::DeliveryIdentityConflict);
    }
    let sha256 = sha256_bytes(&bytes);
    Ok(Some((document, bytes, sha256)))
}

fn read_record(
    paths: &LedgerPaths,
    envelope_id: &DeliveryEnvelopeId,
) -> Result<Option<(CaptureDeliveryRecordV1, Vec<u8>, String)>, ProjectError> {
    let path = paths.records.join(record_file_name(envelope_id));
    let Some(bytes) = read_private_document(paths, &path, MAX_DELIVERY_RECORD_BYTES)? else {
        return Ok(None);
    };
    let document = CaptureDeliveryRecordV1::from_json_slice(&bytes)?;
    if document.envelope_id != *envelope_id {
        return Err(ProjectError::DeliveryIdentityConflict);
    }
    let sha256 = sha256_bytes(&bytes);
    Ok(Some((document, bytes, sha256)))
}

fn read_acknowledgement(
    paths: &LedgerPaths,
    acknowledgement_id: &DeliveryAcknowledgementId,
) -> Result<Option<(CaptureDeliveryAcknowledgementV1, Vec<u8>, String)>, ProjectError> {
    let path = paths
        .acknowledgements
        .join(acknowledgement_file_name(acknowledgement_id));
    let Some(bytes) = read_private_document(paths, &path, MAX_DELIVERY_ACKNOWLEDGEMENT_BYTES)?
    else {
        return Ok(None);
    };
    let document = CaptureDeliveryAcknowledgementV1::from_json_slice(&bytes)?;
    if document.acknowledgement_id != *acknowledgement_id {
        return Err(ProjectError::DeliveryIdentityConflict);
    }
    let sha256 = sha256_bytes(&bytes);
    Ok(Some((document, bytes, sha256)))
}

fn read_private_document(
    paths: &LedgerPaths,
    path: &Path,
    maximum_bytes: usize,
) -> Result<Option<Vec<u8>>, ProjectError> {
    let Some(metadata) = project_metadata_if_exists(&paths.state_root, path)? else {
        return Ok(None);
    };
    read_bounded_project_file(&paths.state_root, path, &metadata, maximum_bytes, true).map(Some)
}

fn list_envelope_ids(
    paths: &LedgerPaths,
    directory: &Path,
) -> Result<Vec<DeliveryEnvelopeId>, ProjectError> {
    let metadata = project_metadata_if_exists(&paths.state_root, directory)?
        .ok_or(ProjectError::RecoveryRequired)?;
    validate_private_directory(directory, &metadata)?;
    let mut ids = Vec::new();
    for entry in fs::read_dir(directory).map_err(map_io)? {
        let file_name = entry
            .map_err(map_io)?
            .file_name()
            .into_string()
            .map_err(|_| ProjectError::InvalidDeliveryDocument)?;
        let value = file_name
            .strip_suffix(".json")
            .ok_or(ProjectError::InvalidDeliveryDocument)?;
        ids.push(DeliveryEnvelopeId::parse(value.to_owned())?);
        if ids.len() > MAX_CAPTURE_DELIVERY_ENTRIES {
            return Err(ProjectError::DocumentTooLarge);
        }
    }
    ids.sort();
    Ok(ids)
}

fn list_acknowledgement_ids(
    paths: &LedgerPaths,
) -> Result<Vec<DeliveryAcknowledgementId>, ProjectError> {
    let metadata = project_metadata_if_exists(&paths.state_root, &paths.acknowledgements)?
        .ok_or(ProjectError::RecoveryRequired)?;
    validate_private_directory(&paths.acknowledgements, &metadata)?;
    let mut ids = Vec::new();
    for entry in fs::read_dir(&paths.acknowledgements).map_err(map_io)? {
        let file_name = entry
            .map_err(map_io)?
            .file_name()
            .into_string()
            .map_err(|_| ProjectError::InvalidDeliveryDocument)?;
        let value = file_name
            .strip_suffix(".json")
            .ok_or(ProjectError::InvalidDeliveryDocument)?;
        ids.push(DeliveryAcknowledgementId::parse(value.to_owned())?);
        if ids.len() > MAX_CAPTURE_DELIVERY_ENTRIES {
            return Err(ProjectError::DocumentTooLarge);
        }
    }
    ids.sort();
    Ok(ids)
}

fn cleanup_owned_staging_files(paths: &LedgerPaths) -> Result<(), ProjectError> {
    for (directory, kind, maximum_bytes) in [
        (
            &paths.envelopes,
            StoredDocumentKind::Envelope,
            MAX_DELIVERY_ENVELOPE_BYTES,
        ),
        (
            &paths.records,
            StoredDocumentKind::Record,
            MAX_DELIVERY_RECORD_BYTES,
        ),
        (
            &paths.acknowledgements,
            StoredDocumentKind::Acknowledgement,
            MAX_DELIVERY_ACKNOWLEDGEMENT_BYTES,
        ),
    ] {
        let metadata = project_metadata_if_exists(&paths.state_root, directory)?
            .ok_or(ProjectError::RecoveryRequired)?;
        validate_private_directory(directory, &metadata)?;
        let mut owned_stages = Vec::new();
        for entry in fs::read_dir(directory).map_err(map_io)? {
            let entry = entry.map_err(map_io)?;
            let file_name = entry
                .file_name()
                .into_string()
                .map_err(|_| ProjectError::InvalidDeliveryDocument)?;
            if is_owned_staging_file(&file_name, kind) {
                owned_stages.push(entry.path());
            }
        }
        for stage in owned_stages {
            remove_private_state_file(&paths.state_root, &stage, maximum_bytes)?;
        }
    }
    Ok(())
}

#[derive(Clone, Copy)]
enum StoredDocumentKind {
    Envelope,
    Record,
    Acknowledgement,
}

fn is_owned_staging_file(file_name: &str, kind: StoredDocumentKind) -> bool {
    let Some(value) = file_name.strip_prefix('.') else {
        return false;
    };
    let Some((target, token)) = value.rsplit_once(".qiongli-stage-") else {
        return false;
    };
    if !valid_lower_hex(token, 24) {
        return false;
    }
    let Some(id) = target.strip_suffix(".json") else {
        return false;
    };
    match kind {
        StoredDocumentKind::Envelope | StoredDocumentKind::Record => {
            DeliveryEnvelopeId::parse(id.to_owned()).is_ok()
        }
        StoredDocumentKind::Acknowledgement => {
            DeliveryAcknowledgementId::parse(id.to_owned()).is_ok()
        }
    }
}

fn envelope_file_name(envelope_id: &DeliveryEnvelopeId) -> String {
    format!("{}.json", envelope_id.as_str())
}

fn record_file_name(envelope_id: &DeliveryEnvelopeId) -> String {
    format!("{}.json", envelope_id.as_str())
}

fn acknowledgement_file_name(acknowledgement_id: &DeliveryAcknowledgementId) -> String {
    format!("{}.json", acknowledgement_id.as_str())
}

fn map_io(error: std::io::Error) -> ProjectError {
    ProjectError::PersistenceFailed(error.kind())
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};

    use qiongli_config::resolve_config_root;

    use crate::{
        CaptureArea, CaptureDelivery, CaptureDeliveryDestinationV1, CapturePolicy, CaptureSource,
        ContradictionV1, DecisionCandidateV1, DecisionRelation, EvidenceLocatorKind,
        EvidenceReferenceV1, ProjectBindingV1, ProjectId, ProjectStage, ResearchCaptureDraftV1,
        SemanticChangeV1,
    };

    use super::*;

    static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

    struct Fixture {
        root: PathBuf,
        config_root: ConfigRoot,
        store: CaptureDeliveryStore,
    }

    impl Fixture {
        fn new() -> Self {
            let root = std::env::temp_dir().join(format!(
                "qiongli-capture-delivery-ledger-{}-{}",
                std::process::id(),
                NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed),
            ));
            let _ = fs::remove_dir_all(&root);
            fs::create_dir(&root).unwrap();
            let root = fs::canonicalize(root).unwrap();
            let home = root.join("home");
            fs::create_dir(&home).unwrap();
            let config_root = resolve_config_root(None, &home).unwrap();
            let store = CaptureDeliveryStore::new(config_root.clone());
            Self {
                root,
                config_root,
                store,
            }
        }

        fn ledger_root(&self) -> PathBuf {
            self.config_root
                .state_root()
                .join(CAPTURE_DELIVERY_DIRECTORY)
                .join(CAPTURE_DELIVERY_STORAGE_VERSION)
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    fn project_id(value: &str) -> ProjectId {
        ProjectId::parse(value).unwrap()
    }

    fn capture(captured_at_unix: u64, suffix: &str) -> crate::ResearchCaptureV1 {
        ResearchCaptureDraftV1 {
            binding: ProjectBindingV1::new(
                project_id("prj_0123456789abcdef0123456789abcdef"),
                4,
                ProjectStage::Literature,
                format!("Route the verified capture {suffix}"),
                CapturePolicy::ReviewRequired,
            )
            .unwrap(),
            source: CaptureSource::Codex,
            delivery: CaptureDelivery::Connected,
            captured_at_unix,
            summary: format!("The methods evidence requires a validity branch {suffix}."),
            changes: vec![SemanticChangeV1 {
                area: CaptureArea::Method,
                summary: "Separate validity from reliability evidence.".to_string(),
            }],
            decisions: vec![DecisionCandidateV1 {
                relation: DecisionRelation::Candidate,
                statement: "Track validity as an independent method concern.".to_string(),
                rationale: "The source set treats it independently.".to_string(),
                target: None,
            }],
            evidence: vec![EvidenceReferenceV1 {
                locator_kind: EvidenceLocatorKind::Doi,
                locator: format!("10.1000/delivery-{suffix}"),
                relevance: "Supports the proposed distinction.".to_string(),
                limitation: None,
            }],
            contradictions: Vec::<ContradictionV1>::new(),
            next_actions: vec!["Review the candidate before consolidation.".to_string()],
        }
        .into_capture()
        .unwrap()
    }

    fn envelope(created_at_unix: u64, suffix: &str) -> CaptureDeliveryEnvelopeV1 {
        CaptureDeliveryEnvelopeV1::new(
            capture(created_at_unix - 10, suffix),
            Some(
                CaptureDeliveryDestinationV1::new(
                    project_id("prj_0123456789abcdef0123456789abcdef"),
                    4,
                )
                .unwrap(),
            ),
            created_at_unix,
        )
        .unwrap()
    }

    fn advance(
        fixture: &Fixture,
        current: StoredCaptureDelivery,
        state: CaptureDeliveryState,
        timestamp: u64,
        reason: CaptureDeliveryReason,
    ) -> StoredCaptureDelivery {
        let next = current
            .record
            .transition(state, timestamp, reason, None)
            .unwrap();
        fixture
            .store
            .replace_record(
                &current.envelope.envelope_id,
                current.record.generation,
                &current.record_sha256,
                &next,
            )
            .unwrap()
    }

    #[test]
    fn enqueue_cas_replay_and_reopen_rebuild_one_delivery() {
        let fixture = Fixture::new();
        let envelope = envelope(1_800_000_010, "one");
        let queued = fixture.store.enqueue(&envelope).unwrap();
        assert_eq!(queued.record.state, CaptureDeliveryState::Queued);
        assert_eq!(queued.record.created_at_unix, envelope.created_at_unix);
        assert_eq!(fixture.store.enqueue(&envelope).unwrap(), queued);

        let delivering_next = queued
            .record
            .transition(
                CaptureDeliveryState::Delivering,
                1_800_000_011,
                CaptureDeliveryReason::DeliveryAttemptStarted,
                None,
            )
            .unwrap();
        let delivering = fixture
            .store
            .replace_record(
                &envelope.envelope_id,
                queued.record.generation,
                &queued.record_sha256,
                &delivering_next,
            )
            .unwrap();
        assert_eq!(delivering.record.state, CaptureDeliveryState::Delivering);
        assert_eq!(
            fixture
                .store
                .replace_record(
                    &envelope.envelope_id,
                    queued.record.generation,
                    &queued.record_sha256,
                    &delivering_next,
                )
                .unwrap(),
            delivering
        );

        let stale = queued
            .record
            .transition(
                CaptureDeliveryState::Cancelled,
                1_800_000_012,
                CaptureDeliveryReason::DeliveryCancelled,
                None,
            )
            .unwrap();
        assert_eq!(
            fixture.store.replace_record(
                &envelope.envelope_id,
                queued.record.generation,
                &queued.record_sha256,
                &stale,
            ),
            Err(ProjectError::RevisionConflict)
        );

        let reopened = CaptureDeliveryStore::new(fixture.config_root.clone())
            .rebuild()
            .unwrap();
        assert_eq!(reopened.entries, vec![delivering]);
        assert!(!format!("{reopened:?}").contains(&envelope.capture.summary));

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            for directory in [
                fixture.ledger_root(),
                fixture
                    .ledger_root()
                    .join(CAPTURE_DELIVERY_ENVELOPES_DIRECTORY),
                fixture
                    .ledger_root()
                    .join(CAPTURE_DELIVERY_RECORDS_DIRECTORY),
                fixture
                    .ledger_root()
                    .join(CAPTURE_DELIVERY_ACKNOWLEDGEMENTS_DIRECTORY),
            ] {
                assert_eq!(
                    fs::metadata(directory).unwrap().permissions().mode() & 0o077,
                    0
                );
            }
        }
    }

    #[test]
    fn acknowledgement_is_exactly_once_and_recovers_after_its_write_boundary() {
        let fixture = Fixture::new();
        let first_envelope = envelope(1_800_000_010, "ack-one");
        let first = fixture.store.enqueue(&first_envelope).unwrap();
        let first = advance(
            &fixture,
            first,
            CaptureDeliveryState::Delivering,
            1_800_000_011,
            CaptureDeliveryReason::DeliveryAttemptStarted,
        );
        let first = advance(
            &fixture,
            first,
            CaptureDeliveryState::Delivered,
            1_800_000_012,
            CaptureDeliveryReason::DeliveryAccepted,
        );
        let acknowledgement = CaptureDeliveryAcknowledgementV1::new(
            &first_envelope,
            first_envelope.capture_id.clone(),
            5,
            1_800_000_013,
        )
        .unwrap();
        let acknowledged = fixture
            .store
            .acknowledge(
                &acknowledgement,
                first.record.generation,
                &first.record_sha256,
            )
            .unwrap();
        assert_eq!(
            acknowledged.record.state,
            CaptureDeliveryState::Acknowledged
        );
        assert_eq!(
            fixture
                .store
                .acknowledge(
                    &acknowledgement,
                    first.record.generation,
                    &first.record_sha256,
                )
                .unwrap(),
            acknowledged
        );
        let conflicting = CaptureDeliveryAcknowledgementV1::new(
            &first_envelope,
            first_envelope.capture_id.clone(),
            6,
            1_800_000_014,
        )
        .unwrap();
        assert_eq!(
            fixture
                .store
                .acknowledge(&conflicting, first.record.generation, &first.record_sha256,),
            Err(ProjectError::DeliveryAcknowledgementConflict)
        );

        let second_envelope = envelope(1_800_000_020, "ack-two");
        let second = fixture.store.enqueue(&second_envelope).unwrap();
        let second = advance(
            &fixture,
            second,
            CaptureDeliveryState::Delivering,
            1_800_000_021,
            CaptureDeliveryReason::DeliveryAttemptStarted,
        );
        let second = advance(
            &fixture,
            second,
            CaptureDeliveryState::Delivered,
            1_800_000_022,
            CaptureDeliveryReason::DeliveryAccepted,
        );
        let interrupted_acknowledgement = CaptureDeliveryAcknowledgementV1::new(
            &second_envelope,
            second_envelope.capture_id.clone(),
            5,
            1_800_000_023,
        )
        .unwrap();
        let paths = fixture.store.prepare().unwrap();
        atomic_write(
            &paths.acknowledgements,
            &acknowledgement_file_name(&interrupted_acknowledgement.acknowledgement_id),
            &interrupted_acknowledgement.to_canonical_json().unwrap(),
            true,
        )
        .unwrap();

        let recovered = CaptureDeliveryStore::new(fixture.config_root.clone())
            .read(&second_envelope.envelope_id)
            .unwrap()
            .unwrap();
        assert_eq!(recovered.record.state, CaptureDeliveryState::Acknowledged);
        assert_eq!(recovered.acknowledgement, Some(interrupted_acknowledgement));
        assert_eq!(second.record.state, CaptureDeliveryState::Delivered);
    }

    #[test]
    fn interrupted_enqueue_and_owned_atomic_stage_recover_deterministically() {
        let fixture = Fixture::new();
        let envelope = envelope(1_800_000_010, "interrupted");
        let paths = fixture.store.prepare().unwrap();
        atomic_write(
            &paths.envelopes,
            &envelope_file_name(&envelope.envelope_id),
            &envelope.to_canonical_json().unwrap(),
            true,
        )
        .unwrap();
        let stage = paths.records.join(format!(
            ".{}.json.qiongli-stage-{}",
            envelope.envelope_id.as_str(),
            "a".repeat(24)
        ));
        fs::write(&stage, b"interrupted").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&stage, fs::Permissions::from_mode(0o600)).unwrap();
        }

        let recovered = CaptureDeliveryStore::new(fixture.config_root.clone())
            .read(&envelope.envelope_id)
            .unwrap()
            .unwrap();
        assert_eq!(recovered.record.state, CaptureDeliveryState::Queued);
        assert_eq!(recovered.record.created_at_unix, envelope.created_at_unix);
        assert!(!stage.exists());
    }

    #[cfg(unix)]
    #[test]
    fn ledger_rejects_linked_and_permission_broadened_authoritative_files() {
        use std::os::unix::fs::{PermissionsExt, symlink};

        let linked_fixture = Fixture::new();
        let linked_envelope = envelope(1_800_000_010, "hard-link");
        linked_fixture.store.enqueue(&linked_envelope).unwrap();
        let linked_path = linked_fixture
            .ledger_root()
            .join(CAPTURE_DELIVERY_ENVELOPES_DIRECTORY)
            .join(envelope_file_name(&linked_envelope.envelope_id));
        fs::hard_link(&linked_path, linked_fixture.root.join("second-link")).unwrap();
        assert_eq!(
            linked_fixture.store.rebuild(),
            Err(ProjectError::UnsafeProjectRoot)
        );

        let permission_fixture = Fixture::new();
        let permission_envelope = envelope(1_800_000_010, "permission");
        permission_fixture
            .store
            .enqueue(&permission_envelope)
            .unwrap();
        let permission_path = permission_fixture
            .ledger_root()
            .join(CAPTURE_DELIVERY_RECORDS_DIRECTORY)
            .join(record_file_name(&permission_envelope.envelope_id));
        fs::set_permissions(&permission_path, fs::Permissions::from_mode(0o644)).unwrap();
        assert_eq!(
            permission_fixture.store.rebuild(),
            Err(ProjectError::UnsafeProjectRoot)
        );

        let symlink_fixture = Fixture::new();
        let paths = symlink_fixture.store.prepare().unwrap();
        fs::remove_dir(&paths.envelopes).unwrap();
        symlink(&paths.records, &paths.envelopes).unwrap();
        assert_eq!(
            symlink_fixture.store.rebuild(),
            Err(ProjectError::UnsafeProjectRoot)
        );
    }

    #[test]
    fn ledger_rejects_corruption_orphans_and_over_count_indexes() {
        let corrupt_fixture = Fixture::new();
        let corrupt_envelope = envelope(1_800_000_010, "corrupt");
        corrupt_fixture.store.enqueue(&corrupt_envelope).unwrap();
        let corrupt_record = corrupt_fixture
            .ledger_root()
            .join(CAPTURE_DELIVERY_RECORDS_DIRECTORY)
            .join(record_file_name(&corrupt_envelope.envelope_id));
        fs::write(&corrupt_record, b"{\"schema_version\":1}").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&corrupt_record, fs::Permissions::from_mode(0o600)).unwrap();
        }
        assert_eq!(
            corrupt_fixture.store.rebuild(),
            Err(ProjectError::InvalidDeliveryDocument)
        );

        let oversized_fixture = Fixture::new();
        let oversized_envelope = envelope(1_800_000_010, "oversized");
        oversized_fixture
            .store
            .enqueue(&oversized_envelope)
            .unwrap();
        let oversized_record = oversized_fixture
            .ledger_root()
            .join(CAPTURE_DELIVERY_RECORDS_DIRECTORY)
            .join(record_file_name(&oversized_envelope.envelope_id));
        fs::write(&oversized_record, vec![b' '; MAX_DELIVERY_RECORD_BYTES + 1]).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&oversized_record, fs::Permissions::from_mode(0o600)).unwrap();
        }
        assert_eq!(
            oversized_fixture.store.rebuild(),
            Err(ProjectError::DocumentTooLarge)
        );

        let unknown_fixture = Fixture::new();
        let paths = unknown_fixture.store.prepare().unwrap();
        let unknown = paths.envelopes.join("notes.json");
        fs::write(&unknown, b"{}").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&unknown, fs::Permissions::from_mode(0o600)).unwrap();
        }
        assert_eq!(
            unknown_fixture.store.rebuild(),
            Err(ProjectError::InvalidDeliveryDocument)
        );

        let orphan_fixture = Fixture::new();
        let orphan_envelope = envelope(1_800_000_010, "orphan");
        let paths = orphan_fixture.store.prepare().unwrap();
        let record =
            CaptureDeliveryRecordV1::queued(&orphan_envelope, orphan_envelope.created_at_unix)
                .unwrap();
        atomic_write(
            &paths.records,
            &record_file_name(&orphan_envelope.envelope_id),
            &record.to_canonical_json().unwrap(),
            true,
        )
        .unwrap();
        assert_eq!(
            orphan_fixture.store.rebuild(),
            Err(ProjectError::RecoveryRequired)
        );

        let count_fixture = Fixture::new();
        let paths = count_fixture.store.prepare().unwrap();
        for index in 0..=MAX_CAPTURE_DELIVERY_ENTRIES {
            let envelope_id = DeliveryEnvelopeId::parse(format!("env_{index:064x}")).unwrap();
            fs::write(
                paths.envelopes.join(envelope_file_name(&envelope_id)),
                b"{}",
            )
            .unwrap();
        }
        assert_eq!(
            count_fixture.store.rebuild(),
            Err(ProjectError::DocumentTooLarge)
        );
    }
}
