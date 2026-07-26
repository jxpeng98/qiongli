use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::consolidation::read_consolidation_receipt;
use crate::json::parse_unique_json;
use crate::{
    AcademicGraphEdgeV1, AcademicGraphIdentityScope, AcademicGraphNodeType, AcademicGraphNodeV1,
    AcademicGraphRelation, CaptureAssignmentOutcome, CaptureDelivery, CaptureDeliveryState,
    CaptureInboxState, CaptureSource, IncrementalPortfolioService, PortfolioContributionV1,
    ProjectError, ProjectHealth, ProjectId, ProjectLifecycle, ProjectStage, ProjectStateService,
};

pub const PORTFOLIO_QUERY_SCHEMA_VERSION: u32 = 1;
pub const PORTFOLIO_QUERY_DOCUMENT_KIND: &str = "qiongli-portfolio-query";
pub const PORTFOLIO_QUERY_RESULT_DOCUMENT_KIND: &str = "qiongli-portfolio-query-result";
const MAX_QUERY_DOCUMENT_BYTES: usize = 32 * 1024;
const MIN_QUERY_RESULT_BYTES: usize = 64 * 1024;
const MAX_QUERY_RESULT_BYTES: usize = 4 * 1024 * 1024;
const MAX_QUERY_TEXT_BYTES: usize = 256;
const MAX_QUERY_IDENTITY_BYTES: usize = 512;
const MAX_QUERY_LINEAGE_ID_BYTES: usize = 160;
const MAX_QUERY_PROJECTS: usize = 128;
const MAX_QUERY_NODES: usize = 256;
const MAX_QUERY_EDGES: usize = 256;
const MAX_QUERY_LINEAGE: usize = 256;
const QUERY_RESULT_RESERVE_BYTES: usize = 16 * 1024;
const MAX_LINEAGE_SNAPSHOT_RECORDS: usize = 65_536;
const MAX_LINEAGE_SNAPSHOT_BYTES: usize = 64 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PortfolioEvidenceSignal {
    Gap,
    Contradiction,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PortfolioSharedIdentityFilterV1 {
    pub node_type: AcademicGraphNodeType,
    pub canonical_id: String,
}

impl PortfolioSharedIdentityFilterV1 {
    fn validate(&self) -> Result<(), ProjectError> {
        if !matches!(
            self.node_type,
            AcademicGraphNodeType::Paper
                | AcademicGraphNodeType::Concept
                | AcademicGraphNodeType::Method
        ) || !valid_filter_text(&self.canonical_id, MAX_QUERY_IDENTITY_BYTES)
        {
            return Err(ProjectError::InvalidPortfolioQuery);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PortfolioQueryFiltersV1 {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_id: Option<ProjectId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stage: Option<ProjectStage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence_signal: Option<PortfolioEvidenceSignal>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub manuscript_section: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shared_identity: Option<PortfolioSharedIdentityFilterV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capture_source: Option<CaptureSource>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capture_delivery: Option<CaptureDelivery>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delivery_state: Option<CaptureDeliveryState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assignment_outcome: Option<CaptureAssignmentOutcome>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lineage_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

impl PortfolioQueryFiltersV1 {
    fn validate(&self) -> Result<(), ProjectError> {
        if self
            .project_id
            .as_ref()
            .is_some_and(|project_id| project_id.validate().is_err())
            || self
                .manuscript_section
                .as_deref()
                .is_some_and(|value| !valid_filter_text(value, MAX_QUERY_IDENTITY_BYTES))
            || self
                .shared_identity
                .as_ref()
                .is_some_and(|identity| identity.validate().is_err())
            || self
                .lineage_id
                .as_deref()
                .is_some_and(|value| !valid_filter_text(value, MAX_QUERY_LINEAGE_ID_BYTES))
            || self
                .text
                .as_deref()
                .is_some_and(|value| !valid_filter_text(value, MAX_QUERY_TEXT_BYTES))
        {
            return Err(ProjectError::InvalidPortfolioQuery);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PortfolioQueryLimitsV1 {
    pub projects: usize,
    pub nodes: usize,
    pub edges: usize,
    pub lineage: usize,
    pub max_bytes: usize,
}

impl Default for PortfolioQueryLimitsV1 {
    fn default() -> Self {
        Self {
            projects: 32,
            nodes: 128,
            edges: 128,
            lineage: 128,
            max_bytes: 2 * 1024 * 1024,
        }
    }
}

impl PortfolioQueryLimitsV1 {
    fn validate(&self) -> Result<(), ProjectError> {
        if self.projects == 0
            || self.projects > MAX_QUERY_PROJECTS
            || self.nodes == 0
            || self.nodes > MAX_QUERY_NODES
            || self.edges == 0
            || self.edges > MAX_QUERY_EDGES
            || self.lineage == 0
            || self.lineage > MAX_QUERY_LINEAGE
            || self.max_bytes < MIN_QUERY_RESULT_BYTES
            || self.max_bytes > MAX_QUERY_RESULT_BYTES
        {
            return Err(ProjectError::InvalidPortfolioQuery);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PortfolioQueryCursorV1 {
    pub cursor_id: String,
    pub query_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_after: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_after: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub edge_after: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lineage_after: Option<String>,
}

#[derive(Serialize)]
struct CursorIdentity<'a> {
    query_id: &'a str,
    project_after: &'a Option<String>,
    node_after: &'a Option<String>,
    edge_after: &'a Option<String>,
    lineage_after: &'a Option<String>,
}

impl PortfolioQueryCursorV1 {
    fn new(
        query_id: String,
        project_after: Option<String>,
        node_after: Option<String>,
        edge_after: Option<String>,
        lineage_after: Option<String>,
    ) -> Result<Self, ProjectError> {
        let mut cursor = Self {
            cursor_id: String::new(),
            query_id,
            project_after,
            node_after,
            edge_after,
            lineage_after,
        };
        cursor.cursor_id = cursor.identity()?;
        cursor.validate()?;
        Ok(cursor)
    }

    fn validate(&self) -> Result<(), ProjectError> {
        if !valid_prefixed_digest(&self.cursor_id, "pqc_")
            || !valid_prefixed_digest(&self.query_id, "pqy_")
            || [
                self.project_after.as_deref(),
                self.node_after.as_deref(),
                self.edge_after.as_deref(),
                self.lineage_after.as_deref(),
            ]
            .into_iter()
            .flatten()
            .any(|value| !valid_filter_text(value, MAX_QUERY_IDENTITY_BYTES))
            || self.cursor_id != self.identity()?
        {
            return Err(ProjectError::InvalidPortfolioQuery);
        }
        Ok(())
    }

    fn identity(&self) -> Result<String, ProjectError> {
        prefixed_digest(
            "pqc_",
            b"qiongli-portfolio-query-cursor-v1\0",
            &CursorIdentity {
                query_id: &self.query_id,
                project_after: &self.project_after,
                node_after: &self.node_after,
                edge_after: &self.edge_after,
                lineage_after: &self.lineage_after,
            },
        )
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PortfolioQueryV1 {
    pub schema_version: u32,
    pub document_kind: String,
    pub request_id: String,
    pub catalog_id: String,
    pub filters: PortfolioQueryFiltersV1,
    pub limits: PortfolioQueryLimitsV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursor: Option<PortfolioQueryCursorV1>,
}

#[derive(Serialize)]
struct QueryRequestIdentity<'a> {
    schema_version: u32,
    catalog_id: &'a str,
    filters: &'a PortfolioQueryFiltersV1,
    limits: &'a PortfolioQueryLimitsV1,
}

impl PortfolioQueryV1 {
    pub fn new(catalog_id: impl Into<String>) -> Result<Self, ProjectError> {
        let mut query = Self {
            schema_version: PORTFOLIO_QUERY_SCHEMA_VERSION,
            document_kind: PORTFOLIO_QUERY_DOCUMENT_KIND.to_string(),
            request_id: String::new(),
            catalog_id: catalog_id.into(),
            filters: PortfolioQueryFiltersV1::default(),
            limits: PortfolioQueryLimitsV1::default(),
            cursor: None,
        };
        query.request_id = query.identity()?;
        query.validate()?;
        Ok(query)
    }

    pub fn with_filters(mut self, filters: PortfolioQueryFiltersV1) -> Result<Self, ProjectError> {
        self.filters = filters;
        self.cursor = None;
        self.request_id = self.identity()?;
        self.validate()?;
        Ok(self)
    }

    pub fn with_limits(mut self, limits: PortfolioQueryLimitsV1) -> Result<Self, ProjectError> {
        self.limits = limits;
        self.cursor = None;
        self.request_id = self.identity()?;
        self.validate()?;
        Ok(self)
    }

    pub fn with_cursor(mut self, cursor: PortfolioQueryCursorV1) -> Result<Self, ProjectError> {
        self.cursor = Some(cursor);
        self.validate()?;
        Ok(self)
    }

    pub fn from_json_slice(bytes: &[u8]) -> Result<Self, ProjectError> {
        if bytes.len() > MAX_QUERY_DOCUMENT_BYTES {
            return Err(ProjectError::DocumentTooLarge);
        }
        let value = parse_unique_json(bytes).map_err(|_| ProjectError::InvalidPortfolioQuery)?;
        let query: Self =
            serde_json::from_value(value).map_err(|_| ProjectError::InvalidPortfolioQuery)?;
        query.validate()?;
        if query.to_canonical_json()? != bytes {
            return Err(ProjectError::InvalidPortfolioQuery);
        }
        Ok(query)
    }

    pub fn to_canonical_json(&self) -> Result<Vec<u8>, ProjectError> {
        self.validate()?;
        let bytes = serde_json_canonicalizer::to_vec(self)
            .map_err(|_| ProjectError::InvalidPortfolioQuery)?;
        if bytes.len() > MAX_QUERY_DOCUMENT_BYTES {
            return Err(ProjectError::DocumentTooLarge);
        }
        Ok(bytes)
    }

    pub fn validate(&self) -> Result<(), ProjectError> {
        self.filters.validate()?;
        self.limits.validate()?;
        if self.schema_version != PORTFOLIO_QUERY_SCHEMA_VERSION
            || self.document_kind != PORTFOLIO_QUERY_DOCUMENT_KIND
            || !valid_prefixed_digest(&self.request_id, "pqr_")
            || !valid_prefixed_digest(&self.catalog_id, "pca_")
            || self.request_id != self.identity()?
            || self
                .cursor
                .as_ref()
                .is_some_and(|cursor| cursor.validate().is_err())
        {
            return Err(ProjectError::InvalidPortfolioQuery);
        }
        Ok(())
    }

    fn identity(&self) -> Result<String, ProjectError> {
        prefixed_digest(
            "pqr_",
            b"qiongli-portfolio-query-request-v1\0",
            &QueryRequestIdentity {
                schema_version: self.schema_version,
                catalog_id: &self.catalog_id,
                filters: &self.filters,
                limits: &self.limits,
            },
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PortfolioLineageKind {
    Capture,
    Consolidation,
    Delivery,
    Assignment,
    Resolution,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PortfolioLineageRecordV1 {
    pub lineage_id: String,
    pub kind: PortfolioLineageKind,
    pub project_ids: Vec<ProjectId>,
    pub related_ids: Vec<String>,
    pub occurred_at_unix: u64,
    pub source: Option<CaptureSource>,
    pub delivery: Option<CaptureDelivery>,
    pub delivery_state: Option<CaptureDeliveryState>,
    pub assignment_outcome: Option<CaptureAssignmentOutcome>,
    pub from_project_revision: Option<u64>,
    pub to_project_revision: Option<u64>,
}

#[derive(Serialize)]
struct LineageIdentity<'a> {
    kind: PortfolioLineageKind,
    project_ids: &'a [ProjectId],
    related_ids: &'a [String],
    occurred_at_unix: u64,
    source: Option<CaptureSource>,
    delivery: Option<CaptureDelivery>,
    delivery_state: Option<CaptureDeliveryState>,
    assignment_outcome: Option<CaptureAssignmentOutcome>,
    from_project_revision: Option<u64>,
    to_project_revision: Option<u64>,
}

impl PortfolioLineageRecordV1 {
    #[allow(clippy::too_many_arguments)]
    fn new(
        kind: PortfolioLineageKind,
        mut project_ids: Vec<ProjectId>,
        mut related_ids: Vec<String>,
        occurred_at_unix: u64,
        source: Option<CaptureSource>,
        delivery: Option<CaptureDelivery>,
        delivery_state: Option<CaptureDeliveryState>,
        assignment_outcome: Option<CaptureAssignmentOutcome>,
        from_project_revision: Option<u64>,
        to_project_revision: Option<u64>,
    ) -> Result<Self, ProjectError> {
        project_ids.sort_unstable();
        project_ids.dedup();
        related_ids.sort_unstable();
        related_ids.dedup();
        let identity = LineageIdentity {
            kind,
            project_ids: &project_ids,
            related_ids: &related_ids,
            occurred_at_unix,
            source,
            delivery,
            delivery_state,
            assignment_outcome,
            from_project_revision,
            to_project_revision,
        };
        Ok(Self {
            lineage_id: prefixed_digest(
                "lin_",
                b"qiongli-portfolio-lineage-record-v1\0",
                &identity,
            )?,
            kind,
            project_ids,
            related_ids,
            occurred_at_unix,
            source,
            delivery,
            delivery_state,
            assignment_outcome,
            from_project_revision,
            to_project_revision,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PortfolioQueryProjectV1 {
    pub result_id: String,
    pub project_id: ProjectId,
    pub display_name: String,
    pub stage: ProjectStage,
    pub lifecycle: ProjectLifecycle,
    pub health: ProjectHealth,
    pub semantic_revision: u64,
    pub projection_id: String,
    pub node_count: usize,
    pub edge_count: usize,
    pub lineage_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PortfolioQueryNodeV1 {
    pub result_id: String,
    pub project_id: ProjectId,
    pub projection_id: String,
    pub node: AcademicGraphNodeV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PortfolioQueryEdgeV1 {
    pub result_id: String,
    pub project_id: ProjectId,
    pub projection_id: String,
    pub edge: AcademicGraphEdgeV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PortfolioQueryResultV1 {
    pub schema_version: u32,
    pub document_kind: String,
    pub request_id: String,
    pub query_id: String,
    pub catalog_id: String,
    pub portfolio_id: String,
    pub lineage_digest: String,
    pub matched_project_count: usize,
    pub matched_node_count: usize,
    pub matched_edge_count: usize,
    pub matched_lineage_count: usize,
    pub projects_truncated: bool,
    pub nodes_truncated: bool,
    pub edges_truncated: bool,
    pub lineage_truncated: bool,
    pub projects: Vec<PortfolioQueryProjectV1>,
    pub nodes: Vec<PortfolioQueryNodeV1>,
    pub edges: Vec<PortfolioQueryEdgeV1>,
    pub lineage: Vec<PortfolioLineageRecordV1>,
    pub next_cursor: Option<PortfolioQueryCursorV1>,
}

#[derive(Clone)]
pub struct PortfolioQueryService {
    projects: ProjectStateService,
}

impl PortfolioQueryService {
    #[must_use]
    pub const fn new(projects: ProjectStateService) -> Self {
        Self { projects }
    }

    pub fn query(&self, query: &PortfolioQueryV1) -> Result<PortfolioQueryResultV1, ProjectError> {
        query.validate()?;
        let current = IncrementalPortfolioService::new(self.projects.clone()).current()?;
        if current.catalog.catalog_id != query.catalog_id {
            return Err(ProjectError::PortfolioCatalogConflict);
        }
        let catalog = self
            .projects
            .portfolio_catalog_store
            .rebuild()?
            .ok_or(ProjectError::RecoveryRequired)?;
        if catalog.manifest.catalog_id != current.catalog.catalog_id {
            return Err(ProjectError::RevisionConflict);
        }
        let lineage = collect_lineage(&self.projects, &catalog.contributions)?;
        let lineage_digest =
            prefixed_digest("plg_", b"qiongli-portfolio-lineage-snapshot-v1\0", &lineage)?;
        let query_id = prefixed_digest(
            "pqy_",
            b"qiongli-portfolio-query-execution-v1\0",
            &(
                &query.request_id,
                &current.catalog.catalog_id,
                &current.portfolio.portfolio_id,
                &lineage_digest,
            ),
        )?;
        if query
            .cursor
            .as_ref()
            .is_some_and(|cursor| cursor.query_id != query_id)
        {
            return Err(ProjectError::InvalidPortfolioQuery);
        }

        let matched = match_query(
            query,
            &current.portfolio.projects,
            &catalog.contributions,
            &lineage,
        )?;
        let budget = query
            .limits
            .max_bytes
            .checked_sub(QUERY_RESULT_RESERVE_BYTES)
            .ok_or(ProjectError::InvalidPortfolioQuery)?
            / 4;
        let cursor = query.cursor.as_ref();
        let project_page = paginate(
            matched.projects,
            cursor.and_then(|cursor| cursor.project_after.as_deref()),
            query.limits.projects,
            budget,
            |project| project.result_id.as_str(),
        )?;
        let node_page = paginate(
            matched.nodes,
            cursor.and_then(|cursor| cursor.node_after.as_deref()),
            query.limits.nodes,
            budget,
            |node| node.result_id.as_str(),
        )?;
        let edge_page = paginate(
            matched.edges,
            cursor.and_then(|cursor| cursor.edge_after.as_deref()),
            query.limits.edges,
            budget,
            |edge| edge.result_id.as_str(),
        )?;
        let lineage_page = paginate(
            matched.lineage,
            cursor.and_then(|cursor| cursor.lineage_after.as_deref()),
            query.limits.lineage,
            budget,
            |record| record.lineage_id.as_str(),
        )?;
        let next_cursor = if project_page.truncated
            || node_page.truncated
            || edge_page.truncated
            || lineage_page.truncated
        {
            Some(PortfolioQueryCursorV1::new(
                query_id.clone(),
                project_page.next_after.clone(),
                node_page.next_after.clone(),
                edge_page.next_after.clone(),
                lineage_page.next_after.clone(),
            )?)
        } else {
            None
        };
        let result = PortfolioQueryResultV1 {
            schema_version: PORTFOLIO_QUERY_SCHEMA_VERSION,
            document_kind: PORTFOLIO_QUERY_RESULT_DOCUMENT_KIND.to_string(),
            request_id: query.request_id.clone(),
            query_id,
            catalog_id: current.catalog.catalog_id,
            portfolio_id: current.portfolio.portfolio_id,
            lineage_digest,
            matched_project_count: project_page.matched_count,
            matched_node_count: node_page.matched_count,
            matched_edge_count: edge_page.matched_count,
            matched_lineage_count: lineage_page.matched_count,
            projects_truncated: project_page.truncated,
            nodes_truncated: node_page.truncated,
            edges_truncated: edge_page.truncated,
            lineage_truncated: lineage_page.truncated,
            projects: project_page.items,
            nodes: node_page.items,
            edges: edge_page.items,
            lineage: lineage_page.items,
            next_cursor,
        };
        let confirmed_lineage = collect_lineage(&self.projects, &catalog.contributions)?;
        let confirmed = IncrementalPortfolioService::new(self.projects.clone()).current()?;
        if confirmed.catalog.catalog_id != result.catalog_id
            || confirmed.portfolio.portfolio_id != result.portfolio_id
            || confirmed_lineage != lineage
        {
            return Err(ProjectError::RevisionConflict);
        }
        let bytes = serde_json_canonicalizer::to_vec(&result)
            .map_err(|_| ProjectError::InvalidPortfolioQuery)?;
        if bytes.len() > query.limits.max_bytes {
            return Err(ProjectError::DocumentTooLarge);
        }
        Ok(result)
    }
}

#[derive(Serialize)]
struct LineageSnapshot<'a> {
    records: &'a [PortfolioLineageRecordV1],
}

fn collect_lineage(
    projects: &ProjectStateService,
    contributions: &[PortfolioContributionV1],
) -> Result<Vec<PortfolioLineageRecordV1>, ProjectError> {
    let included = contributions
        .iter()
        .map(|contribution| contribution.project_id.clone())
        .collect::<BTreeSet<_>>();
    let mut records = Vec::new();
    for contribution in contributions {
        let inbox = projects.capture_inbox(&contribution.project_id)?;
        let root = projects.resolve_project_root(&contribution.project_id)?;
        for entry in inbox.entries {
            records.push(PortfolioLineageRecordV1::new(
                PortfolioLineageKind::Capture,
                vec![contribution.project_id.clone()],
                vec![entry.capture_id.as_str().to_string()],
                entry.captured_at_unix,
                Some(entry.source),
                Some(entry.delivery),
                None,
                None,
                Some(entry.base_revision),
                None,
            )?);
            if entry.state == CaptureInboxState::Applied {
                let (receipt, _) = read_consolidation_receipt(root.path(), &entry.capture_id)?
                    .ok_or(ProjectError::RecoveryRequired)?;
                records.push(PortfolioLineageRecordV1::new(
                    PortfolioLineageKind::Consolidation,
                    vec![contribution.project_id.clone()],
                    vec![
                        receipt.capture_id.as_str().to_string(),
                        receipt.acknowledgement,
                    ],
                    receipt.consolidated_at_unix,
                    Some(entry.source),
                    Some(entry.delivery),
                    None,
                    None,
                    Some(receipt.from_project_revision),
                    Some(receipt.to_project_revision),
                )?);
            }
        }
    }

    let deliveries = projects.delivery_store.rebuild()?.entries;
    let delivery_metadata = deliveries
        .iter()
        .map(|stored| {
            (
                stored.envelope.envelope_id.clone(),
                (stored.envelope.source, stored.envelope.delivery),
            )
        })
        .collect::<BTreeMap<_, _>>();
    for stored in deliveries {
        let mut project_ids = vec![stored.envelope.capture.binding.project_id.clone()];
        if let Some(destination) = &stored.envelope.destination {
            project_ids.push(destination.project_id.clone());
        }
        project_ids.retain(|project_id| included.contains(project_id));
        if project_ids.is_empty() {
            continue;
        }
        let mut related_ids = vec![
            stored.envelope.envelope_id.as_str().to_string(),
            stored.envelope.capture_id.as_str().to_string(),
        ];
        if let Some(acknowledgement) = &stored.acknowledgement {
            related_ids.push(acknowledgement.acknowledgement_id.as_str().to_string());
            related_ids.push(acknowledgement.accepted_capture_id.as_str().to_string());
        }
        records.push(PortfolioLineageRecordV1::new(
            PortfolioLineageKind::Delivery,
            project_ids,
            related_ids,
            stored.record.updated_at_unix,
            Some(stored.envelope.source),
            Some(stored.envelope.delivery),
            Some(stored.record.state),
            None,
            stored
                .envelope
                .destination
                .as_ref()
                .map(|destination| destination.expected_project_revision),
            stored
                .acknowledgement
                .as_ref()
                .map(|acknowledgement| acknowledgement.resulting_project_revision),
        )?);
    }
    for assignment in projects.list_capture_assignments()? {
        if !included.contains(&assignment.target_project_id) {
            continue;
        }
        let mut related_ids = vec![
            assignment.intent_id.as_str().to_string(),
            assignment.source_envelope_id.as_str().to_string(),
            assignment.source_capture_id.as_str().to_string(),
        ];
        related_ids.extend(
            assignment
                .receipt_id
                .iter()
                .map(|id| id.as_str().to_string()),
        );
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
        let (source, delivery) = delivery_metadata
            .get(&assignment.source_envelope_id)
            .copied()
            .map_or((None, None), |(source, delivery)| {
                (Some(source), Some(delivery))
            });
        records.push(PortfolioLineageRecordV1::new(
            PortfolioLineageKind::Assignment,
            vec![assignment.target_project_id],
            related_ids,
            assignment
                .decided_at_unix
                .unwrap_or(assignment.created_at_unix),
            source,
            delivery,
            None,
            assignment.outcome,
            Some(assignment.target_project_revision),
            None,
        )?);
    }
    for contribution in contributions {
        for resolution in projects.list_capture_resolutions(&contribution.project_id)? {
            let receipt = resolution.receipt;
            let (source, delivery) = delivery_metadata
                .get(&receipt.source_envelope_id)
                .copied()
                .map_or((None, None), |(source, delivery)| {
                    (Some(source), Some(delivery))
                });
            records.push(PortfolioLineageRecordV1::new(
                PortfolioLineageKind::Resolution,
                vec![receipt.target_project_id],
                vec![
                    resolution.receipt_id.as_str().to_string(),
                    receipt.assignment_receipt_id.as_str().to_string(),
                    receipt.source_envelope_id.as_str().to_string(),
                    receipt.source_capture_id.as_str().to_string(),
                    receipt.derived_capture_id.as_str().to_string(),
                    receipt.child_envelope_id.as_str().to_string(),
                ],
                receipt.resolved_at_unix,
                source,
                delivery,
                Some(CaptureDeliveryState::Acknowledged),
                Some(CaptureAssignmentOutcome::Assigned),
                Some(receipt.from_project_revision),
                Some(receipt.to_project_revision),
            )?);
        }
    }
    records.sort_by(|left, right| left.lineage_id.cmp(&right.lineage_id));
    if records.len() > MAX_LINEAGE_SNAPSHOT_RECORDS
        || records
            .windows(2)
            .any(|pair| pair[0].lineage_id == pair[1].lineage_id)
    {
        return Err(ProjectError::InvalidPortfolioQuery);
    }
    let snapshot = LineageSnapshot { records: &records };
    let bytes = serde_json_canonicalizer::to_vec(&snapshot)
        .map_err(|_| ProjectError::InvalidPortfolioQuery)?;
    if bytes.len() > MAX_LINEAGE_SNAPSHOT_BYTES {
        return Err(ProjectError::DocumentTooLarge);
    }
    Ok(records)
}

struct MatchedQuery {
    projects: Vec<PortfolioQueryProjectV1>,
    nodes: Vec<PortfolioQueryNodeV1>,
    edges: Vec<PortfolioQueryEdgeV1>,
    lineage: Vec<PortfolioLineageRecordV1>,
}

fn match_query(
    query: &PortfolioQueryV1,
    portfolio_projects: &[crate::AcademicGraphPortfolioProjectV1],
    contributions: &[PortfolioContributionV1],
    lineage: &[PortfolioLineageRecordV1],
) -> Result<MatchedQuery, ProjectError> {
    let contribution_by_project = contributions
        .iter()
        .map(|contribution| (contribution.project_id.clone(), contribution))
        .collect::<BTreeMap<_, _>>();
    let portfolio_by_project = portfolio_projects
        .iter()
        .map(|project| (project.project_id.clone(), project))
        .collect::<BTreeMap<_, _>>();
    let lineage_by_project = lineage_counts(lineage);
    let mut qualifying = contribution_by_project
        .keys()
        .cloned()
        .collect::<BTreeSet<_>>();
    if let Some(project_id) = &query.filters.project_id {
        qualifying.retain(|candidate| candidate == project_id);
    }
    if let Some(stage) = query.filters.stage {
        qualifying.retain(|project_id| {
            contribution_by_project
                .get(project_id)
                .is_some_and(|contribution| contribution.graph.project_stage == stage)
        });
    }
    if let Some(signal) = query.filters.evidence_signal {
        qualifying.retain(|project_id| {
            contribution_by_project
                .get(project_id)
                .is_some_and(|contribution| graph_has_signal(&contribution.graph, signal))
        });
    }
    if let Some(section) = query.filters.manuscript_section.as_deref() {
        qualifying.retain(|project_id| {
            contribution_by_project
                .get(project_id)
                .is_some_and(|contribution| graph_has_section(&contribution.graph, section))
        });
    }
    if let Some(identity) = &query.filters.shared_identity {
        let shared_projects = contributions
            .iter()
            .filter(|contribution| graph_has_shared_identity(contribution, identity))
            .map(|contribution| contribution.project_id.clone())
            .collect::<BTreeSet<_>>();
        if shared_projects.len() < 2 {
            qualifying.clear();
        } else {
            qualifying.retain(|project_id| shared_projects.contains(project_id));
        }
    }
    let has_lineage_filter = query.filters.capture_source.is_some()
        || query.filters.capture_delivery.is_some()
        || query.filters.delivery_state.is_some()
        || query.filters.assignment_outcome.is_some()
        || query.filters.lineage_id.is_some();
    if has_lineage_filter {
        let lineage_projects = lineage
            .iter()
            .filter(|record| lineage_matches(record, &query.filters))
            .flat_map(|record| record.project_ids.iter().cloned())
            .collect::<BTreeSet<_>>();
        qualifying.retain(|project_id| lineage_projects.contains(project_id));
    }

    let mut projects = Vec::new();
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    for project_id in &qualifying {
        let contribution = contribution_by_project
            .get(project_id)
            .copied()
            .ok_or(ProjectError::RevisionConflict)?;
        let portfolio_project = portfolio_by_project
            .get(project_id)
            .copied()
            .ok_or(ProjectError::RevisionConflict)?;
        if !portfolio_project.included
            || portfolio_project.lifecycle != ProjectLifecycle::Active
            || portfolio_project.health != ProjectHealth::Ready
        {
            return Err(ProjectError::RevisionConflict);
        }
        projects.push(PortfolioQueryProjectV1 {
            result_id: format!("project:{}", project_id.as_str()),
            project_id: project_id.clone(),
            display_name: portfolio_project.display_name.clone(),
            stage: contribution.graph.project_stage,
            lifecycle: contribution.lifecycle,
            health: contribution.health,
            semantic_revision: contribution.semantic_revision,
            projection_id: contribution.projection_id.clone(),
            node_count: contribution.node_count,
            edge_count: contribution.edge_count,
            lineage_count: lineage_by_project.get(project_id).copied().unwrap_or(0),
        });
        let selected_nodes = selected_node_ids(contribution, &query.filters);
        for node in &contribution.graph.nodes {
            if selected_nodes.contains(&node.node_id) {
                nodes.push(PortfolioQueryNodeV1 {
                    result_id: format!(
                        "node:{}/{}",
                        contribution.project_id.as_str(),
                        node.node_id
                    ),
                    project_id: contribution.project_id.clone(),
                    projection_id: contribution.projection_id.clone(),
                    node: node.clone(),
                });
            }
        }
        for edge in &contribution.graph.edges {
            if edge_matches(edge, &selected_nodes, &query.filters) {
                edges.push(PortfolioQueryEdgeV1 {
                    result_id: format!(
                        "edge:{}/{}",
                        contribution.project_id.as_str(),
                        edge.edge_id
                    ),
                    project_id: contribution.project_id.clone(),
                    projection_id: contribution.projection_id.clone(),
                    edge: edge.clone(),
                });
            }
        }
    }
    let mut matched_lineage = lineage
        .iter()
        .filter(|record| {
            record
                .project_ids
                .iter()
                .any(|project_id| qualifying.contains(project_id))
                && (!has_lineage_filter || lineage_matches(record, &query.filters))
        })
        .cloned()
        .collect::<Vec<_>>();
    projects.sort_by(|left, right| left.result_id.cmp(&right.result_id));
    nodes.sort_by(|left, right| left.result_id.cmp(&right.result_id));
    edges.sort_by(|left, right| left.result_id.cmp(&right.result_id));
    matched_lineage.sort_by(|left, right| left.lineage_id.cmp(&right.lineage_id));
    Ok(MatchedQuery {
        projects,
        nodes,
        edges,
        lineage: matched_lineage,
    })
}

fn selected_node_ids(
    contribution: &PortfolioContributionV1,
    filters: &PortfolioQueryFiltersV1,
) -> BTreeSet<String> {
    let section_ids = filters
        .manuscript_section
        .as_deref()
        .map(|section| section_node_ids(&contribution.graph, section))
        .unwrap_or_default();
    let signal_ids = filters
        .evidence_signal
        .map(|signal| signal_node_ids(&contribution.graph, signal))
        .unwrap_or_default();
    contribution
        .graph
        .nodes
        .iter()
        .filter(|node| {
            (filters.manuscript_section.is_none() || section_ids.contains(&node.node_id))
                && (filters.evidence_signal.is_none() || signal_ids.contains(&node.node_id))
                && filters.shared_identity.as_ref().is_none_or(|identity| {
                    node.node_type == identity.node_type
                        && node.identity_scope == AcademicGraphIdentityScope::Global
                        && node.canonical_id == identity.canonical_id
                })
                && filters
                    .text
                    .as_deref()
                    .is_none_or(|text| node.label.to_lowercase().contains(&text.to_lowercase()))
        })
        .map(|node| node.node_id.clone())
        .collect()
}

fn edge_matches(
    edge: &AcademicGraphEdgeV1,
    selected_nodes: &BTreeSet<String>,
    filters: &PortfolioQueryFiltersV1,
) -> bool {
    let node_filter = filters.manuscript_section.is_some()
        || filters.evidence_signal.is_some()
        || filters.shared_identity.is_some()
        || filters.text.is_some();
    (!node_filter
        || selected_nodes.contains(&edge.source_node_id)
        || selected_nodes.contains(&edge.target_node_id))
        && filters.evidence_signal.is_none_or(|signal| match signal {
            PortfolioEvidenceSignal::Gap => true,
            PortfolioEvidenceSignal::Contradiction => {
                edge.relation == AcademicGraphRelation::Contradicts
            }
        })
}

fn graph_has_signal(
    graph: &crate::AcademicGraphSnapshotV1,
    signal: PortfolioEvidenceSignal,
) -> bool {
    match signal {
        PortfolioEvidenceSignal::Gap => graph
            .nodes
            .iter()
            .any(|node| node.node_type == AcademicGraphNodeType::Gap),
        PortfolioEvidenceSignal::Contradiction => graph
            .edges
            .iter()
            .any(|edge| edge.relation == AcademicGraphRelation::Contradicts),
    }
}

fn signal_node_ids(
    graph: &crate::AcademicGraphSnapshotV1,
    signal: PortfolioEvidenceSignal,
) -> BTreeSet<String> {
    match signal {
        PortfolioEvidenceSignal::Gap => graph
            .nodes
            .iter()
            .filter(|node| node.node_type == AcademicGraphNodeType::Gap)
            .map(|node| node.node_id.clone())
            .collect(),
        PortfolioEvidenceSignal::Contradiction => graph
            .edges
            .iter()
            .filter(|edge| edge.relation == AcademicGraphRelation::Contradicts)
            .flat_map(|edge| [edge.source_node_id.clone(), edge.target_node_id.clone()])
            .collect(),
    }
}

fn graph_has_section(graph: &crate::AcademicGraphSnapshotV1, section: &str) -> bool {
    !section_node_ids(graph, section).is_empty()
}

fn section_node_ids(graph: &crate::AcademicGraphSnapshotV1, section: &str) -> BTreeSet<String> {
    let section_lower = section.to_lowercase();
    let direct = graph
        .nodes
        .iter()
        .filter(|node| {
            node.node_type == AcademicGraphNodeType::ManuscriptSection
                && (node.canonical_id == section || node.label.to_lowercase() == section_lower)
        })
        .map(|node| node.node_id.clone())
        .collect::<BTreeSet<_>>();
    let mut selected = direct.clone();
    for edge in graph
        .edges
        .iter()
        .filter(|edge| edge.relation == AcademicGraphRelation::AppearsInSection)
    {
        if direct.contains(&edge.source_node_id) || direct.contains(&edge.target_node_id) {
            selected.insert(edge.source_node_id.clone());
            selected.insert(edge.target_node_id.clone());
        }
    }
    selected
}

fn graph_has_shared_identity(
    contribution: &PortfolioContributionV1,
    identity: &PortfolioSharedIdentityFilterV1,
) -> bool {
    contribution.graph.nodes.iter().any(|node| {
        node.node_type == identity.node_type
            && node.identity_scope == AcademicGraphIdentityScope::Global
            && node.canonical_id == identity.canonical_id
    })
}

fn lineage_matches(record: &PortfolioLineageRecordV1, filters: &PortfolioQueryFiltersV1) -> bool {
    filters
        .capture_source
        .is_none_or(|source| record.source == Some(source))
        && filters
            .capture_delivery
            .is_none_or(|delivery| record.delivery == Some(delivery))
        && filters
            .delivery_state
            .is_none_or(|state| record.delivery_state == Some(state))
        && filters
            .assignment_outcome
            .is_none_or(|outcome| record.assignment_outcome == Some(outcome))
        && filters
            .lineage_id
            .as_deref()
            .is_none_or(|id| record.lineage_id == id || record.related_ids.iter().any(|v| v == id))
}

fn lineage_counts(records: &[PortfolioLineageRecordV1]) -> BTreeMap<ProjectId, usize> {
    let mut counts = BTreeMap::new();
    for record in records {
        for project_id in &record.project_ids {
            *counts.entry(project_id.clone()).or_default() += 1;
        }
    }
    counts
}

struct Page<T> {
    matched_count: usize,
    truncated: bool,
    next_after: Option<String>,
    items: Vec<T>,
}

fn paginate<T: Clone + Serialize>(
    items: Vec<T>,
    after: Option<&str>,
    limit: usize,
    byte_budget: usize,
    key: impl Fn(&T) -> &str,
) -> Result<Page<T>, ProjectError> {
    let matched_count = items.len();
    let candidates = items
        .into_iter()
        .filter(|item| after.is_none_or(|after| key(item) > after))
        .collect::<Vec<_>>();
    let mut selected = Vec::new();
    let mut used_bytes = 0usize;
    for item in &candidates {
        if selected.len() == limit {
            break;
        }
        let item_bytes = serde_json_canonicalizer::to_vec(item)
            .map_err(|_| ProjectError::InvalidPortfolioQuery)?
            .len();
        if used_bytes
            .checked_add(item_bytes)
            .is_none_or(|next| next > byte_budget)
        {
            break;
        }
        used_bytes += item_bytes;
        selected.push(item.clone());
    }
    let truncated = selected.len() < candidates.len();
    if truncated && selected.is_empty() && !candidates.is_empty() {
        return Err(ProjectError::DocumentTooLarge);
    }
    let next_after = selected
        .last()
        .map(|item| key(item).to_string())
        .or_else(|| after.map(str::to_string));
    Ok(Page {
        matched_count,
        truncated,
        next_after,
        items: selected,
    })
}

fn valid_filter_text(value: &str, maximum_bytes: usize) -> bool {
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
    let bytes =
        serde_json_canonicalizer::to_vec(value).map_err(|_| ProjectError::InvalidPortfolioQuery)?;
    let mut digest = Sha256::new();
    digest.update(domain);
    digest.update(bytes);
    Ok(format!("{prefix}{:x}", digest.finalize()))
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    use qiongli_config::{ConfigRoot, resolve_config_root};
    use serde_json::json;

    use super::*;
    use crate::{
        AcademicGraphConfidence, AcademicGraphEdgeStatus, AcademicGraphLayer,
        AcademicInferenceStrength, ApprovedCaptureAssignment, ApprovedCaptureIntake,
        ApprovedProjectMutation, CaptureArea, CaptureAssignmentDecision, CaptureDeliveryEnvelopeV1,
        CapturePolicy, DecisionCandidateV1, DecisionRelation, EvidenceLocatorKind,
        EvidenceReferenceV1, ProjectBindingV1, ProjectKind, ProjectRegistrationOptions,
        ResearchCaptureDraftV1, SemanticChangeV1, VerifiedProjectMutation,
    };

    static NEXT_FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

    struct Fixture {
        root: PathBuf,
        _config: ConfigRoot,
        projects: ProjectStateService,
        incremental: IncrementalPortfolioService,
        query: PortfolioQueryService,
    }

    impl Fixture {
        fn new() -> Self {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time is available")
                .as_nanos();
            let root = std::env::temp_dir().join(format!(
                "qiongli-portfolio-query-{}-{nonce}-{}",
                std::process::id(),
                NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&root).expect("fixture root can be created");
            let root = fs::canonicalize(root).expect("fixture root can be canonicalized");
            let home = root.join("home");
            fs::create_dir(&home).expect("fixture home can be created");
            let config = resolve_config_root(Some(root.join("config").as_os_str()), &home)
                .expect("config is valid");
            let projects = ProjectStateService::new(config.clone());
            Self {
                root,
                _config: config,
                incremental: IncrementalPortfolioService::new(projects.clone()),
                query: PortfolioQueryService::new(projects.clone()),
                projects,
            }
        }

        fn create_project(&self, name: &str, now_unix: u64) -> (ProjectId, PathBuf) {
            let project_root = self.root.join(name.to_lowercase().replace(' ', "-"));
            let plan = self
                .projects
                .preview_create(
                    &project_root,
                    ProjectRegistrationOptions::new(name, ProjectKind::Article),
                    now_unix,
                )
                .expect("project create can be previewed");
            let project_id = plan.preview().project_id.clone();
            self.apply(&plan, now_unix);
            (project_id, project_root)
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

        fn refresh(&self, project_id: &ProjectId, now_unix: u64) {
            let plan = self
                .projects
                .preview_refresh(project_id, now_unix)
                .expect("refresh can be previewed");
            self.apply(&plan, now_unix);
        }

        fn write_graph(
            &self,
            project_id: &ProjectId,
            project_root: &Path,
            paper_label: &str,
            include_signals: bool,
        ) {
            let paper = AcademicGraphNodeV1::new(
                project_id,
                AcademicGraphNodeType::Paper,
                AcademicGraphIdentityScope::Global,
                "doi:10.1000/shared-query",
                paper_label,
                vec![AcademicGraphLayer::Literature],
                "graph/semantic_links.jsonl",
                "paper:shared",
            )
            .expect("paper node is valid");
            let mut records = vec![node_record(project_id, &paper)];
            if include_signals {
                let claim = AcademicGraphNodeV1::new(
                    project_id,
                    AcademicGraphNodeType::Claim,
                    AcademicGraphIdentityScope::Project,
                    "claim:query",
                    "Contradicted portfolio claim",
                    vec![AcademicGraphLayer::Argument, AcademicGraphLayer::Manuscript],
                    "graph/semantic_links.jsonl",
                    "claim:query",
                )
                .expect("claim node is valid");
                let evidence = AcademicGraphNodeV1::new(
                    project_id,
                    AcademicGraphNodeType::Evidence,
                    AcademicGraphIdentityScope::Project,
                    "evidence:query",
                    "Counterevidence",
                    vec![AcademicGraphLayer::Argument],
                    "graph/semantic_links.jsonl",
                    "evidence:query",
                )
                .expect("evidence node is valid");
                let gap = AcademicGraphNodeV1::new(
                    project_id,
                    AcademicGraphNodeType::Gap,
                    AcademicGraphIdentityScope::Project,
                    "gap:query",
                    "Unresolved evidence gap",
                    vec![AcademicGraphLayer::Argument],
                    "graph/semantic_links.jsonl",
                    "gap:query",
                )
                .expect("gap node is valid");
                let section = AcademicGraphNodeV1::new(
                    project_id,
                    AcademicGraphNodeType::ManuscriptSection,
                    AcademicGraphIdentityScope::Project,
                    "section:results",
                    "Results",
                    vec![AcademicGraphLayer::Manuscript],
                    "graph/semantic_links.jsonl",
                    "section:results",
                )
                .expect("section node is valid");
                let contradiction = AcademicGraphEdgeV1::new(
                    project_id,
                    &evidence.node_id,
                    AcademicGraphRelation::Contradicts,
                    &claim.node_id,
                    vec![AcademicGraphLayer::Argument],
                    "The exact reviewed counterevidence contradicts this claim.",
                    "graph/semantic_links.jsonl",
                    "edge:contradiction",
                    "One reviewed counterexample; scope remains bounded.",
                    AcademicInferenceStrength::DirectEvidence,
                    AcademicGraphConfidence::High,
                    AcademicGraphEdgeStatus::Reviewed,
                    None,
                )
                .expect("contradiction edge is valid");
                let appears = AcademicGraphEdgeV1::new(
                    project_id,
                    &claim.node_id,
                    AcademicGraphRelation::AppearsInSection,
                    &section.node_id,
                    vec![AcademicGraphLayer::Manuscript],
                    "The claim is mapped to the Results section.",
                    "graph/semantic_links.jsonl",
                    "edge:section",
                    "Section placement does not establish evidential support.",
                    AcademicInferenceStrength::DirectEvidence,
                    AcademicGraphConfidence::High,
                    AcademicGraphEdgeStatus::Reviewed,
                    None,
                )
                .expect("section edge is valid");
                records.extend([
                    node_record(project_id, &claim),
                    node_record(project_id, &evidence),
                    node_record(project_id, &gap),
                    node_record(project_id, &section),
                    edge_record(project_id, &contradiction),
                    edge_record(project_id, &appears),
                ]);
            }
            fs::create_dir_all(project_root.join("graph")).expect("graph directory can be created");
            let mut bytes = Vec::new();
            for record in records {
                bytes.extend(serde_json::to_vec(&record).expect("record serializes"));
                bytes.push(b'\n');
            }
            fs::write(project_root.join("graph/semantic_links.jsonl"), bytes)
                .expect("semantic graph can be written");
        }

        fn add_capture_and_rejected_assignment(
            &self,
            source_project: &ProjectId,
            target_project: &ProjectId,
        ) -> (String, String) {
            let source = self
                .projects
                .snapshot()
                .expect("library is readable")
                .projects
                .into_iter()
                .find(|project| &project.project_id == source_project)
                .expect("source project is present");
            let capture = ResearchCaptureDraftV1 {
                binding: ProjectBindingV1::new(
                    source_project.clone(),
                    source.semantic_revision,
                    source.stage,
                    "Inspect the shared evidence",
                    CapturePolicy::ReviewRequired,
                )
                .expect("binding is valid"),
                source: CaptureSource::Codex,
                delivery: CaptureDelivery::Connected,
                captured_at_unix: 10,
                summary: "A source-bound capture for portfolio query tests.".to_string(),
                changes: vec![SemanticChangeV1 {
                    area: CaptureArea::Evidence,
                    summary: "Review the counterevidence.".to_string(),
                }],
                decisions: vec![DecisionCandidateV1 {
                    relation: DecisionRelation::Candidate,
                    statement: "Retain the bounded contradiction.".to_string(),
                    rationale: "The source remains limited.".to_string(),
                    target: None,
                }],
                evidence: vec![EvidenceReferenceV1 {
                    locator_kind: EvidenceLocatorKind::Doi,
                    locator: "10.1000/query-capture".to_string(),
                    relevance: "Provides the exact query lineage fixture.".to_string(),
                    limitation: Some("Fixture evidence only.".to_string()),
                }],
                contradictions: Vec::new(),
                next_actions: vec!["Review the capture.".to_string()],
            }
            .into_capture()
            .expect("capture is valid");
            let intake = self
                .projects
                .preview_capture(capture.clone())
                .expect("capture can be previewed");
            self.projects
                .apply_capture(
                    &intake,
                    &ApprovedCaptureIntake::new(intake.preview().plan_digest.clone(), true),
                    11,
                )
                .expect("capture can be accepted");
            let envelope = CaptureDeliveryEnvelopeV1::new(capture, None, 12)
                .expect("delivery envelope is valid");
            let envelope_id = envelope.envelope_id.as_str().to_string();
            self.projects
                .enqueue_capture_delivery(envelope)
                .expect("delivery can be queued");
            let assignment = self
                .projects
                .preview_capture_assignment(
                    &crate::DeliveryEnvelopeId::parse(envelope_id.clone())
                        .expect("envelope id is valid"),
                    target_project,
                    CaptureAssignmentDecision::Reject,
                    13,
                )
                .expect("assignment can be previewed");
            let commit = self
                .projects
                .apply_capture_assignment(
                    &assignment,
                    &ApprovedCaptureAssignment::new(assignment.preview().plan_digest.clone(), true),
                )
                .expect("assignment can be rejected");
            (envelope_id, commit.receipt_id.as_str().to_string())
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    fn node_record(project_id: &ProjectId, node: &AcademicGraphNodeV1) -> serde_json::Value {
        json!({
            "schema_version": 1,
            "document_kind": "qiongli-academic-graph-node",
            "project_id": project_id,
            "node_id": node.node_id,
            "node_type": node.node_type,
            "identity_scope": node.identity_scope,
            "canonical_id": node.canonical_id,
            "label": node.label,
            "layers": node.layers,
            "artifact_path": node.artifact_path,
            "source_anchor": node.source_anchor,
        })
    }

    fn edge_record(project_id: &ProjectId, edge: &AcademicGraphEdgeV1) -> serde_json::Value {
        json!({
            "schema_version": 1,
            "document_kind": "qiongli-academic-semantic-link",
            "project_id": project_id,
            "edge_id": edge.edge_id,
            "source_node_id": edge.source_node_id,
            "relation": edge.relation,
            "target_node_id": edge.target_node_id,
            "layers": edge.layers,
            "rationale": edge.rationale,
            "artifact_path": edge.artifact_path,
            "source_anchor": edge.source_anchor,
            "evidence_limit": edge.evidence_limit,
            "inference_strength": edge.inference_strength,
            "confidence": edge.confidence,
            "status": edge.status,
            "created_from_capture": edge.created_from_capture,
        })
    }

    fn prepared_fixture() -> (Fixture, ProjectId, ProjectId, String) {
        let fixture = Fixture::new();
        let (project_a, root_a) = fixture.create_project("Query A", 1);
        let (project_b, root_b) = fixture.create_project("Query B", 2);
        fixture.write_graph(&project_a, &root_a, "Shared paper A", true);
        fixture.refresh(&project_a, 3);
        fixture.write_graph(&project_b, &root_b, "Shared paper B", false);
        fixture.refresh(&project_b, 4);
        let reconciled = fixture
            .incremental
            .reconcile(5)
            .expect("catalog can be reconciled");
        (
            fixture,
            project_a,
            project_b,
            reconciled.snapshot.catalog.catalog_id,
        )
    }

    #[test]
    fn query_filters_exact_graph_identity_signal_section_and_text_without_merging_labels() {
        let (fixture, project_a, _project_b, catalog_id) = prepared_fixture();
        let shared = PortfolioQueryV1::new(catalog_id.clone())
            .expect("query is valid")
            .with_filters(PortfolioQueryFiltersV1 {
                shared_identity: Some(PortfolioSharedIdentityFilterV1 {
                    node_type: AcademicGraphNodeType::Paper,
                    canonical_id: "doi:10.1000/shared-query".to_string(),
                }),
                ..PortfolioQueryFiltersV1::default()
            })
            .expect("shared query is valid");
        let shared_result = fixture.query.query(&shared).expect("shared query succeeds");
        assert_eq!(shared_result.matched_project_count, 2);
        assert_eq!(shared_result.matched_node_count, 2);
        assert!(
            shared_result
                .nodes
                .iter()
                .all(|node| node.node.canonical_id == "doi:10.1000/shared-query")
        );
        assert_ne!(
            shared_result.nodes[0].node.label,
            shared_result.nodes[1].node.label
        );

        for (filters, expected_node_type) in [
            (
                PortfolioQueryFiltersV1 {
                    evidence_signal: Some(PortfolioEvidenceSignal::Gap),
                    ..PortfolioQueryFiltersV1::default()
                },
                AcademicGraphNodeType::Gap,
            ),
            (
                PortfolioQueryFiltersV1 {
                    manuscript_section: Some("section:results".to_string()),
                    ..PortfolioQueryFiltersV1::default()
                },
                AcademicGraphNodeType::ManuscriptSection,
            ),
            (
                PortfolioQueryFiltersV1 {
                    evidence_signal: Some(PortfolioEvidenceSignal::Contradiction),
                    ..PortfolioQueryFiltersV1::default()
                },
                AcademicGraphNodeType::Evidence,
            ),
        ] {
            let query = PortfolioQueryV1::new(catalog_id.clone())
                .expect("query is valid")
                .with_filters(filters)
                .expect("filters are valid");
            let result = fixture.query.query(&query).expect("query succeeds");
            assert_eq!(result.projects[0].project_id, project_a);
            assert!(
                result
                    .nodes
                    .iter()
                    .any(|node| node.node.node_type == expected_node_type)
            );
        }

        let text = PortfolioQueryV1::new(catalog_id)
            .expect("query is valid")
            .with_filters(PortfolioQueryFiltersV1 {
                text: Some("shared paper a".to_string()),
                ..PortfolioQueryFiltersV1::default()
            })
            .expect("text filter is valid");
        let text_result = fixture.query.query(&text).expect("text query succeeds");
        assert_eq!(text_result.matched_project_count, 2);
        assert_eq!(text_result.matched_node_count, 1);
        assert_eq!(text_result.nodes[0].project_id, project_a);
        assert!(
            !serde_json::to_string(&text_result)
                .expect("result serializes")
                .contains(fixture.root.to_string_lossy().as_ref())
        );
    }

    #[test]
    fn query_joins_capture_delivery_and_assignment_only_by_exact_lineage_ids() {
        let (fixture, project_a, project_b, catalog_id) = prepared_fixture();
        let (envelope_id, assignment_receipt_id) =
            fixture.add_capture_and_rejected_assignment(&project_a, &project_b);

        for (filters, expected_project) in [
            (
                PortfolioQueryFiltersV1 {
                    capture_source: Some(CaptureSource::Codex),
                    ..PortfolioQueryFiltersV1::default()
                },
                project_a.clone(),
            ),
            (
                PortfolioQueryFiltersV1 {
                    delivery_state: Some(CaptureDeliveryState::Cancelled),
                    ..PortfolioQueryFiltersV1::default()
                },
                project_a.clone(),
            ),
            (
                PortfolioQueryFiltersV1 {
                    assignment_outcome: Some(CaptureAssignmentOutcome::Rejected),
                    ..PortfolioQueryFiltersV1::default()
                },
                project_b.clone(),
            ),
            (
                PortfolioQueryFiltersV1 {
                    capture_source: Some(CaptureSource::Codex),
                    assignment_outcome: Some(CaptureAssignmentOutcome::Rejected),
                    ..PortfolioQueryFiltersV1::default()
                },
                project_b.clone(),
            ),
            (
                PortfolioQueryFiltersV1 {
                    lineage_id: Some(assignment_receipt_id.clone()),
                    ..PortfolioQueryFiltersV1::default()
                },
                project_b,
            ),
        ] {
            let query = PortfolioQueryV1::new(catalog_id.clone())
                .expect("query is valid")
                .with_filters(filters)
                .expect("filters are valid");
            let result = fixture.query.query(&query).expect("lineage query succeeds");
            assert!(
                result
                    .projects
                    .iter()
                    .any(|project| project.project_id == expected_project)
            );
        }
        let envelope_query = PortfolioQueryV1::new(catalog_id)
            .expect("query is valid")
            .with_filters(PortfolioQueryFiltersV1 {
                lineage_id: Some(envelope_id),
                ..PortfolioQueryFiltersV1::default()
            })
            .expect("lineage filter is valid");
        let result = fixture
            .query
            .query(&envelope_query)
            .expect("envelope query succeeds");
        let kinds = result
            .lineage
            .iter()
            .map(|record| record.kind)
            .collect::<BTreeSet<_>>();
        assert_eq!(
            kinds,
            [
                PortfolioLineageKind::Delivery,
                PortfolioLineageKind::Assignment
            ]
            .into_iter()
            .collect()
        );
    }

    #[test]
    fn query_cursor_is_content_bound_bounded_and_rejects_lineage_drift() {
        let (fixture, project_a, project_b, catalog_id) = prepared_fixture();
        let query = PortfolioQueryV1::new(catalog_id)
            .expect("query is valid")
            .with_limits(PortfolioQueryLimitsV1 {
                projects: 1,
                nodes: 1,
                edges: 1,
                lineage: 1,
                max_bytes: MIN_QUERY_RESULT_BYTES,
            })
            .expect("limits are valid");
        let first = fixture.query.query(&query).expect("first page succeeds");
        let cursor = first.next_cursor.clone().expect("first page is truncated");
        let second_query = query.clone().with_cursor(cursor).expect("cursor is valid");
        let second = fixture
            .query
            .query(&second_query)
            .expect("second page succeeds");
        let first_ids = first
            .projects
            .iter()
            .map(|item| item.result_id.clone())
            .chain(first.nodes.iter().map(|item| item.result_id.clone()))
            .chain(first.edges.iter().map(|item| item.result_id.clone()))
            .collect::<BTreeSet<_>>();
        assert!(
            second
                .projects
                .iter()
                .map(|item| &item.result_id)
                .chain(second.nodes.iter().map(|item| &item.result_id))
                .chain(second.edges.iter().map(|item| &item.result_id))
                .all(|id| !first_ids.contains(id))
        );
        assert!(
            serde_json_canonicalizer::to_vec(&first)
                .expect("result serializes")
                .len()
                <= MIN_QUERY_RESULT_BYTES
        );

        let _ = fixture.add_capture_and_rejected_assignment(&project_a, &project_b);
        assert_eq!(
            fixture.query.query(&second_query).unwrap_err(),
            ProjectError::InvalidPortfolioQuery
        );
    }

    #[test]
    fn query_contract_rejects_unknown_fields_noncanonical_json_and_wrong_catalog() {
        let (fixture, _project_a, _project_b, catalog_id) = prepared_fixture();
        let query = PortfolioQueryV1::new(catalog_id).expect("query is valid");
        let bytes = query.to_canonical_json().expect("query serializes");
        let mut value: serde_json::Value = serde_json::from_slice(&bytes).expect("query is JSON");
        value
            .as_object_mut()
            .expect("query is an object")
            .insert("projectRoot".to_string(), json!("/private/project"));
        assert_eq!(
            PortfolioQueryV1::from_json_slice(
                &serde_json::to_vec(&value).expect("tampered query serializes")
            )
            .unwrap_err(),
            ProjectError::InvalidPortfolioQuery
        );
        let pretty = serde_json::to_vec_pretty(&query).expect("pretty query serializes");
        assert_eq!(
            PortfolioQueryV1::from_json_slice(&pretty).unwrap_err(),
            ProjectError::InvalidPortfolioQuery
        );
        let wrong = PortfolioQueryV1::new(format!("pca_{}", "f".repeat(64)))
            .expect("shape-valid catalog query");
        assert_eq!(
            fixture.query.query(&wrong).unwrap_err(),
            ProjectError::PortfolioCatalogConflict
        );
    }
}
