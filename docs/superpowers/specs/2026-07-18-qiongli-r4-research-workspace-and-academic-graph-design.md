# Qiongli R4 Research Workspace And Academic Graph Design

Status: proposed R4 Alpha.2 foundation and R5 Beta maturity contract

Decision date: July 18, 2026

Roadmap authority:
`docs/superpowers/roadmaps/2026-07-13-qiongli-2-accelerated-rust-migration-roadmap.md`

## Problem

Qiongli already defines durable academic artifacts for one paper, including
research state, decision history, literature maps, evidence ledgers, and
claim-evidence maps. Those artifacts are currently easier for a workflow agent
to write than for a researcher to discover, compare, resume, or inspect as one
coherent product.

Two user journeys are missing:

1. A researcher needs a library of multiple article projects, with each
   project's idea, stage, decisions, evidence position, risks, and next actions
   preserved across long periods of work.
2. A researcher needs an Obsidian-like visual map that explains not only which
   papers are connected, but which concept, claim, contradiction, method, or
   manuscript section connects them and why.

Saving host conversation sessions does not solve either journey. A session is
runtime history owned by Codex, Claude, ChatGPT, or another client. It contains
tool chatter, prompts, transient errors, credentials risk, and platform-specific
metadata. Qiongli needs the durable academic meaning extracted from that
conversation, not a second chat archive.

## Product Decision

R4 adds one local-first Research Workspace with four first-class concepts:

- **Research Library:** a cross-project index of article projects known to the
  installed Qiongli product;
- **Article Project:** one durable paper-level unit rooted in the existing
  `RESEARCH/<topic>/` contract;
- **Research Capture:** a normalized academic summary or delta imported from a
  conversation, note, CLI run, or manual entry without retaining the source
  session;
- **Academic Graph:** a typed, source-anchored projection of the project's
  literature, ideas, decisions, claims, evidence, and manuscript structure.

Research Capture and Academic Graph use the same project state. They are not
separate note-taking and visualization products.

## Goals

- manage multiple article projects from the native App, CLI, and Full MCP;
- preserve each project's intellectual state across platforms and model
  switches;
- import a concise article-level update without scraping or copying a complete
  host session;
- show how papers, concepts, claims, evidence, decisions, and manuscript
  sections connect;
- make every scholarly graph relation inspectable through a source artifact,
  anchor, rationale, and evidence limit;
- retain portable Markdown, CSV, BibTeX, and JSON artifacts as the durable
  project record;
- keep all mutation preview-first, revision-checked, atomic, and recoverable;
- run with no Python or Node production dependency.

## Non-Goals For Alpha.2

- cloning Codex, Claude, ChatGPT, or other clients' session stores;
- silently reading undocumented host caches or conversation databases;
- real-time multi-user editing or a hosted collaboration service;
- replacing Zotero as the authority for a user's reference library;
- treating proximity in a force-directed graph as scholarly evidence;
- allowing an inferred graph edge to overwrite an academic artifact silently;
- requiring Git, Obsidian, a language runtime, or a cloud account.

## Conceptual Model

### Research Library

The Research Library is an installed-product index. It stores only the minimum
information required to find and present registered projects:

- stable `project_id`;
- display name;
- registered project-root reference;
- current stage and lifecycle status;
- last known semantic revision;
- last opened and last academically updated timestamps;
- health state and next safe action.

The index lives under the versioned Qiongli configuration home. Exact paths are
available only through explicit path inspection and remain redacted from
ordinary logs, errors, and copied diagnostics. The library does not own or copy
the project's academic files. Its Portfolio projection may federate the graphs
of registered projects to show shared concepts, sources, methods, datasets, and
explicit idea lineage. It does not create a second global authority for those
relations.

### Article Project

One `ArticleProject` represents one paper, review, dissertation article, or
other manuscript unit under `RESEARCH/<topic>/`. A project receives a stable,
portable identity in:

```text
RESEARCH/<topic>/context/project_manifest.json
```

The manifest contains only versioned identity and lifecycle fields. It does not
duplicate the full academic state already owned by:

- `context/research_state.md`;
- `context/decision_log.md`;
- `context/stage_handoff.md`;
- `context/boundary_review.md` and `context/idea_funnel.md` when present;
- `literature/literature_map.md`;
- `evidence/claim-evidence-ledger.csv`;
- `manuscript/claims_evidence_map.md`;
- paper notes, bibliography, synthesis, design, analysis, and manuscript
  artifacts referenced by those files.

The existing repository-level `.qiongli/guidance_manifest.yaml` remains the
subject and guidance contract. It must not be repurposed as a multi-article
library database.

### Research Capture

A Research Capture is a bounded, versioned academic packet. It records what the
researcher wants the paper project to remember after a conversation or work
session has ended.

Required fields:

```text
schema_version
capture_id
project_id
source_surface
captured_at
binding_state
base_project_revision
parent_capture_ids
capture_reason
summary
idea_changes
decisions_or_candidates
evidence_references
unresolved_questions
risks_or_contradictions
suggested_next_actions
affected_artifacts
transport
idempotency_key
semantic_digest
```

`source_surface` may identify a broad surface such as `codex`, `claude`,
`chatgpt`, `cli`, or `manual`. The packet does not contain a session ID, full
transcript, raw prompt/response history, tool log, credential, environment
dump, or hidden host path.

`base_project_revision` lets the merge service detect that another surface has
changed the paper since the conversation began. `parent_capture_ids` preserve
idea lineage across surfaces without preserving conversation lineage.
`idempotency_key` makes retry and repository sync safe.

Accepted captures are stored as append-only semantic records in:

```text
RESEARCH/<topic>/context/research_captures.jsonl
```

The capture history explains how the article idea changed over time. Major
academic choices still belong in `decision_log.md`; the capture ledger must not
become a second generic activity log.

### Academic Graph

The Academic Graph is a deterministic projection, not a proprietary source of
truth. It is rebuilt from canonical project artifacts plus explicit semantic
links stored in:

```text
RESEARCH/<topic>/graph/semantic_links.jsonl
```

The installed product may maintain a rebuildable local search and layout index
under its versioned data root. Deleting that index must not delete academic
meaning. Re-indexing the same project revision must produce the same semantic
node and edge identities.

## Graph Contract

### Node Types

The first graph schema supports:

- `project`;
- `research_question`;
- `idea`;
- `contribution`;
- `concept`;
- `literature_cluster`;
- `paper`;
- `claim`;
- `evidence`;
- `decision`;
- `gap`;
- `method`;
- `manuscript_section`;
- `artifact`;
- `task`.

Stable IDs reuse canonical identifiers where possible: `project_id`, citekey or
DOI, claim ID, decision ID, task ID, and normalized artifact anchor. Alias
resolution is explicit and reviewable; it must not merge two papers or concepts
only because their display labels are similar.

### Edge Types

The first schema supports typed relations including:

- `cites` and `cited_by`;
- `supports` and `weakens`;
- `contradicts`;
- `extends`;
- `defines` and `operationalizes`;
- `uses_method`;
- `belongs_to_cluster`;
- `complements` and `competes_with`;
- `combines_with`;
- `motivates` and `informs`;
- `addresses_gap`;
- `appears_in_section`;
- `derived_from`;
- `supersedes`;
- `bounded_by`;
- `shares_source` and `shares_concept`;
- `forked_from` and `extends_project`.

Every scholarly edge other than a purely structural containment edge must
carry:

```text
edge_id
source_node_id
relation
target_node_id
rationale
artifact_path
source_anchor
evidence_limit
inference_strength
confidence
status
created_from_capture
```

`inference_strength` follows the existing academic contract:
`direct_evidence`, `reasonable_inference`, or `unsupported_gap`. A visual edge
without a rationale and anchor is navigation metadata, not a scholarly claim.

### Graph Layers

The UI presents five interoperable layers rather than one undifferentiated
hairball:

1. **Portfolio topology:** multiple article projects, shared concepts, reused
   sources, common methods or datasets, and explicit idea ancestry.
2. **Literature topology:** papers, citation links, clusters, methods,
   contradictions, and open problems.
