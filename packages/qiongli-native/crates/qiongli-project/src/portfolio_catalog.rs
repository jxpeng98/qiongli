use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::json::parse_unique_json;
use crate::model::{MAX_SEMANTIC_REVISION, valid_lower_hex};
use crate::{AcademicGraphSnapshotV1, ProjectError, ProjectHealth, ProjectId, ProjectLifecycle};

pub const PORTFOLIO_CONTRIBUTION_SCHEMA_VERSION: u32 = 1;
pub const PORTFOLIO_CONTRIBUTION_DOCUMENT_KIND: &str = "qiongli-portfolio-project-contribution";
pub const PORTFOLIO_CATALOG_SCHEMA_VERSION: u32 = 1;
pub const PORTFOLIO_CATALOG_MANIFEST_DOCUMENT_KIND: &str = "qiongli-portfolio-catalog-manifest";
pub const PORTFOLIO_CATALOG_SNAPSHOT_DOCUMENT_KIND: &str = "qiongli-portfolio-catalog-snapshot";
pub(crate) const PORTFOLIO_CATALOG_TRANSACTION_DOCUMENT_KIND: &str =
    "qiongli-portfolio-catalog-transaction";
pub(crate) const MAX_PORTFOLIO_CONTRIBUTIONS: usize = 1_024;
pub(crate) const MAX_PORTFOLIO_CHANGED_PROJECTS: usize = 1_024;
pub(crate) const MAX_PORTFOLIO_CONTRIBUTION_BYTES: usize = 5 * 1024 * 1024;
pub(crate) const MAX_PORTFOLIO_CATALOG_MANIFEST_BYTES: usize = 1024 * 1024;
pub(crate) const MAX_PORTFOLIO_CATALOG_TRANSACTION_BYTES: usize = 32 * 1024 * 1024;

const CONTRIBUTION_ID_PREFIX: &str = "pct_";
const CATALOG_ID_PREFIX: &str = "pca_";
const TRANSACTION_ID_PREFIX: &str = "ptx_";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PortfolioContributionV1 {
    pub schema_version: u32,
    pub document_kind: String,
    pub contribution_id: String,
    pub contribution_digest: String,
    pub project_id: ProjectId,
    pub lifecycle: ProjectLifecycle,
    pub health: ProjectHealth,
    pub semantic_revision: u64,
    pub semantic_digest: String,
    pub projection_id: String,
    pub node_count: usize,
    pub edge_count: usize,
    pub diagnostic_count: usize,
    pub graph: AcademicGraphSnapshotV1,
}

#[derive(Serialize)]
struct ContributionIdentity<'a> {
    schema_version: u32,
    project_id: &'a ProjectId,
    lifecycle: ProjectLifecycle,
    health: ProjectHealth,
    semantic_revision: u64,
    semantic_digest: &'a str,
    projection_id: &'a str,
    graph: &'a AcademicGraphSnapshotV1,
}

impl PortfolioContributionV1 {
    pub fn from_graph(
        graph: AcademicGraphSnapshotV1,
        health: ProjectHealth,
    ) -> Result<Self, ProjectError> {
        graph.validate()?;
        let mut contribution = Self {
            schema_version: PORTFOLIO_CONTRIBUTION_SCHEMA_VERSION,
            document_kind: PORTFOLIO_CONTRIBUTION_DOCUMENT_KIND.to_string(),
            contribution_id: String::new(),
            contribution_digest: String::new(),
            project_id: graph.project_id.clone(),
            lifecycle: graph.project_lifecycle,
            health,
            semantic_revision: graph.project_revision,
            semantic_digest: graph.project_semantic_digest.clone(),
            projection_id: graph.projection_id.clone(),
            node_count: graph.node_count,
            edge_count: graph.edge_count,
            diagnostic_count: graph.diagnostic_count,
            graph,
        };
        contribution.contribution_digest = contribution.identity_digest()?;
        contribution.contribution_id = format!(
            "{CONTRIBUTION_ID_PREFIX}{}",
            contribution.contribution_digest
        );
        contribution.validate()?;
        Ok(contribution)
    }

