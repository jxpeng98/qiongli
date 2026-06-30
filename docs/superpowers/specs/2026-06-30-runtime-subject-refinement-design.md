# Runtime Subject Refinement Design

## Goal

Make Qiongli feel complete on every install path while allowing the active
discipline layer to emerge during use. Users should not need to choose a subject
at installation time. Instead, Qiongli should install an adaptive core package,
infer project-specific subject needs from the user's work, and gradually apply
subject refinements through runtime packets and local guidance.

The design must work for CLI installs, marketplace plugins, client-native skill
installs, and constrained clients that cannot write local files.

## Product Model

Qiongli has one default user-facing install model:

```bash
qiongli install --profile full --target <client>
```

or, on plugin/client surfaces, "Install Qiongli".

This installs the adaptive core experience:

- Canonical workflow, tasks, templates, quality gates, and core skills.
- Subject catalog metadata.
- Domain profiles.
- Subject overlays.
- Subject-specific skill cards.
- Venue profiles and subject templates where platform file budgets allow them.

The installed package starts with `active_subject: auto`. Runtime subject
selection is a project state, not an installation decision.

## Current Context

The repository already contains most of the building blocks:

- `content/subjects/catalog.yaml` defines subject packages, overlays, domain
  profiles, subject-specific skills, and venue profiles.
- `packages/python-qiongli/src/qiongli/bridges/project_manifest.py` defines
  `.qiongli/guidance_manifest.yaml` with `active_subject`, secondary subjects,
  venue profiles, method lenses, and strictness.
- `packages/python-qiongli/src/qiongli/bridges/project_inference.py` can infer
  finance and economics hints from task content.
- `packages/python-qiongli/src/qiongli/bridges/subject_runtime.py` maps subjects
  to domains for task packets.
- `packages/python-qiongli/src/qiongli/bridges/guidance_runtime.py` reads local
  guidance and writes guidance update proposals.
- `packages/python-qiongli/src/qiongli/bridges/mcp_tool_handlers.py` includes
  project subject and effective domain fields in task-run previews.

The gap is that subject inference is still too coarse. It should not treat every
edge touch with economics or finance as a full subject activation. It also
needs a cross-platform persistence contract for clients that can or cannot write
`.qiongli` files.

## Recommended Architecture

Introduce a Runtime Subject Refinement layer.

```text
Installed Adaptive Core
        |
        v
Project Context + Current Task
        |
        v
SubjectResolver
        |
        v
SubjectRefinementPacket
        |
        +--> temporary runtime constraints
        +--> guidance proposal
        +--> confirmed local guidance
        +--> locked project subject
```

The package remains core-first. Subject refinements are overlays on top of core,
never replacements for canonical workflow contracts.

## Subject Refinement Packet

Every task-run preview, task-run packet, and skill-only subject preflight should
be able to expose this shape:

```yaml
subject_refinement:
  mode: auto | suggested | confirmed | locked
  active_subject: auto | economics | finance | business | ...
  primary_subject: finance
  secondary_subjects:
    - economics
  candidate_subjects:
    - subject: finance
      confidence: 0.82
      evidence:
        - "event study"
        - "abnormal returns"
      matched_dimensions:
        - method
        - outcome
        - venue
  method_lenses:
    - event-study
    - asset-pricing
  borrowed_lenses:
    - source_subject: finance
      lens: event-study
      reason: "The project uses abnormal-return event-study methods but is not necessarily a finance paper."
  loaded_resources:
    domain_profiles:
      - skills/domain-profiles/finance.yaml
    overlays:
      - subjects/finance/overlays/skills/study-designer.md
      - subjects/finance/overlays/skills/stats-engine.md
    subject_skills:
      - finance-identification-risk-auditor
    venue_profiles: []
  persistence:
    status: temporary | proposed | applied | locked
    proposal_path: ".qiongli/trace/runs/[run_id]/guidance_update_proposal.md"
```

This packet is the cross-platform contract. Platforms may differ in persistence,
but they should report the same decision shape.

## Boundary Classification

Subject inference should use multiple dimensions, not keyword hits alone:

- `topic`: the phenomenon or literature area.
- `method`: DID, event study, factor model, survey, qualitative coding, etc.
- `data`: CRSP/Compustat, census, interviews, experiments, trials, code repos.
- `outcome`: returns, policy outcomes, health endpoints, learning outcomes.
- `venue`: JF, AER, CHI, JAMA, ACL, etc.
- `claim_type`: causal, predictive, descriptive, theoretical, normative.
- `artifact_history`: repeated project artifacts and prior guidance choices.
- `user_confirmation`: explicit user answer or manual manifest setting.

The resolver should produce a confidence score and a decision class:

| Decision | Meaning | Runtime Behavior |
|---|---|---|
| `no_subject` | No subject signal beyond core | Use core only |
| `borrow_lens` | A method or standard belongs to a subject, but the project is not clearly that subject | Load method pack or diagnostics only |
| `suggest_subject` | Several dimensions point to the same subject | Use subject refinement temporarily and propose guidance |
| `confirm_subject` | High confidence or repeated evidence | Apply guidance if mode permits; otherwise ask user |
| `lock_subject` | User explicitly selected the subject | Do not auto-switch without user action |

