# Guidance delivery and unavailable-image recovery

October 22, 2026. This increment follows the [attention/version pilot](../attention/README.md)
and reuses the earlier public education papers. It tests whether long guidance
actually reaches the Host model, then tests resumed writing with an unavailable
original image. The [observation](observation.json) binds frozen inputs, original
captures, output-delivery metadata, reviews and native save receipts. Reviews are
by the coordinator model; they do not establish expert or program acceptance.

## Diagnosis and change

Successful nested file reads did not guarantee delivery to the model. The earlier
attention baseline/candidate/balanced rollouts contain 3/1/3 outer truncation
warnings and 0/1/1 script failures. These are separate from the inner tool counts
already reported. Long guidance, broad tool catalogs and duplicate result
representations exceeded the outer budget; parsing a failed tool response also
discarded other results before they were returned.

The root Skill now distinguishes file-reader limits from outer wrapper budgets.
When the latter is unknown, it calls for one bounded section per returned batch,
retaining received context and recovering missing ranges from the same version.
It also avoids duplicate result representations. Workflow/content ownership and
research-write permissions stay with their existing owners.

The existing evaluation module adds `inspect_tool_delivery` for authorized public
rollouts. It retains call IDs, input/output hashes, warning/failure counts and
unknown/unmatched/pending cases without exporting payloads or reasoning. These
are observed markers, not complete resource-coverage evidence. A nested failure
can also cause an outer failure; the counts must not be added as independent
events. Missing rollout evidence is not zero failures.

## Frozen observations

All fresh sessions retain Codex 0.162.1 and the configured `gpt-6-astra` / `xhigh`.
They use the same native binary, direct Full MCP and frozen canonical guidance.
Strict running app-server readback verifies that only project-read, document-list
and document-read are enabled and approved. Network access and project writes are
disabled. This does not qualify an installed Plugin or another Host/model.

The education baseline and first candidate use the same saved revision-6 project,
two source packets, two reviewed notes and partial bilingual seed. Only the root
Skill differs; the prompt changes only its guidance-root path. Both trees and
review criteria are frozen before either observation. After the first candidate
still concatenates three long files under the default outer limit, a second
candidate explicitly distinguishes inner and outer budgets. Its inputs are frozen
before its run. This adaptive follow-up is not a confirmatory sample.

Each run has its own Host profile. Criteria are outside the model's read scope.
The allowed input directory contains sibling frozen guidance trees and fixture
metadata; observed guidance commands must be checked for cross-variant reads.
An unrelated missing-image fixture is prepared during the education observations;
the education project, copied configuration and selected guidance remain frozen.
The comparison is conditional on actual accessed inputs, not identical whole-root
filesystem snapshots.

| Education attempt | Seconds | Recorded calls / failed | Document reads | Content bytes | MCP response bytes | Input tokens | Outer truncations / failures |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| baseline | 239.7 | 23 / 2 | 15 | 129,186 | 142,595 | 604,228 | 2 / 1 |
| first candidate | 206.9 | 20 / 0 | 12 | 120,488 | 135,705 | 742,936 | 2 / 0 |
| explicit budgets | 228.8 | 35 / 2 | 16 | 94,034 | 109,994 | 890,436 | 1 / 1 |

The first candidate does not eliminate the three-guide bundle truncation. The
explicit-budget candidate reads later long guides in sections without another
guidance-bundle truncation or whole Stage F reread. Its initial root-plus-catalog
output still truncates before the model can act on the newly loaded guidance.
Its final draft retains the source distinctions, but additional calls and input
tokens preclude an overall efficiency claim. Single-run times also include
service variation and different failures; the lowest latency is the candidate
whose guide batching still fails.

The baseline and explicit-budget candidate each submit two search contexts above
the existing 512-byte maximum. They recover after refusal; their outer failures
come from parsing/storing failed results. Tool help now exposes the existing
128-byte default and 4..512-byte range, and directs longer passages to returned
read arguments. These observations use the preceding frozen binary; prevention
of the same error by the final help is not established by a new model run.

All three education runs preserve project/config bytes, recover the partial seed
and current claim bindings, and actually receive both original comparison images.
Coordinator review retains relative percentages versus score units, means versus
medians, assisted versus unaided outcomes, nonsignificance versus equivalence,
and package effects versus isolated mechanisms. Denominator, post-test access,
long-term and transfer gaps remain explicit. The bilingual arguments are coherent;
there is no independent prose-quality ranking or publication-ready claim.

