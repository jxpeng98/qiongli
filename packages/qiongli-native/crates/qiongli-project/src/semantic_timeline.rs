use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::consolidation::read_consolidation_receipt;
use crate::json::parse_unique_json;
use crate::model::MAX_SEMANTIC_REVISION;
use crate::{
    CaptureAssignmentOutcome, CaptureDelivery, CaptureDeliveryReason, CaptureDeliveryState,
    CaptureResolutionDisposition, CaptureResolutionItemKind, CaptureSource,
    IncrementalPortfolioService, PortfolioContributionV1, ProjectError, ProjectId,
    ProjectLifecycle, ProjectStateService,
};

pub const SEMANTIC_TIMELINE_SCHEMA_VERSION: u32 = 1;
pub const SEMANTIC_TIMELINE_DOCUMENT_KIND: &str = "qiongli-semantic-timeline-query";
pub const SEMANTIC_TIMELINE_RESULT_DOCUMENT_KIND: &str = "qiongli-semantic-timeline-result";
const MAX_TIMELINE_QUERY_BYTES: usize = 32 * 1024;
const MIN_TIMELINE_RESULT_BYTES: usize = 64 * 1024;
const MAX_TIMELINE_RESULT_BYTES: usize = 4 * 1024 * 1024;
const MAX_TIMELINE_PAGE_EVENTS: usize = 512;
const MAX_TIMELINE_SNAPSHOT_EVENTS: usize = 65_536;
const MAX_TIMELINE_SNAPSHOT_BYTES: usize = 64 * 1024 * 1024;
const MAX_RELATED_IDS: usize = 24;
const MAX_RELATED_ID_BYTES: usize = 160;
const TIMELINE_RESULT_RESERVE_BYTES: usize = 16 * 1024;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SemanticTimelineView {
    Activity,
    RevisionHistory,
    MergeResolutionHistory,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SemanticActivityKind {
    ProjectRegistered,
    ProjectRevisionObserved,
    ProjectLifecycleObserved,
    CaptureAccepted,
    CaptureConsolidated,
    DeliveryQueued,
    DeliveryStarted,
    DeliveryDelivered,
    DeliveryAcknowledged,
    DeliveryRetryRequired,
    DeliveryConflicted,
    DeliveryCancelled,
    AssignmentCreated,
    CaptureAssigned,
    CaptureAssignmentRejected,
    ResolutionReviewed,
    ResolutionItemResolved,
    ResolutionCompleted,
}

impl SemanticActivityKind {
    const fn is_revision_history(self) -> bool {
        matches!(
            self,
            Self::ProjectRegistered
                | Self::ProjectRevisionObserved
                | Self::ProjectLifecycleObserved
                | Self::CaptureConsolidated
                | Self::DeliveryAcknowledged
                | Self::ResolutionCompleted
        )
    }

    const fn is_merge_resolution_history(self) -> bool {
        matches!(
            self,
            Self::CaptureConsolidated
                | Self::ResolutionReviewed
                | Self::ResolutionItemResolved
                | Self::ResolutionCompleted
        )
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SemanticActivityTimestampSource {
    ProjectRegisteredAt,
    ProjectAcademicallyUpdatedAt,
    CaptureCapturedAt,
    ConsolidationConsolidatedAt,
    DeliveryTransitionedAt,
    AssignmentCreatedAt,
    AssignmentDecidedAt,
    ResolutionReviewedAt,
    ResolutionResolvedAt,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SemanticActivityV1 {
    pub event_id: String,
    pub kind: SemanticActivityKind,
    pub occurred_at_unix: u64,
    pub timestamp_source: SemanticActivityTimestampSource,
    pub project_ids: Vec<ProjectId>,
    pub related_ids: Vec<String>,
    pub from_project_revision: Option<u64>,
    pub to_project_revision: Option<u64>,
    pub lifecycle: Option<ProjectLifecycle>,
    pub source: Option<CaptureSource>,
    pub delivery: Option<CaptureDelivery>,
    pub delivery_state: Option<CaptureDeliveryState>,
    pub delivery_reason: Option<CaptureDeliveryReason>,
    pub delivery_generation: Option<u64>,
    pub assignment_outcome: Option<CaptureAssignmentOutcome>,
    pub resolution_item_id: Option<String>,
    pub resolution_item_kind: Option<CaptureResolutionItemKind>,
    pub resolution_disposition: Option<CaptureResolutionDisposition>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SemanticActivityIdentity<'a> {
    kind: SemanticActivityKind,
    occurred_at_unix: u64,
    timestamp_source: SemanticActivityTimestampSource,
    project_ids: &'a [ProjectId],
    related_ids: &'a [String],
    from_project_revision: Option<u64>,
    to_project_revision: Option<u64>,
    lifecycle: Option<ProjectLifecycle>,
    source: Option<CaptureSource>,
    delivery: Option<CaptureDelivery>,
    delivery_state: Option<CaptureDeliveryState>,
    delivery_reason: Option<CaptureDeliveryReason>,
    delivery_generation: Option<u64>,
    assignment_outcome: Option<CaptureAssignmentOutcome>,
    resolution_item_id: &'a Option<String>,
    resolution_item_kind: Option<CaptureResolutionItemKind>,
    resolution_disposition: Option<CaptureResolutionDisposition>,
}

#[derive(Default)]
struct SemanticActivityDetails {
    from_project_revision: Option<u64>,
    to_project_revision: Option<u64>,
    lifecycle: Option<ProjectLifecycle>,
    source: Option<CaptureSource>,
    delivery: Option<CaptureDelivery>,
    delivery_state: Option<CaptureDeliveryState>,
    delivery_reason: Option<CaptureDeliveryReason>,
    delivery_generation: Option<u64>,
    assignment_outcome: Option<CaptureAssignmentOutcome>,
    resolution_item_id: Option<String>,
    resolution_item_kind: Option<CaptureResolutionItemKind>,
    resolution_disposition: Option<CaptureResolutionDisposition>,
}

impl SemanticActivityV1 {
    fn new(
        kind: SemanticActivityKind,
        occurred_at_unix: u64,
        timestamp_source: SemanticActivityTimestampSource,
        mut project_ids: Vec<ProjectId>,
        mut related_ids: Vec<String>,
        details: SemanticActivityDetails,
    ) -> Result<Self, ProjectError> {
        project_ids.sort_unstable();
        project_ids.dedup();
        related_ids.sort_unstable();
        related_ids.dedup();
        let mut event = Self {
            event_id: String::new(),
            kind,
            occurred_at_unix,
            timestamp_source,
            project_ids,
            related_ids,
            from_project_revision: details.from_project_revision,
            to_project_revision: details.to_project_revision,
            lifecycle: details.lifecycle,
            source: details.source,
            delivery: details.delivery,
            delivery_state: details.delivery_state,
            delivery_reason: details.delivery_reason,
            delivery_generation: details.delivery_generation,
            assignment_outcome: details.assignment_outcome,
            resolution_item_id: details.resolution_item_id,
            resolution_item_kind: details.resolution_item_kind,
            resolution_disposition: details.resolution_disposition,
        };
        event.event_id = event.identity()?;
        event.validate()?;
        Ok(event)
    }

    fn validate(&self) -> Result<(), ProjectError> {
        if !valid_prefixed_digest(&self.event_id, "pte_")
            || self.occurred_at_unix > MAX_SEMANTIC_REVISION
            || self.project_ids.is_empty()
            || self.project_ids.len() > 2
            || self.project_ids.windows(2).any(|pair| pair[0] >= pair[1])
            || self
                .project_ids
                .iter()
                .any(|project_id| project_id.validate().is_err())
            || self.related_ids.is_empty()
            || self.related_ids.len() > MAX_RELATED_IDS
            || self.related_ids.windows(2).any(|pair| pair[0] >= pair[1])
            || self
                .related_ids
                .iter()
                .any(|value| !valid_text(value, MAX_RELATED_ID_BYTES))
            || self
                .from_project_revision
                .is_some_and(|revision| !valid_revision(revision))
            || self
                .to_project_revision
                .is_some_and(|revision| !valid_revision(revision))
            || matches!(
                (self.from_project_revision, self.to_project_revision),
                (Some(from), Some(to)) if to < from
            )
            || self
                .delivery_generation
                .is_some_and(|generation| generation == 0 || generation > MAX_SEMANTIC_REVISION)
            || self
                .resolution_item_id
                .as_deref()
                .is_some_and(|value| !valid_text(value, MAX_RELATED_ID_BYTES))
            || (self.kind == SemanticActivityKind::ResolutionItemResolved)
                != (self.resolution_item_id.is_some()
                    && self.resolution_item_kind.is_some()
                    && self.resolution_disposition.is_some())
            || self.event_id != self.identity()?
        {
            return Err(ProjectError::InvalidSemanticTimeline);
        }
        Ok(())
    }

    fn identity(&self) -> Result<String, ProjectError> {
        prefixed_digest(
            "pte_",
            b"qiongli-semantic-activity-v1\0",
            &SemanticActivityIdentity {
                kind: self.kind,
                occurred_at_unix: self.occurred_at_unix,
                timestamp_source: self.timestamp_source,
                project_ids: &self.project_ids,
                related_ids: &self.related_ids,
                from_project_revision: self.from_project_revision,
                to_project_revision: self.to_project_revision,
                lifecycle: self.lifecycle,
                source: self.source,
                delivery: self.delivery,
                delivery_state: self.delivery_state,
                delivery_reason: self.delivery_reason,
                delivery_generation: self.delivery_generation,
                assignment_outcome: self.assignment_outcome,
                resolution_item_id: &self.resolution_item_id,
                resolution_item_kind: self.resolution_item_kind,
                resolution_disposition: self.resolution_disposition,
            },
        )
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SemanticTimelineCursorV1 {
    pub cursor_id: String,
    pub query_id: String,
    pub after_occurred_at_unix: u64,
    pub after_event_id: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SemanticTimelineCursorIdentity<'a> {
    query_id: &'a str,
    after_occurred_at_unix: u64,
    after_event_id: &'a str,
}

impl SemanticTimelineCursorV1 {
    fn new(
        query_id: String,
        after_occurred_at_unix: u64,
        after_event_id: String,
    ) -> Result<Self, ProjectError> {
        let mut cursor = Self {
            cursor_id: String::new(),
            query_id,
            after_occurred_at_unix,
            after_event_id,
        };
        cursor.cursor_id = cursor.identity()?;
        cursor.validate()?;
        Ok(cursor)
    }

    fn validate(&self) -> Result<(), ProjectError> {
        if !valid_prefixed_digest(&self.cursor_id, "ptc_")
            || !valid_prefixed_digest(&self.query_id, "pty_")
            || self.after_occurred_at_unix > MAX_SEMANTIC_REVISION
            || !valid_prefixed_digest(&self.after_event_id, "pte_")
            || self.cursor_id != self.identity()?
        {
            return Err(ProjectError::InvalidSemanticTimeline);
        }
        Ok(())
    }

    fn identity(&self) -> Result<String, ProjectError> {
        prefixed_digest(
            "ptc_",
            b"qiongli-semantic-timeline-cursor-v1\0",
            &SemanticTimelineCursorIdentity {
                query_id: &self.query_id,
                after_occurred_at_unix: self.after_occurred_at_unix,
                after_event_id: &self.after_event_id,
            },
        )
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SemanticTimelineQueryV1 {
    pub schema_version: u32,
    pub document_kind: String,
    pub request_id: String,
    pub catalog_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_id: Option<ProjectId>,
    pub view: SemanticTimelineView,
    pub limit: usize,
    pub max_bytes: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursor: Option<SemanticTimelineCursorV1>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SemanticTimelineRequestIdentity<'a> {
    schema_version: u32,
    catalog_id: &'a str,
    project_id: &'a Option<ProjectId>,
    view: SemanticTimelineView,
    limit: usize,
    max_bytes: usize,
}

impl SemanticTimelineQueryV1 {
    pub fn new(catalog_id: impl Into<String>) -> Result<Self, ProjectError> {
        let mut query = Self {
            schema_version: SEMANTIC_TIMELINE_SCHEMA_VERSION,
            document_kind: SEMANTIC_TIMELINE_DOCUMENT_KIND.to_string(),
            request_id: String::new(),
            catalog_id: catalog_id.into(),
            project_id: None,
            view: SemanticTimelineView::Activity,
            limit: 128,
            max_bytes: 2 * 1024 * 1024,
            cursor: None,
        };
        query.request_id = query.identity()?;
        query.validate()?;
        Ok(query)
    }

    pub fn for_project(mut self, project_id: ProjectId) -> Result<Self, ProjectError> {
        self.project_id = Some(project_id);
        self.cursor = None;
        self.request_id = self.identity()?;
        self.validate()?;
        Ok(self)
    }

    pub fn with_view(mut self, view: SemanticTimelineView) -> Result<Self, ProjectError> {
        self.view = view;
        self.cursor = None;
        self.request_id = self.identity()?;
        self.validate()?;
        Ok(self)
    }

    pub fn with_limits(mut self, limit: usize, max_bytes: usize) -> Result<Self, ProjectError> {
        self.limit = limit;
        self.max_bytes = max_bytes;
        self.cursor = None;
        self.request_id = self.identity()?;
        self.validate()?;
        Ok(self)
    }

    pub fn with_cursor(mut self, cursor: SemanticTimelineCursorV1) -> Result<Self, ProjectError> {
        cursor.validate()?;
        self.cursor = Some(cursor);
        self.validate()?;
        Ok(self)
    }

    pub fn from_json_slice(bytes: &[u8]) -> Result<Self, ProjectError> {
        if bytes.len() > MAX_TIMELINE_QUERY_BYTES {
            return Err(ProjectError::DocumentTooLarge);
        }
        let value = parse_unique_json(bytes).map_err(|_| ProjectError::InvalidSemanticTimeline)?;
        let query: Self =
            serde_json::from_value(value).map_err(|_| ProjectError::InvalidSemanticTimeline)?;
        query.validate()?;
        if query.to_canonical_json()? != bytes {
            return Err(ProjectError::InvalidSemanticTimeline);
        }
        Ok(query)
    }

    pub fn to_canonical_json(&self) -> Result<Vec<u8>, ProjectError> {
        self.validate()?;
        let bytes = serde_json_canonicalizer::to_vec(self)
            .map_err(|_| ProjectError::InvalidSemanticTimeline)?;
        if bytes.len() > MAX_TIMELINE_QUERY_BYTES {
            return Err(ProjectError::DocumentTooLarge);
        }
        Ok(bytes)
    }

    fn validate(&self) -> Result<(), ProjectError> {
        if self.schema_version != SEMANTIC_TIMELINE_SCHEMA_VERSION
            || self.document_kind != SEMANTIC_TIMELINE_DOCUMENT_KIND
            || !valid_prefixed_digest(&self.request_id, "ptr_")
            || !valid_prefixed_digest(&self.catalog_id, "pca_")
            || self
                .project_id
                .as_ref()
                .is_some_and(|project_id| project_id.validate().is_err())
            || self.limit == 0
            || self.limit > MAX_TIMELINE_PAGE_EVENTS
            || self.max_bytes < MIN_TIMELINE_RESULT_BYTES
            || self.max_bytes > MAX_TIMELINE_RESULT_BYTES
            || self.request_id != self.identity()?
            || self
                .cursor
                .as_ref()
                .is_some_and(|cursor| cursor.validate().is_err())
        {
            return Err(ProjectError::InvalidSemanticTimeline);
        }
        Ok(())
    }

    fn identity(&self) -> Result<String, ProjectError> {
        prefixed_digest(
            "ptr_",
            b"qiongli-semantic-timeline-request-v1\0",
            &SemanticTimelineRequestIdentity {
                schema_version: self.schema_version,
                catalog_id: &self.catalog_id,
                project_id: &self.project_id,
                view: self.view,
                limit: self.limit,
                max_bytes: self.max_bytes,
            },
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SemanticTimelineResultV1 {
    pub schema_version: u32,
    pub document_kind: String,
    pub request_id: String,
    pub query_id: String,
    pub catalog_id: String,
    pub portfolio_id: String,
    pub timeline_digest: String,
    pub project_id: Option<ProjectId>,
    pub view: SemanticTimelineView,
    pub matched_event_count: usize,
    pub truncated: bool,
    pub events: Vec<SemanticActivityV1>,
    pub next_cursor: Option<SemanticTimelineCursorV1>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SemanticTimelineQueryIdentity<'a> {
    request_id: &'a str,
    catalog_id: &'a str,
    portfolio_id: &'a str,
    timeline_digest: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SemanticTimelineSnapshot<'a> {
    events: &'a [SemanticActivityV1],
}

#[derive(Clone)]
pub struct SemanticTimelineService {
    projects: ProjectStateService,
}

impl SemanticTimelineService {
    #[must_use]
    pub const fn new(projects: ProjectStateService) -> Self {
        Self { projects }
    }

    pub fn query(
        &self,
        query: &SemanticTimelineQueryV1,
    ) -> Result<SemanticTimelineResultV1, ProjectError> {
        query.validate()?;
        let current = IncrementalPortfolioService::new(self.projects.clone()).current()?;
        if current.catalog.catalog_id != query.catalog_id {
            return Err(ProjectError::PortfolioCatalogConflict);
        }
        if query.project_id.as_ref().is_some_and(|project_id| {
            !current
                .catalog
                .contributions
                .iter()
                .any(|contribution| &contribution.project_id == project_id)
        }) {
            return Err(ProjectError::ProjectNotRegistered);
        }
        let catalog = self
            .projects
            .portfolio_catalog_store
            .rebuild()?
            .ok_or(ProjectError::RecoveryRequired)?;
        if catalog.manifest.catalog_id != current.catalog.catalog_id {
            return Err(ProjectError::RevisionConflict);
        }
        let events = collect_semantic_activity(&self.projects, &catalog.contributions)?;
        let timeline_digest_value = timeline_digest(&events)?;
        let query_id = prefixed_digest(
            "pty_",
            b"qiongli-semantic-timeline-query-v1\0",
            &SemanticTimelineQueryIdentity {
                request_id: &query.request_id,
                catalog_id: &current.catalog.catalog_id,
                portfolio_id: &current.portfolio.portfolio_id,
                timeline_digest: &timeline_digest_value,
            },
        )?;
        if query
            .cursor
            .as_ref()
            .is_some_and(|cursor| cursor.query_id != query_id)
        {
            return Err(ProjectError::InvalidSemanticTimeline);
        }
        let mut matched = events
            .iter()
            .filter(|event| {
                query.project_id.as_ref().is_none_or(|project_id| {
                    event
                        .project_ids
                        .iter()
                        .any(|candidate| candidate == project_id)
                }) && match query.view {
                    SemanticTimelineView::Activity => true,
                    SemanticTimelineView::RevisionHistory => event.kind.is_revision_history(),
                    SemanticTimelineView::MergeResolutionHistory => {
                        event.kind.is_merge_resolution_history()
                    }
                }
            })
            .cloned()
            .collect::<Vec<_>>();
        let matched_event_count = matched.len();
        if let Some(cursor) = query.cursor.as_ref() {
            matched.retain(|event| {
                (event.occurred_at_unix, event.event_id.as_str())
                    > (
                        cursor.after_occurred_at_unix,
                        cursor.after_event_id.as_str(),
                    )
            });
        }
        let byte_budget = query
            .max_bytes
            .checked_sub(TIMELINE_RESULT_RESERVE_BYTES)
            .ok_or(ProjectError::InvalidSemanticTimeline)?;
        let mut selected = Vec::new();
        let mut used_bytes = 0_usize;
        for event in matched.iter().take(query.limit) {
            let event_bytes = serde_json_canonicalizer::to_vec(event)
                .map_err(|_| ProjectError::InvalidSemanticTimeline)?
                .len();
            if used_bytes
                .checked_add(event_bytes)
                .is_none_or(|next| next > byte_budget)
            {
                break;
            }
            used_bytes += event_bytes;
            selected.push(event.clone());
        }
        let truncated = selected.len() < matched.len();
        if truncated && selected.is_empty() && !matched.is_empty() {
            return Err(ProjectError::DocumentTooLarge);
        }
        let next_cursor = if truncated {
            let last = selected
                .last()
                .ok_or(ProjectError::InvalidSemanticTimeline)?;
            Some(SemanticTimelineCursorV1::new(
                query_id.clone(),
                last.occurred_at_unix,
                last.event_id.clone(),
            )?)
        } else {
            None
        };
        let result = SemanticTimelineResultV1 {
            schema_version: SEMANTIC_TIMELINE_SCHEMA_VERSION,
            document_kind: SEMANTIC_TIMELINE_RESULT_DOCUMENT_KIND.to_string(),
            request_id: query.request_id.clone(),
            query_id,
            catalog_id: current.catalog.catalog_id,
            portfolio_id: current.portfolio.portfolio_id,
            timeline_digest: timeline_digest_value,
            project_id: query.project_id.clone(),
            view: query.view,
            matched_event_count,
            truncated,
            events: selected,
            next_cursor,
        };
        let confirmed = IncrementalPortfolioService::new(self.projects.clone()).current()?;
        let confirmed_catalog = self
            .projects
            .portfolio_catalog_store
            .rebuild()?
            .ok_or(ProjectError::RevisionConflict)?;
        let confirmed_events =
            collect_semantic_activity(&self.projects, &confirmed_catalog.contributions)?;
        if confirmed.catalog.catalog_id != result.catalog_id
            || confirmed.portfolio.portfolio_id != result.portfolio_id
            || timeline_digest(&confirmed_events)? != result.timeline_digest
        {
            return Err(ProjectError::RevisionConflict);
        }
        let bytes = serde_json_canonicalizer::to_vec(&result)
            .map_err(|_| ProjectError::InvalidSemanticTimeline)?;
        if bytes.len() > query.max_bytes {
            return Err(ProjectError::DocumentTooLarge);
        }
        Ok(result)
    }
}

fn collect_semantic_activity(
    projects: &ProjectStateService,
    contributions: &[PortfolioContributionV1],
) -> Result<Vec<SemanticActivityV1>, ProjectError> {
    let included = contributions
        .iter()
        .map(|contribution| contribution.project_id.clone())
        .collect::<BTreeSet<_>>();
    let library = projects.snapshot()?;
    let summaries = library
        .projects
        .into_iter()
        .filter(|summary| included.contains(&summary.project_id))
        .map(|summary| (summary.project_id.clone(), summary))
        .collect::<BTreeMap<_, _>>();
    if summaries.len() != included.len() {
        return Err(ProjectError::RevisionConflict);
    }
    let mut events = Vec::new();
    for contribution in contributions {
        let summary = summaries
            .get(&contribution.project_id)
            .ok_or(ProjectError::RevisionConflict)?;
        events.push(SemanticActivityV1::new(
            SemanticActivityKind::ProjectRegistered,
            summary.registered_at_unix,
            SemanticActivityTimestampSource::ProjectRegisteredAt,
            vec![summary.project_id.clone()],
            vec![summary.project_id.as_str().to_string()],
            SemanticActivityDetails::default(),
        )?);
        events.push(SemanticActivityV1::new(
            SemanticActivityKind::ProjectRevisionObserved,
            summary.academically_updated_at_unix,
            SemanticActivityTimestampSource::ProjectAcademicallyUpdatedAt,
            vec![summary.project_id.clone()],
            vec![
                summary.project_id.as_str().to_string(),
                contribution.projection_id.clone(),
            ],
            SemanticActivityDetails {
                to_project_revision: Some(summary.semantic_revision),
                ..SemanticActivityDetails::default()
            },
        )?);
        events.push(SemanticActivityV1::new(
            SemanticActivityKind::ProjectLifecycleObserved,
            summary.academically_updated_at_unix,
            SemanticActivityTimestampSource::ProjectAcademicallyUpdatedAt,
            vec![summary.project_id.clone()],
            vec![summary.project_id.as_str().to_string()],
            SemanticActivityDetails {
                to_project_revision: Some(summary.semantic_revision),
                lifecycle: Some(summary.lifecycle),
                ..SemanticActivityDetails::default()
            },
        )?);

        let inbox = projects.capture_inbox(&contribution.project_id)?;
        let root = projects.resolve_project_root(&contribution.project_id)?;
        for entry in inbox.entries {
            events.push(SemanticActivityV1::new(
                SemanticActivityKind::CaptureAccepted,
                entry.captured_at_unix,
                SemanticActivityTimestampSource::CaptureCapturedAt,
                vec![contribution.project_id.clone()],
                vec![entry.capture_id.as_str().to_string()],
                SemanticActivityDetails {
                    from_project_revision: Some(entry.base_revision),
                    source: Some(entry.source),
                    delivery: Some(entry.delivery),
                    ..SemanticActivityDetails::default()
                },
            )?);
            if let Some((receipt, _)) = read_consolidation_receipt(root.path(), &entry.capture_id)?
            {
                events.push(SemanticActivityV1::new(
                    SemanticActivityKind::CaptureConsolidated,
                    receipt.consolidated_at_unix,
                    SemanticActivityTimestampSource::ConsolidationConsolidatedAt,
                    vec![contribution.project_id.clone()],
                    vec![
                        receipt.capture_id.as_str().to_string(),
                        receipt.acknowledgement,
                    ],
                    SemanticActivityDetails {
                        from_project_revision: Some(receipt.from_project_revision),
                        to_project_revision: Some(receipt.to_project_revision),
                        source: Some(entry.source),
                        delivery: Some(entry.delivery),
                        ..SemanticActivityDetails::default()
                    },
                )?);
            }
        }
    }

    let deliveries = projects.delivery_store.rebuild()?.entries;
    let delivery_by_id = deliveries
        .iter()
        .map(|stored| (stored.envelope.envelope_id.clone(), stored))
        .collect::<BTreeMap<_, _>>();
    for stored in &deliveries {
        let mut project_ids = vec![stored.envelope.capture.binding.project_id.clone()];
        if let Some(destination) = stored.envelope.destination.as_ref() {
            project_ids.push(destination.project_id.clone());
        }
        project_ids.retain(|project_id| included.contains(project_id));
        if project_ids.is_empty() {
            continue;
        }
        for transition in &stored.record.transitions {
            let kind = delivery_activity_kind(transition.to_state);
            let mut related_ids = vec![
                stored.envelope.envelope_id.as_str().to_string(),
                stored.envelope.capture_id.as_str().to_string(),
            ];
            related_ids.extend(
                transition
                    .acknowledgement_id
                    .iter()
                    .map(|id| id.as_str().to_string()),
            );
            let to_project_revision = if transition.to_state == CaptureDeliveryState::Acknowledged {
                stored
                    .acknowledgement
                    .as_ref()
                    .map(|acknowledgement| acknowledgement.resulting_project_revision)
            } else {
                None
            };
            events.push(SemanticActivityV1::new(
                kind,
                transition.transitioned_at_unix,
                SemanticActivityTimestampSource::DeliveryTransitionedAt,
                project_ids.clone(),
                related_ids,
                SemanticActivityDetails {
                    from_project_revision: stored
                        .envelope
                        .destination
                        .as_ref()
                        .map(|destination| destination.expected_project_revision),
                    to_project_revision,
                    source: Some(stored.envelope.source),
                    delivery: Some(stored.envelope.delivery),
                    delivery_state: Some(transition.to_state),
                    delivery_reason: Some(transition.reason_code),
                    delivery_generation: Some(transition.generation),
                    ..SemanticActivityDetails::default()
                },
            )?);
        }
    }

    for assignment in projects.list_capture_assignments()? {
        if !included.contains(&assignment.target_project_id) {
            continue;
        }
        let source_project_id = delivery_by_id
            .get(&assignment.source_envelope_id)
            .map(|stored| stored.envelope.capture.binding.project_id.clone());
        let mut project_ids = vec![assignment.target_project_id.clone()];
        project_ids.extend(source_project_id.filter(|project_id| included.contains(project_id)));
        events.push(SemanticActivityV1::new(
            SemanticActivityKind::AssignmentCreated,
            assignment.created_at_unix,
            SemanticActivityTimestampSource::AssignmentCreatedAt,
            project_ids.clone(),
            vec![
                assignment.intent_id.as_str().to_string(),
                assignment.source_envelope_id.as_str().to_string(),
                assignment.source_capture_id.as_str().to_string(),
            ],
            SemanticActivityDetails {
                from_project_revision: Some(assignment.target_project_revision),
                ..SemanticActivityDetails::default()
            },
        )?);
        if let (Some(outcome), Some(decided_at_unix), Some(receipt_id)) = (
            assignment.outcome,
            assignment.decided_at_unix,
            assignment.receipt_id,
        ) {
            let mut related_ids = vec![
                assignment.intent_id.as_str().to_string(),
                receipt_id.as_str().to_string(),
                assignment.source_envelope_id.as_str().to_string(),
                assignment.source_capture_id.as_str().to_string(),
            ];
            related_ids.extend(
                assignment
                    .derived_capture_id
                    .iter()
                    .map(|id| id.as_str().to_string()),
            );
            related_ids.extend(
                assignment
                    .child_envelope_id
                    .iter()
                    .map(|id| id.as_str().to_string()),
            );
            events.push(SemanticActivityV1::new(
                match outcome {
                    CaptureAssignmentOutcome::Assigned => SemanticActivityKind::CaptureAssigned,
                    CaptureAssignmentOutcome::Rejected => {
                        SemanticActivityKind::CaptureAssignmentRejected
                    }
                },
                decided_at_unix,
                SemanticActivityTimestampSource::AssignmentDecidedAt,
                project_ids,
                related_ids,
                SemanticActivityDetails {
                    from_project_revision: Some(assignment.target_project_revision),
                    assignment_outcome: Some(outcome),
                    ..SemanticActivityDetails::default()
                },
            )?);
        }
    }

    for contribution in contributions {
        for resolution in projects.list_capture_resolutions(&contribution.project_id)? {
            append_resolution_events(&mut events, resolution)?;
        }
    }
    events.sort_by(|left, right| {
        left.occurred_at_unix
            .cmp(&right.occurred_at_unix)
            .then_with(|| left.event_id.cmp(&right.event_id))
    });
    if events.len() > MAX_TIMELINE_SNAPSHOT_EVENTS
        || events
            .windows(2)
            .any(|pair| pair[0].event_id == pair[1].event_id)
    {
        return Err(ProjectError::InvalidSemanticTimeline);
    }
    let bytes = serde_json_canonicalizer::to_vec(&SemanticTimelineSnapshot { events: &events })
        .map_err(|_| ProjectError::InvalidSemanticTimeline)?;
    if bytes.len() > MAX_TIMELINE_SNAPSHOT_BYTES {
        return Err(ProjectError::DocumentTooLarge);
    }
    Ok(events)
}

fn append_resolution_events(
    events: &mut Vec<SemanticActivityV1>,
    resolution: crate::CaptureResolutionReceiptV1,
) -> Result<(), ProjectError> {
    let receipt = resolution.receipt;
    let base_related_ids = vec![
        resolution.receipt_id.as_str().to_string(),
        receipt.assignment_receipt_id.as_str().to_string(),
        receipt.source_envelope_id.as_str().to_string(),
        receipt.source_capture_id.as_str().to_string(),
        receipt.derived_capture_id.as_str().to_string(),
        receipt.child_envelope_id.as_str().to_string(),
    ];
    events.push(SemanticActivityV1::new(
        SemanticActivityKind::ResolutionReviewed,
        receipt.reviewed_at_unix,
        SemanticActivityTimestampSource::ResolutionReviewedAt,
        vec![receipt.target_project_id.clone()],
        base_related_ids.clone(),
        SemanticActivityDetails {
            from_project_revision: Some(receipt.from_project_revision),
            ..SemanticActivityDetails::default()
        },
    )?);
    for decision in &receipt.decisions {
        let mut related_ids = base_related_ids.clone();
        related_ids.push(decision.item.item_id.as_str().to_string());
        events.push(SemanticActivityV1::new(
            SemanticActivityKind::ResolutionItemResolved,
            receipt.resolved_at_unix,
            SemanticActivityTimestampSource::ResolutionResolvedAt,
            vec![receipt.target_project_id.clone()],
            related_ids,
            SemanticActivityDetails {
                from_project_revision: Some(receipt.from_project_revision),
                to_project_revision: Some(receipt.to_project_revision),
                resolution_item_id: Some(decision.item.item_id.as_str().to_string()),
                resolution_item_kind: Some(decision.item.kind),
                resolution_disposition: Some(decision.disposition),
                ..SemanticActivityDetails::default()
            },
        )?);
    }
    events.push(SemanticActivityV1::new(
        SemanticActivityKind::ResolutionCompleted,
        receipt.resolved_at_unix,
        SemanticActivityTimestampSource::ResolutionResolvedAt,
        vec![receipt.target_project_id],
        base_related_ids,
        SemanticActivityDetails {
            from_project_revision: Some(receipt.from_project_revision),
            to_project_revision: Some(receipt.to_project_revision),
            ..SemanticActivityDetails::default()
        },
    )?);
    Ok(())
}

const fn delivery_activity_kind(state: CaptureDeliveryState) -> SemanticActivityKind {
    match state {
        CaptureDeliveryState::Queued => SemanticActivityKind::DeliveryQueued,
        CaptureDeliveryState::Delivering => SemanticActivityKind::DeliveryStarted,
        CaptureDeliveryState::Delivered => SemanticActivityKind::DeliveryDelivered,
        CaptureDeliveryState::Acknowledged => SemanticActivityKind::DeliveryAcknowledged,
        CaptureDeliveryState::RetryRequired => SemanticActivityKind::DeliveryRetryRequired,
        CaptureDeliveryState::Conflicted => SemanticActivityKind::DeliveryConflicted,
        CaptureDeliveryState::Cancelled => SemanticActivityKind::DeliveryCancelled,
    }
}

fn timeline_digest(events: &[SemanticActivityV1]) -> Result<String, ProjectError> {
    prefixed_digest(
        "ptl_",
        b"qiongli-semantic-timeline-snapshot-v1\0",
        &SemanticTimelineSnapshot { events },
    )
}

const fn valid_revision(revision: u64) -> bool {
    revision > 0 && revision <= MAX_SEMANTIC_REVISION
}

fn valid_text(value: &str, maximum_bytes: usize) -> bool {
    !value.is_empty()
        && value == value.trim()
        && value.len() <= maximum_bytes
        && !value.chars().any(char::is_control)
}

fn valid_prefixed_digest(value: &str, prefix: &str) -> bool {
    value.strip_prefix(prefix).is_some_and(|digest| {
        digest.len() == 64
            && digest
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    })
}

fn prefixed_digest<T: Serialize>(
    prefix: &str,
    domain: &[u8],
    value: &T,
) -> Result<String, ProjectError> {
    let bytes = serde_json_canonicalizer::to_vec(value)
        .map_err(|_| ProjectError::InvalidSemanticTimeline)?;
    let mut digest = Sha256::new();
    digest.update(domain);
    digest.update(bytes);
    Ok(format!("{prefix}{:x}", digest.finalize()))
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    use qiongli_config::{ConfigRoot, resolve_config_root};
    use serde_json::json;

    use super::*;
    use crate::{
        ApprovedCaptureAssignment, ApprovedCaptureConsolidation, ApprovedCaptureIntake,
        ApprovedProjectMutation, CaptureArea, CaptureAssignmentDecision,
        CaptureDeliveryDestinationV1, CaptureDeliveryEnvelopeV1, CaptureDeliveryRetryCause,
        CapturePolicy, CaptureResolutionCounterpartState, CaptureResolutionDecisionV1,
        CaptureResolutionItemId, CaptureResolutionItemV1, CaptureResolutionReceiptBodyV1,
        CaptureResolutionReceiptId, CaptureResolutionReceiptV1, DecisionCandidateV1,
        DecisionRelation, DeliveryEnvelopeId, EvidenceLocatorKind, EvidenceReferenceV1,
        ProjectBindingV1, ProjectKind, ProjectRegistrationOptions, ProjectStage,
        ResearchCaptureDraftV1, ResearchCaptureV1, SemanticChangeV1, VerifiedProjectMutation,
    };

    static NEXT_FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

    struct Fixture {
        root: PathBuf,
        _config: ConfigRoot,
        projects: ProjectStateService,
        incremental: IncrementalPortfolioService,
        timeline: SemanticTimelineService,
    }

    impl Fixture {
        fn new() -> Self {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time is available")
                .as_nanos();
            let root = std::env::temp_dir().join(format!(
                "qiongli-semantic-timeline-{}-{nonce}-{}",
                std::process::id(),
                NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&root).expect("fixture root can be created");
            let root = fs::canonicalize(root).expect("fixture root can be canonicalized");
            let home = root.join("home");
            fs::create_dir(&home).expect("fixture home can be created");
            let config = resolve_config_root(Some(root.join("config").as_os_str()), &home)
                .expect("config root is valid");
            let projects = ProjectStateService::new(config.clone());
            Self {
                root,
                _config: config,
                incremental: IncrementalPortfolioService::new(projects.clone()),
                timeline: SemanticTimelineService::new(projects.clone()),
                projects,
            }
        }

        fn create_project(&self, name: &str, now_unix: u64) -> ProjectId {
            let root = self.root.join(name.to_lowercase().replace(' ', "-"));
            let plan = self
                .projects
                .preview_create(
                    &root,
                    ProjectRegistrationOptions::new(name, ProjectKind::Article),
                    now_unix,
                )
                .expect("project create can be previewed");
            let project_id = plan.preview().project_id.clone();
            self.apply(&plan, now_unix);
            project_id
        }

        fn apply(&self, plan: &VerifiedProjectMutation, now_unix: u64) {
            self.projects
                .apply(
                    plan,
                    &ApprovedProjectMutation::new(plan.preview().plan_digest.clone(), true),
                    now_unix,
                )
                .expect("project mutation can be applied");
        }

        fn capture(&self, project_id: &ProjectId, captured_at_unix: u64) -> ResearchCaptureV1 {
            let summary = self
                .projects
                .snapshot()
                .expect("library is readable")
                .projects
                .into_iter()
                .find(|summary| &summary.project_id == project_id)
                .expect("project is registered");
            ResearchCaptureDraftV1 {
                binding: ProjectBindingV1::new(
                    project_id.clone(),
                    summary.semantic_revision,
                    summary.stage,
                    "Build deterministic timeline evidence",
                    CapturePolicy::ReviewRequired,
                )
                .expect("binding is valid"),
                source: CaptureSource::Codex,
                delivery: CaptureDelivery::Connected,
                captured_at_unix,
                summary: "A bounded semantic activity fixture.".to_string(),
                changes: vec![SemanticChangeV1 {
                    area: CaptureArea::Evidence,
                    summary: "Record exact timeline evidence.".to_string(),
                }],
                decisions: vec![DecisionCandidateV1 {
                    relation: DecisionRelation::Candidate,
                    statement: "Keep activity derived from receipts.".to_string(),
                    rationale: "Receipts remain authoritative.".to_string(),
                    target: None,
                }],
                evidence: vec![EvidenceReferenceV1 {
                    locator_kind: EvidenceLocatorKind::Doi,
                    locator: "10.1000/semantic-timeline".to_string(),
                    relevance: "Provides deterministic fixture evidence.".to_string(),
                    limitation: Some("Fixture evidence only.".to_string()),
                }],
                contradictions: Vec::new(),
                next_actions: vec!["Review the derived timeline.".to_string()],
            }
            .into_capture()
            .expect("capture is valid")
        }

        fn accept_capture(&self, capture: ResearchCaptureV1, applied_at_unix: u64) {
            let intake = self
                .projects
                .preview_capture(capture)
                .expect("capture can be previewed");
            self.projects
                .apply_capture(
                    &intake,
                    &ApprovedCaptureIntake::new(intake.preview().plan_digest.clone(), true),
                    applied_at_unix,
                )
                .expect("capture can be accepted");
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn project_and_portfolio_views_are_deterministic_bounded_and_path_redacted() {
        let fixture = Fixture::new();
        let project_a = fixture.create_project("Timeline A", 10);
        let _project_b = fixture.create_project("Timeline B", 20);
        let reconciled = fixture
            .incremental
            .reconcile(21)
            .expect("catalog can be reconciled");
        let query = SemanticTimelineQueryV1::new(reconciled.snapshot.catalog.catalog_id.clone())
            .expect("timeline query is valid")
            .for_project(project_a.clone())
            .expect("project scope is valid");
        let first = fixture.timeline.query(&query).expect("timeline succeeds");
        let repeated = fixture.timeline.query(&query).expect("timeline repeats");
        assert_eq!(first, repeated);
        assert!(
            first
                .events
                .iter()
                .all(|event| event.project_ids.contains(&project_a))
        );
        assert_eq!(
            first
                .events
                .iter()
                .map(|event| (event.occurred_at_unix, event.event_id.as_str()))
                .collect::<Vec<_>>(),
            {
                let mut ordering = first
                    .events
                    .iter()
                    .map(|event| (event.occurred_at_unix, event.event_id.as_str()))
                    .collect::<Vec<_>>();
                ordering.sort_unstable();
                ordering
            }
        );
        assert!(
            !serde_json::to_string(&first)
                .expect("result serializes")
                .contains(fixture.root.to_string_lossy().as_ref())
        );

        let revision_query = SemanticTimelineQueryV1::new(reconciled.snapshot.catalog.catalog_id)
            .expect("timeline query is valid")
            .with_view(SemanticTimelineView::RevisionHistory)
            .expect("revision view is valid");
        let revisions = fixture
            .timeline
            .query(&revision_query)
            .expect("revision history succeeds");
        assert!(
            revisions
                .events
                .iter()
                .all(|event| event.kind.is_revision_history())
        );
        assert!(
            revisions
                .events
                .iter()
                .any(|event| event.kind == SemanticActivityKind::ProjectRegistered)
        );
    }

    #[test]
    fn delivery_and_assignment_history_uses_every_exact_transition_and_lineage_id() {
        let fixture = Fixture::new();
        let source_project = fixture.create_project("Timeline Source", 10);
        let target_project = fixture.create_project("Timeline Target", 20);
        let reconciled = fixture
            .incremental
            .reconcile(21)
            .expect("catalog can be reconciled");
        let capture = fixture.capture(&source_project, 30);
        fixture.accept_capture(capture.clone(), 31);
        let target = fixture
            .projects
            .snapshot()
            .expect("library is readable")
            .projects
            .into_iter()
            .find(|summary| summary.project_id == target_project)
            .expect("target is registered");
        let envelope = CaptureDeliveryEnvelopeV1::new(
            capture.clone(),
            Some(
                CaptureDeliveryDestinationV1::new(target_project.clone(), target.semantic_revision)
                    .expect("destination is valid"),
            ),
            32,
        )
        .expect("envelope is valid");
        let envelope_id = envelope.envelope_id.clone();
        let queued = fixture
            .projects
            .enqueue_capture_delivery(envelope)
            .expect("delivery can be queued");
        let delivering = fixture
            .projects
            .begin_capture_delivery(&envelope_id, queued.generation, &queued.record_sha256, 33)
            .expect("delivery can begin");
        let retry = fixture
            .projects
            .retry_capture_delivery(
                &envelope_id,
                delivering.generation,
                &delivering.record_sha256,
                34,
                CaptureDeliveryRetryCause::TransportUnavailable,
            )
            .expect("delivery can request retry");
        let redelivering = fixture
            .projects
            .begin_capture_delivery(&envelope_id, retry.generation, &retry.record_sha256, 35)
            .expect("delivery can retry");
        fixture
            .projects
            .cancel_capture_delivery(
                &envelope_id,
                redelivering.generation,
                &redelivering.record_sha256,
                36,
            )
            .expect("delivery can be cancelled");

        let unbound =
            CaptureDeliveryEnvelopeV1::new(capture, None, 37).expect("unbound envelope is valid");
        let unbound_id = unbound.envelope_id.clone();
        fixture
            .projects
            .enqueue_capture_delivery(unbound)
            .expect("unbound delivery can be queued");
        let assignment = fixture
            .projects
            .preview_capture_assignment(
                &unbound_id,
                &target_project,
                CaptureAssignmentDecision::Reject,
                38,
            )
            .expect("assignment can be previewed");
        let assignment_commit = fixture
            .projects
            .apply_capture_assignment(
                &assignment,
                &ApprovedCaptureAssignment::new(assignment.preview().plan_digest.clone(), true),
            )
            .expect("assignment can be rejected");

        let query = SemanticTimelineQueryV1::new(reconciled.snapshot.catalog.catalog_id)
            .expect("timeline query is valid");
        let result = fixture.timeline.query(&query).expect("timeline succeeds");
        let envelope_events = result
            .events
            .iter()
            .filter(|event| {
                event
                    .related_ids
                    .iter()
                    .any(|id| id == envelope_id.as_str())
            })
            .collect::<Vec<_>>();
        assert_eq!(
            envelope_events
                .iter()
                .map(|event| event.kind)
                .collect::<Vec<_>>(),
            vec![
                SemanticActivityKind::DeliveryQueued,
                SemanticActivityKind::DeliveryStarted,
                SemanticActivityKind::DeliveryRetryRequired,
                SemanticActivityKind::DeliveryStarted,
                SemanticActivityKind::DeliveryCancelled,
            ]
        );
        assert_eq!(
            envelope_events
                .iter()
                .filter_map(|event| event.delivery_generation)
                .collect::<Vec<_>>(),
            vec![1, 2, 3, 4, 5]
        );
        let rejected = result
            .events
            .iter()
            .find(|event| event.kind == SemanticActivityKind::CaptureAssignmentRejected)
            .expect("rejected assignment event exists");
        assert!(
            rejected
                .related_ids
                .contains(&assignment_commit.receipt_id.as_str().to_string())
        );
        assert_eq!(
            rejected.assignment_outcome,
            Some(CaptureAssignmentOutcome::Rejected)
        );
        assert!(rejected.project_ids.contains(&source_project));
        assert!(rejected.project_ids.contains(&target_project));
    }

    #[test]
    fn merge_history_and_cursor_are_bound_to_exact_receipt_state() {
        let fixture = Fixture::new();
        let project_id = fixture.create_project("Timeline Merge", 10);
        let initial = fixture
            .incremental
            .reconcile(11)
            .expect("catalog can be reconciled");
        let capture = fixture.capture(&project_id, 20);
        fixture.accept_capture(capture.clone(), 21);
        let consolidation = fixture
            .projects
            .preview_capture_consolidation(&project_id, &capture.capture_id, 22)
            .expect("consolidation can be previewed");
        fixture
            .projects
            .apply_capture_consolidation(
                &consolidation,
                &ApprovedCaptureConsolidation::new(
                    consolidation.preview().plan_digest.clone(),
                    true,
                    true,
                ),
            )
            .expect("capture can be consolidated");
        let reconciled = fixture
            .incremental
            .reconcile(23)
            .expect("catalog can be refreshed");
        assert_ne!(
            initial.snapshot.catalog.catalog_id,
            reconciled.snapshot.catalog.catalog_id
        );
        let merge_query =
            SemanticTimelineQueryV1::new(reconciled.snapshot.catalog.catalog_id.clone())
                .expect("timeline query is valid")
                .with_view(SemanticTimelineView::MergeResolutionHistory)
                .expect("merge history view is valid");
        let merge_history = fixture
            .timeline
            .query(&merge_query)
            .expect("merge history succeeds");
        assert!(merge_history.events.iter().any(|event| event.kind
            == SemanticActivityKind::CaptureConsolidated
            && event.from_project_revision.is_some()
            && event.to_project_revision.is_some()));

        let query = SemanticTimelineQueryV1::new(reconciled.snapshot.catalog.catalog_id.clone())
            .expect("timeline query is valid")
            .with_limits(1, MIN_TIMELINE_RESULT_BYTES)
            .expect("bounded timeline is valid");
        let first = fixture.timeline.query(&query).expect("first page succeeds");
        assert!(first.truncated);
        let cursor = first.next_cursor.clone().expect("cursor is returned");
        let second = fixture
            .timeline
            .query(
                &query
                    .clone()
                    .with_cursor(cursor.clone())
                    .expect("cursor is valid"),
            )
            .expect("second page succeeds");
        assert!(second.events.iter().all(|event| {
            !first
                .events
                .iter()
                .any(|seen| seen.event_id == event.event_id)
        }));
        let next_capture = fixture.capture(&project_id, 30);
        fixture.accept_capture(next_capture, 31);
        let stale_cursor_query = query
            .with_cursor(cursor)
            .expect("cursor still has a valid shape");
        assert_eq!(
            fixture.timeline.query(&stale_cursor_query).unwrap_err(),
            ProjectError::InvalidSemanticTimeline
        );
    }

    #[test]
    fn timeline_contract_rejects_unknown_fields_noncanonical_json_and_invalid_item_shape() {
        let query = SemanticTimelineQueryV1::new(format!("pca_{}", "a".repeat(64)))
            .expect("query shape is valid");
        let mut value: serde_json::Value =
            serde_json::from_slice(&query.to_canonical_json().expect("query serializes"))
                .expect("query is JSON");
        value
            .as_object_mut()
            .expect("query is an object")
            .insert("projectRoot".to_string(), json!("/private/project"));
        assert_eq!(
            SemanticTimelineQueryV1::from_json_slice(
                &serde_json::to_vec(&value).expect("tampered query serializes")
            )
            .unwrap_err(),
            ProjectError::InvalidSemanticTimeline
        );
        assert_eq!(
            SemanticTimelineQueryV1::from_json_slice(
                &serde_json::to_vec_pretty(&query).expect("pretty query serializes")
            )
            .unwrap_err(),
            ProjectError::InvalidSemanticTimeline
        );
        let project_id =
            ProjectId::parse(format!("prj_{}", "b".repeat(32))).expect("project id is valid");
        assert_eq!(
            SemanticActivityV1::new(
                SemanticActivityKind::ResolutionItemResolved,
                1,
                SemanticActivityTimestampSource::ResolutionResolvedAt,
                vec![project_id],
                vec![format!("crr_{}", "c".repeat(64))],
                SemanticActivityDetails::default(),
            )
            .unwrap_err(),
            ProjectError::InvalidSemanticTimeline
        );
    }

    #[test]
    fn resolution_receipt_projects_one_event_for_every_exact_item_decision() {
        let project_id =
            ProjectId::parse(format!("prj_{}", "1".repeat(32))).expect("project id is valid");
        let source_envelope_id = DeliveryEnvelopeId::parse(format!("env_{}", "2".repeat(64)))
            .expect("source envelope id is valid");
        let child_envelope_id = DeliveryEnvelopeId::parse(format!("env_{}", "3".repeat(64)))
            .expect("child envelope id is valid");
        let source_capture_id = crate::CaptureId::parse(format!("cap_{}", "4".repeat(64)))
            .expect("source capture id is valid");
        let derived_capture_id = crate::CaptureId::parse(format!("cap_{}", "5".repeat(64)))
            .expect("derived capture id is valid");
        let assignment_receipt_id =
            crate::CaptureAssignmentReceiptId::parse(format!("car_{}", "6".repeat(64)))
                .expect("assignment receipt id is valid");
        let resolution_item_id = CaptureResolutionItemId::parse(format!("cri_{}", "7".repeat(64)))
            .expect("resolution item id is valid");
        let resolution_receipt_id =
            CaptureResolutionReceiptId::parse(format!("crr_{}", "8".repeat(64)))
                .expect("resolution receipt id is valid");
        let disposition = CaptureResolutionDisposition::RetainBoth;
        let receipt = CaptureResolutionReceiptV1 {
            schema_version: crate::CAPTURE_RESOLUTION_SCHEMA_VERSION,
            document_kind: crate::CAPTURE_RESOLUTION_RECEIPT_DOCUMENT_KIND.to_string(),
            receipt_id: resolution_receipt_id.clone(),
            receipt: CaptureResolutionReceiptBodyV1 {
                resolution_plan_digest: "9".repeat(64),
                assignment_receipt_id,
                assignment_receipt_sha256: "a".repeat(64),
                source_envelope_id: source_envelope_id.clone(),
                source_envelope_sha256: "b".repeat(64),
                source_record_generation: 2,
                source_record_sha256: "c".repeat(64),
                source_capture_id,
                source_capture_sha256: "d".repeat(64),
                derived_capture_id,
                derived_capture_sha256: "e".repeat(64),
                child_envelope_id,
                target_project_id: project_id.clone(),
                assigned_library_revision: 1,
                assigned_project_revision: 1,
                expected_library_revision: 1,
                target_stage: ProjectStage::Idea,
                from_project_revision: 1,
                to_project_revision: 2,
                previous_manifest_sha256: "1".repeat(64),
                resulting_manifest_sha256: "2".repeat(64),
                observed_artifacts: Vec::new(),
                resulting_artifacts: Vec::new(),
                item_set_sha256: "3".repeat(64),
                decisions: vec![CaptureResolutionDecisionV1 {
                    item: CaptureResolutionItemV1 {
                        item_id: resolution_item_id.clone(),
                        source_envelope_id,
                        kind: CaptureResolutionItemKind::Contradiction,
                        source_index: 0,
                        source_item_sha256: "4".repeat(64),
                        counterpart_state:
                            CaptureResolutionCounterpartState::ExactIdentityDivergent,
                        current_item_sha256: Some("5".repeat(64)),
                        allowed_dispositions: vec![
                            CaptureResolutionDisposition::AcceptCurrent,
                            CaptureResolutionDisposition::AcceptCapture,
                            CaptureResolutionDisposition::RetainBoth,
                            CaptureResolutionDisposition::RejectCapture,
                        ],
                    },
                    disposition,
                }],
                reviewed_at_unix: 40,
                resolved_at_unix: 41,
            },
        };
        let mut events = Vec::new();
        append_resolution_events(&mut events, receipt).expect("receipt projects into events");
        assert_eq!(
            events.iter().map(|event| event.kind).collect::<Vec<_>>(),
            vec![
                SemanticActivityKind::ResolutionReviewed,
                SemanticActivityKind::ResolutionItemResolved,
                SemanticActivityKind::ResolutionCompleted,
            ]
        );
        let item = &events[1];
        assert_eq!(
            item.resolution_item_id.as_deref(),
            Some(resolution_item_id.as_str())
        );
        assert_eq!(
            item.resolution_item_kind,
            Some(CaptureResolutionItemKind::Contradiction)
        );
        assert_eq!(item.resolution_disposition, Some(disposition));
        assert_eq!(item.from_project_revision, Some(1));
        assert_eq!(item.to_project_revision, Some(2));
        assert!(
            item.related_ids
                .contains(&resolution_receipt_id.as_str().to_string())
        );
        assert_eq!(item.project_ids, vec![project_id]);
    }
}
