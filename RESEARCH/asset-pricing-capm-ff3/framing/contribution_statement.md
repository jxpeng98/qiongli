# Contribution Statement (A2)

## One-Sentence Thesis

This project provides a current-vintage, digest-pinned, diagnostics-aware reproduction of how CAPM and FF3 differ in portfolio-level pricing errors for 25 canonical size/book-to-market test assets.

## Primary Contribution

- Type: empirical replication and reproducibility artifact.
- Claim: the exact magnitude and distribution of benchmark-relative alpha attenuation can be reproduced from two official public archives with explicit validation and HAC inference.
- Who cares / why now: researchers and tool builders who need a real, auditable asset-pricing workflow rather than a synthetic fixture.
- Evidence required: official-source provenance, locked analysis, deterministic outputs, diagnostics, and claim-evidence mapping.

## Secondary Contributions

1. Exposes portfolio-level cases where adding SMB/HML increases rather than reduces absolute alpha.
2. Separates the value-weighted primary result from an equal-weighted sensitivity using the same source archive.
3. Makes data-vintage drift an explicit stop condition instead of silently changing results.
4. Provides a canonical scholarly artifact set suitable for Qiongli 1.19-to-2.x migration and Graph v1 acceptance.

## Non-Goals

- No new asset-pricing factor or theory.
- No proof that FF3 is the true or universally superior pricing model.
- No causal identification, trading strategy, or investment recommendation.
- No systematic literature review or submission-ready manuscript.
