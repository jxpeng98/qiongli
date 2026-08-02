use std::collections::BTreeMap;
use std::fmt::{self, Debug, Formatter};
use std::fs;
use std::path::{Path, PathBuf};

use qiongli_config::ConfigRoot;
use serde::{Deserialize, Serialize};

use crate::ProjectError;
use crate::capture_delivery::{
    CaptureDeliveryEnvelopeV1, CaptureDeliveryReason, CaptureDeliveryRecordV1, CaptureDeliveryState,
};
use crate::capture_delivery_storage::CaptureDeliveryStore;
use crate::capture_resolution::{
    CaptureAssignmentIntentId, CaptureAssignmentIntentV1, CaptureAssignmentOutcome,
    CaptureAssignmentReceiptId, CaptureAssignmentReceiptV1, MAX_ASSIGNMENT_INTENT_BYTES,
    MAX_ASSIGNMENT_RECEIPT_BYTES,
};
use crate::model::valid_lower_hex;
use crate::storage::{
    acquire_lock, atomic_write, ensure_private_directory_beneath, prepare_private_state_directory,
    project_metadata_if_exists, read_bounded_project_file, remove_private_state_file, sha256_bytes,
    validate_private_directory,
};

const CAPTURE_RESOLUTION_DIRECTORY: &str = "capture-resolution";
const CAPTURE_RESOLUTION_STORAGE_VERSION: &str = "v1";
const CAPTURE_ASSIGNMENT_INTENTS_DIRECTORY: &str = "assignment-intents";
const CAPTURE_ASSIGNMENT_RECEIPTS_DIRECTORY: &str = "assignment-receipts";
const CAPTURE_ASSIGNMENT_TRANSACTIONS_DIRECTORY: &str = "transactions";
const CAPTURE_RESOLUTION_LOCK_FILE: &str = ".ledger.lock";
const CAPTURE_ASSIGNMENT_TRANSACTION_SCHEMA_VERSION: u32 = 1;
const CAPTURE_ASSIGNMENT_TRANSACTION_DOCUMENT_KIND: &str = "qiongli-capture-assignment-transaction";
const MAX_CAPTURE_ASSIGNMENTS: usize = 1_024;
const MAX_ASSIGNMENT_TRANSACTION_BYTES: usize = 192 * 1024;

#[derive(Clone)]
pub(crate) struct CaptureResolutionStore {
    config_root: ConfigRoot,
    delivery_store: CaptureDeliveryStore,
}

#[derive(Clone, Eq, PartialEq)]
pub(crate) struct StoredCaptureAssignment {
    pub(crate) intent: CaptureAssignmentIntentV1,
    pub(crate) intent_sha256: String,
    pub(crate) receipt: Option<CaptureAssignmentReceiptV1>,
    pub(crate) receipt_sha256: Option<String>,
}

