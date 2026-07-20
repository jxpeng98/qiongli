use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::model::{ProjectId, ProjectStage};
use crate::storage::{
    SEMANTIC_ARTIFACTS, empty_semantic_digest, project_root_from_string, read_manifest,
    read_semantic_artifact, semantic_digest,
};
use crate::{ProjectError, ProjectStateService};

pub const ARTIFACT_CHANGE_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArtifactChangeState {
    Current,
    Unattributed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArtifactChangeDetection {
    Exact,
    Aggregate,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArtifactChangeEffect {
    Created,
    ChangedSet,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArtifactChangeReason {
    NoAcceptedCaptureLineage,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RegisteredArtifact {
    ResearchState,
    DecisionLog,
    StageHandoff,
    BoundaryReview,
    IdeaFunnel,
    LiteratureMap,
    ClaimEvidenceLedger,
    ManuscriptClaimMap,
}

impl RegisteredArtifact {
    fn from_relative_path(relative_path: &str) -> Self {
        match relative_path.as_bytes() {
            b"context/research_state.md" => Self::ResearchState,
            b"context/decision_log.md" => Self::DecisionLog,
            b"context/stage_handoff.md" => Self::StageHandoff,
            b"context/boundary_review.md" => Self::BoundaryReview,
            b"context/idea_funnel.md" => Self::IdeaFunnel,
            b"literature/literature_map.md" => Self::LiteratureMap,
            b"evidence/claim-evidence-ledger.csv" => Self::ClaimEvidenceLedger,
            b"manuscript/claims_evidence_map.md" => Self::ManuscriptClaimMap,
            _ => unreachable!("registered semantic artifact inventory must remain closed"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisteredArtifactObservationV1 {
    pub artifact: RegisteredArtifact,
    pub relative_path: String,
    pub present: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisteredArtifactChangeV1 {
    pub change_id: String,
    pub state: ArtifactChangeState,
    pub detection: ArtifactChangeDetection,
    pub effect: ArtifactChangeEffect,
    pub base_revision: u64,
    pub relative_paths: Vec<String>,
    pub reason: ArtifactChangeReason,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactChangeSnapshotV1 {
    pub schema_version: u32,
    pub project_id: ProjectId,
    pub project_revision: u64,
    pub project_stage: ProjectStage,
    pub state: ArtifactChangeState,
    pub registered_artifact_count: usize,
    pub present_artifact_count: usize,
    pub change_count: usize,
    pub unattributed_count: usize,
    pub changes: Vec<RegisteredArtifactChangeV1>,
    pub artifacts: Vec<RegisteredArtifactObservationV1>,
}

impl ProjectStateService {
    pub fn artifact_changes(
        &self,
        project_id: &ProjectId,
    ) -> Result<ArtifactChangeSnapshotV1, ProjectError> {
        let library = self.store.load()?;
        library.validate()?;
        let entry = library
            .projects
            .iter()
            .find(|entry| &entry.project_id == project_id)
            .ok_or(ProjectError::ProjectNotRegistered)?;
        let root = project_root_from_string(&entry.root_path)?;
        let (manifest, _) = read_manifest(&root)?.ok_or(ProjectError::ProjectManifestMissing)?;
        if manifest.project_id != *project_id
            || entry.project_id != manifest.project_id
            || entry.semantic_revision != manifest.semantic_revision
            || entry.semantic_digest != manifest.semantic_digest
            || entry.stage != manifest.stage
            || entry.lifecycle != manifest.lifecycle
        {
            return Err(ProjectError::RevisionConflict);
        }

        let artifacts = SEMANTIC_ARTIFACTS
            .iter()
            .map(|relative_path| {
                read_semantic_artifact(&root, relative_path).map(|observed| {
                    RegisteredArtifactObservationV1 {
                        artifact: RegisteredArtifact::from_relative_path(relative_path),
                        relative_path: (*relative_path).to_string(),
                        present: observed.is_some(),
                    }
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let observed_digest = semantic_digest(&root)?;
        let changed = observed_digest != manifest.semantic_digest;
        let changes = if changed {
            let exact = manifest.semantic_digest == empty_semantic_digest();
            let relative_paths = if exact {
                artifacts
                    .iter()
                    .filter(|artifact| artifact.present)
                    .map(|artifact| artifact.relative_path.clone())
                    .collect()
            } else {
                Vec::new()
            };
            vec![RegisteredArtifactChangeV1 {
                change_id: change_id(
                    project_id,
                    manifest.semantic_revision,
                    &manifest.semantic_digest,
                    &observed_digest,
                ),
                state: ArtifactChangeState::Unattributed,
                detection: if exact {
                    ArtifactChangeDetection::Exact
                } else {
                    ArtifactChangeDetection::Aggregate
                },
                effect: if exact {
                    ArtifactChangeEffect::Created
                } else {
                    ArtifactChangeEffect::ChangedSet
                },
                base_revision: manifest.semantic_revision,
                relative_paths,
                reason: ArtifactChangeReason::NoAcceptedCaptureLineage,
            }]
        } else {
            Vec::new()
        };

        Ok(ArtifactChangeSnapshotV1 {
            schema_version: ARTIFACT_CHANGE_SCHEMA_VERSION,
            project_id: project_id.clone(),
            project_revision: manifest.semantic_revision,
            project_stage: manifest.stage,
            state: if changed {
                ArtifactChangeState::Unattributed
            } else {
                ArtifactChangeState::Current
            },
            registered_artifact_count: artifacts.len(),
            present_artifact_count: artifacts.iter().filter(|artifact| artifact.present).count(),
            change_count: changes.len(),
            unattributed_count: changes.len(),
            changes,
            artifacts,
        })
    }
}

fn change_id(
    project_id: &ProjectId,
    project_revision: u64,
    expected_digest: &str,
    observed_digest: &str,
) -> String {
    let mut digest = Sha256::new();
    digest.update(b"qiongli-artifact-change-v1\0");
    digest.update(project_id.as_str().as_bytes());
    digest.update([0]);
    digest.update(project_revision.to_be_bytes());
    digest.update(expected_digest.as_bytes());
    digest.update(observed_digest.as_bytes());
    format!("chg_{:x}", digest.finalize())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    use qiongli_config::resolve_config_root;

    use super::*;
    use crate::{
        ApprovedProjectMutation, ProjectKind, ProjectRegistrationOptions, ProjectStateService,
    };

    static NEXT_FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

    struct Fixture {
        root: PathBuf,
        service: ProjectStateService,
    }

    impl Fixture {
        fn new() -> Self {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let root = std::env::temp_dir().join(format!(
                "qiongli-artifact-changes-{}-{nonce}-{}",
                std::process::id(),
                NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&root).unwrap();
            let root = fs::canonicalize(root).unwrap();
            let home = root.join("home");
            fs::create_dir(&home).unwrap();
            let config = resolve_config_root(Some(root.join("config").as_os_str()), &home).unwrap();
            Self {
                root,
                service: ProjectStateService::new(config),
            }
        }

        fn create_project(&self) -> (PathBuf, ProjectId) {
            let project_root = self.root.join("paper");
            let plan = self
                .service
                .preview_create(
                    &project_root,
                    ProjectRegistrationOptions::new("Paper", ProjectKind::Article),
                    1,
                )
                .unwrap();
            let project_id = plan.preview().project_id.clone();
            self.service
                .apply(
                    &plan,
                    &ApprovedProjectMutation::new(plan.preview().plan_digest.clone(), true),
                    1,
                )
                .unwrap();
            (project_root, project_id)
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn detects_an_exact_unattributed_change_from_the_empty_registered_baseline() {
        let fixture = Fixture::new();
        let (root, project_id) = fixture.create_project();
        let current = fixture.service.artifact_changes(&project_id).unwrap();
        assert_eq!(current.state, ArtifactChangeState::Current);
        assert!(current.changes.is_empty());
        assert_eq!(current.artifacts.len(), 8);

        fs::write(
            root.join("context/research_state.md"),
            "RQ: Which registered artifact changed?\n",
        )
        .unwrap();
        let changed = fixture.service.artifact_changes(&project_id).unwrap();
        assert_eq!(changed.state, ArtifactChangeState::Unattributed);
        assert_eq!(changed.change_count, 1);
        assert_eq!(changed.unattributed_count, 1);
        assert_eq!(changed.changes[0].detection, ArtifactChangeDetection::Exact);
        assert_eq!(changed.changes[0].effect, ArtifactChangeEffect::Created);
        assert_eq!(
            changed.changes[0].relative_paths,
            ["context/research_state.md"]
        );
        let rendered = serde_json::to_string(&changed).unwrap();
        assert!(!rendered.contains("session"));
        assert!(!rendered.contains(root.to_string_lossy().as_ref()));
    }

    #[test]
    fn reports_only_aggregate_drift_when_the_prior_file_baseline_is_not_known() {
        let fixture = Fixture::new();
        let (root, project_id) = fixture.create_project();
        fs::write(root.join("context/research_state.md"), "Thesis: First\n").unwrap();
        let refresh = fixture.service.preview_refresh(&project_id, 2).unwrap();
        fixture
            .service
            .apply(
                &refresh,
                &ApprovedProjectMutation::new(refresh.preview().plan_digest.clone(), true),
                2,
            )
            .unwrap();
        fs::write(root.join("context/research_state.md"), "Thesis: Second\n").unwrap();

        let changed = fixture.service.artifact_changes(&project_id).unwrap();
        assert_eq!(changed.state, ArtifactChangeState::Unattributed);
        assert_eq!(
            changed.changes[0].detection,
            ArtifactChangeDetection::Aggregate
        );
        assert_eq!(changed.changes[0].effect, ArtifactChangeEffect::ChangedSet);
        assert!(changed.changes[0].relative_paths.is_empty());
    }
}