Content bytes count decoded document `content` fields and omit search-hit text;
MCP bytes count structured responses once, including searches. Recorded calls omit
nested image tools; exact returned image bytes are checked separately. Input tokens
include repeated/cached context. Neither bytes nor zero warnings prove complete
resource delivery. October 23 correction: the loaded Stage F contract already
specified the default Chinese count (non-whitespace code points, including
punctuation/digits, excluding source-anchor citations). The earlier review's
claim that no precise convention was available was incorrect. Recounting the
three education bodies gives 506/502/507 characters; the missing-image body has
531 after excluding its six claim-anchor citations. All fit the requested
400–600 range. Raw reviews remain preserved; the observation appends this
correction without changing their hashes. Length compliance is separate from
the qualitative judgments above.

## Missing-image fixture

A separate project imports the previously reviewed attention source text and
claim distinctions through existing preview/academic approval/filesystem approval/
CAS. Fresh bindings point to fixture-local images. The v7 p.14 image is absent;
its historical expected digest and locator remain in the packet. The three other
page renders are present. Historical visual review is explicitly labeled as such.
No original image or earlier project is removed or edited.

Two coordinator capture requests are refused because the driver supplied invalid
evidence entries. A subsequent consolidation preview refuses a research-state
binding mistakenly selected as a source. The corrected driver uses the inspected
locator schema and selects source/note bindings by path, reusing the existing
pending capture. All failed inputs and logs are retained separately from model
outcomes. Source packets and the final four current bindings are verified before
the fresh recovery starts.

The completed recovery takes 232.7 seconds, with 24 recorded calls, 8 document
reads, 87,856 content bytes and 97,601 structured MCP bytes. It reports 781,813
input tokens, one outer bootstrap truncation and no script failures. The nested
image attempt actually rejects the absent p.14 path; the model handles that result
and receives the other three original images. This rejection is visible in the
outer output but not counted as a failed tool in simplified exec JSON.

The bilingual answer uses the checked caption for AIAYN-C07 and explicitly defers
AIAYN-C06's historical “its→Law/application” details. It neither claims to reopen
the absent image nor invents attention weights or a causal translation mechanism.
Version conflicts and test/development distinctions remain intact. Project/config
bytes remain unchanged. Existing unavailable-image guidance is sufficient for this
case; no new rule is added on the strength of an already correct behavior.

## Checks and remaining work

Affected checks total **71 distinct tests**: 38 Python evaluation/content gates,
9 MCP contract fixtures, 16 native content tests, 1 CLI embedded-pack test and
7 ledger/index tests. Checks repeated after changed inputs are counted once.
All bounded native processes exit successfully, are reaped and leave no process
group. Final native receipts and source/pack identity are bound by the observation.
The 446-resource pack remains version 2.5.3. The final 35-tool inventory differs
from the observed binary only in saved-document search help; validation and read/
write schemas are unchanged. The cache budget passes at about 1.75 GB listed
caches and 73.0 GB free, so no deletion is needed.

Final review covers canonical ownership, malformed/unknown delivery records,
non-disclosure of raw payloads, actual image delivery/rejection, unchanged saved
bytes and the distinction between nested success and model-visible output.
Next are scoped tool discovery during initial loading, a measured balance between
section size and round trips, and author/subject-expert review. The final search
help also needs a fresh behavior observation before claiming it prevents invalid
requests. These local outcomes do not establish overall speed or language-quality
improvement across papers, models or Hosts.

## Scope and replay

The education sources remain the Kestin publisher PDF and Bastani author PDF
identified in the [parent pilot](../README.md). The attention sources remain the
proceedings and arXiv v7 PDFs identified in the preceding report. No new source
retrieval occurs. Original PDFs, page renders, full answers and local rollouts
stay outside Git; the observation retains exact hashes and artifact locations.

Replay uses the [existing public-pilot setup](../tasks.md) and capture helpers in
a new private root. Preserve the selected model, effective read-tool policy,
before/after project maps and actual image-return evidence. Never overwrite an
earlier attempt. Temporary authentication links are removed after each run;
credentials are neither copied into captures nor committed.

This bounded development work does not promote task states, accepted evidence,
publication readiness or independent scientific review. Local integration targets
`2.x`; no push, installed-profile change or publication is requested.