impl Debug for StoredCaptureAssignment {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("StoredCaptureAssignment")
            .field("intent", &self.intent)
            .field("intent_sha256", &self.intent_sha256)
            .field("receipt", &self.receipt)
            .field("receipt_sha256", &self.receipt_sha256)
            .finish()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CaptureResolutionLedgerSnapshot {
    pub(crate) assignments: Vec<StoredCaptureAssignment>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct CaptureAssignmentTransactionV1 {
    schema_version: u32,
    document_kind: String,
    intent: CaptureAssignmentIntentV1,
    receipt: CaptureAssignmentReceiptV1,
    child_envelope: Option<CaptureDeliveryEnvelopeV1>,
    source_record_after: CaptureDeliveryRecordV1,
}

impl CaptureAssignmentTransactionV1 {
    pub(crate) fn new(
        intent: CaptureAssignmentIntentV1,
        receipt: CaptureAssignmentReceiptV1,
        child_envelope: Option<CaptureDeliveryEnvelopeV1>,
        source_record_after: CaptureDeliveryRecordV1,
    ) -> Result<Self, ProjectError> {
        let transaction = Self {
            schema_version: CAPTURE_ASSIGNMENT_TRANSACTION_SCHEMA_VERSION,
            document_kind: CAPTURE_ASSIGNMENT_TRANSACTION_DOCUMENT_KIND.to_string(),
            intent,
            receipt,
            child_envelope,
            source_record_after,
        };
        transaction.validate()?;
        Ok(transaction)
    }

    fn from_json_slice(bytes: &[u8]) -> Result<Self, ProjectError> {
        if bytes.len() > MAX_ASSIGNMENT_TRANSACTION_BYTES {
            return Err(ProjectError::DocumentTooLarge);
        }
        let value = crate::json::parse_unique_json(bytes)
            .map_err(|_| ProjectError::InvalidResolutionDocument)?;
        let transaction: Self =
            serde_json::from_value(value).map_err(|_| ProjectError::InvalidResolutionDocument)?;
        transaction.validate()?;
        Ok(transaction)
    }

    pub(crate) fn intent(&self) -> &CaptureAssignmentIntentV1 {
        &self.intent
    }

    pub(crate) fn receipt(&self) -> &CaptureAssignmentReceiptV1 {
        &self.receipt
    }

    pub(crate) fn child_envelope(&self) -> Option<&CaptureDeliveryEnvelopeV1> {
        self.child_envelope.as_ref()
    }

    pub(crate) fn source_record_after(&self) -> &CaptureDeliveryRecordV1 {
        &self.source_record_after
    }

    pub(crate) fn to_canonical_json(&self) -> Result<Vec<u8>, ProjectError> {
        self.validate()?;
        let bytes = serde_json_canonicalizer::to_vec(self)
            .map_err(|_| ProjectError::InvalidResolutionDocument)?;
        if bytes.len() > MAX_ASSIGNMENT_TRANSACTION_BYTES {
            return Err(ProjectError::DocumentTooLarge);
        }
        Ok(bytes)
    }

    pub(crate) fn validate(&self) -> Result<(), ProjectError> {
        if self.schema_version != CAPTURE_ASSIGNMENT_TRANSACTION_SCHEMA_VERSION
            || self.document_kind != CAPTURE_ASSIGNMENT_TRANSACTION_DOCUMENT_KIND
        {
            return Err(ProjectError::InvalidResolutionDocument);
        }
        let intent_bytes = self.intent.to_canonical_json()?;
        let receipt_bytes = self.receipt.to_canonical_json()?;
        validate_assignment_pair(&self.intent, &intent_bytes, &self.receipt, &receipt_bytes)?;
        self.source_record_after.validate()?;
        if self.source_record_after.envelope_id != self.intent.intent.source_envelope_id
            || self.source_record_after.envelope_sha256 != self.intent.intent.source_envelope_sha256
            || self.source_record_after.state != CaptureDeliveryState::Cancelled
            || self.source_record_after.generation
                != self.receipt.receipt.source_record_generation_after
            || sha256_bytes(&self.source_record_after.to_canonical_json()?)
                != self.receipt.receipt.source_record_sha256_after
            || self.source_record_after.updated_at_unix != self.receipt.receipt.decided_at_unix
        {
            return Err(ProjectError::InvalidResolutionDocument);
        }
        let transition = self
            .source_record_after
            .transitions
            .last()
            .ok_or(ProjectError::InvalidResolutionDocument)?;
        if transition.from_state != Some(self.intent.intent.source_record_state)
            || transition.to_state != CaptureDeliveryState::Cancelled
            || transition.reason_code != CaptureDeliveryReason::DeliveryCancelled
            || transition.acknowledgement_id.is_some()
        {
            return Err(ProjectError::InvalidResolutionDocument);
        }
        let previous = self.source_record_after.previous()?;
        if previous.state != self.intent.intent.source_record_state
            || previous.generation != self.intent.intent.source_record_generation
            || sha256_bytes(&previous.to_canonical_json()?)
                != self.intent.intent.source_record_sha256
        {
            return Err(ProjectError::InvalidResolutionDocument);
        }
        match (
            self.receipt.receipt.result.outcome,
            self.child_envelope.as_ref(),
        ) {
            (CaptureAssignmentOutcome::Assigned, Some(child)) => {
                validate_child_envelope(&self.intent, &self.receipt, child)?;
            }
            (CaptureAssignmentOutcome::Rejected, None) => {}
            _ => return Err(ProjectError::InvalidResolutionDocument),
        }
        let bytes = serde_json_canonicalizer::to_vec(self)
            .map_err(|_| ProjectError::InvalidResolutionDocument)?;
        if bytes.len() > MAX_ASSIGNMENT_TRANSACTION_BYTES {
            return Err(ProjectError::DocumentTooLarge);
        }
        Ok(())
    }
}

struct ResolutionPaths {
    state_root: PathBuf,
    root: PathBuf,
    intents: PathBuf,
    receipts: PathBuf,
    transactions: PathBuf,
}

impl CaptureResolutionStore {
    pub(crate) fn new(config_root: ConfigRoot) -> Self {
        Self {
            delivery_store: CaptureDeliveryStore::new(config_root.clone()),
            config_root,
        }
    }

    pub(crate) fn commit_assignment(
        &self,
        transaction: &CaptureAssignmentTransactionV1,
    ) -> Result<StoredCaptureAssignment, ProjectError> {
        transaction.validate()?;
        let paths = self.prepare()?;
        let _lock = acquire_lock(&paths.root.join(CAPTURE_RESOLUTION_LOCK_FILE))?;
        recover_transactions_locked(&paths, &self.delivery_store)?;

        let intent_bytes = transaction.intent.to_canonical_json()?;
        let snapshot = rebuild_snapshot_locked(&paths, &self.delivery_store)?;
        if let Some(existing) = snapshot
            .assignments
            .iter()
            .find(|assignment| assignment.intent.intent_id == transaction.intent.intent_id)
        {
            if existing.intent == transaction.intent
                && existing.receipt.as_ref() == Some(&transaction.receipt)
            {
                return Ok(existing.clone());
            }
            if existing.receipt.is_some() {
                return Err(ProjectError::ResolutionIdentityConflict);
            }
        }

        validate_source_before_transaction(transaction, &self.delivery_store)?;
        write_immutable_intent(&paths, &transaction.intent, &intent_bytes)?;
        write_transaction(&paths, transaction)?;
        complete_transaction_locked(&paths, &self.delivery_store, transaction)?;

        rebuild_snapshot_locked(&paths, &self.delivery_store)?
            .assignments
            .into_iter()
            .find(|assignment| assignment.intent.intent_id == transaction.intent.intent_id)
            .ok_or(ProjectError::RecoveryRequired)
    }

    pub(crate) fn read_assignment(
        &self,
        intent_id: &CaptureAssignmentIntentId,
    ) -> Result<Option<StoredCaptureAssignment>, ProjectError> {
        CaptureAssignmentIntentId::parse(intent_id.as_str().to_owned())?;
        self.rebuild().map(|snapshot| {
            snapshot
                .assignments
                .into_iter()
                .find(|assignment| &assignment.intent.intent_id == intent_id)
        })
    }

    pub(crate) fn read_assignment_by_receipt_id(
        &self,
        receipt_id: &CaptureAssignmentReceiptId,
    ) -> Result<Option<StoredCaptureAssignment>, ProjectError> {
        CaptureAssignmentReceiptId::parse(receipt_id.as_str().to_owned())?;
        self.rebuild().map(|snapshot| {
            snapshot.assignments.into_iter().find(|assignment| {
                assignment
                    .receipt
                    .as_ref()
                    .is_some_and(|receipt| &receipt.receipt_id == receipt_id)
            })
        })
    }

    pub(crate) fn rebuild(&self) -> Result<CaptureResolutionLedgerSnapshot, ProjectError> {
        let paths = self.prepare()?;
        let _lock = acquire_lock(&paths.root.join(CAPTURE_RESOLUTION_LOCK_FILE))?;
        recover_transactions_locked(&paths, &self.delivery_store)?;
        rebuild_snapshot_locked(&paths, &self.delivery_store)
    }

    fn prepare(&self) -> Result<ResolutionPaths, ProjectError> {
        let root = prepare_private_state_directory(
            &self.config_root,
            &[
                CAPTURE_RESOLUTION_DIRECTORY,
                CAPTURE_RESOLUTION_STORAGE_VERSION,
            ],
        )?;
        let state_root = self.config_root.state_root().to_path_buf();
        let intents = root.join(CAPTURE_ASSIGNMENT_INTENTS_DIRECTORY);
        let receipts = root.join(CAPTURE_ASSIGNMENT_RECEIPTS_DIRECTORY);
        let transactions = root.join(CAPTURE_ASSIGNMENT_TRANSACTIONS_DIRECTORY);
        for directory in [&intents, &receipts, &transactions] {
            ensure_private_directory_beneath(&state_root, directory)?;
        }
        Ok(ResolutionPaths {
            state_root,
            root,
            intents,
            receipts,
            transactions,
        })
    }
}

fn validate_source_before_transaction(
    transaction: &CaptureAssignmentTransactionV1,
    delivery_store: &CaptureDeliveryStore,
) -> Result<(), ProjectError> {
    let source = delivery_store
        .read(&transaction.intent.intent.source_envelope_id)?
        .ok_or(ProjectError::DeliveryNotFound)?;
    if source.envelope_sha256 != transaction.intent.intent.source_envelope_sha256 {
        return Err(ProjectError::ResolutionIdentityConflict);
    }
    if source.record == transaction.source_record_after
        && source.record_sha256 == transaction.receipt.receipt.source_record_sha256_after
    {
        return Ok(());
    }
    if source.record.state != transaction.intent.intent.source_record_state
        || source.record.generation != transaction.intent.intent.source_record_generation
        || source.record_sha256 != transaction.intent.intent.source_record_sha256
    {
        return Err(ProjectError::RevisionConflict);
    }
    Ok(())
}

fn recover_transactions_locked(
    paths: &ResolutionPaths,
    delivery_store: &CaptureDeliveryStore,
) -> Result<(), ProjectError> {
    cleanup_owned_staging_files(paths)?;
    for intent_id in list_intent_ids(paths, &paths.transactions)? {
        let transaction =
            read_transaction(paths, &intent_id)?.ok_or(ProjectError::RecoveryRequired)?;
        write_immutable_intent(
            paths,
            &transaction.intent,
            &transaction.intent.to_canonical_json()?,
        )?;
        complete_transaction_locked(paths, delivery_store, &transaction)?;
    }
    Ok(())
}

fn complete_transaction_locked(
    paths: &ResolutionPaths,
    delivery_store: &CaptureDeliveryStore,
    transaction: &CaptureAssignmentTransactionV1,
) -> Result<(), ProjectError> {
    transaction.validate()?;
    if let Some(child) = transaction.child_envelope.as_ref() {
        let stored_child = delivery_store.enqueue(child)?;
        if stored_child.envelope != *child
            || stored_child.envelope_sha256 != sha256_bytes(&child.to_canonical_json()?)
        {
            return Err(ProjectError::ResolutionIdentityConflict);
        }
    }

    let source = delivery_store
        .read(&transaction.intent.intent.source_envelope_id)?
        .ok_or(ProjectError::DeliveryNotFound)?;
    if source.record != transaction.source_record_after
        || source.record_sha256 != transaction.receipt.receipt.source_record_sha256_after
    {
        if source.record.state != transaction.intent.intent.source_record_state
            || source.record.generation != transaction.intent.intent.source_record_generation
            || source.record_sha256 != transaction.intent.intent.source_record_sha256
        {
            return Err(ProjectError::RevisionConflict);
        }
        let committed = delivery_store.replace_record(
            &transaction.intent.intent.source_envelope_id,
            transaction.intent.intent.source_record_generation,
            &transaction.intent.intent.source_record_sha256,
            &transaction.source_record_after,
        )?;
        if committed.record != transaction.source_record_after
            || committed.record_sha256 != transaction.receipt.receipt.source_record_sha256_after
        {
            return Err(ProjectError::RecoveryRequired);
        }
    }

    write_immutable_receipt(
        paths,
        &transaction.receipt,
        &transaction.receipt.to_canonical_json()?,
    )?;
    remove_private_state_file(
        &paths.state_root,
        &paths
            .transactions
            .join(intent_file_name(&transaction.intent.intent_id)),
        MAX_ASSIGNMENT_TRANSACTION_BYTES,
    )
}

fn rebuild_snapshot_locked(
    paths: &ResolutionPaths,
    delivery_store: &CaptureDeliveryStore,
) -> Result<CaptureResolutionLedgerSnapshot, ProjectError> {
    cleanup_owned_staging_files(paths)?;
    if !list_intent_ids(paths, &paths.transactions)?.is_empty() {
        return Err(ProjectError::RecoveryRequired);
    }
    let intent_ids = list_intent_ids(paths, &paths.intents)?;
    let receipt_ids = list_receipt_ids(paths)?;
    let mut receipts = BTreeMap::new();
    for receipt_id in receipt_ids {
        let (receipt, receipt_bytes, receipt_sha256) =
            read_receipt(paths, &receipt_id)?.ok_or(ProjectError::RecoveryRequired)?;
        let intent_id = receipt.receipt.intent_id.clone();
        if receipts
            .insert(intent_id, (receipt, receipt_bytes, receipt_sha256))
            .is_some()
        {
            return Err(ProjectError::ResolutionIdentityConflict);
        }
    }

    let mut assignments = Vec::with_capacity(intent_ids.len());
    for intent_id in intent_ids {
        let (intent, intent_bytes, intent_sha256) =
            read_intent(paths, &intent_id)?.ok_or(ProjectError::RecoveryRequired)?;
        let receipt = receipts.remove(&intent_id);
        if let Some((receipt, receipt_bytes, receipt_sha256)) = receipt {
            validate_assignment_pair(&intent, &intent_bytes, &receipt, &receipt_bytes)?;
            validate_committed_assignment(&intent, &receipt, delivery_store)?;
            assignments.push(StoredCaptureAssignment {
                intent,
                intent_sha256,
                receipt: Some(receipt),
                receipt_sha256: Some(receipt_sha256),
            });
        } else {
            assignments.push(StoredCaptureAssignment {
                intent,
                intent_sha256,
                receipt: None,
                receipt_sha256: None,
            });
        }
    }
    if !receipts.is_empty() {
        return Err(ProjectError::ResolutionIdentityConflict);
    }
    Ok(CaptureResolutionLedgerSnapshot { assignments })
}

fn validate_assignment_pair(
    intent: &CaptureAssignmentIntentV1,
    intent_bytes: &[u8],
    receipt: &CaptureAssignmentReceiptV1,
    receipt_bytes: &[u8],
) -> Result<(), ProjectError> {
    let intent_canonical = intent.to_canonical_json()?;
    let receipt_canonical = receipt.to_canonical_json()?;
    if intent_bytes != intent_canonical
        || receipt_bytes != receipt_canonical
        || receipt.receipt.intent_id != intent.intent_id
        || receipt.receipt.intent_sha256 != sha256_bytes(&intent_canonical)
        || receipt.receipt.source_envelope_id != intent.intent.source_envelope_id
        || receipt.receipt.source_envelope_sha256 != intent.intent.source_envelope_sha256
        || receipt.receipt.source_capture_id != intent.intent.source_capture_id
        || receipt.receipt.source_capture_sha256 != intent.intent.source_capture_sha256
        || receipt.receipt.target_project_id != intent.intent.target_project_id
        || receipt.receipt.target_library_revision != intent.intent.expected_library_revision
        || receipt.receipt.target_project_revision != intent.intent.expected_project_revision
        || receipt.receipt.target_stage != intent.intent.target_stage
        || receipt.receipt.target_manifest_sha256 != intent.intent.target_manifest_sha256
        || receipt.receipt.observed_artifacts != intent.intent.observed_artifacts
        || receipt.receipt.source_record_generation_before != intent.intent.source_record_generation
        || receipt.receipt.source_record_sha256_before != intent.intent.source_record_sha256
        || receipt.receipt.intent_created_at_unix != intent.intent.created_at_unix
    {
        return Err(ProjectError::ResolutionIdentityConflict);
    }
    Ok(())
}

fn validate_committed_assignment(
    intent: &CaptureAssignmentIntentV1,
    receipt: &CaptureAssignmentReceiptV1,
    delivery_store: &CaptureDeliveryStore,
) -> Result<(), ProjectError> {
    let source = delivery_store
        .read(&intent.intent.source_envelope_id)?
        .ok_or(ProjectError::DeliveryNotFound)?;
    if source.envelope_sha256 != intent.intent.source_envelope_sha256
        || source.envelope.capture_id != intent.intent.source_capture_id
        || source.envelope.capture_sha256 != intent.intent.source_capture_sha256
        || source.record.state != CaptureDeliveryState::Cancelled
        || source.record.generation != receipt.receipt.source_record_generation_after
        || source.record_sha256 != receipt.receipt.source_record_sha256_after
    {
        return Err(ProjectError::ResolutionIdentityConflict);
    }
    let transition = source
        .record
        .transitions
        .last()
        .ok_or(ProjectError::ResolutionIdentityConflict)?;
    let previous = source.record.previous()?;
    if transition.reason_code != CaptureDeliveryReason::DeliveryCancelled
        || transition.transitioned_at_unix != receipt.receipt.decided_at_unix
        || previous.state != intent.intent.source_record_state
        || previous.generation != intent.intent.source_record_generation
        || sha256_bytes(&previous.to_canonical_json()?) != intent.intent.source_record_sha256
    {
        return Err(ProjectError::ResolutionIdentityConflict);
    }
    match receipt.receipt.result.outcome {
        CaptureAssignmentOutcome::Assigned => {
            let child_id = receipt
                .receipt
                .result
                .child_envelope_id
                .as_ref()
                .ok_or(ProjectError::InvalidResolutionDocument)?;
            let child = delivery_store
                .read(child_id)?
                .ok_or(ProjectError::DeliveryNotFound)?;
            validate_child_envelope(intent, receipt, &child.envelope)?;
        }
        CaptureAssignmentOutcome::Rejected => {}
    }
    Ok(())
}

fn validate_child_envelope(
    intent: &CaptureAssignmentIntentV1,
    receipt: &CaptureAssignmentReceiptV1,
    child: &CaptureDeliveryEnvelopeV1,
) -> Result<(), ProjectError> {
    child.validate()?;
    let result = &receipt.receipt.result;
    let destination = child
        .destination
        .as_ref()
        .ok_or(ProjectError::InvalidResolutionDocument)?;
    if result.child_envelope_id.as_ref() != Some(&child.envelope_id)
        || child.envelope_id == intent.intent.source_envelope_id
        || result.derived_capture_id.as_ref() != Some(&child.capture_id)
        || result.derived_capture_sha256.as_deref() != Some(child.capture_sha256.as_str())
        || destination.project_id != intent.intent.target_project_id
        || destination.expected_project_revision != intent.intent.expected_project_revision
        || child.capture.binding.project_id != intent.intent.target_project_id
        || child.capture.binding.base_revision != intent.intent.expected_project_revision
        || child.capture.binding.stage != intent.intent.target_stage
        || child.created_at_unix > receipt.receipt.decided_at_unix
    {
        return Err(ProjectError::ResolutionIdentityConflict);
    }
    Ok(())
}

fn write_immutable_intent(
    paths: &ResolutionPaths,
    intent: &CaptureAssignmentIntentV1,
    bytes: &[u8],
) -> Result<(), ProjectError> {
    write_immutable_document(
        paths,
        &paths.intents,
        &intent_file_name(&intent.intent_id),
        bytes,
        MAX_ASSIGNMENT_INTENT_BYTES,
    )
}

fn write_immutable_receipt(
    paths: &ResolutionPaths,
    receipt: &CaptureAssignmentReceiptV1,
    bytes: &[u8],
) -> Result<(), ProjectError> {
    write_immutable_document(
        paths,
        &paths.receipts,
        &receipt_file_name(&receipt.receipt_id),
        bytes,
        MAX_ASSIGNMENT_RECEIPT_BYTES,
    )
}

fn write_transaction(
    paths: &ResolutionPaths,
    transaction: &CaptureAssignmentTransactionV1,
) -> Result<(), ProjectError> {
    let bytes = transaction.to_canonical_json()?;
    write_immutable_document(
        paths,
        &paths.transactions,
        &intent_file_name(&transaction.intent.intent_id),
        &bytes,
        MAX_ASSIGNMENT_TRANSACTION_BYTES,
    )
}

fn write_immutable_document(
    paths: &ResolutionPaths,
    directory: &Path,
    file_name: &str,
    bytes: &[u8],
    maximum_bytes: usize,
) -> Result<(), ProjectError> {
    let path = directory.join(file_name);
    match read_private_document(paths, &path, maximum_bytes)? {
        Some(existing) if existing == bytes => Ok(()),
        Some(_) => Err(ProjectError::ResolutionIdentityConflict),
        None => atomic_write(directory, file_name, bytes, true),
    }
}

fn read_intent(
    paths: &ResolutionPaths,
    intent_id: &CaptureAssignmentIntentId,
) -> Result<Option<(CaptureAssignmentIntentV1, Vec<u8>, String)>, ProjectError> {
    let path = paths.intents.join(intent_file_name(intent_id));
    let Some(bytes) = read_private_document(paths, &path, MAX_ASSIGNMENT_INTENT_BYTES)? else {
        return Ok(None);
    };
    let document = CaptureAssignmentIntentV1::from_json_slice(&bytes)?;
    if document.intent_id != *intent_id {
        return Err(ProjectError::ResolutionIdentityConflict);
    }
    let sha256 = sha256_bytes(&bytes);
    Ok(Some((document, bytes, sha256)))
}

fn read_receipt(
    paths: &ResolutionPaths,
    receipt_id: &CaptureAssignmentReceiptId,
) -> Result<Option<(CaptureAssignmentReceiptV1, Vec<u8>, String)>, ProjectError> {
    let path = paths.receipts.join(receipt_file_name(receipt_id));
    let Some(bytes) = read_private_document(paths, &path, MAX_ASSIGNMENT_RECEIPT_BYTES)? else {
        return Ok(None);
    };
    let document = CaptureAssignmentReceiptV1::from_json_slice(&bytes)?;
    if document.receipt_id != *receipt_id {
        return Err(ProjectError::ResolutionIdentityConflict);
    }
    let sha256 = sha256_bytes(&bytes);
    Ok(Some((document, bytes, sha256)))
}

fn read_transaction(
    paths: &ResolutionPaths,
    intent_id: &CaptureAssignmentIntentId,
) -> Result<Option<CaptureAssignmentTransactionV1>, ProjectError> {
    let path = paths.transactions.join(intent_file_name(intent_id));
    let Some(bytes) = read_private_document(paths, &path, MAX_ASSIGNMENT_TRANSACTION_BYTES)? else {
        return Ok(None);
    };
    let transaction = CaptureAssignmentTransactionV1::from_json_slice(&bytes)?;
    if transaction.intent.intent_id != *intent_id {
        return Err(ProjectError::ResolutionIdentityConflict);
    }
    Ok(Some(transaction))
}

fn read_private_document(
    paths: &ResolutionPaths,
    path: &Path,
    maximum_bytes: usize,
) -> Result<Option<Vec<u8>>, ProjectError> {
    let Some(metadata) = project_metadata_if_exists(&paths.state_root, path)? else {
        return Ok(None);
    };
    read_bounded_project_file(&paths.state_root, path, &metadata, maximum_bytes, true).map(Some)
}

fn list_intent_ids(
    paths: &ResolutionPaths,
    directory: &Path,
) -> Result<Vec<CaptureAssignmentIntentId>, ProjectError> {
    let metadata = project_metadata_if_exists(&paths.state_root, directory)?
        .ok_or(ProjectError::RecoveryRequired)?;
    validate_private_directory(directory, &metadata)?;
    let mut ids = Vec::new();
    for entry in fs::read_dir(directory).map_err(map_io)? {
        let file_name = entry
            .map_err(map_io)?
            .file_name()
            .into_string()
            .map_err(|_| ProjectError::InvalidResolutionDocument)?;
        let id = file_name
            .strip_suffix(".json")
            .ok_or(ProjectError::InvalidResolutionDocument)?;
        ids.push(CaptureAssignmentIntentId::parse(id.to_owned())?);
        if ids.len() > MAX_CAPTURE_ASSIGNMENTS {
            return Err(ProjectError::DocumentTooLarge);
        }
    }
    ids.sort();
    Ok(ids)
}

fn list_receipt_ids(
    paths: &ResolutionPaths,
) -> Result<Vec<CaptureAssignmentReceiptId>, ProjectError> {
    let metadata = project_metadata_if_exists(&paths.state_root, &paths.receipts)?
        .ok_or(ProjectError::RecoveryRequired)?;
    validate_private_directory(&paths.receipts, &metadata)?;
    let mut ids = Vec::new();
    for entry in fs::read_dir(&paths.receipts).map_err(map_io)? {
        let file_name = entry
            .map_err(map_io)?
            .file_name()
            .into_string()
            .map_err(|_| ProjectError::InvalidResolutionDocument)?;
        let id = file_name
            .strip_suffix(".json")
            .ok_or(ProjectError::InvalidResolutionDocument)?;
        ids.push(CaptureAssignmentReceiptId::parse(id.to_owned())?);
        if ids.len() > MAX_CAPTURE_ASSIGNMENTS {
            return Err(ProjectError::DocumentTooLarge);
        }
    }
    ids.sort();
    Ok(ids)
}

fn cleanup_owned_staging_files(paths: &ResolutionPaths) -> Result<(), ProjectError> {
    for (directory, kind, maximum_bytes) in [
        (
            &paths.intents,
            StoredResolutionDocumentKind::Intent,
            MAX_ASSIGNMENT_INTENT_BYTES,
        ),
        (
            &paths.receipts,
            StoredResolutionDocumentKind::Receipt,
            MAX_ASSIGNMENT_RECEIPT_BYTES,
        ),
        (
            &paths.transactions,
            StoredResolutionDocumentKind::Transaction,
            MAX_ASSIGNMENT_TRANSACTION_BYTES,
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
                .map_err(|_| ProjectError::InvalidResolutionDocument)?;
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
enum StoredResolutionDocumentKind {
    Intent,
    Receipt,
    Transaction,
}

fn is_owned_staging_file(file_name: &str, kind: StoredResolutionDocumentKind) -> bool {
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
        StoredResolutionDocumentKind::Intent | StoredResolutionDocumentKind::Transaction => {
            CaptureAssignmentIntentId::parse(id.to_owned()).is_ok()
        }
        StoredResolutionDocumentKind::Receipt => {
            CaptureAssignmentReceiptId::parse(id.to_owned()).is_ok()
        }
    }
}

fn intent_file_name(intent_id: &CaptureAssignmentIntentId) -> String {
    format!("{}.json", intent_id.as_str())
}

fn receipt_file_name(receipt_id: &CaptureAssignmentReceiptId) -> String {
    format!("{}.json", receipt_id.as_str())
}

fn map_io(error: std::io::Error) -> ProjectError {
    ProjectError::PersistenceFailed(error.kind())
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};

    use qiongli_config::resolve_config_root;

    use crate::{
        CaptureAssignmentIntentBodyV1, CaptureAssignmentResultV1, CaptureDelivery,
        CaptureDeliveryDestinationV1, CapturePolicy, CaptureResolutionArtifact,
        CaptureResolutionArtifactObservationV1, CaptureSource, ProjectBindingV1, ProjectId,
        ProjectStage, ResearchCaptureDraftV1,
    };

    use super::*;

    static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

    struct Fixture {
        root: PathBuf,
        config_root: ConfigRoot,
        delivery_store: CaptureDeliveryStore,
        resolution_store: CaptureResolutionStore,
    }

    impl Fixture {
        fn new() -> Self {
            let root = std::env::temp_dir().join(format!(
                "qiongli-capture-resolution-ledger-{}-{}",
                std::process::id(),
                NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed),
            ));
            let _ = fs::remove_dir_all(&root);
            fs::create_dir(&root).unwrap();
            let root = fs::canonicalize(root).unwrap();
            let home = root.join("home");
            fs::create_dir(&home).unwrap();
            let config_root = resolve_config_root(None, &home).unwrap();
            Self {
                root,
                delivery_store: CaptureDeliveryStore::new(config_root.clone()),
                resolution_store: CaptureResolutionStore::new(config_root.clone()),
                config_root,
            }
        }

        #[cfg(unix)]
        fn ledger_root(&self) -> PathBuf {
            self.config_root
                .state_root()
                .join(CAPTURE_RESOLUTION_DIRECTORY)
                .join(CAPTURE_RESOLUTION_STORAGE_VERSION)
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    fn digest(character: char) -> String {
        character.to_string().repeat(64)
    }

    fn project_id() -> ProjectId {
        ProjectId::parse("prj_0123456789abcdef0123456789abcdef").unwrap()
    }

    fn observations() -> Vec<CaptureResolutionArtifactObservationV1> {
        [
            CaptureResolutionArtifact::ResearchState,
            CaptureResolutionArtifact::DecisionLog,
            CaptureResolutionArtifact::CaptureHistory,
            CaptureResolutionArtifact::ConsolidationHistory,
        ]
        .into_iter()
        .zip(['1', '2', '3', '4'])
        .map(|(artifact, character)| {
            CaptureResolutionArtifactObservationV1::new(artifact, Some(digest(character)))
        })
        .collect()
    }

    fn capture(suffix: &str) -> crate::ResearchCaptureV1 {
        ResearchCaptureDraftV1 {
            binding: ProjectBindingV1::new(
                project_id(),
                4,
                ProjectStage::Literature,
                format!("Assign capture {suffix}"),
                CapturePolicy::ReviewRequired,
            )
            .unwrap(),
            source: CaptureSource::Codex,
            delivery: CaptureDelivery::Connected,
            captured_at_unix: 1_800_000_000,
            summary: format!("Private assignment summary {suffix}."),
            changes: Vec::new(),
            decisions: Vec::new(),
            evidence: Vec::new(),
            contradictions: Vec::new(),
            next_actions: vec!["Review the assigned capture.".to_string()],
        }
        .into_capture()
        .unwrap()
    }

    fn transaction(
        fixture: &Fixture,
        suffix: &str,
        assigned: bool,
    ) -> CaptureAssignmentTransactionV1 {
        let capture = capture(suffix);
        let source_envelope =
            CaptureDeliveryEnvelopeV1::new(capture.clone(), None, 1_800_000_010).unwrap();
        let source = fixture.delivery_store.enqueue(&source_envelope).unwrap();
        let intent = CaptureAssignmentIntentV1::new(CaptureAssignmentIntentBodyV1 {
            source_envelope_id: source_envelope.envelope_id.clone(),
            source_envelope_sha256: source.envelope_sha256.clone(),
            source_record_state: source.record.state,
            source_record_generation: source.record.generation,
            source_record_sha256: source.record_sha256.clone(),
            source_capture_id: source_envelope.capture_id.clone(),
            source_capture_sha256: source_envelope.capture_sha256.clone(),
            target_project_id: project_id(),
            expected_library_revision: 7,
            expected_project_revision: 4,
            target_stage: ProjectStage::Literature,
            target_manifest_sha256: digest('5'),
            observed_artifacts: observations(),
            created_at_unix: 1_800_000_011,
        })
        .unwrap();
        let source_record_after = source
            .record
            .transition(
                CaptureDeliveryState::Cancelled,
                1_800_000_012,
                CaptureDeliveryReason::DeliveryCancelled,
                None,
            )
            .unwrap();
        let source_record_sha256_after =
            sha256_bytes(&source_record_after.to_canonical_json().unwrap());
        let (result, child_envelope) = if assigned {
            let child = CaptureDeliveryEnvelopeV1::new(
                capture,
                Some(CaptureDeliveryDestinationV1::new(project_id(), 4).unwrap()),
                1_800_000_011,
            )
            .unwrap();
            (
                CaptureAssignmentResultV1::assigned(
                    child.capture_id.clone(),
                    child.capture_sha256.clone(),
                    child.envelope_id.clone(),
                ),
                Some(child),
            )
        } else {
            (CaptureAssignmentResultV1::rejected(), None)
        };
        let receipt = CaptureAssignmentReceiptV1::new(
            &intent,
            result,
            source_record_after.generation,
            source_record_sha256_after,
            1_800_000_012,
        )
        .unwrap();
        CaptureAssignmentTransactionV1::new(intent, receipt, child_envelope, source_record_after)
            .unwrap()
    }

    fn assert_completed(
        fixture: &Fixture,
        transaction: &CaptureAssignmentTransactionV1,
    ) -> StoredCaptureAssignment {
        let assignment = fixture
            .resolution_store
            .read_assignment(&transaction.intent.intent_id)
            .unwrap()
            .unwrap();
        assert_eq!(assignment.intent, transaction.intent);
        assert_eq!(assignment.receipt.as_ref(), Some(&transaction.receipt));
        let source = fixture
            .delivery_store
            .read(&transaction.intent.intent.source_envelope_id)
            .unwrap()
            .unwrap();
        assert_eq!(source.record, transaction.source_record_after);
        if let Some(child) = transaction.child_envelope.as_ref() {
            assert_eq!(
                fixture
                    .delivery_store
                    .read(&child.envelope_id)
                    .unwrap()
                    .unwrap()
                    .envelope,
                *child
            );
        }
        let paths = fixture.resolution_store.prepare().unwrap();
        assert_eq!(fs::read_dir(paths.transactions).unwrap().count(), 0);
        assignment
    }

    #[test]
    fn assignment_commit_replays_reopens_and_indexes_assigned_and_rejected() {
        let fixture = Fixture::new();
        let assigned_transaction = transaction(&fixture, "assigned", true);
        let assigned = fixture
            .resolution_store
            .commit_assignment(&assigned_transaction)
            .unwrap();
        assert_eq!(
            assigned.receipt.as_ref().unwrap().receipt.result.outcome,
            CaptureAssignmentOutcome::Assigned
        );
        assert_eq!(
            fixture
                .resolution_store
                .commit_assignment(&assigned_transaction)
                .unwrap(),
            assigned
        );

        let rejected_transaction = transaction(&fixture, "rejected", false);
        let rejected = fixture
            .resolution_store
            .commit_assignment(&rejected_transaction)
            .unwrap();
        assert_eq!(
            rejected.receipt.as_ref().unwrap().receipt.result.outcome,
            CaptureAssignmentOutcome::Rejected
        );
        let reopened = CaptureResolutionStore::new(fixture.config_root.clone())
            .rebuild()
            .unwrap();
        assert_eq!(reopened.assignments.len(), 2);
        assert!(
            reopened
                .assignments
                .windows(2)
                .all(|pair| pair[0].intent.intent_id < pair[1].intent.intent_id)
        );
        assert!(!format!("{reopened:?}").contains("Private assignment summary"));

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            for directory in [
                fixture.ledger_root(),
                fixture
                    .ledger_root()
                    .join(CAPTURE_ASSIGNMENT_INTENTS_DIRECTORY),
                fixture
                    .ledger_root()
                    .join(CAPTURE_ASSIGNMENT_RECEIPTS_DIRECTORY),
                fixture
                    .ledger_root()
                    .join(CAPTURE_ASSIGNMENT_TRANSACTIONS_DIRECTORY),
            ] {
                assert_eq!(
                    fs::metadata(directory).unwrap().permissions().mode() & 0o077,
                    0
                );
            }
        }
    }

    #[test]
    fn every_assignment_write_boundary_recovers_the_exact_completed_state() {
        for boundary in 0..4 {
            let fixture = Fixture::new();
            let transaction = transaction(&fixture, &format!("boundary-{boundary}"), true);
            let paths = fixture.resolution_store.prepare().unwrap();
            if boundary > 0 {
                write_immutable_intent(
                    &paths,
                    &transaction.intent,
                    &transaction.intent.to_canonical_json().unwrap(),
                )
                .unwrap();
            }
            write_transaction(&paths, &transaction).unwrap();
            if boundary >= 1 {
                fixture
                    .delivery_store
                    .enqueue(transaction.child_envelope.as_ref().unwrap())
                    .unwrap();
            }
            if boundary >= 2 {
                fixture
                    .delivery_store
                    .replace_record(
                        &transaction.intent.intent.source_envelope_id,
                        transaction.intent.intent.source_record_generation,
                        &transaction.intent.intent.source_record_sha256,
                        &transaction.source_record_after,
                    )
                    .unwrap();
            }
            if boundary >= 3 {
                write_immutable_receipt(
                    &paths,
                    &transaction.receipt,
                    &transaction.receipt.to_canonical_json().unwrap(),
                )
                .unwrap();
            }

            let reopened = CaptureResolutionStore::new(fixture.config_root.clone());
            assert_eq!(reopened.rebuild().unwrap().assignments.len(), 1);
            assert_completed(&fixture, &transaction);
        }
    }

    #[test]
    fn stale_source_fails_before_assignment_state_is_written() {
        let fixture = Fixture::new();
        let transaction = transaction(&fixture, "stale-source", true);
        let stale = transaction
            .source_record_after
            .previous()
            .unwrap()
            .transition(
                CaptureDeliveryState::Cancelled,
                1_800_000_013,
                CaptureDeliveryReason::DeliveryCancelled,
                None,
            )
            .unwrap();
        fixture
            .delivery_store
            .replace_record(
                &transaction.intent.intent.source_envelope_id,
                transaction.intent.intent.source_record_generation,
                &transaction.intent.intent.source_record_sha256,
                &stale,
            )
            .unwrap();

        assert_eq!(
            fixture.resolution_store.commit_assignment(&transaction),
            Err(ProjectError::RevisionConflict)
        );
        let paths = fixture.resolution_store.prepare().unwrap();
        assert_eq!(fs::read_dir(paths.intents).unwrap().count(), 0);
        assert_eq!(fs::read_dir(paths.receipts).unwrap().count(), 0);
        assert_eq!(fs::read_dir(paths.transactions).unwrap().count(), 0);
        assert!(
            fixture
                .delivery_store
                .read(
                    transaction
                        .child_envelope
                        .as_ref()
                        .map(|child| &child.envelope_id)
                        .unwrap()
                )
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn resolution_lock_contention_fails_without_creating_state() {
        let fixture = Fixture::new();
        let paths = fixture.resolution_store.prepare().unwrap();
        let lock = acquire_lock(&paths.root.join(CAPTURE_RESOLUTION_LOCK_FILE)).unwrap();
        assert_eq!(
            fixture.resolution_store.rebuild(),
            Err(ProjectError::LockBusy)
        );
        drop(lock);
        assert!(
            fixture
                .resolution_store
                .rebuild()
                .unwrap()
                .assignments
                .is_empty()
        );
    }

    #[test]
    fn ledger_rejects_corruption_orphans_unknowns_and_over_count_indexes() {
        let corrupt_fixture = Fixture::new();
        let corrupt_paths = corrupt_fixture.resolution_store.prepare().unwrap();
        let corrupt_id = CaptureAssignmentIntentId::parse(format!("cai_{}", digest('a'))).unwrap();
        atomic_write(
            &corrupt_paths.intents,
            &intent_file_name(&corrupt_id),
            b"{\"schemaVersion\":1}",
            true,
        )
        .unwrap();
        assert_eq!(
            corrupt_fixture.resolution_store.rebuild(),
            Err(ProjectError::InvalidResolutionDocument)
        );

        let orphan_fixture = Fixture::new();
        let orphan_transaction = transaction(&orphan_fixture, "orphan", true);
        let orphan_paths = orphan_fixture.resolution_store.prepare().unwrap();
        write_immutable_receipt(
            &orphan_paths,
            &orphan_transaction.receipt,
            &orphan_transaction.receipt.to_canonical_json().unwrap(),
        )
        .unwrap();
        assert_eq!(
            orphan_fixture.resolution_store.rebuild(),
            Err(ProjectError::ResolutionIdentityConflict)
        );

        let unknown_fixture = Fixture::new();
        let unknown_paths = unknown_fixture.resolution_store.prepare().unwrap();
        atomic_write(&unknown_paths.intents, "notes.json", b"{}", true).unwrap();
        assert_eq!(
            unknown_fixture.resolution_store.rebuild(),
            Err(ProjectError::InvalidResolutionDocument)
        );

        let over_count_fixture = Fixture::new();
        let over_count_paths = over_count_fixture.resolution_store.prepare().unwrap();
        for index in 0..=MAX_CAPTURE_ASSIGNMENTS {
            let id = CaptureAssignmentIntentId::parse(format!("cai_{index:064x}")).unwrap();
            fs::write(over_count_paths.intents.join(intent_file_name(&id)), b"{}").unwrap();
        }
        assert_eq!(
            over_count_fixture.resolution_store.rebuild(),
            Err(ProjectError::DocumentTooLarge)
        );
    }

    #[cfg(unix)]
    #[test]
    fn ledger_rejects_links_and_permission_broadened_documents() {
        use std::os::unix::fs::{PermissionsExt, symlink};

        let permission_fixture = Fixture::new();
        let permission_transaction = transaction(&permission_fixture, "permissions", true);
        permission_fixture
            .resolution_store
            .commit_assignment(&permission_transaction)
            .unwrap();
        let permission_path = permission_fixture
            .ledger_root()
            .join(CAPTURE_ASSIGNMENT_INTENTS_DIRECTORY)
            .join(intent_file_name(&permission_transaction.intent.intent_id));
        fs::set_permissions(&permission_path, fs::Permissions::from_mode(0o644)).unwrap();
        assert_eq!(
            permission_fixture.resolution_store.rebuild(),
            Err(ProjectError::UnsafeProjectRoot)
        );

        let hard_link_fixture = Fixture::new();
        let hard_link_transaction = transaction(&hard_link_fixture, "hard-link", true);
        hard_link_fixture
            .resolution_store
            .commit_assignment(&hard_link_transaction)
            .unwrap();
        let hard_link_path = hard_link_fixture
            .ledger_root()
            .join(CAPTURE_ASSIGNMENT_RECEIPTS_DIRECTORY)
            .join(receipt_file_name(&hard_link_transaction.receipt.receipt_id));
        fs::hard_link(&hard_link_path, hard_link_fixture.root.join("second-link")).unwrap();
        assert_eq!(
            hard_link_fixture.resolution_store.rebuild(),
            Err(ProjectError::UnsafeProjectRoot)
        );

        let symlink_fixture = Fixture::new();
        let symlink_paths = symlink_fixture.resolution_store.prepare().unwrap();
        fs::remove_dir(&symlink_paths.intents).unwrap();
        symlink(&symlink_paths.receipts, &symlink_paths.intents).unwrap();
        assert_eq!(
            symlink_fixture.resolution_store.rebuild(),
            Err(ProjectError::UnsafeProjectRoot)
        );
    }
}
