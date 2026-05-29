# Political Economy and Geoeconomics Subject Design

## Goal

Add `political-economy` and `geoeconomics` as two independent Qiongli official subjects optimized for Claude Desktop/Web focused ZIP use.

The subjects should preserve the canonical Qiongli workflow, task IDs, standards, templates, and quality gates while adding discipline-specific profiles, overlays, venue norms, and small expert audit skills.

## User Model

The primary user installs one active `qiongli-workflow` package in Claude Desktop or Claude Web and expects the package to behave as a subject-specialized academic workflow.

The main release artifacts are:

- `qiongli-claude-desktop-skill-political-economy-<tag>.zip`
- `qiongli-claude-desktop-skill-geoeconomics-<tag>.zip`

CLI and npm installs should support the same subjects through `--subject political-economy` and `--subject geoeconomics`, but Desktop focused ZIPs are the primary product surface.

## Subject Boundaries

### Political Economy

`political-economy` focuses on how political institutions, actors, incentives, coalitions, distributional conflict, and policy processes shape economic outcomes, and how economic interests shape political behavior.

Include:

- institutional constraints and policy formation
- distributional conflict and coalition politics
- elections, bureaucracy, parties, interest groups, and state capacity
- public policy, development, redistribution, regulation, and political accountability
- causal or interpretive designs that connect political mechanisms to economic outcomes

Exclude as the main focus:

- general economics without political mechanism
- generic political science without economic outcome or economic interest structure
- national-security statecraft where economic tools are primarily strategic instruments; use `geoeconomics`

### Geoeconomics

`geoeconomics` focuses on state use of economic tools for strategic competition, security, influence, coercion, resilience, and geopolitical positioning.

Include:

- sanctions, export controls, investment screening, trade restrictions, and industrial policy
- supply-chain security, critical minerals, energy security, and technology controls
- financial statecraft, currency and payment infrastructure, and sovereign risk
- corporate and market responses to geopolitical economic policy
- strategic inference that separates policy rhetoric, mechanism evidence, and observed economic effects

Exclude as the main focus:

- domestic political economy without strategic statecraft
- general international trade without security, coercion, strategic competition, or resilience logic
- broad international relations essays without economic instruments or measurable economic mechanisms

## Packaging Design

Both subjects use:

```text
subject / coverage / flavor = political-economy / focused / desktop
subject / coverage / flavor = geoeconomics / focused / desktop
```

`coverage=focused` selects only the subject-relevant effective skills, profiles, overlays, templates, standards, and venues. `flavor=desktop` keeps file count and package size suitable for Claude Desktop/Web upload limits.

`coverage=complete` should still materialize for CLI/npm users. Complete packages keep the full core framework plus subject overlays and subject-specific skills.

Each subject remains a single active `qiongli-workflow` package. They are not composite subjects and should not be installed together in one Desktop package.

## Source Layout

Add the following source structure:

```text
subjects/political-economy/
  overlays/skills/
    literature-mapper.md
    manuscript-architect.md
    stats-engine.md
    study-designer.md
  skills/
    registry.yaml
    political-economy-mechanism-auditor.md
  venue-profiles/
    ajps.yaml
    apsr.yaml
    comparative-political-studies.yaml
    journal-of-politics.yaml
    world-politics.yaml

subjects/geoeconomics/
  overlays/skills/
    literature-mapper.md
    manuscript-architect.md
    study-designer.md
    venue-analyzer.md
  skills/
    registry.yaml
    geoeconomic-statecraft-auditor.md
  venue-profiles/
    global-policy.yaml
    international-organization.yaml
    international-security.yaml
    review-of-international-political-economy.yaml
    world-politics.yaml

skills/domain-profiles/
  political-economy.yaml
  geoeconomics.yaml
```

The domain profiles live in `skills/domain-profiles/` because official subject materialization already resolves domain profiles from the canonical domain profile tree.

## Catalog Entries

Add two `subjects/catalog.yaml` entries.

`political-economy`:

- extends `core`
- selects `political-economy` domain profile
- selects political science and political economy venues
- applies overlays to literature mapping, manuscript architecture, study design, and stats review
- adds `political-economy-mechanism-auditor`

`geoeconomics`:

- extends `core`
- selects `geoeconomics` domain profile
- selects security, international political economy, and policy venues
- applies overlays to literature mapping, manuscript architecture, study design, and venue analysis
- adds `geoeconomic-statecraft-auditor`

Focused skill groups should be compact and workflow-shaped:

- Research framing and venue fit
- Literature positioning
- Design, mechanism, and evidence
- Results and manuscript
- Review, compliance, and readiness

## Subject-Specific Skills

### `political-economy-mechanism-auditor`

Purpose: audit whether a draft clearly connects actors, institutions, incentives, distributional stakes, and policy or economic outcomes.

Quality bar:

- identifies the political actor set and their incentives
- names the institutional setting and constraint structure
- distinguishes mechanism, correlation, causal claim, and interpretation
- checks whether evidence supports the stated mechanism
- flags unsupported jumps from policy outcome to political cause

Primary stages: A, C, F, H.

### `geoeconomic-statecraft-auditor`

Purpose: audit whether a draft treats economic instruments as strategic tools with clear actors, targets, mechanisms, responses, and evidence boundaries.

Quality bar:

- identifies the instrument, sender, target, intermediaries, and affected markets or firms
- separates policy rhetoric from mechanism evidence
- checks timing, selection, exposure, and countermeasure risks
- distinguishes coercion, resilience, signaling, deterrence, and industrial policy claims
- flags unsupported national-security or geopolitical overclaiming

Primary stages: A, C, F, H.

## Overlay Design

Overlays should deepen existing generic skills rather than duplicate them.

Political economy overlays:

- `literature-mapper`: map literature by mechanism, institution, actor, and outcome family
- `study-designer`: require actor-institution-outcome alignment and mechanism identification
- `manuscript-architect`: foreground political mechanism and contribution to political economy debates
- `stats-engine`: add diagnostics for policy timing, institutional clustering, endogeneity, and treatment assignment

Geoeconomics overlays:

- `literature-mapper`: map literature by instrument, strategic objective, target response, and outcome channel
- `study-designer`: require sender-target-instrument-response logic and timing/exposure checks
- `manuscript-architect`: foreground strategic economic statecraft and avoid unsupported security rhetoric
- `venue-analyzer`: match manuscripts to security, IPE, policy, or management-facing venues based on claim type

Use `append` overlays by default. Use `replace_sections` only when replacing an existing `Quality Bar` or `Common Pitfalls` section is necessary and the exact section name is declared.

## Domain Profiles

`political-economy.yaml` should include:

- common designs: comparative historical analysis, panel policy analysis, quasi-experimental policy designs, survey or elite data, archival institutional datasets
- common risks: omitted institutional confounders, ecological inference, endogenous policy adoption, weak mechanism evidence, overgeneralized regime claims
- minimum diagnostics: actor-incentive map, institutional scope, policy timing, alternative mechanisms, evidence boundary
- reporting expectations: institutional context, mechanism trace, outcome definition, claim strength calibration

`geoeconomics.yaml` should include:

- common designs: sanctions/event studies, supply-chain exposure analysis, policy shock designs, firm or trade flow analysis, case-process tracing
- common risks: anticipatory effects, target selection, geopolitical confounding, policy bundling, data opacity, strategic substitution
- minimum diagnostics: instrument taxonomy, sender-target map, timing window, exposure construction, countermeasure assessment
- reporting expectations: strategic objective, instrument channel, market or firm response, security/economic claim separation

## Venue Profiles

Political economy venue profiles should cover general political science and comparative/international political economy outlets:

- APSR
- AJPS
- Journal of Politics
- World Politics
- Comparative Political Studies

Geoeconomics venue profiles should cover security, IPE, and policy-facing outlets:

- International Security
- International Organization
- Review of International Political Economy
- World Politics
- Global Policy

Shared venues such as `world-politics` may appear in both subjects, but each subject should select it through its own catalog entry.

## Tests and Validation

Add or extend tests to verify:

- `validate_subject_catalog()` accepts both new subjects
- focused materialization includes only declared domain profiles
- focused materialization includes selected venue profiles and excludes unrelated profiles
- focused materialization includes each subject-specific skill
- subject overlays appear in effective selected skill markdown
- complete materialization keeps core registry coverage plus subject-specific skills
- Claude Desktop subject ZIP file count remains under the current release budget
- subject specialization audit expects terms for both subjects

Expected subject audit terms:

- `political-economy`: political mechanism, institution, distributional conflict, policy
- `geoeconomics`: sanctions, statecraft, supply chain, strategic competition

## Documentation and Release

Update:

- `README.md` and `README_CN.md` subject lists
- `docs/advanced/subject-packaging-model.md`
- `docs/zh/advanced/subject-packaging-model.md`
- install/quickstart references for Desktop ZIP assets
- release automation subject artifact lists
- npm package payload tests if npm distributes the new subjects
- marketplace metadata outside this repository when publishing public entries

Release artifacts should be named consistently with existing Desktop ZIP conventions.

## Acceptance Criteria

- `qiongli install --subject political-economy --target all` resolves to the new official subject.
- `qiongli install --subject geoeconomics --target all` resolves to the new official subject.
- Focused Desktop materialization produces subject-specialized packages with effective skill markdown, selected domain profile, selected venues, standards, references, templates, and manifest metadata.
- Neither subject is implemented as a composite of the other.
- Political economy outputs emphasize actor-institution-mechanism-outcome alignment.
- Geoeconomics outputs emphasize instrument-sender-target-response logic and calibrated strategic claims.
- Existing core, economics, accounting, business, finance, and economics-accounting subject tests continue to pass.
