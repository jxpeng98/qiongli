use std::fmt::{self, Debug, Formatter};
use std::path::{Component, Path};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use url::Url;

use crate::ProjectError;
use crate::model::{MAX_SEMANTIC_REVISION, ProjectId, ProjectStage, valid_lower_hex};

pub const RESEARCH_CAPTURE_SCHEMA_VERSION: u32 = 1;
pub const RESEARCH_CAPTURE_DOCUMENT_KIND: &str = "qiongli-research-capture";
pub const PROJECT_BINDING_SCHEMA_VERSION: u32 = 1;
pub const PROJECT_BINDING_DOCUMENT_KIND: &str = "qiongli-project-binding";
pub const CAPTURE_ID_PREFIX: &str = "cap_";

const MAX_CAPTURE_BYTES: usize = 64 * 1024;
const MAX_TASK_BYTES: usize = 300;
const MAX_SUMMARY_BYTES: usize = 2_000;
const MAX_ITEM_TEXT_BYTES: usize = 1_000;
const MAX_LOCATOR_BYTES: usize = 500;
const MAX_ITEMS_PER_FIELD: usize = 16;

#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CaptureId(String);

impl CaptureId {
    pub fn parse(value: impl Into<String>) -> Result<Self, ProjectError> {
        let value = value.into();
        if value.len() != CAPTURE_ID_PREFIX.len() + 64
            || !value.starts_with(CAPTURE_ID_PREFIX)
            || !valid_lower_hex(&value[CAPTURE_ID_PREFIX.len()..], 64)
        {
            return Err(ProjectError::InvalidCaptureDocument);
        }
        Ok(Self(value))
    }

