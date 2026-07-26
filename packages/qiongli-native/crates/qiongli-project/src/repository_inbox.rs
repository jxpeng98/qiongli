use std::fmt::{self, Debug, Formatter};

use serde::Serialize;

use crate::capture::{CaptureDisposition, CaptureId, classify_capture};
use crate::capture_delivery::{
    CaptureDeliveryDestinationV1, CaptureDeliveryEnvelopeV1, DeliveryEnvelopeId,
};
use crate::capture_delivery_service::CaptureDeliveryStatusV1;
use crate::model::{ProjectId, ProjectLifecycle, ProjectStage};
use crate::storage::{
    list_repository_capture_documents, project_root_from_string, read_capture_document,
    read_manifest, read_repository_capture_document, repository_capture_inbox_relative_path,
    sha256_bytes, validate_existing_project_root,
};
use crate::{
    ApprovedCaptureIntake, ArtifactChangeSnapshotV1, CaptureDelivery, CaptureIntakeCommitV1,
    CaptureIntakePreviewV1, CapturePolicy, CaptureSource, ProjectError, ProjectStateService,
    ResearchCaptureV1, VerifiedCaptureIntake,
};

pub const REPOSITORY_CAPTURE_INBOX_SCHEMA_VERSION: u32 = 1;
pub const REPOSITORY_CAPTURE_DELIVERY_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RepositoryCaptureInboxState {
    Pending,
    Accepted,
    Stale,
    Conflicted,
    Unbound,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryCaptureInboxEntryV1 {
    pub capture_id: CaptureId,
    pub project_id: ProjectId,
    pub state: RepositoryCaptureInboxState,
    pub disposition: CaptureDisposition,
    pub source: CaptureSource,
    pub captured_at_unix: u64,
    pub base_revision: u64,
    pub bound_stage: ProjectStage,
    pub task: String,
    pub capture_policy: CapturePolicy,
    pub summary: String,
    pub change_count: usize,
    pub decision_count: usize,
    pub evidence_count: usize,
    pub contradiction_count: usize,
    pub next_action_count: usize,
    pub repository_entry: String,
    pub history_entry: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryCaptureInboxSnapshotV1 {
    pub schema_version: u32,
    pub project_id: ProjectId,
    pub project_revision: u64,
    pub project_stage: ProjectStage,
    pub pending_count: usize,
    pub accepted_count: usize,
    pub stale_count: usize,
    pub conflicted_count: usize,
    pub unbound_count: usize,
    pub entries: Vec<RepositoryCaptureInboxEntryV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryCaptureIntakePreviewV1 {
    pub schema_version: u32,
    pub repository_entry: String,
    pub intake: CaptureIntakePreviewV1,
}

#[derive(Clone)]
pub struct VerifiedRepositoryCaptureIntake {
    preview: RepositoryCaptureIntakePreviewV1,
    intake: VerifiedCaptureIntake,
    capture: ResearchCaptureV1,
    repository_digest: String,
}

impl VerifiedRepositoryCaptureIntake {
    #[must_use]
    pub const fn preview(&self) -> &RepositoryCaptureIntakePreviewV1 {
        &self.preview
    }
}

impl Debug for VerifiedRepositoryCaptureIntake {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("VerifiedRepositoryCaptureIntake")
            .field("preview", &self.preview)
            .field("capture", &"<bounded-repository-capture>")
            .field("repository_digest", &"<repository-source-digest>")
            .finish()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryCaptureDeliveryPreviewV1 {
    pub schema_version: u32,
    pub plan_digest: String,
    pub repository_project_id: ProjectId,
    pub source_capture_id: CaptureId,
    pub source_capture_sha256: String,
    pub repository_sha256: String,
    pub envelope_id: DeliveryEnvelopeId,
    pub destination_project_id: Option<ProjectId>,
    pub expected_destination_revision: Option<u64>,
    pub queued_at_unix: u64,
    pub artifact_change_ids: Vec<String>,
    pub unattributed_change_count: usize,
    pub approvals_required: Vec<String>,
}

#[derive(Clone)]
pub struct VerifiedRepositoryCaptureDelivery {
    preview: RepositoryCaptureDeliveryPreviewV1,
    envelope: CaptureDeliveryEnvelopeV1,
    capture: ResearchCaptureV1,
    repository_digest: String,
}

impl VerifiedRepositoryCaptureDelivery {
    #[must_use]
    pub const fn preview(&self) -> &RepositoryCaptureDeliveryPreviewV1 {
        &self.preview
    }
}

impl Debug for VerifiedRepositoryCaptureDelivery {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("VerifiedRepositoryCaptureDelivery")
            .field("preview", &self.preview)
            .field("envelope", &"<bounded-delivery-envelope>")
            .field("capture", &"<bounded-repository-capture>")
            .field("repository_digest", &"<repository-source-digest>")
            .finish()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApprovedRepositoryCaptureDelivery {
    expected_plan_digest: String,
    delivery_write: bool,
}

impl ApprovedRepositoryCaptureDelivery {
    #[must_use]
    pub fn new(expected_plan_digest: impl Into<String>, delivery_write: bool) -> Self {
        Self {
            expected_plan_digest: expected_plan_digest.into(),
            delivery_write,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryCaptureDeliveryCommitV1 {
    pub schema_version: u32,
    pub repository_project_id: ProjectId,
    pub artifact_change_ids: Vec<String>,
    pub delivery: CaptureDeliveryStatusV1,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RepositoryCaptureDeliveryPlanIdentity<'a> {
    schema_version: u32,
    repository_project_id: &'a ProjectId,
    source_capture_id: &'a CaptureId,
    source_capture_sha256: &'a str,
    repository_sha256: &'a str,
    envelope_id: &'a DeliveryEnvelopeId,
    destination_project_id: Option<&'a ProjectId>,
    expected_destination_revision: Option<u64>,
    queued_at_unix: u64,
    artifact_change_ids: &'a [String],
    unattributed_change_count: usize,
    approvals_required: &'a [String],
}

impl ProjectStateService {
    pub fn repository_capture_inbox(
        &self,
        project_id: &ProjectId,
    ) -> Result<RepositoryCaptureInboxSnapshotV1, ProjectError> {
        let library = self.store.load()?;
        library.validate()?;
        let entry = library
            .projects
            .iter()
            .find(|entry| &entry.project_id == project_id)
            .ok_or(ProjectError::ProjectNotRegistered)?;
        let root = project_root_from_string(&entry.root_path)?;
        validate_existing_project_root(&root)?;
        let (manifest, _) = read_manifest(&root)?.ok_or(ProjectError::ProjectManifestMissing)?;
        if manifest.project_id != *project_id
            || entry.semantic_revision != manifest.semantic_revision
            || entry.semantic_digest != manifest.semantic_digest
            || entry.stage != manifest.stage
            || entry.lifecycle != manifest.lifecycle
        {
            return Err(ProjectError::RevisionConflict);
        }

        let mut entries = list_repository_capture_documents(&root)?
            .into_iter()
            .map(|(capture, _)| {
                validate_repository_delivery(&capture)?;
                let accepted = match read_capture_document(&root, &capture.capture_id)? {
                    Some((committed, _)) if committed == capture => true,
                    Some(_) => return Err(ProjectError::CaptureIdentityConflict),
                    None => false,
                };
                let state = repository_capture_state(
                    accepted,
                    project_id,
                    entry.lifecycle,
                    manifest.semantic_revision,
                    manifest.stage,
                    &capture,
                );
                Ok(RepositoryCaptureInboxEntryV1 {
                    capture_id: capture.capture_id.clone(),
                    project_id: capture.binding.project_id.clone(),
                    state,
                    disposition: classify_capture(&capture, accepted),
                    source: capture.source,
                    captured_at_unix: capture.captured_at_unix,
                    base_revision: capture.binding.base_revision,
                    bound_stage: capture.binding.stage,
                    task: capture.binding.task.clone(),
                    capture_policy: capture.binding.capture_policy,
                    summary: capture.summary.clone(),
                    change_count: capture.changes.len(),
                    decision_count: capture.decisions.len(),
                    evidence_count: capture.evidence.len(),
                    contradiction_count: capture.contradictions.len(),
                    next_action_count: capture.next_actions.len(),
                    repository_entry: repository_capture_inbox_relative_path(&capture.capture_id),
                    history_entry: accepted.then(|| {
                        crate::storage::capture_history_relative_path(&capture.capture_id)
                    }),
                })
            })
            .collect::<Result<Vec<_>, ProjectError>>()?;
        entries.sort_by(|left, right| {
            right
                .captured_at_unix
                .cmp(&left.captured_at_unix)
                .then_with(|| left.capture_id.cmp(&right.capture_id))
        });

        Ok(RepositoryCaptureInboxSnapshotV1 {
            schema_version: REPOSITORY_CAPTURE_INBOX_SCHEMA_VERSION,
            project_id: project_id.clone(),
            project_revision: manifest.semantic_revision,
            project_stage: manifest.stage,
            pending_count: count_state(&entries, RepositoryCaptureInboxState::Pending),
            accepted_count: count_state(&entries, RepositoryCaptureInboxState::Accepted),
            stale_count: count_state(&entries, RepositoryCaptureInboxState::Stale),
            conflicted_count: count_state(&entries, RepositoryCaptureInboxState::Conflicted),
            unbound_count: count_state(&entries, RepositoryCaptureInboxState::Unbound),
            entries,
        })
    }

    pub fn read_repository_capture(
        &self,
        project_id: &ProjectId,
        capture_id: &CaptureId,
    ) -> Result<Option<ResearchCaptureV1>, ProjectError> {
        let root = self.resolve_project_root(project_id)?;
        read_repository_capture_document(root.path(), capture_id).and_then(|capture| {
            capture
                .map(|(capture, _)| {
                    validate_repository_delivery(&capture)?;
                    Ok(capture)
                })
                .transpose()
        })
    }

    pub fn preview_repository_capture(
        &self,
        project_id: &ProjectId,
        capture_id: &CaptureId,
    ) -> Result<VerifiedRepositoryCaptureIntake, ProjectError> {
        let root = self.resolve_project_root(project_id)?;
        let (capture, repository_digest) =
            read_repository_capture_document(root.path(), capture_id)?
                .ok_or(ProjectError::CaptureNotFound)?;
        validate_repository_delivery(&capture)?;
        if capture.binding.project_id != *project_id {
            return Err(ProjectError::CaptureIdentityConflict);
        }
        let intake = self.preview_capture(capture.clone())?;
        let preview = RepositoryCaptureIntakePreviewV1 {
            schema_version: REPOSITORY_CAPTURE_INBOX_SCHEMA_VERSION,
            repository_entry: repository_capture_inbox_relative_path(capture_id),
            intake: intake.preview().clone(),
        };
        Ok(VerifiedRepositoryCaptureIntake {
            preview,
            intake,
            capture,
            repository_digest,
        })
    }

    pub fn apply_repository_capture(
        &self,
        plan: &VerifiedRepositoryCaptureIntake,
        approval: &ApprovedCaptureIntake,
        now_unix: u64,
    ) -> Result<CaptureIntakeCommitV1, ProjectError> {
        let root = self.resolve_project_root(&plan.capture.binding.project_id)?;
        let (capture, repository_digest) =
            read_repository_capture_document(root.path(), &plan.capture.capture_id)?
                .ok_or(ProjectError::CaptureNotFound)?;
        if capture != plan.capture || repository_digest != plan.repository_digest {
            return Err(ProjectError::RevisionConflict);
        }
        self.apply_capture(&plan.intake, approval, now_unix)
    }

    pub fn preview_repository_capture_delivery(
        &self,
        repository_project_id: &ProjectId,
        capture_id: &CaptureId,
        queued_at_unix: u64,
    ) -> Result<VerifiedRepositoryCaptureDelivery, ProjectError> {
        let inbox = self.repository_capture_inbox(repository_project_id)?;
        let root = self.resolve_project_root(repository_project_id)?;
        let (capture, repository_digest) =
            read_repository_capture_document(root.path(), capture_id)?
                .ok_or(ProjectError::CaptureNotFound)?;
        validate_repository_delivery(&capture)?;
        let destination = if capture.binding.project_id == *repository_project_id {
            Some(CaptureDeliveryDestinationV1::new(
                repository_project_id.clone(),
                inbox.project_revision,
            )?)
        } else {
            None
        };
        let envelope =
            CaptureDeliveryEnvelopeV1::new(capture.clone(), destination, queued_at_unix)?;
        let artifact_changes = self.artifact_changes(repository_project_id)?;
        build_verified_repository_delivery(
            repository_project_id,
            capture,
            repository_digest,
            envelope,
            &artifact_changes,
        )
    }

    pub fn apply_repository_capture_delivery(
        &self,
        plan: &VerifiedRepositoryCaptureDelivery,
        approval: &ApprovedRepositoryCaptureDelivery,
    ) -> Result<RepositoryCaptureDeliveryCommitV1, ProjectError> {
        if !approval.delivery_write {
            return Err(ProjectError::ApprovalRequired);
        }
        if approval.expected_plan_digest != plan.preview.plan_digest {
            return Err(ProjectError::PlanMismatch);
        }
        let rebuilt = self.preview_repository_capture_delivery(
            &plan.preview.repository_project_id,
            &plan.capture.capture_id,
            plan.preview.queued_at_unix,
        )?;
        if rebuilt.preview != plan.preview
            || rebuilt.envelope != plan.envelope
            || rebuilt.capture != plan.capture
            || rebuilt.repository_digest != plan.repository_digest
        {
            return Err(ProjectError::RevisionConflict);
        }
        let delivery = self.enqueue_capture_delivery(plan.envelope.clone())?;
        Ok(RepositoryCaptureDeliveryCommitV1 {
            schema_version: REPOSITORY_CAPTURE_DELIVERY_SCHEMA_VERSION,
            repository_project_id: plan.preview.repository_project_id.clone(),
            artifact_change_ids: plan.preview.artifact_change_ids.clone(),
            delivery,
        })
    }
}

fn build_verified_repository_delivery(
    repository_project_id: &ProjectId,
    capture: ResearchCaptureV1,
    repository_digest: String,
    envelope: CaptureDeliveryEnvelopeV1,
    artifact_changes: &ArtifactChangeSnapshotV1,
) -> Result<VerifiedRepositoryCaptureDelivery, ProjectError> {
    if artifact_changes.project_id != *repository_project_id {
        return Err(ProjectError::RevisionConflict);
    }
    let artifact_change_ids = artifact_changes
        .changes
        .iter()
        .map(|change| change.change_id.clone())
        .collect::<Vec<_>>();
    let approvals_required = vec!["delivery-write".to_string()];
    let destination_project_id = envelope
        .destination
        .as_ref()
        .map(|destination| destination.project_id.clone());
    let expected_destination_revision = envelope
        .destination
        .as_ref()
        .map(|destination| destination.expected_project_revision);
    let identity = RepositoryCaptureDeliveryPlanIdentity {
        schema_version: REPOSITORY_CAPTURE_DELIVERY_SCHEMA_VERSION,
        repository_project_id,
        source_capture_id: &envelope.capture_id,
        source_capture_sha256: &envelope.capture_sha256,
        repository_sha256: &repository_digest,
        envelope_id: &envelope.envelope_id,
        destination_project_id: destination_project_id.as_ref(),
        expected_destination_revision,
        queued_at_unix: envelope.created_at_unix,
        artifact_change_ids: &artifact_change_ids,
        unattributed_change_count: artifact_changes.unattributed_count,
        approvals_required: &approvals_required,
    };
    let plan_digest = sha256_bytes(
        &serde_json_canonicalizer::to_vec(&identity)
            .map_err(|_| ProjectError::InvalidDeliveryDocument)?,
    );
    Ok(VerifiedRepositoryCaptureDelivery {
        preview: RepositoryCaptureDeliveryPreviewV1 {
            schema_version: REPOSITORY_CAPTURE_DELIVERY_SCHEMA_VERSION,
            plan_digest,
            repository_project_id: repository_project_id.clone(),
            source_capture_id: envelope.capture_id.clone(),
            source_capture_sha256: envelope.capture_sha256.clone(),
            repository_sha256: repository_digest.clone(),
            envelope_id: envelope.envelope_id.clone(),
            destination_project_id,
            expected_destination_revision,
            queued_at_unix: envelope.created_at_unix,
            artifact_change_ids,
            unattributed_change_count: artifact_changes.unattributed_count,
            approvals_required,
        },
        envelope,
        capture,
        repository_digest,
    })
}

fn validate_repository_delivery(capture: &ResearchCaptureV1) -> Result<(), ProjectError> {
    if capture.delivery != CaptureDelivery::RepositoryBacked {
        return Err(ProjectError::InvalidCaptureDocument);
    }
    Ok(())
}

fn repository_capture_state(
    accepted: bool,
    project_id: &ProjectId,
    lifecycle: ProjectLifecycle,
    current_revision: u64,
    current_stage: ProjectStage,
    capture: &ResearchCaptureV1,
) -> RepositoryCaptureInboxState {
    if accepted {
        RepositoryCaptureInboxState::Accepted
    } else if &capture.binding.project_id != project_id {
        RepositoryCaptureInboxState::Unbound
    } else if lifecycle != ProjectLifecycle::Active
        || capture.binding.base_revision > current_revision
        || (capture.binding.base_revision == current_revision
            && capture.binding.stage != current_stage)
    {
        RepositoryCaptureInboxState::Conflicted
    } else if capture.binding.base_revision < current_revision {
        RepositoryCaptureInboxState::Stale
    } else {
        RepositoryCaptureInboxState::Pending
    }
}

fn count_state(
    entries: &[RepositoryCaptureInboxEntryV1],
    state: RepositoryCaptureInboxState,
) -> usize {
    entries.iter().filter(|entry| entry.state == state).count()
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    use qiongli_config::resolve_config_root;

    use super::*;
    use crate::{
        ApprovedProjectMutation, CaptureArea, CapturePolicy, EvidenceLocatorKind,
        EvidenceReferenceV1, ProjectBindingV1, ProjectKind, ProjectRegistrationOptions,
        ResearchCaptureDraftV1, SemanticChangeV1,
    };

    static NEXT_FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

    struct Fixture {
        root: PathBuf,
        project_root: PathBuf,
        service: ProjectStateService,
        project_id: ProjectId,
    }

    impl Fixture {
        fn new() -> Self {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let root = std::env::temp_dir().join(format!(
                "qiongli-repository-inbox-{}-{nonce}-{}",
                std::process::id(),
                NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&root).unwrap();
            set_private_directory_mode(&root);
            let root = fs::canonicalize(root).unwrap();
            let home = root.join("home");
            fs::create_dir(&home).unwrap();
            set_private_directory_mode(&home);
            let projects = root.join("projects");
            fs::create_dir(&projects).unwrap();
            let config = resolve_config_root(None, &home).unwrap();
            let service = ProjectStateService::new(config);
            let project_root = projects.join("article");
            let create = service
                .preview_create(
                    &project_root,
                    ProjectRegistrationOptions::new("Repository article", ProjectKind::Article),
                    1,
                )
                .unwrap();
            service
                .apply(
                    &create,
                    &ApprovedProjectMutation::new(create.preview().plan_digest.clone(), true),
                    1,
                )
                .unwrap();
            let project_id = create.preview().project_id.clone();
            Self {
                root,
                project_root,
                service,
                project_id,
            }
        }

        fn capture(&self, summary: &str, captured_at_unix: u64) -> ResearchCaptureV1 {
            ResearchCaptureDraftV1 {
                binding: ProjectBindingV1::new(
                    self.project_id.clone(),
                    1,
                    ProjectStage::Idea,
                    "Synchronize the article argument through the repository",
                    CapturePolicy::ReviewRequired,
                )
                .unwrap(),
                source: CaptureSource::Repository,
                delivery: CaptureDelivery::RepositoryBacked,
                captured_at_unix,
                summary: summary.to_string(),
                changes: vec![SemanticChangeV1 {
                    area: CaptureArea::Thesis,
                    summary: "Keep research meaning separate from client sessions.".to_string(),
                }],
                decisions: vec![],
                evidence: vec![EvidenceReferenceV1 {
                    locator_kind: EvidenceLocatorKind::Doi,
                    locator: "10.1000/repository-inbox".to_string(),
                    relevance: "Anchors the repository delivery fixture.".to_string(),
                    limitation: None,
                }],
                contradictions: vec![],
                next_actions: vec!["Review before academic consolidation.".to_string()],
            }
            .into_capture()
            .unwrap()
        }

        fn write_inbox(&self, capture: &ResearchCaptureV1) -> PathBuf {
            let inbox = self.project_root.join("context/capture-inbox");
            fs::create_dir_all(&inbox).unwrap();
            let path = inbox.join(format!("{}.json", capture.capture_id.as_str()));
            fs::write(&path, capture.to_canonical_json().unwrap()).unwrap();
            path
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    #[cfg(unix)]
    fn set_private_directory_mode(path: &Path) {
        use std::os::unix::fs::PermissionsExt;

        fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
    }

    #[cfg(not(unix))]
    fn set_private_directory_mode(_path: &Path) {}

    #[test]
    fn repository_inbox_previews_applies_acknowledges_and_rejects_replay() {
        let fixture = Fixture::new();
        let capture = fixture.capture("Repository-backed article memory", 10);
        fixture.write_inbox(&capture);

        let inbox = fixture
            .service
            .repository_capture_inbox(&fixture.project_id)
            .unwrap();
        assert_eq!(inbox.pending_count, 1);
        assert_eq!(inbox.accepted_count, 0);
        assert_eq!(inbox.entries[0].state, RepositoryCaptureInboxState::Pending);
        assert_eq!(
            inbox.entries[0].repository_entry,
            repository_capture_inbox_relative_path(&capture.capture_id)
        );
        assert_eq!(inbox.entries[0].history_entry, None);

        let plan = fixture
            .service
            .preview_repository_capture(&fixture.project_id, &capture.capture_id)
            .unwrap();
        assert_eq!(plan.preview().intake.project_id, fixture.project_id);
        assert_eq!(
            fixture.service.apply_repository_capture(
                &plan,
                &ApprovedCaptureIntake::new(plan.preview().intake.plan_digest.clone(), false),
                11,
            ),
            Err(ProjectError::ApprovalRequired)
        );
        let commit = fixture
            .service
            .apply_repository_capture(
                &plan,
                &ApprovedCaptureIntake::new(plan.preview().intake.plan_digest.clone(), true),
                11,
            )
            .unwrap();
        assert_eq!(commit.capture_id, capture.capture_id);
        assert!(commit.acknowledgement.starts_with("ack_"));

        let accepted = fixture
            .service
            .repository_capture_inbox(&fixture.project_id)
            .unwrap();
        assert_eq!(accepted.pending_count, 0);
        assert_eq!(accepted.accepted_count, 1);
        assert_eq!(
            accepted.entries[0].state,
            RepositoryCaptureInboxState::Accepted
        );
        let expected_history = crate::storage::capture_history_relative_path(&capture.capture_id);
        assert_eq!(
            accepted.entries[0].history_entry.as_deref(),
            Some(expected_history.as_str())
        );
        let replay = fixture
            .service
            .preview_repository_capture(&fixture.project_id, &capture.capture_id)
            .unwrap();
        assert_eq!(
            fixture.service.apply_repository_capture(
                &replay,
                &ApprovedCaptureIntake::new(replay.preview().intake.plan_digest.clone(), true),
                12,
            ),
            Err(ProjectError::CaptureAlreadyApplied)
        );
    }

    #[test]
    fn repository_inbox_rejects_source_drift_and_projects_unbound_packets() {
        let fixture = Fixture::new();
        let capture = fixture.capture("Previewed repository memory", 20);
        let path = fixture.write_inbox(&capture);
        let plan = fixture
            .service
            .preview_repository_capture(&fixture.project_id, &capture.capture_id)
            .unwrap();
        let mut drifted = capture.to_canonical_json().unwrap();
        drifted.push(b'\n');
        fs::write(&path, drifted).unwrap();
        assert_eq!(
            fixture.service.apply_repository_capture(
                &plan,
                &ApprovedCaptureIntake::new(plan.preview().intake.plan_digest.clone(), true),
                21,
            ),
            Err(ProjectError::RevisionConflict)
        );

        fs::remove_file(path).unwrap();
        let other_project =
            ProjectId::parse("prj_0123456789abcdef0123456789abcdef".to_string()).unwrap();
        let unbound = ResearchCaptureDraftV1 {
            binding: ProjectBindingV1::new(
                other_project,
                1,
                ProjectStage::Idea,
                "Inspect an unbound repository packet",
                CapturePolicy::ReviewRequired,
            )
            .unwrap(),
            source: CaptureSource::Repository,
            delivery: CaptureDelivery::RepositoryBacked,
            captured_at_unix: 22,
            summary: "Packet belongs to another article project.".to_string(),
            changes: vec![],
            decisions: vec![],
            evidence: vec![],
            contradictions: vec![],
            next_actions: vec![],
        }
        .into_capture()
        .unwrap();
        fixture.write_inbox(&unbound);
        let inbox = fixture
            .service
            .repository_capture_inbox(&fixture.project_id)
            .unwrap();
        assert_eq!(inbox.unbound_count, 1);
        assert_eq!(inbox.entries[0].state, RepositoryCaptureInboxState::Unbound);
        assert!(matches!(
            fixture
                .service
                .preview_repository_capture(&fixture.project_id, &unbound.capture_id),
            Err(ProjectError::CaptureIdentityConflict)
        ));
    }

    #[test]
    fn repository_delivery_binds_unattributed_drift_and_replays_exactly() {
        let fixture = Fixture::new();
        let capture = fixture.capture("Route repository work through delivery", 30);
        fixture.write_inbox(&capture);
        fs::write(
            fixture.project_root.join("context/research_state.md"),
            "RQ: Which repository change entered the delivery ledger?\n",
        )
        .unwrap();

        let plan = fixture
            .service
            .preview_repository_capture_delivery(&fixture.project_id, &capture.capture_id, 31)
            .unwrap();
        assert_eq!(plan.preview().repository_project_id, fixture.project_id);
        assert_eq!(
            plan.preview().destination_project_id.as_ref(),
            Some(&fixture.project_id)
        );
        assert_eq!(plan.preview().expected_destination_revision, Some(1));
        assert_eq!(plan.preview().unattributed_change_count, 1);
        assert_eq!(plan.preview().artifact_change_ids.len(), 1);
        assert_eq!(
            fixture.service.apply_repository_capture_delivery(
                &plan,
                &ApprovedRepositoryCaptureDelivery::new(plan.preview().plan_digest.clone(), false,),
            ),
            Err(ProjectError::ApprovalRequired)
        );
        let commit = fixture
            .service
            .apply_repository_capture_delivery(
                &plan,
                &ApprovedRepositoryCaptureDelivery::new(plan.preview().plan_digest.clone(), true),
            )
            .unwrap();
        assert_eq!(commit.delivery.envelope_id, plan.preview().envelope_id);
        assert_eq!(commit.delivery.state, crate::CaptureDeliveryState::Queued);
        assert_eq!(
            commit.artifact_change_ids,
            plan.preview().artifact_change_ids
        );

        let replay = fixture
            .service
            .apply_repository_capture_delivery(
                &plan,
                &ApprovedRepositoryCaptureDelivery::new(plan.preview().plan_digest.clone(), true),
            )
            .unwrap();
        assert_eq!(replay, commit);
    }

    #[test]
    fn repository_delivery_rejects_source_drift_and_keeps_unbound_destination_empty() {
        let fixture = Fixture::new();
        let capture = fixture.capture("Bind the raw repository source", 40);
        let path = fixture.write_inbox(&capture);
        let plan = fixture
            .service
            .preview_repository_capture_delivery(&fixture.project_id, &capture.capture_id, 41)
            .unwrap();
        let mut changed_bytes = capture.to_canonical_json().unwrap();
        changed_bytes.push(b'\n');
        fs::write(path, changed_bytes).unwrap();
        assert_eq!(
            fixture.service.apply_repository_capture_delivery(
                &plan,
                &ApprovedRepositoryCaptureDelivery::new(plan.preview().plan_digest.clone(), true,),
            ),
            Err(ProjectError::RevisionConflict)
        );

        let other_project =
            ProjectId::parse("prj_0123456789abcdef0123456789abcdef".to_string()).unwrap();
        let unbound = ResearchCaptureDraftV1 {
            binding: ProjectBindingV1::new(
                other_project,
                1,
                ProjectStage::Idea,
                "Route an unbound repository capture",
                CapturePolicy::ReviewRequired,
            )
            .unwrap(),
            source: CaptureSource::Repository,
            delivery: CaptureDelivery::RepositoryBacked,
            captured_at_unix: 42,
            summary: "The destination must remain unresolved until assignment.".to_string(),
            changes: vec![],
            decisions: vec![],
            evidence: vec![],
            contradictions: vec![],
            next_actions: vec![],
        }
        .into_capture()
        .unwrap();
        fixture.write_inbox(&unbound);
        let unbound_plan = fixture
            .service
            .preview_repository_capture_delivery(&fixture.project_id, &unbound.capture_id, 43)
            .unwrap();
        assert_eq!(unbound_plan.preview().destination_project_id, None);
        assert_eq!(unbound_plan.preview().expected_destination_revision, None);
    }
}