    pub fn from_json_slice(bytes: &[u8]) -> Result<Self, ProjectError> {
        if bytes.len() > MAX_PORTFOLIO_CONTRIBUTION_BYTES {
            return Err(ProjectError::DocumentTooLarge);
        }
        let value = parse_unique_json(bytes).map_err(|_| ProjectError::InvalidPortfolioCatalog)?;
        let contribution: Self =
            serde_json::from_value(value).map_err(|_| ProjectError::InvalidPortfolioCatalog)?;
        contribution.validate()?;
        if contribution.to_canonical_json()? != bytes {
            return Err(ProjectError::InvalidPortfolioCatalog);
        }
        Ok(contribution)
    }

    pub fn to_canonical_json(&self) -> Result<Vec<u8>, ProjectError> {
        self.validate()?;
        let bytes = serde_json_canonicalizer::to_vec(self)
            .map_err(|_| ProjectError::InvalidPortfolioCatalog)?;
        if bytes.len() > MAX_PORTFOLIO_CONTRIBUTION_BYTES {
            return Err(ProjectError::DocumentTooLarge);
        }
        Ok(bytes)
    }

    pub fn validate(&self) -> Result<(), ProjectError> {
        self.graph
            .validate()
            .map_err(|_| ProjectError::InvalidPortfolioCatalog)?;
        let expected_digest = self.identity_digest()?;
        if self.schema_version != PORTFOLIO_CONTRIBUTION_SCHEMA_VERSION
            || self.document_kind != PORTFOLIO_CONTRIBUTION_DOCUMENT_KIND
            || !valid_prefixed_digest(&self.contribution_id, CONTRIBUTION_ID_PREFIX)
            || !valid_lower_hex(&self.contribution_digest, 64)
            || self.contribution_id
                != format!("{CONTRIBUTION_ID_PREFIX}{}", self.contribution_digest)
            || self.contribution_digest != expected_digest
            || self.health != ProjectHealth::Ready
            || self.semantic_revision == 0
            || self.semantic_revision > MAX_SEMANTIC_REVISION
            || !valid_lower_hex(&self.semantic_digest, 64)
            || self.project_id != self.graph.project_id
            || self.lifecycle != self.graph.project_lifecycle
            || self.semantic_revision != self.graph.project_revision
            || self.semantic_digest != self.graph.project_semantic_digest
            || self.projection_id != self.graph.projection_id
            || self.node_count != self.graph.node_count
            || self.edge_count != self.graph.edge_count
            || self.diagnostic_count != self.graph.diagnostic_count
        {
            return Err(ProjectError::InvalidPortfolioCatalog);
        }
        self.project_id
            .validate()
            .map_err(|_| ProjectError::InvalidPortfolioCatalog)
    }