3. **Idea and decision history:** research questions, candidate ideas, locked
   decisions, rejected alternatives, revisit triggers, and supersession.
4. **Argument topology:** claims, supporting or weakening evidence, gaps,
   confidence, and limitations.
5. **Manuscript topology:** sections, claims used in each section, cited papers,
   and the point where multiple literature streams are combined.

The Portfolio layer federates project-local projections. Cross-project
connections require an exact shared identifier or an explicit reviewed link;
display-name similarity alone is never enough to claim that two ideas, papers,
or concepts are the same.

The combined view is available, but filters and focus paths must let a user ask
questions such as:

- Which article ideas share a source or concept, and which one was forked from
  another project?
- Which papers support this claim?
- Through which concept are these two literature streams connected?
- Where does the manuscript combine those streams?
- Which decision narrowed this research question?
- Which central claims remain unsupported or contradicted?
- What changed between two project revisions?

## Capture And Merge Flow

```text
Codex / Claude / ChatGPT / CLI / manual note
  -> Research Capture v1
  -> validate, redact, deduplicate, and resolve project identity
  -> preview academic delta and conflicts
  -> exact revision plus explicit approval
  -> atomic project-artifact update
  -> append capture and decision provenance
  -> rebuild graph projection
  -> show changed nodes, edges, risks, and next actions
```

The merge service classifies each proposed item as:

- new;
- duplicate;
- refinement;
- contradiction;
- supersession;
- unresolved candidate;
- unsupported gap.

No capture may silently replace a locked decision, broaden a boundary, upgrade
an inference to direct evidence, or turn an unsupported claim into a citation.
Those cases require an explicit conflict plan and preserve the prior state.

## Cross-Surface Observability Boundary

Qiongli cannot passively discover every place where a user discussed a paper.
A local App cannot inspect a private Codex Cloud, Claude, ChatGPT, or other web
conversation, and a skill-only cloud session cannot write into a user's local
Qiongli data root. Host session databases and hidden caches are not supported
integration contracts.

Qiongli can observe only explicit, auditable signals:

- a Qiongli workflow bound to a known article project;
- a Research Capture delivered through a connected MCP, repository inbox,
  portable packet, or later approved relay;
- a semantic revision to a registered project artifact;
- an accepted capture, conflict resolution, decision, or graph update produced
  by the native services.

If a registered project changes without a matching capture, Qiongli records an
`unattributed_change` and asks the user to reconcile it. It does not guess which
model, person, or session made the change. A session that never invokes
Qiongli, writes a registered artifact, or exports a capture remains `unknown`.

### Project Binding Contract

Every cross-platform Qiongli workflow starts by resolving a bounded
`ProjectBinding`:

```text
schema_version
project_id
project_revision
paper_type
current_stage
active_task_ids
capture_policy
project_digest
```

The binding contains no absolute local path, secret, transcript, or full paper
content. A local Full MCP resolves it directly. A repository-backed agent reads
the portable project manifest and current research-state revision. A cloud
surface without either asks the user to select or name the project and produces
an `unbound` capture that can be assigned later.

Bindings are optimistic, not exclusive locks. The capture records the base
revision it saw; the native merge service handles later divergence.

### Semantic Capture Checkpoints

Qiongli emits or offers a capture when academic meaning changes, not after
every chat message. Required checkpoints are:

- research question, thesis, contribution, scope, or definition changes;
- a decision becomes tentative, locked, reopened, rejected, or superseded;
- a paper, dataset, analysis result, or citation changes claim support;
- a contradiction, null, limitation, boundary, or unsupported gap appears;
- a canonical stage closes or a high-risk stage handoff occurs;
- the user explicitly chooses `Save to Qiongli`;
- a workflow ends after material academic changes.

Pure wording edits, tool diagnostics, retries, and runtime chatter do not create
academic captures. An abrupt cloud-session close may prevent the final
checkpoint; the coverage view must show that limitation rather than infer a
complete history.

## Cross-Platform Intake And Delivery

Local clients with the Full MCP can submit or query captures through the same
typed service used by the App and CLI. The initial public command family is:

```text
qiongli project create
qiongli project register
qiongli project list
qiongli project show
qiongli project capture preview
qiongli project capture apply
qiongli project graph query
qiongli project export
qiongli project import
qiongli project doctor
```

Full MCP exposes equivalent project-list/read, capture-preview/apply, and
graph-query operations through the shared services. Mutating MCP calls retain
the native ToolHost project-root, revision, approval, redaction, limit, and
audit policy.

R4 supports three delivery modes:

1. **Connected bridge:** a local Codex, Claude Code, desktop, CLI, or other
   approved client calls the Full MCP and receives an immediate capture receipt.
2. **Repository inbox:** a cloud agent with access to the article repository
   writes a content-addressed packet under
   `RESEARCH/<topic>/context/capture_inbox/`. Qiongli previews it after the
   repository reaches the local machine; the agent does not mutate accepted
   capture history directly.
3. **Portable packet:** a skill-only or web session emits a `.qcapture.json`
   file or copyable capture block. The user imports or drops it into the App.

Accepted semantic history remains in `context/research_captures.jsonl`.
Transport acknowledgements, retry state, and device-specific delivery metadata
remain under the versioned Qiongli data root rather than becoming academic
content.

R5 may add a fourth **authenticated capture relay** for supported remote
surfaces. That relay requires a separate identity, pairing, encryption,
retention, abuse, deletion, and threat-model decision. It may carry bounded
capture envelopes and acknowledgements only; it does not receive raw sessions
or direct authority to mutate a project. Until that gate passes, the product
must describe remote coverage as repository-backed or user-mediated rather
than automatic.

### Coverage And Delivery States

The App and CLI expose explicit states instead of a misleading universal-sync
claim:

| State | Meaning |
|---|---|
| `connected` | A supported surface can reach the project service and receive an acknowledgement. |
| `repository_backed` | Capture packets or artifact changes arrive through a registered repository. |
| `portable_pending` | A packet exists but has not been imported or accepted. |
| `pending_review` | The native service received a capture that still needs merge approval. |
| `conflicted` | The base revision or academic meaning conflicts with current project state. |
| `current` | All received captures through the declared transport are applied or resolved. |
| `stale` | The project or surface has exceeded its declared capture-freshness policy. |
| `unbound` | A capture exists but is not assigned to a stable article project. |
| `unknown` | Qiongli has no supported observation signal for possible work on that surface. |

These states describe evidence Qiongli actually has. They are not a percentage
estimate of how much thinking occurred in private conversations.

## Native App Experience

### Research Library View

The App adds a first-class Research Library showing all registered article
projects with:

- title and paper type;
- current stage and status;
- last academic update;
- current thesis or focal question;
- strongest evidence position;
- unresolved contradiction or risk count;
- claim-evidence coverage;
- next-stage priorities;
- open, capture, graph, export, and doctor actions.

Projects may be filtered, sorted, archived, and reopened without deleting
their project directories. The same view offers a Portfolio map and timeline so
a researcher can see how article ideas branched, which projects reuse a source
or concept, and where two projects should remain separate. A cross-surface
coverage summary shows pending, conflicted, stale, unbound, and unknown work
without presenting silence as successful synchronization.

### Project View

Each project provides Overview, Timeline, Graph, Captures, Artifacts, and
Diagnostics surfaces. Timeline shows semantic project revisions and capture
summaries, not raw client sessions. Captures provides Inbox, Pending Review,
Conflicts, Applied, Rejected, Unbound, and Delivery Health queues. A capture
lineage view shows which surface proposed a semantic change, which project
revision it used, what it affected, and how it was resolved.

### Graph View

The graph supports:

- layer and relation filters;
- search and focus mode;
- shortest explanatory path between two nodes;
- before/after revision comparison;
- contradiction, gap, and low-confidence emphasis;
- an inspector showing rationale, anchor, evidence limit, and affected
  manuscript locations;
- opening the exact project artifact behind a node or edge;
- an accessible synchronized table/list representation for keyboard and screen
  reader use;
- optional surface and capture-revision overlays that show where a node or edge
  changed without implying that surface identity proves scholarly validity.

Node position, color, or size must not be the only carrier of meaning.

