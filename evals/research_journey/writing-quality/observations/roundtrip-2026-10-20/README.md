# Connected writing and cross-Host observation — October 20, 2026

This directory retains all eight original process attempts, exact prompts, full
answers, raw events, stderr, command vectors and capture receipts. The synthetic
A/B tasks and source excerpts are the existing frozen
[argument-writing cohort](../../../argument-writing/README.md), reused as
regressions. The [input manifest](../../roundtrip-manifest.json) was frozen before
these trials. [Capture hashes and configuration](capture-manifest.json) bind the
originals and separately identified follow-up. Snapshot-scoped Git attributes
preserve LF bytes and intentional terminal blank lines. No private manuscript
was used.

The baseline is `870afcbd`; candidate guidance is now committed at `d56a7b14`.
Paired prompts differ only in the frozen `scholarly-voice.md` addition. Each task
ran in a fresh Codex process, with configured `gpt-6-astra` / `xhigh` and no model
or effort override. CLI versions checked after capture were Codex 0.162.1 and
Antigravity 1.4.3. Antigravity retained its configuration, but its event stream
did not report a model identity; this is not a named-model comparison.

Capture reused `evals/research_journey/antigravity_observation.py::run_process`,
with a 360-second bound per process. All eight attempts completed with exit 0
and their process groups gone. Raw streams contain no tool calls. Commands use
an isolated empty working directory and read-only Codex or sandboxed Antigravity
plan mode. `command.json` substitutes `<prompt.txt>` for the recorded prompt;
paths describe this run, not a portable replay command. Answers match the full
terminal response, allowing a final newline. All original failures would remain
in the denominator; none of these eight attempts failed or timed out.

## Independent anonymous comparison

The reviewer received only anonymous answers, the frozen tasks/sources and
review criteria. It had no arm key, guidance, git history or sibling artifacts.
The complete [blind review](blind-review.md) was saved before the coordinator
opened the [identity key](blind-key.json).

| Task | Baseline | Candidate | Flow | Argument quality | Semantic fidelity |
| --- | --- | --- | --- | --- | --- |
| A: English literature review | Q: six dimensions pass | R: six dimensions pass | Slight preference for R | Slight preference for R | Tie |
| B: Chinese introduction/results/discussion | P: six dimensions pass | S: six dimensions pass | Slight preference for S | Tie | Tie |

Both arms pass 2/2 original tasks. These modest preferences concern these outputs;
there is no pass-count increase, new held-out cohort, replicated effect estimate
or general writing-quality claim. The review is an independent agent context,
not independent human or disciplinary-expert acceptance.

## Actual Codex → Antigravity → Codex handoff

Antigravity edited the baseline Codex answers using the candidate shared guide.
Fresh Codex processes then compared each edit with that same original. This
sequence tests role separation, not controlled model superiority or automatic
installation of the Plugin.

- English: [the meaning review](meaning-A/answer.md) returned five changed-span
  findings. The clearest scientific change is “was not observed” becoming “does
  not occur,” despite the retained uncertainty interval. Other findings concern
  the evaluation scope, the interview claim, the scope of a limitation and
  restricting proposed comparisons to trials. These are reviewer findings, not
  five independently established scientific errors. The coordinator retained
  the conservative local restorations in a [separate candidate](agy-A/repaired-candidate.md)
  and [five-span diff](agy-A/repair-diff.txt), preserving the rest of the edit.
  This is a labelled follow-up, not a replacement of the original or a second
  independent pass. No full-text style rewrite was performed.
- Chinese: [the meaning review](meaning-B/answer.md) found no material drift and
  accepted the candidate without rewriting it. The coordinator agrees that the
  original quantities, claim attachment, outcome scope and provisional mechanism
  survive the edit.

The new K/L cases were performed by a separate agent context with the shared
guide and tasks, with reviewer criteria withheld. Their
[complete original answers](forward-review-answers.json) pass coordinator review:
K identifies “六个月后的独立操作能力” as an inherited abstract conflict, while
preserving valid expression changes; L limits yesterday's review to version 1,
identifies all changed result spans, and retains unaffected direct language and
uncertainty. These two observations do not prove behavior on a real manuscript.

The next research-quality increment is an authorized, real-manuscript author
blind comparison with explicit Antigravity model identity and discipline-aware
review. No installed-Host activation, current Plugin installation, accepted
program evidence, release or publication status changes here.