    fn from_digest(digest: &[u8]) -> Self {
        let mut value = String::with_capacity(CAPTURE_ID_PREFIX.len() + digest.len() * 2);
        value.push_str(CAPTURE_ID_PREFIX);
        for byte in digest {
            use std::fmt::Write as _;
            let _ = write!(value, "{byte:02x}");
        }
        Self(value)
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    fn validate(&self) -> Result<(), ProjectError> {
        Self::parse(self.0.clone()).map(|_| ())
    }
}

impl Debug for CaptureId {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.debug_tuple("CaptureId").field(&self.0).finish()
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CapturePolicy {
    ReviewRequired,
    HistoryOnly,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectBindingV1 {
    pub schema_version: u32,
    pub document_kind: String,
    pub project_id: ProjectId,
    pub base_revision: u64,
    pub stage: ProjectStage,
    pub task: String,
    pub capture_policy: CapturePolicy,
}

impl ProjectBindingV1 {
    pub fn new(
        project_id: ProjectId,
        base_revision: u64,
        stage: ProjectStage,
        task: impl Into<String>,
        capture_policy: CapturePolicy,
    ) -> Result<Self, ProjectError> {
        let binding = Self {
            schema_version: PROJECT_BINDING_SCHEMA_VERSION,
            document_kind: PROJECT_BINDING_DOCUMENT_KIND.to_string(),
            project_id,
            base_revision,
            stage,
            task: task.into(),
            capture_policy,
        };
        binding.validate()?;
        Ok(binding)
    }

    pub fn validate(&self) -> Result<(), ProjectError> {
        if self.schema_version != PROJECT_BINDING_SCHEMA_VERSION
            || self.document_kind != PROJECT_BINDING_DOCUMENT_KIND
            || self.base_revision == 0
            || self.base_revision > MAX_SEMANTIC_REVISION
            || !valid_text(&self.task, MAX_TASK_BYTES)
        {
            return Err(ProjectError::InvalidCaptureDocument);
        }
        self.project_id
            .validate()
            .map_err(|_| ProjectError::InvalidCaptureDocument)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CaptureSource {
    Codex,
    ClaudeCode,
    ChatGpt,
    Cli,
    Manual,
    Repository,
    PortableFile,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CaptureDelivery {
    Connected,
    RepositoryBacked,
    Portable,
    Manual,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CaptureArea {
    ResearchQuestion,
    Thesis,
    Literature,
    Method,
    Evidence,
    Analysis,
    Manuscript,
    Scope,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SemanticChangeV1 {
    pub area: CaptureArea,
    pub summary: String,
}

impl SemanticChangeV1 {
    fn validate(&self) -> Result<(), ProjectError> {
        valid_text(&self.summary, MAX_ITEM_TEXT_BYTES)
            .then_some(())
            .ok_or(ProjectError::InvalidCaptureDocument)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DecisionRelation {
    Candidate,
    Refinement,
    Challenge,
    Supersession,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionCandidateV1 {
    pub relation: DecisionRelation,
    pub statement: String,
    pub rationale: String,
    pub target: Option<String>,
}

impl DecisionCandidateV1 {
    fn validate(&self) -> Result<(), ProjectError> {
        if !valid_text(&self.statement, MAX_ITEM_TEXT_BYTES)
            || !valid_text(&self.rationale, MAX_ITEM_TEXT_BYTES)
            || self
                .target
                .as_deref()
                .is_some_and(|value| !valid_text(value, MAX_ITEM_TEXT_BYTES))
        {
            return Err(ProjectError::InvalidCaptureDocument);
        }
        if self.relation == DecisionRelation::Candidate && self.target.is_some() {
            return Err(ProjectError::InvalidCaptureDocument);
        }
        if self.relation != DecisionRelation::Candidate && self.target.is_none() {
            return Err(ProjectError::InvalidCaptureDocument);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum EvidenceLocatorKind {
    Doi,
    CitationKey,
    HttpsUrl,
    ArtifactAnchor,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceReferenceV1 {
    pub locator_kind: EvidenceLocatorKind,
    pub locator: String,
    pub relevance: String,
    pub limitation: Option<String>,
}

impl EvidenceReferenceV1 {
    fn validate(&self) -> Result<(), ProjectError> {
        if !valid_text(&self.locator, MAX_LOCATOR_BYTES)
            || !valid_text(&self.relevance, MAX_ITEM_TEXT_BYTES)
            || self
                .limitation
                .as_deref()
                .is_some_and(|value| !valid_text(value, MAX_ITEM_TEXT_BYTES))
            || !valid_locator(self.locator_kind, &self.locator)
        {
            return Err(ProjectError::InvalidCaptureDocument);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ContradictionV1 {
    pub statement: String,
    pub conflicts_with: String,
    pub consequence: String,
}

impl ContradictionV1 {
    fn validate(&self) -> Result<(), ProjectError> {
        [
            self.statement.as_str(),
            self.conflicts_with.as_str(),
            self.consequence.as_str(),
        ]
        .into_iter()
        .all(|value| valid_text(value, MAX_ITEM_TEXT_BYTES))
        .then_some(())
        .ok_or(ProjectError::InvalidCaptureDocument)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResearchCaptureDraftV1 {
    pub binding: ProjectBindingV1,
    pub source: CaptureSource,
    pub delivery: CaptureDelivery,
    pub captured_at_unix: u64,
    pub summary: String,
    pub changes: Vec<SemanticChangeV1>,
    pub decisions: Vec<DecisionCandidateV1>,
    pub evidence: Vec<EvidenceReferenceV1>,
    pub contradictions: Vec<ContradictionV1>,
    pub next_actions: Vec<String>,
}

impl ResearchCaptureDraftV1 {
    pub fn validate(&self) -> Result<(), ProjectError> {
        self.binding.validate()?;
        if self.captured_at_unix > MAX_SEMANTIC_REVISION
            || !valid_text(&self.summary, MAX_SUMMARY_BYTES)
            || !valid_collection(&self.changes, SemanticChangeV1::validate)
            || !valid_collection(&self.decisions, DecisionCandidateV1::validate)
            || !valid_collection(&self.evidence, EvidenceReferenceV1::validate)
            || !valid_collection(&self.contradictions, ContradictionV1::validate)
            || self.next_actions.len() > MAX_ITEMS_PER_FIELD
            || self
                .next_actions
                .iter()
                .any(|value| !valid_text(value, MAX_ITEM_TEXT_BYTES))
        {
            return Err(ProjectError::InvalidCaptureDocument);
        }
        let bytes = serde_json_canonicalizer::to_vec(self)
            .map_err(|_| ProjectError::InvalidCaptureDocument)?;
        if bytes.len() > MAX_CAPTURE_BYTES {
            return Err(ProjectError::InvalidCaptureDocument);
        }
        Ok(())
    }

    pub fn into_capture(self) -> Result<ResearchCaptureV1, ProjectError> {
        self.validate()?;
        let capture_id = capture_id(&self)?;
        let capture = ResearchCaptureV1 {
            schema_version: RESEARCH_CAPTURE_SCHEMA_VERSION,
            document_kind: RESEARCH_CAPTURE_DOCUMENT_KIND.to_string(),
            capture_id,
            binding: self.binding,
            source: self.source,
            delivery: self.delivery,
            captured_at_unix: self.captured_at_unix,
            summary: self.summary,
            changes: self.changes,
            decisions: self.decisions,
            evidence: self.evidence,
            contradictions: self.contradictions,
            next_actions: self.next_actions,
        };
        capture.validate()?;
        Ok(capture)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResearchCaptureV1 {
    pub schema_version: u32,
    pub document_kind: String,
    pub capture_id: CaptureId,
    pub binding: ProjectBindingV1,
    pub source: CaptureSource,
    pub delivery: CaptureDelivery,
    pub captured_at_unix: u64,
    pub summary: String,
    pub changes: Vec<SemanticChangeV1>,
    pub decisions: Vec<DecisionCandidateV1>,
    pub evidence: Vec<EvidenceReferenceV1>,
    pub contradictions: Vec<ContradictionV1>,
    pub next_actions: Vec<String>,
}

impl ResearchCaptureV1 {
    pub fn validate(&self) -> Result<(), ProjectError> {
        if self.schema_version != RESEARCH_CAPTURE_SCHEMA_VERSION
            || self.document_kind != RESEARCH_CAPTURE_DOCUMENT_KIND
        {
            return Err(ProjectError::InvalidCaptureDocument);
        }
        self.capture_id.validate()?;
        let draft = ResearchCaptureDraftV1 {
            binding: self.binding.clone(),
            source: self.source,
            delivery: self.delivery,
            captured_at_unix: self.captured_at_unix,
            summary: self.summary.clone(),
            changes: self.changes.clone(),
            decisions: self.decisions.clone(),
            evidence: self.evidence.clone(),
            contradictions: self.contradictions.clone(),
            next_actions: self.next_actions.clone(),
        };
        draft.validate()?;
        if capture_id(&draft)? != self.capture_id {
            return Err(ProjectError::CaptureIdentityConflict);
        }
        let bytes = serde_json_canonicalizer::to_vec(self)
            .map_err(|_| ProjectError::InvalidCaptureDocument)?;
        (bytes.len() <= MAX_CAPTURE_BYTES)
            .then_some(())
            .ok_or(ProjectError::InvalidCaptureDocument)
    }
}

fn capture_id(draft: &ResearchCaptureDraftV1) -> Result<CaptureId, ProjectError> {
    let bytes = serde_json_canonicalizer::to_vec(draft)
        .map_err(|_| ProjectError::InvalidCaptureDocument)?;
    Ok(CaptureId::from_digest(&Sha256::digest(bytes)))
}

fn valid_collection<T>(values: &[T], validate: impl Fn(&T) -> Result<(), ProjectError>) -> bool {
    values.len() <= MAX_ITEMS_PER_FIELD && values.iter().all(|value| validate(value).is_ok())
}

fn valid_text(value: &str, max_bytes: usize) -> bool {
    !value.is_empty()
        && value.len() <= max_bytes
        && value.trim() == value
        && !value.chars().any(char::is_control)
}

fn valid_locator(kind: EvidenceLocatorKind, value: &str) -> bool {
    match kind {
        EvidenceLocatorKind::Doi => {
            value.starts_with("10.")
                && value.contains('/')
                && value.bytes().all(|byte| !byte.is_ascii_whitespace())
        }
        EvidenceLocatorKind::CitationKey => value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':' | b'/')
        }),
        EvidenceLocatorKind::HttpsUrl => Url::parse(value).is_ok_and(|url| {
            url.scheme() == "https"
                && url.host_str().is_some()
                && url.username().is_empty()
                && url.password().is_none()
        }),
        EvidenceLocatorKind::ArtifactAnchor => valid_artifact_anchor(value),
    }
}

fn valid_artifact_anchor(value: &str) -> bool {
    let path = value.split_once('#').map_or(value, |(path, _)| path);
    if path.is_empty()
        || path.starts_with(['/', '\\', '~'])
        || path.contains(['\\', ':'])
        || path.ends_with('/')
    {
        return false;
    }
    Path::new(path)
        .components()
        .all(|component| matches!(component, Component::Normal(value) if !value.is_empty()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_draft() -> ResearchCaptureDraftV1 {
        ResearchCaptureDraftV1 {
            binding: ProjectBindingV1::new(
                ProjectId::parse("prj_0123456789abcdef0123456789abcdef").unwrap(),
                3,
                ProjectStage::Literature,
                "Reconcile the methods literature",
                CapturePolicy::ReviewRequired,
            )
            .unwrap(),
            source: CaptureSource::Codex,
            delivery: CaptureDelivery::Connected,
            captured_at_unix: 1_721_337_600,
            summary: "The measurement literature separates construct validity from reliability."
                .to_string(),
            changes: vec![SemanticChangeV1 {
                area: CaptureArea::Literature,
                summary: "Split the measurement cluster into validity and reliability streams."
                    .to_string(),
            }],
            decisions: vec![DecisionCandidateV1 {
                relation: DecisionRelation::Candidate,
                statement: "Use construct validity as the organizing distinction.".to_string(),
                rationale: "It explains the disagreement between the two source clusters."
                    .to_string(),
                target: None,
            }],
            evidence: vec![EvidenceReferenceV1 {
                locator_kind: EvidenceLocatorKind::Doi,
                locator: "10.1000/example".to_string(),
                relevance: "Defines the construct-validity distinction.".to_string(),
                limitation: Some("Conceptual rather than empirical evidence.".to_string()),
            }],
            contradictions: Vec::new(),
            next_actions: vec![
                "Check whether the distinction survives the empirical papers.".to_string(),
            ],
        }
    }

    #[test]
    fn content_addressed_capture_round_trips_without_session_or_path_fields() {
        let capture = valid_draft().into_capture().unwrap();
        let bytes = serde_json_canonicalizer::to_vec(&capture).unwrap();
        let decoded: ResearchCaptureV1 = serde_json::from_slice(&bytes).unwrap();
        decoded.validate().unwrap();
        assert_eq!(decoded, capture);
        assert_eq!(
            capture.capture_id.as_str().len(),
            CAPTURE_ID_PREFIX.len() + 64
        );

        let text = String::from_utf8(bytes).unwrap();
        for forbidden in ["session", "transcript", "root_path", "paper_body"] {
            assert!(!text.contains(forbidden));
        }
    }

    #[test]
    fn semantic_change_changes_capture_identity() {
        let first = valid_draft().into_capture().unwrap();
        let mut changed = valid_draft();
        changed
            .summary
            .push_str(" Reliability remains a secondary axis.");
        let changed = changed.into_capture().unwrap();
        assert_ne!(first.capture_id, changed.capture_id);
    }

    #[test]
    fn unknown_raw_payload_and_host_path_fields_fail_closed() {
        let capture = valid_draft().into_capture().unwrap();
        let mut value = serde_json::to_value(capture).unwrap();
        value
            .as_object_mut()
            .unwrap()
            .insert("transcript".to_string(), serde_json::json!(["raw prompt"]));
        assert!(serde_json::from_value::<ResearchCaptureV1>(value).is_err());

        let mut binding = serde_json::to_value(valid_draft().binding).unwrap();
        binding.as_object_mut().unwrap().insert(
            "root_path".to_string(),
            serde_json::json!("/Users/example/paper"),
        );
        assert!(serde_json::from_value::<ProjectBindingV1>(binding).is_err());
    }

    #[test]
    fn binding_and_evidence_locators_are_bounded_and_portable() {
        assert!(
            ProjectBindingV1::new(
                ProjectId::parse("prj_0123456789abcdef0123456789abcdef").unwrap(),
                0,
                ProjectStage::Idea,
                "Task",
                CapturePolicy::ReviewRequired,
            )
            .is_err()
        );
        assert!(valid_locator(
            EvidenceLocatorKind::ArtifactAnchor,
            "literature/paper_notes.md#measurement"
        ));
        assert!(!valid_locator(
            EvidenceLocatorKind::ArtifactAnchor,
            "/Users/example/private.md"
        ));
        assert!(!valid_locator(
            EvidenceLocatorKind::ArtifactAnchor,
            "C:\\Users\\example\\private.md"
        ));
        assert!(valid_locator(
            EvidenceLocatorKind::HttpsUrl,
            "https://example.org/paper"
        ));
        assert!(!valid_locator(
            EvidenceLocatorKind::HttpsUrl,
            "file:///Users/example/paper.pdf"
        ));
    }

    #[test]
    fn capture_identity_is_revalidated_after_deserialization() {
        let capture = valid_draft().into_capture().unwrap();
        let mut value = serde_json::to_value(capture).unwrap();
        value["summary"] = serde_json::json!("Tampered summary");
        let decoded: ResearchCaptureV1 = serde_json::from_value(value).unwrap();
        assert_eq!(
            decoded.validate(),
            Err(ProjectError::CaptureIdentityConflict)
        );
    }
}