    fn identity_digest(&self) -> Result<String, ProjectError> {
        canonical_domain_digest(
            b"qiongli-portfolio-project-contribution-v1\0",
            &ContributionIdentity {
                schema_version: self.schema_version,
                project_id: &self.project_id,
                lifecycle: self.lifecycle,
                health: self.health,
                semantic_revision: self.semantic_revision,
                semantic_digest: &self.semantic_digest,
                projection_id: &self.projection_id,
                graph: &self.graph,
            },
        )
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PortfolioContributionRefV1 {
    pub project_id: ProjectId,
    pub semantic_revision: u64,
    pub projection_id: String,
    pub contribution_id: String,
    pub contribution_sha256: String,
}

impl PortfolioContributionRefV1 {
    pub(crate) fn from_contribution(
        contribution: &PortfolioContributionV1,
    ) -> Result<Self, ProjectError> {
        contribution.validate()?;
        Ok(Self {
            project_id: contribution.project_id.clone(),
            semantic_revision: contribution.semantic_revision,
            projection_id: contribution.projection_id.clone(),
            contribution_id: contribution.contribution_id.clone(),
            contribution_sha256: sha256_bytes(&contribution.to_canonical_json()?),
        })
    }

    fn validate(&self) -> Result<(), ProjectError> {
        if self.semantic_revision == 0
            || self.semantic_revision > MAX_SEMANTIC_REVISION
            || !valid_prefixed_digest(&self.projection_id, "grp_")
            || !valid_prefixed_digest(&self.contribution_id, CONTRIBUTION_ID_PREFIX)
            || !valid_lower_hex(&self.contribution_sha256, 64)
        {
            return Err(ProjectError::InvalidPortfolioCatalog);
        }
        self.project_id
            .validate()
            .map_err(|_| ProjectError::InvalidPortfolioCatalog)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PortfolioCatalogManifestV1 {
    pub schema_version: u32,
    pub document_kind: String,
    pub catalog_id: String,
    pub generation: u64,
    pub library_revision: u64,
    pub created_at_unix: u64,
    pub contribution_count: usize,
    pub contributions: Vec<PortfolioContributionRefV1>,
}

#[derive(Serialize)]
struct CatalogIdentity<'a> {
    schema_version: u32,
    library_revision: u64,
    contributions: &'a [PortfolioContributionRefV1],
}

impl PortfolioCatalogManifestV1 {
    pub fn from_json_slice(bytes: &[u8]) -> Result<Self, ProjectError> {
        if bytes.len() > MAX_PORTFOLIO_CATALOG_MANIFEST_BYTES {
            return Err(ProjectError::DocumentTooLarge);
        }
        let value = parse_unique_json(bytes).map_err(|_| ProjectError::InvalidPortfolioCatalog)?;
        let manifest: Self =
            serde_json::from_value(value).map_err(|_| ProjectError::InvalidPortfolioCatalog)?;
        manifest.validate()?;
        if manifest.to_canonical_json()? != bytes {
            return Err(ProjectError::InvalidPortfolioCatalog);
        }
        Ok(manifest)
    }

    pub fn to_canonical_json(&self) -> Result<Vec<u8>, ProjectError> {
        self.validate()?;
        let bytes = serde_json_canonicalizer::to_vec(self)
            .map_err(|_| ProjectError::InvalidPortfolioCatalog)?;
        if bytes.len() > MAX_PORTFOLIO_CATALOG_MANIFEST_BYTES {
            return Err(ProjectError::DocumentTooLarge);
        }
        Ok(bytes)
    }

    pub fn validate(&self) -> Result<(), ProjectError> {
        if self.schema_version != PORTFOLIO_CATALOG_SCHEMA_VERSION
            || self.document_kind != PORTFOLIO_CATALOG_MANIFEST_DOCUMENT_KIND
            || !valid_prefixed_digest(&self.catalog_id, CATALOG_ID_PREFIX)
            || self.generation == 0
            || self.generation > MAX_SEMANTIC_REVISION
            || self.library_revision > MAX_SEMANTIC_REVISION
            || self.created_at_unix > MAX_SEMANTIC_REVISION
            || self.contribution_count != self.contributions.len()
            || self.contributions.len() > MAX_PORTFOLIO_CONTRIBUTIONS
            || !strictly_sorted_by_project(&self.contributions)
            || self
                .contributions
                .iter()
                .any(|contribution| contribution.validate().is_err())
            || self.catalog_id != self.identity()?
        {
            return Err(ProjectError::InvalidPortfolioCatalog);
        }
        Ok(())
    }

    fn new(
        generation: u64,
        library_revision: u64,
        created_at_unix: u64,
        contributions: Vec<PortfolioContributionRefV1>,
    ) -> Result<Self, ProjectError> {
        let mut manifest = Self {
            schema_version: PORTFOLIO_CATALOG_SCHEMA_VERSION,
            document_kind: PORTFOLIO_CATALOG_MANIFEST_DOCUMENT_KIND.to_string(),
            catalog_id: String::new(),
            generation,
            library_revision,
            created_at_unix,
            contribution_count: contributions.len(),
            contributions,
        };
        manifest.catalog_id = manifest.identity()?;
        manifest.validate()?;
        Ok(manifest)
    }

    fn identity(&self) -> Result<String, ProjectError> {
        let digest = canonical_domain_digest(
            b"qiongli-portfolio-catalog-v1\0",
            &CatalogIdentity {
                schema_version: self.schema_version,
                library_revision: self.library_revision,
                contributions: &self.contributions,
            },
        )?;
        Ok(format!("{CATALOG_ID_PREFIX}{digest}"))
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PortfolioCatalogSnapshotV1 {
    pub schema_version: u32,
    pub document_kind: String,
    pub catalog_id: String,
    pub generation: u64,
    pub library_revision: u64,
    pub created_at_unix: u64,
    pub contribution_count: usize,
    pub node_count: usize,
    pub edge_count: usize,
    pub diagnostic_count: usize,
    pub contributions: Vec<PortfolioContributionRefV1>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct PortfolioCatalogTransactionV1 {
    pub(crate) schema_version: u32,
    pub(crate) document_kind: String,
    pub(crate) transaction_id: String,
    pub(crate) previous_manifest: Option<PortfolioCatalogManifestV1>,
    pub(crate) replacements: Vec<PortfolioContributionV1>,
    pub(crate) removals: Vec<ProjectId>,
    pub(crate) next_manifest: PortfolioCatalogManifestV1,
}

#[derive(Serialize)]
struct TransactionIdentity<'a> {
    schema_version: u32,
    previous_manifest: &'a Option<PortfolioCatalogManifestV1>,
    replacements: &'a [PortfolioContributionV1],
    removals: &'a [ProjectId],
    next_manifest: &'a PortfolioCatalogManifestV1,
}

impl PortfolioCatalogTransactionV1 {
    pub(crate) fn new(
        previous_manifest: Option<PortfolioCatalogManifestV1>,
        mut replacements: Vec<PortfolioContributionV1>,
        mut removals: Vec<ProjectId>,
        library_revision: u64,
        created_at_unix: u64,
    ) -> Result<Self, ProjectError> {
        replacements.sort_by(|left, right| left.project_id.cmp(&right.project_id));
        removals.sort_unstable();
        let contributions = apply_changes(previous_manifest.as_ref(), &replacements, &removals)?;
        let generation = previous_manifest.as_ref().map_or(Ok(1), |manifest| {
            manifest
                .generation
                .checked_add(1)
                .ok_or(ProjectError::InvalidPortfolioCatalog)
        })?;
        let next_manifest = PortfolioCatalogManifestV1::new(
            generation,
            library_revision,
            created_at_unix,
            contributions,
        )?;
        let mut transaction = Self {
            schema_version: PORTFOLIO_CATALOG_SCHEMA_VERSION,
            document_kind: PORTFOLIO_CATALOG_TRANSACTION_DOCUMENT_KIND.to_string(),
            transaction_id: String::new(),
            previous_manifest,
            replacements,
            removals,
            next_manifest,
        };
        transaction.transaction_id = transaction.identity()?;
        transaction.validate()?;
        Ok(transaction)
    }

    pub(crate) fn from_json_slice(bytes: &[u8]) -> Result<Self, ProjectError> {
        if bytes.len() > MAX_PORTFOLIO_CATALOG_TRANSACTION_BYTES {
            return Err(ProjectError::DocumentTooLarge);
        }
        let value = parse_unique_json(bytes).map_err(|_| ProjectError::InvalidPortfolioCatalog)?;
        let transaction: Self =
            serde_json::from_value(value).map_err(|_| ProjectError::InvalidPortfolioCatalog)?;
        transaction.validate()?;
        if transaction.to_canonical_json()? != bytes {
            return Err(ProjectError::InvalidPortfolioCatalog);
        }
        Ok(transaction)
    }

    pub(crate) fn to_canonical_json(&self) -> Result<Vec<u8>, ProjectError> {
        self.validate()?;
        let bytes = serde_json_canonicalizer::to_vec(self)
            .map_err(|_| ProjectError::InvalidPortfolioCatalog)?;
        if bytes.len() > MAX_PORTFOLIO_CATALOG_TRANSACTION_BYTES {
            return Err(ProjectError::DocumentTooLarge);
        }
        Ok(bytes)
    }

    pub(crate) fn validate(&self) -> Result<(), ProjectError> {
        if self.schema_version != PORTFOLIO_CATALOG_SCHEMA_VERSION
            || self.document_kind != PORTFOLIO_CATALOG_TRANSACTION_DOCUMENT_KIND
            || !valid_prefixed_digest(&self.transaction_id, TRANSACTION_ID_PREFIX)
            || self.replacements.len() > MAX_PORTFOLIO_CHANGED_PROJECTS
            || self.removals.len() > MAX_PORTFOLIO_CHANGED_PROJECTS
            || self
                .replacements
                .len()
                .checked_add(self.removals.len())
                .is_none_or(|count| count > MAX_PORTFOLIO_CHANGED_PROJECTS)
            || self
                .previous_manifest
                .as_ref()
                .is_some_and(|manifest| manifest.validate().is_err())
            || self
                .replacements
                .iter()
                .any(|contribution| contribution.validate().is_err())
            || !strictly_sorted_contributions(&self.replacements)
            || !strictly_sorted_project_ids(&self.removals)
            || overlaps(&self.replacements, &self.removals)
            || self.next_manifest.validate().is_err()
        {
            return Err(ProjectError::InvalidPortfolioCatalog);
        }
        let expected_generation = self.previous_manifest.as_ref().map_or(Ok(1), |manifest| {
            manifest
                .generation
                .checked_add(1)
                .ok_or(ProjectError::InvalidPortfolioCatalog)
        })?;
        let expected_contributions = apply_changes(
            self.previous_manifest.as_ref(),
            &self.replacements,
            &self.removals,
        )?;
        if self.next_manifest.generation != expected_generation
            || self.next_manifest.contributions != expected_contributions
            || self.next_manifest.contribution_count != expected_contributions.len()
            || self.previous_manifest.as_ref().is_some_and(|previous| {
                self.next_manifest.created_at_unix < previous.created_at_unix
                    || self.next_manifest.library_revision < previous.library_revision
            })
            || self.transaction_id != self.identity()?
        {
            return Err(ProjectError::InvalidPortfolioCatalog);
        }
        let bytes = serde_json_canonicalizer::to_vec(self)
            .map_err(|_| ProjectError::InvalidPortfolioCatalog)?;
        if bytes.len() > MAX_PORTFOLIO_CATALOG_TRANSACTION_BYTES {
            return Err(ProjectError::DocumentTooLarge);
        }
        Ok(())
    }

    fn identity(&self) -> Result<String, ProjectError> {
        let digest = canonical_domain_digest(
            b"qiongli-portfolio-catalog-transaction-v1\0",
            &TransactionIdentity {
                schema_version: self.schema_version,
                previous_manifest: &self.previous_manifest,
                replacements: &self.replacements,
                removals: &self.removals,
                next_manifest: &self.next_manifest,
            },
        )?;
        Ok(format!("{TRANSACTION_ID_PREFIX}{digest}"))
    }
}

fn apply_changes(
    previous: Option<&PortfolioCatalogManifestV1>,
    replacements: &[PortfolioContributionV1],
    removals: &[ProjectId],
) -> Result<Vec<PortfolioContributionRefV1>, ProjectError> {
    let mut by_project = previous
        .map(|manifest| {
            manifest
                .contributions
                .iter()
                .cloned()
                .map(|contribution| (contribution.project_id.clone(), contribution))
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();
    for project_id in removals {
        by_project.remove(project_id);
    }
    for replacement in replacements {
        by_project.insert(
            replacement.project_id.clone(),
            PortfolioContributionRefV1::from_contribution(replacement)?,
        );
    }
    if by_project.len() > MAX_PORTFOLIO_CONTRIBUTIONS {
        return Err(ProjectError::InvalidPortfolioCatalog);
    }
    Ok(by_project.into_values().collect())
}

fn overlaps(replacements: &[PortfolioContributionV1], removals: &[ProjectId]) -> bool {
    let removals = removals.iter().collect::<BTreeSet<_>>();
    replacements
        .iter()
        .any(|replacement| removals.contains(&replacement.project_id))
}

fn strictly_sorted_by_project(values: &[PortfolioContributionRefV1]) -> bool {
    values
        .windows(2)
        .all(|pair| pair[0].project_id < pair[1].project_id)
}

fn strictly_sorted_contributions(values: &[PortfolioContributionV1]) -> bool {
    values
        .windows(2)
        .all(|pair| pair[0].project_id < pair[1].project_id)
}

fn strictly_sorted_project_ids(values: &[ProjectId]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}

fn valid_prefixed_digest(value: &str, prefix: &str) -> bool {
    value
        .strip_prefix(prefix)
        .is_some_and(|digest| valid_lower_hex(digest, 64))
}

fn canonical_domain_digest<T: Serialize>(domain: &[u8], value: &T) -> Result<String, ProjectError> {
    let bytes = serde_json_canonicalizer::to_vec(value)
        .map_err(|_| ProjectError::InvalidPortfolioCatalog)?;
    let mut digest = Sha256::new();
    digest.update(domain);
    digest.update(bytes);
    Ok(format!("{:x}", digest.finalize()))
}

fn sha256_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
