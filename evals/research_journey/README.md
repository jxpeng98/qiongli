# Bounded research evidence journey

These two Evaluation Truth V1 cases share one explicitly synthetic abstract and
source registry. They check declared evidence links and requested claim coverage,
not academic truth or real Host execution. No private research or model call is
needed. Use the existing single-case runner with the shared fixture:

```sh
.venv/bin/python evals/runner/run_eval.py evals/research_journey/cases/reading-to-manuscript.yaml \
  evals/research_journey/fixtures/reading-to-manuscript --json-receipt /tmp/reading-journey.json
.venv/bin/python evals/runner/run_eval.py evals/research_journey/cases/source-to-paragraph.yaml \
  evals/research_journey/fixtures/reading-to-manuscript --json-receipt /tmp/paragraph-journey.json
.venv/bin/python -m unittest tests.test_research_journey_evals -v
```

The reading task requires reading observations. The paragraph task consumes the
source directly and does not inspect or require a new reading artifact. Scope is
declared before evaluation; deleting a required step from a failed result does
not turn it into a completed narrower task. Reordering rows, reusing evidence,
adding annotations and safely paraphrasing prose are allowed. Both requests need
the association C1 and causal limitation C2. Unused C3 can remain `needs_evidence`.

The fixed inputs are `source.md`, `sources.csv`, `references.bib` and
`required_claims.csv`; exact-byte digests bind them. Changing a source invalidates
this checkpoint, not the scientific claim itself. Review the affected claim and
bind fresh evidence before claiming verification again; never silently update a
digest to obtain a pass. Outputs are not compared to an exact prose answer.

`reading.csv` contains tabular observations of a note, summary and matrix;
`ledger.csv` retains the canonical evidence-ledger columns plus evidence scope;
`manuscript.csv` puts a passage alongside its declared claim/source mapping.
These CSVs are **test observations**, not replacements for canonical Markdown
artifacts or evidence that arbitrary Markdown has been parsed. The optional
adapter below binds model observations to actual answer and source bytes.

The case enforces source identity, locator and artifact binding, abstract-only
scope, supported status for used claims, nonempty narrative fields, and inclusion
of requested claim IDs. Cross-artifact checks compare whole tuples so swapping
sources between claims cannot hide behind matching individual column values.
In a multi-column `subset`, unused incomplete ledger rows may remain; they cannot
support a complete active claim. Paths are data in these tuples; only the pinned
source registry can establish their binding to a checked source file.

Semantic review remains necessary: a syntactically valid locator need not support
the claim, a nonempty passage may misstate the study, and a model may omit an
unmarked claim from its observations. A passing receipt proves only the declared
structural checks. It does not establish full B2 completion, causal validity,
submission readiness, installed Skill activation or live approval/CAS behavior.
The status-only full-cycle harness remains historical coverage, not this gate.

## Answer-bound observations

`observe.py` reuses the routing probe's isolated Codex capture and snapshot-only
`read_resource` MCP, then projects reviewed answer spans into the existing V1
cases. It has no new dependency, production parser or research-project writes.
Both cases are fixed before capture. Only the four input files above, the
request and entry are supplied; passing fixture CSVs and assertions are withheld.
The current configured model and reasoning effort are retained. As with the
routing probe, a custom provider or profile is unsupported, not silently changed.

Choose fresh local paths; capture invokes the model twice (up to 180 seconds per
turn). Prepare and score are offline and never call a model:

```sh
.venv/bin/python evals/research_journey/observe.py capture /tmp/journey-capture
.venv/bin/python evals/research_journey/observe.py prepare /tmp/journey-capture \
  --review /tmp/journey-review.json
# Review the actual answer/source, then edit journey-review.json as described below.
.venv/bin/python evals/research_journey/observe.py score /tmp/journey-capture \
  --review /tmp/journey-review.json --report /tmp/journey-report
.venv/bin/python -m unittest tests.test_research_journey_observations tests.test_skill_routing_probe -v
```

