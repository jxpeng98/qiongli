use serde::Serialize;

use crate::{
    CaptureDelivery, CaptureId, CaptureInboxEntryV1, CaptureInboxSnapshotV1, CaptureInboxState,
    CaptureSource, ProjectError, ProjectId, ProjectStage, ProjectStateService,
    RepositoryCaptureInboxEntryV1, RepositoryCaptureInboxSnapshotV1, RepositoryCaptureInboxState,
};

pub const CAPTURE_COVERAGE_SCHEMA_VERSION: u32 = 1;

const CAPTURE_SOURCES: [CaptureSource; 7] = [
    CaptureSource::Codex,
    CaptureSource::ClaudeCode,
    CaptureSource::ChatGpt,
    CaptureSource::Cli,
    CaptureSource::Manual,
    CaptureSource::Repository,
    CaptureSource::PortableFile,
];

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CaptureCoverageState {
    PendingReview,
    Current,
    Stale,
    Conflicted,
    Unbound,
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CaptureCoverageDelivery {
    Connected,
    RepositoryBacked,
    Portable,
    Manual,
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureSourceCoverageV1 {
    pub source: CaptureSource,
    pub state: CaptureCoverageState,
    pub delivery: CaptureCoverageDelivery,
    pub capture_count: usize,
    pub pending_review_count: usize,
    pub current_count: usize,
    pub stale_count: usize,
    pub conflicted_count: usize,
    pub unbound_count: usize,
    pub latest_capture_id: Option<CaptureId>,
    pub last_captured_at_unix: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureCoverageSnapshotV1 {
    pub schema_version: u32,
    pub project_id: ProjectId,
    pub project_revision: u64,
    pub project_stage: ProjectStage,
    pub capture_count: usize,
    pub connected_count: usize,
    pub repository_backed_count: usize,
    pub portable_count: usize,
    pub manual_count: usize,
    pub pending_review_count: usize,
    pub current_count: usize,
    pub stale_count: usize,
    pub conflicted_count: usize,
    pub unbound_count: usize,
    pub unknown_source_count: usize,
    pub sources: Vec<CaptureSourceCoverageV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ObservedCapture {
    capture_id: CaptureId,
    source: CaptureSource,
    delivery: CaptureCoverageDelivery,
    state: CaptureCoverageState,
    captured_at_unix: u64,
}

impl ProjectStateService {
    pub fn capture_coverage(
        &self,
        project_id: &ProjectId,
    ) -> Result<CaptureCoverageSnapshotV1, ProjectError> {
        let inbox = self.capture_inbox(project_id)?;
        let repository = self.repository_capture_inbox(project_id)?;
        build_capture_coverage(inbox, repository)
    }
}

fn build_capture_coverage(
    inbox: CaptureInboxSnapshotV1,
    repository: RepositoryCaptureInboxSnapshotV1,
) -> Result<CaptureCoverageSnapshotV1, ProjectError> {
    if inbox.project_id != repository.project_id
        || inbox.project_revision != repository.project_revision
        || inbox.project_stage != repository.project_stage
    {
        return Err(ProjectError::RevisionConflict);
    }

    let mut observations = inbox
        .entries
        .iter()
        .map(observation_from_history)
        .collect::<Vec<_>>();
    observations.extend(
        repository
            .entries
            .iter()
            .filter(|entry| entry.state != RepositoryCaptureInboxState::Accepted)
            .map(observation_from_repository),
    );
    observations.sort_by(|left, right| {
        right
            .captured_at_unix
            .cmp(&left.captured_at_unix)
            .then_with(|| left.capture_id.cmp(&right.capture_id))
    });

    let sources = CAPTURE_SOURCES
        .into_iter()
        .map(|source| source_coverage(source, &observations))
        .collect::<Vec<_>>();

    Ok(CaptureCoverageSnapshotV1 {
        schema_version: CAPTURE_COVERAGE_SCHEMA_VERSION,
        project_id: inbox.project_id,
        project_revision: inbox.project_revision,
        project_stage: inbox.project_stage,
        capture_count: observations.len(),
        connected_count: count_delivery(&observations, CaptureCoverageDelivery::Connected),
        repository_backed_count: count_delivery(
            &observations,
            CaptureCoverageDelivery::RepositoryBacked,
        ),
        portable_count: count_delivery(&observations, CaptureCoverageDelivery::Portable),
        manual_count: count_delivery(&observations, CaptureCoverageDelivery::Manual),
        pending_review_count: count_state(&observations, CaptureCoverageState::PendingReview),
        current_count: count_state(&observations, CaptureCoverageState::Current),
        stale_count: count_state(&observations, CaptureCoverageState::Stale),
        conflicted_count: count_state(&observations, CaptureCoverageState::Conflicted),
        unbound_count: count_state(&observations, CaptureCoverageState::Unbound),
        unknown_source_count: sources
            .iter()
            .filter(|source| source.state == CaptureCoverageState::Unknown)
            .count(),
        sources,
    })
}

fn observation_from_history(entry: &CaptureInboxEntryV1) -> ObservedCapture {
    ObservedCapture {
        capture_id: entry.capture_id.clone(),
        source: entry.source,
        delivery: delivery_state(entry.delivery),
        state: match entry.state {
            CaptureInboxState::PendingReview => CaptureCoverageState::PendingReview,
            CaptureInboxState::Stale => CaptureCoverageState::Stale,
            CaptureInboxState::Conflicted => CaptureCoverageState::Conflicted,
            CaptureInboxState::Applied => CaptureCoverageState::Current,
        },
        captured_at_unix: entry.captured_at_unix,
    }
}

fn observation_from_repository(entry: &RepositoryCaptureInboxEntryV1) -> ObservedCapture {
    ObservedCapture {
        capture_id: entry.capture_id.clone(),
        source: entry.source,
        delivery: CaptureCoverageDelivery::RepositoryBacked,
        state: match entry.state {
            RepositoryCaptureInboxState::Pending => CaptureCoverageState::PendingReview,
            RepositoryCaptureInboxState::Accepted => CaptureCoverageState::Current,
            RepositoryCaptureInboxState::Stale => CaptureCoverageState::Stale,
            RepositoryCaptureInboxState::Conflicted => CaptureCoverageState::Conflicted,
            RepositoryCaptureInboxState::Unbound => CaptureCoverageState::Unbound,
        },
        captured_at_unix: entry.captured_at_unix,
    }
}

const fn delivery_state(delivery: CaptureDelivery) -> CaptureCoverageDelivery {
    match delivery {
        CaptureDelivery::Connected => CaptureCoverageDelivery::Connected,
        CaptureDelivery::RepositoryBacked => CaptureCoverageDelivery::RepositoryBacked,
        CaptureDelivery::Portable => CaptureCoverageDelivery::Portable,
        CaptureDelivery::Manual => CaptureCoverageDelivery::Manual,
    }
}

fn source_coverage(
    source: CaptureSource,
    observations: &[ObservedCapture],
) -> CaptureSourceCoverageV1 {
    let matching = observations
        .iter()
        .filter(|observation| observation.source == source)
        .collect::<Vec<_>>();
    let latest = matching.first().copied();
    let state = matching
        .iter()
        .map(|observation| observation.state)
        .max_by_key(|state| coverage_priority(*state))
        .unwrap_or(CaptureCoverageState::Unknown);
    CaptureSourceCoverageV1 {
        source,
        state,
        delivery: latest
            .map(|observation| observation.delivery)
            .unwrap_or(CaptureCoverageDelivery::Unknown),
        capture_count: matching.len(),
        pending_review_count: count_matching_state(&matching, CaptureCoverageState::PendingReview),
        current_count: count_matching_state(&matching, CaptureCoverageState::Current),
        stale_count: count_matching_state(&matching, CaptureCoverageState::Stale),
        conflicted_count: count_matching_state(&matching, CaptureCoverageState::Conflicted),
        unbound_count: count_matching_state(&matching, CaptureCoverageState::Unbound),
        latest_capture_id: latest.map(|observation| observation.capture_id.clone()),
        last_captured_at_unix: latest.map(|observation| observation.captured_at_unix),
    }
}

const fn coverage_priority(state: CaptureCoverageState) -> u8 {
    match state {
        CaptureCoverageState::Unknown => 0,
        CaptureCoverageState::Current => 1,
        CaptureCoverageState::PendingReview => 2,
        CaptureCoverageState::Stale => 3,
        CaptureCoverageState::Unbound => 4,
        CaptureCoverageState::Conflicted => 5,
    }
}

fn count_delivery(observations: &[ObservedCapture], delivery: CaptureCoverageDelivery) -> usize {
    observations
        .iter()
        .filter(|observation| observation.delivery == delivery)
        .count()
}

fn count_state(observations: &[ObservedCapture], state: CaptureCoverageState) -> usize {
    observations
        .iter()
        .filter(|observation| observation.state == state)
        .count()
}

fn count_matching_state(observations: &[&ObservedCapture], state: CaptureCoverageState) -> usize {
    observations
        .iter()
        .filter(|observation| observation.state == state)
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CaptureDisposition, CapturePolicy, ProjectId, RepositoryCaptureInboxState};

    fn project_id() -> ProjectId {
        ProjectId::parse("prj_0123456789abcdef0123456789abcdef").unwrap()
    }

    fn capture_id(character: char) -> CaptureId {
        CaptureId::parse(format!("cap_{}", character.to_string().repeat(64))).unwrap()
    }

    fn history_entry(
        character: char,
        source: CaptureSource,
        delivery: CaptureDelivery,
        state: CaptureInboxState,
        captured_at_unix: u64,
    ) -> CaptureInboxEntryV1 {
        CaptureInboxEntryV1 {
            capture_id: capture_id(character),
            state,
            disposition: CaptureDisposition::UnresolvedCandidate,
            source,
            delivery,
            captured_at_unix,
            base_revision: 1,
            bound_stage: ProjectStage::Writing,
            task: "Retain the article argument".to_string(),
            capture_policy: CapturePolicy::ReviewRequired,
            summary: "A bounded capture summary.".to_string(),
            change_count: 0,
            decision_count: 0,
            evidence_count: 0,
            contradiction_count: 0,
            next_action_count: 1,
            history_entry: format!(
                "context/captures/cap_{}.json",
                character.to_string().repeat(64)
            ),
        }
    }

    fn repository_entry(
        character: char,
        source: CaptureSource,
        state: RepositoryCaptureInboxState,
        captured_at_unix: u64,
    ) -> RepositoryCaptureInboxEntryV1 {
        RepositoryCaptureInboxEntryV1 {
            capture_id: capture_id(character),
            project_id: project_id(),
            state,
            disposition: CaptureDisposition::UnresolvedCandidate,
            source,
            captured_at_unix,
            base_revision: 1,
            bound_stage: ProjectStage::Writing,
            task: "Retain the repository argument".to_string(),
            capture_policy: CapturePolicy::ReviewRequired,
            summary: "A bounded repository capture summary.".to_string(),
            change_count: 0,
            decision_count: 0,
            evidence_count: 0,
            contradiction_count: 0,
            next_action_count: 1,
            repository_entry: format!(
                "context/capture-inbox/cap_{}.json",
                character.to_string().repeat(64)
            ),
            history_entry: None,
        }
    }

    #[test]
    fn coverage_unifies_delivery_review_and_unknown_source_states() {
        let project_id = project_id();
        let inbox = CaptureInboxSnapshotV1 {
            schema_version: 1,
            project_id: project_id.clone(),
            project_revision: 2,
            project_stage: ProjectStage::Writing,
            pending_review_count: 1,
            stale_count: 0,
            conflicted_count: 0,
            applied_count: 1,
            entries: vec![
                history_entry(
                    'a',
                    CaptureSource::Codex,
                    CaptureDelivery::Connected,
                    CaptureInboxState::PendingReview,
                    20,
                ),
                history_entry(
                    'b',
                    CaptureSource::PortableFile,
                    CaptureDelivery::Portable,
                    CaptureInboxState::Applied,
                    10,
                ),
            ],
        };
        let repository = RepositoryCaptureInboxSnapshotV1 {
            schema_version: 1,
            project_id: project_id.clone(),
            project_revision: 2,
            project_stage: ProjectStage::Writing,
            pending_count: 0,
            accepted_count: 0,
            stale_count: 0,
            conflicted_count: 0,
            unbound_count: 1,
            entries: vec![repository_entry(
                'c',
                CaptureSource::ClaudeCode,
                RepositoryCaptureInboxState::Unbound,
                30,
            )],
        };

        let coverage = build_capture_coverage(inbox, repository).unwrap();
        assert_eq!(coverage.project_id, project_id);
        assert_eq!(coverage.capture_count, 3);
        assert_eq!(coverage.connected_count, 1);
        assert_eq!(coverage.repository_backed_count, 1);
        assert_eq!(coverage.portable_count, 1);
        assert_eq!(coverage.pending_review_count, 1);
        assert_eq!(coverage.current_count, 1);
        assert_eq!(coverage.unbound_count, 1);
        assert_eq!(coverage.unknown_source_count, 4);
        assert_eq!(coverage.sources.len(), CAPTURE_SOURCES.len());
        assert_eq!(coverage.sources[0].source, CaptureSource::Codex);
        assert_eq!(
            coverage.sources[0].state,
            CaptureCoverageState::PendingReview
        );
        assert_eq!(
            coverage.sources[1].delivery,
            CaptureCoverageDelivery::RepositoryBacked
        );
        assert_eq!(coverage.sources[1].state, CaptureCoverageState::Unbound);
        assert_eq!(coverage.sources[2].state, CaptureCoverageState::Unknown);
        assert_eq!(
            coverage.sources[6].delivery,
            CaptureCoverageDelivery::Portable
        );
        assert_eq!(coverage.sources[6].state, CaptureCoverageState::Current);
    }

    #[test]
    fn accepted_repository_packets_are_counted_once_from_history() {
        let project_id = project_id();
        let history = history_entry(
            'd',
            CaptureSource::Repository,
            CaptureDelivery::RepositoryBacked,
            CaptureInboxState::PendingReview,
            40,
        );
        let mut accepted = repository_entry(
            'd',
            CaptureSource::Repository,
            RepositoryCaptureInboxState::Accepted,
            40,
        );
        accepted.history_entry = Some(history.history_entry.clone());
        let coverage = build_capture_coverage(
            CaptureInboxSnapshotV1 {
                schema_version: 1,
                project_id: project_id.clone(),
                project_revision: 1,
                project_stage: ProjectStage::Writing,
                pending_review_count: 1,
                stale_count: 0,
                conflicted_count: 0,
                applied_count: 0,
                entries: vec![history],
            },
            RepositoryCaptureInboxSnapshotV1 {
                schema_version: 1,
                project_id,
                project_revision: 1,
                project_stage: ProjectStage::Writing,
                pending_count: 0,
                accepted_count: 1,
                stale_count: 0,
                conflicted_count: 0,
                unbound_count: 0,
                entries: vec![accepted],
            },
        )
        .unwrap();

        assert_eq!(coverage.capture_count, 1);
        assert_eq!(coverage.repository_backed_count, 1);
        assert_eq!(coverage.pending_review_count, 1);
    }
}