## Edge-Subject Rule

If a project only touches the edge of a discipline, do not activate the whole
subject. Borrow the smallest audited unit that protects the claim.

Examples:

- A political science paper uses DID. Borrow economics DID diagnostics and Q1
  failure triggers; do not mark the project as economics unless its venue,
  contribution, and literature are economics-facing.
- A management paper uses an abnormal-return event study. Borrow finance event
  study method diagnostics; do not mark the project as finance unless the
  central claim is asset pricing, corporate finance, market microstructure, or a
  finance venue contribution.
- A public health paper includes cost outcomes. Borrow economics cost or policy
  evaluation lenses only when the paper makes economic welfare, causal policy,
  or resource-allocation claims.

The default rule:

```text
one method signal -> borrowed_lens
method + data/outcome signal -> suggested secondary subject
method + data/outcome + literature/venue signal -> suggested primary subject
user confirmation -> confirmed or locked subject
```

## Resource Loading Levels

Subject refinement should be granular:

1. `method_pack_only`
   - Load domain profile method entries and failure triggers.
   - Use when a project borrows a method from a neighboring discipline.

2. `skill_overlay`
   - Add overlays for active core skills such as `study-designer`,
     `stats-engine`, `manuscript-architect`, or `literature-mapper`.
   - Use when multiple dimensions suggest a subject workflow standard.

3. `subject_skill`
   - Add subject-specific auditors such as
     `finance-identification-risk-auditor` or `econ-identification-auditor`.
   - Use when the central claim depends on that subject's review standards.

4. `venue_profile`
   - Load venue profiles only when the user names or implies a venue family.

5. `project_guidance`
   - Persist only when confidence is high, evidence is repeated, or the user
     confirms the direction.

This prevents overloading a project with irrelevant subject machinery.

## Persistence Model

Use four modes:

- `auto`: default. Runtime inference is temporary.
- `suggested`: write a proposal explaining subject candidates and borrowed
  lenses.
- `confirmed`: write selected subject or secondary subjects to local guidance.
- `locked`: user-selected. Automatic inference can add borrowed lenses but
  cannot replace the locked primary subject.

Persistence targets:

- `.qiongli/guidance_manifest.yaml` for stable subject state.
- `.qiongli/guidance.d/subject-[subject].md` for subject-specific project
  guidance.
- `.qiongli/trace/runs/[run_id]/guidance_update_proposal.md` for proposals and
  audit trail.

## Cross-Platform Behavior

The same refinement logic should run everywhere, but persistence differs.

### CLI Full

Best experience. The runtime can read and write `.qiongli`, inspect project
artifacts, launch local agents when requested, and persist proposals or applied
guidance.

### Marketplace Plugin

Install the adaptive package by default. If the client exposes filesystem or
workspace write access, use the same `.qiongli` contract. If the client is
session-only, return a `subject_refinement` packet and a guidance proposal
artifact that the user can apply manually.

### Claude Desktop / Web Skill

Use a compact adaptive package when file budgets are tight:

- Always include core workflow and a subject refinement index.
- Include compact domain profiles or a subject-resource index.
- Prefer full local guidance persistence only when the client can write project
  files.
- Otherwise, include the refinement packet in the response and ask the user to
  confirm the suggested subject/lenses.

### Other Clients

Use capability detection:

- `can_read_project_files`
- `can_write_project_files`
- `can_call_mcp`
- `can_launch_local_agents`

The resolver should degrade gracefully from applied guidance to proposal-only
guidance when persistence is unavailable.

## User Experience

The user should see decisions in plain language:

```text
I am treating this as a core project with a borrowed finance event-study lens.
I am not switching the whole project to finance yet because the venue and
literature contribution are not finance-specific.
```

For higher confidence:

```text
This project now appears finance-facing: returns data, event-study design,
abnormal-return claims, and JF/RFS-style contribution language all point to
finance. I proposed updating `.qiongli/guidance_manifest.yaml` to
`active_subject: finance`.
```

## Non-Goals

- Do not require users to choose a subject during installation.
- Do not auto-lock a subject from one request.
- Do not treat method borrowing as a full subject switch.
- Do not duplicate canonical workflow contracts inside subject overlays.
- Do not make marketplace or Desktop installs depend on the Python CLI.
- Do not silently persist guidance when the user has not opted into apply mode
  or previously confirmed the subject.

## Acceptance Criteria

- Full installs default to adaptive core with `active_subject: auto`.
- Runtime packets expose subject candidates, confidence, evidence, method
  lenses, borrowed lenses, loaded resources, and persistence status.
- Borderline projects can borrow method-level audited resources without
  switching the whole project subject.
- Explicit or repeated subject evidence can promote from suggested to confirmed
  guidance.
- Locked user subjects are respected.
- CLI, marketplace, and client-native skill installs share the same
  `subject_refinement` decision contract even when persistence differs.