The capture retains exact final answers, raw events, partial timeout output,
source/guidance/case/producer snapshots, prompt/event hashes, elapsed time and
configured model/Host version. A failed trace stops new captures; failed and
unattempted cases still count in the fixed denominator of two. These are isolated
supplied-entry observations, not installed-Plugin activation or project access.
Guidance reads are counted only when matching successful MCP starts/results were
actually observed. Source access is explicitly `supplied-in-prompt`.

`prepare` starts with **unreviewed**, unmapped whole-answer spans. Set `reviewer`
to `{ "kind": "human" | "model", "id": "named reviewer" }`; a model's self-review
must not be labeled independent or human. Keep the captured answer/event hashes.
For each case, replace `segments` with ordered spans covering every non-whitespace
character. `start`/`end` are zero-based Python Unicode character offsets, end
exclusive; `quote` must exactly equal that answer substring. Each segment has:

- `role`: `reading`, `manuscript`, `context` (headings, formatting or task-status
  text that makes no research claim), or
  `unmapped` (unresolved content). Do not hide substantive claims as context.
- `links`: zero or more objects containing exactly `claim_id`, `source_location`,
  `status` and `claim_type`. Known source locations come from `sources.csv`;
  source identity, path and abstract-only scope are derived from that registry,
  never supplied or widened by the reviewer. Duplicate links are rejected.
  Status is `supported`, `needs_evidence` or `unsupported`; claim type is
  `finding`, `limitation` or `speculation`. Only supported findings/limitations
  can pass the original manuscript case. Unused C3 need not produce a row.
- `verdict`: `pass`, `fail`, `unreviewed`, or `not-evidence` (context only), plus
  a `reason`. A completed judgment needs a nonempty reason; passing content
  needs a source link. Preserve defective prose and mark it, never repair it in
  `quote` or substitute a fixture. Mapped reading quotes populate the existing
  note/summary observation cells; this does not assert full B2 completion.
  Multiple spans sharing a source (reading) or full claim/source tuple
  (manuscript/ledger) join with newlines in one record. Annotation granularity
  must not multiply source records; every constituent quote remains separately
  bound in the review. Different claim/source tuples are not merged.

Complete all five `checks`, each with `status` (`pass`, `fail`, `unreviewed`),
nonempty `reason` and affected zero-based segment indices in `segments`:

| Check | Review against the supplied synthetic abstract |
|---|---|
| `association` | C1 remains a positive association; any sample/design/r values agree |
| `causality` | C2 is retained; association is not promoted to a causal effect |
| `statistics` | No invented CI, p-value, adjusted analysis or significance claim |
| `access` | No claimed full-text, external or project access beyond supplied abstract |
| `added_claims` | No unsupported additional finding or use of pending C3 as evidence |

Failure checks must reference at least one failed span. Full span coverage is
mechanical; whether a span is correctly classified and supported is still a
reviewer assertion, not automated entailment. Unassigned/incomplete review cannot
pass. Honest labeling and reviewer identity are not authenticated by local hashes.

Scoring revalidates frozen inputs, answer/event hashes, actual resource results,
span coverage and review binding, creates CSVs only from exact answer quotes,
and invokes the canonical V1 runner once per case. The original 16/13 assertions
plus one binding digest give 17/14 assertions. `summary.json` reports
`structural_passed` and `reviewed_passed` separately: a causal overclaim with valid
tuples can be structural-pass/semantic-fail. Missing or invalid bindings cannot
pass. Regrading uses a new report with a new review digest and leaves capture and
prior reports untouched. A report is local diagnostic evidence, not acceptance.

Keep raw events/stderr and complete capture snapshots in local test output, not
the repository. Portable evidence should include only necessary synthetic final
answers/source excerpts, review reasons, hashes and redacted metadata. Do not
publish private research, user configuration, credentials or raw reasoning.
Two first observations diagnose this configuration; they do not measure an
accuracy rate, model superiority, longer C→F continuity or cross-Host readiness.
