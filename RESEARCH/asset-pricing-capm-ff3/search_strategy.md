# Targeted Literature Search Strategy

## Scope

- Research question: CAPM versus FF3 pricing-error attenuation for 25 U.S. size/BM portfolios.
- Review mode: targeted search, not systematic review.
- Date/language/type: no publication-year restriction; English primary papers and official dataset documentation.
- Search execution: `native_only`; Qiongli MCP tools were not visible in this Codex session.
- Provider capability: `strategy_only` for the unavailable MCP/provider layer.
- Plan: `context/qiongli_search_plan.json`.

## Concept Blocks And Queries

| Query ID | Concept Blocks | Native Query | Known Item |
|---|---|---|---|
| NQ-001 | CAPM; equilibrium; market beta | `Sharpe 1964 capital asset prices primary paper` | Sharpe (1964) |
| NQ-002 | size; book-to-market; expected returns | `Fama French 1992 cross-section expected stock returns primary paper` | Fama and French (1992) |
| NQ-003 | FF3; common factors; portfolio returns | `Fama French 1993 common risk factors primary paper` | Fama and French (1993) |
| NQ-004 | FF3; anomalies; 25 portfolios | `Fama French 1996 multifactor explanations asset pricing anomalies primary paper` | Fama and French (1996) |
| NQ-005 | HAC; autocorrelation; heteroskedasticity | `Newey West 1987 HAC covariance primary paper` | Newey and West (1987) |
| NQ-006 | factor construction; portfolio construction | `Kenneth French Data Library factor and 25 portfolio construction` | official details pages |

## Eligibility And Evidence Limits

- Include the primary publication page, primary full text when directly visible, and official dataset documentation.
- Exclude tertiary summaries from the evidence base.
- Preserve `full_text`, `abstract_only`, `metadata_only`, or `official_documentation` limits in notes and the retrieval manifest.
- A targeted search cannot support exhaustive field-coverage or systematic-review claims.

## Deduplication

Use DOI first, then normalized title/year/author. One canonical citekey is retained per publication.

## Approval

The user explicitly approved the PLT-322 implementation plan, including this targeted literature pass, before search execution.