## Service And Trust Boundaries

- `ProjectStateService` owns project identity, registration, revision checks,
  capture validation, atomic writes, recovery, and portable import/export.
- `AcademicGraphService` owns deterministic projection, graph validation,
  query, comparison, and rebuildable indexing.
- The execution layer may propose capture summaries and semantic links, but it
  receives no direct mutation authority.
- The ToolHost grants writes only inside the selected registered project and
  only for the exact previewed artifact set.
- The UI renders typed snapshots and sends typed intents. It does not crawl
  project roots, parse papers, write artifacts, or edit the graph index.
- The global Research Library index never becomes authority for the project's
  academic content.
- Portable exports contain relative artifact anchors and no secrets, absolute
  host paths, raw sessions, or rebuildable local index.
- A capture transport can enqueue or acknowledge an envelope but cannot apply
  it to canonical project state; only the revision-checked merge service owns
  that authority.
- Connected-client, repository, portable, and future relay provenance is
  descriptive metadata. It does not increase evidence strength or confidence.

## Migration

Existing `RESEARCH/<topic>/` folders remain valid. Registration creates a
preview showing the identity file and any missing canonical continuity
artifacts. It must not rewrite existing research files merely to make the
project visible in the library.

Graph construction is incremental:

1. register the project and index existing artifact identities;
2. parse stable IDs and relations already present in the literature map,
   decision log, evidence ledger, claim map, bibliography, and outline;
3. report missing IDs, ambiguous relations, and unsupported edges as repair
   suggestions;
4. add explicit semantic links only after preview and approval.

Imported 1.x projects stay copy-on-migrate and rollback-safe under the R5 state
migration policy.

## Acceptance Contract

R4 Research Workspace and Academic Graph are accepted only when:

- one installed App can register, reopen, sort, and inspect at least three
  independent article projects after restart;
- App, CLI, and Full MCP consume one project/library service and return the
  same semantic revision and health state;
- captures from two local client surfaces and one portable file normalize to
  the same schema, reject duplicate replay, and never retain a raw session;
- connected, repository-backed, portable, stale, unbound, and unknown coverage
  states remain distinct in App, CLI, and export output;
- a repository-delivered capture and an unattributed artifact change are both
  detected after project refresh without guessing a private source session;
- capture preview identifies duplicate, refinement, contradiction,
  supersession, and unsupported-gap cases before any write;
- accepted changes update only the previewed project artifacts atomically and
  preserve locked decisions, unmanaged bytes, and recovery evidence;
- rebuilding the graph from the same revision produces identical semantic
  node and edge IDs;
- every displayed scholarly edge opens its rationale and source anchor, while
  unsupported relations remain visibly qualified;
- Literature, Argument, Manuscript, and combined graph layers answer the
  cross-stream and claim-evidence queries defined above, while the Portfolio
  layer shows exact-ID or explicitly reviewed relationships across projects;
- the graph has an accessible keyboard and screen-reader-equivalent table/list
  view and remains cancellable on a bounded large-project fixture;
- project export/import round-trips across macOS, Windows, and Linux without
  credentials, absolute paths, raw sessions, or a Python/Node runtime;
- deleting the rebuildable graph index does not lose academic content and a
  Doctor action can reconstruct it from portable artifacts.

## Roadmap Placement

This work is the first dependency slice of R4, before broad agent execution:

```text
R4A Research Library and native project state
  -> R4B Research Capture and conflict-aware consolidation
  -> R4C Academic Graph projection and native visualization
  -> R4D Full MCP, AgentBackend, and ToolHost execution
  -> R4E orchestration, recovery, and Alpha.2 acceptance
```

The ordering is intentional. Agents and orchestration need one durable project
identity, state model, evidence graph, and mutation boundary before they can
safely resume or coordinate a long-running paper.

R5 then matures this foundation through durable Inbox/Outbox delivery,
idempotent retry and acknowledgement, cross-device reconciliation, large
portfolio indexing, capture/decision lineage, coverage and conflict dashboards,
and three-platform recovery acceptance. An authenticated remote relay is a
separate R5 decision gate, not an implied consequence of the local App.
