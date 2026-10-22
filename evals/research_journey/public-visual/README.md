# Public figures, synthesis and recovery — October 22, 2026

The bounded journey now completes: public source reading with six rendered pages,
reviewed source/note persistence, a Chinese/English literature review, and a fresh
project-ID-only continuation with an actual image read. This is coordinator model
review of two papers, not a first-attempt clean sweep, author/expert assessment,
installed-Plugin qualification or program acceptance.

## Sources and scope

| Source | Actual supplied version and coverage | Visual checks |
| --- | --- | --- |
| [Kestin et al., 2025](https://doi.org/10.1038/s41598-025-97652-6) | Publisher PDF, text pp.1–8; supplements unreviewed | pp.3–5, Figures 1–3: means versus medians, SEM, time distribution and perception measures |
| [Bastani et al., 2025](https://doi.org/10.1073/pnas.2422633122) | [Author-hosted 68-page PDF](https://hamsabastani.github.io/education_llm.pdf), text pp.1–22; appendices and final publisher body unreviewed | p.10 Table 1, p.16 Figure 2, p.18 Figure 3: score estimates, model-error distributions and engagement |
| [Bastani correction](https://doi.org/10.1073/pnas.2518204122) | Europe PMC correction body | Author affiliation correction; no reported result revision |

[Inputs](input-manifest.json) retain public download URLs, original SHA-256s and
the criteria frozen before model work. The native reader retrieved both PDFs and
returned the same source hashes as the downloaded/rendered bytes. Its DOI status
remains `unverified`, and its parsing/layout warnings remain in saved packets.
Neither a full download nor a parsed segment count establishes complete reading.
The native Bastani response exposes 50 of 93 segments; later segments were not
requested. The separately supplied Host text/image scope is explicit above.

Codex 0.162.1 retained configured `gpt-6-astra` / `xhigh`; no model or effort was
changed. Canonical reading/writing guides were copied before execution and their
bytes checked against `f3f5fc83` at integration. The frozen runtime is identified
by its binary hash; its `source-build` stamp does not establish a compile-time
commit. This pilot uses direct native Full MCP and explicit canonical guidance,
not an installed Plugin. The rebuilt candidate retains the same 35-tool inventory
with only the corrected saved-summary parameter description differing.

## Original attempts and follow-ups

| Attempt | Process time | Observed outcome |
| --- | --- | --- |
| `extraction` | Incomplete capture | Outer driver exited 143; original events retained, owned orphan group cleaned up; no completed answer claimed |
| `extraction-r2` | 397.285 s | Same task/source/images, bounded background capture; completed notes pass scoped source/visual review |
| `synthesis` | 56.077 s | Process completed but research task blocked: MCP read required approval under `never`; no draft produced |
| `synthesis-r2` | 304.524 s | Same prompt and sources after explicit read-tool policy correction; bilingual prose supported within scope; image reinspection unverified in this capture |
| `recovery` | 219.158 s | Fresh process discovers five saved bindings at revision 6, reopens the original table and returns a 145-word English continuation |

All five attempts remain in [the observation manifest](observation.json). A
completed process is separate from a completed research task. The second
synthesis also tried `limit: 100`, which the existing maximum-64 schema rejected;
it corrected the argument and continued. This tool error remains in its metrics.
Do not label the original sequence as a 3/3 first-pass success or erase retries.
Reported input-token counts include repeated/cached context; they are not unique
corpus size or a price estimate.

The extraction preserves paper-specific claim IDs, original versions/pages,
uncertainty and analysis units. It distinguishes 194 eligible students from 316
reported records, and 2,848 observations from independent participants. The review
separates assisted practice from subsequent testing, avoids pooling unlike
interventions and does not turn a nonsignificant Tutor estimate into equivalence
or a demonstrated independent-learning benefit. Both languages retain the
unresolved Kestin post-test AI restrictions and the limits on durability,
transfer and mechanism. These are judgments against the selected sources, not a
reanalysis of study data or a representative model-quality estimate.

The coordinator saved two exact source packets, then two reviewed note additions,
then the reviewed writing unit as `partial` `STG-F-001`, using the existing
preview/academic-approval/filesystem-approval/digest owner. Every previewed saved
artifact matches its declared hash. This summary is not formal manuscript or
claim-ledger consolidation, and no academic stage was promoted. Synthesis and
recovery preserve all project/config bytes during their model turns.

## Problems found and repairs

- The shared capture owner cleaned up timeout/exception paths but not an external
  SIGTERM. Source `1f1e6791` now handles main-thread SIGTERM/SIGINT, retains a
  cancelled result, cleans up the owned process group and restores prior signal
  handlers. Real subprocess regression tests cover both signals. The original
  interrupted model attempt remains an incomplete capture.
- An enabled MCP tool was incorrectly treated as approved in this pilot's setup.
  Correct only the three already-authorized read tools in the isolated profile;
  strict app-server `config/read` confirms effective per-tool policy. `mcp get
  --json` alone omits those settings. The replay guidance now names this check.
  The user's profile and research-write authority remain unchanged.
- The saved-document parameter description and evaluation README omitted existing
  stage-summary support. They now agree with the implemented reader/list owner;
  the regenerated 446-resource pack is committed at `cc14eceb` without a version
  bump. Existing old-Host capability checks remain applicable.
- The synthesis said it had inspected images, but exec JSON recorded only image
  hashes. Preserve that original and mark this run's reinspection unverified in
  a separate reviewed candidate; its Chinese/English prose is unchanged. The
  fresh recovery retains its public local rollout: a nested `tools.view_image`
  call returns one image whose decoded bytes exactly match saved Table 1. That
  call is absent from the simplified exec stream. This demonstrates a capture
  limitation, not that the earlier model necessarily skipped its image read.
  Replay instructions now require actual image-delivery evidence for this claim.

The model turns retain existing sandbox and model settings. No blanket tool
approval, installed-profile change, private manuscript access or publication was
needed. An initial use of `--strict-config` with schema generation was unsupported;
effective readback used the supported running app-server interface instead.
Temporary driver import/verification errors were corrected before the summary
preview and did not produce extra model observations.

## Verification and retained evidence

- 57 Python tests: capture/preflight, journey observations, portable routing and
  saved-document capability gates; 9 canonical MCP fixture tests.
- 16 native content tests, including corruption, path/limit and source-drift
  refusals; 1 CLI embedded-pack test. Native logs and bounded process receipts
  remain under `/tmp/qiongli-visual-native-checks`.
- 7 program-ledger/index tests; the generated 249-task index is current. These
  suites total 90 distinct tests; qualitative review and live cases are separate.
- Five live negative scenarios: missing academic approval, missing filesystem
  approval, stale revision, changed packet and missing packet. Refused operations
  write no project/config bytes. Each altered-source listing keeps the other four
  documents usable and withholds affected read arguments. Original fixture bytes
  are restored and checked.
- Source/guidance/capture hashes, terminal answers, source-bound recovery,
  before/after preservation and exact delivered table-image bytes are verified.
  The temporary authentication link is absent. Cache budget passes after native
  checks: about 1.75 GB listed caches and 73.5 GB free; no cache deletion needed.

Raw public PDFs/XML, rendered pages, model events, final answers, the separate
reviewed candidate, save receipts and the authorized local recovery rollout stay
outside Git at the `capture_root` in `observation.json`. That manifest binds their
hashes; the repository contains original evaluation metadata and replay tasks,
not redistributed papers or authentication data. The [existing execution
plan](../../../docs/superpowers/plans/2026-10-05-v2-4-plugin-quality-release-plan.md#october-22--public-visual-evidence-journey)
records integration checks and unchanged acceptance boundaries.

Next useful increments are another discipline and figure/table type, author or
subject-expert assessment, and fewer repeated guidance/packet reads while keeping
claim-level context. Antigravity model comparisons and private-manuscript review
remain separate; this pilot does not establish their readiness.
