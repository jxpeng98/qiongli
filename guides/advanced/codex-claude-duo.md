# Codex-Claude Duo

Use duo mode when Codex and Claude should provide complementary review without requiring a third runtime. Codex is strongest for implementation discipline, command evidence, artifact paths, and reproducibility. Claude is strongest for scholarly argument, reviewer empathy, manuscript structure, and assumption pressure.

## Common Patterns

Codex-primary:

```bash
python3 -m bridges.orchestrator task-run \
  --task-id I7 \
  --paper-type methods \
  --topic policy-effects \
  --cwd . \
  --execution-mode duo \
  --controller codex \
  --primary codex \
  --reviewer claude \
  --mcp-strict \
  --skills-strict
```

Claude-primary:

```bash
python3 -m bridges.orchestrator task-run \
  --task-id H3 \
  --paper-type empirical \
  --topic ai-in-education \
  --cwd . \
  --execution-mode duo \
  --controller claude \
  --primary claude \
  --reviewer codex \
  --mcp-strict \
  --skills-strict
```

## Review Contract

Each duo run should produce or preserve:

- primary artifact under `RESEARCH/[topic]/`
- reviewer notes with explicit accept/block/revise status
- `templates/duo-review-report.md` when a human-readable review artifact is needed
- `templates/disagreement-matrix.md` when Codex and Claude conflict on evidence, method, implementation, or acceptance

## Disagreement Matrix

| Case | Codex emphasis | Claude emphasis | Decision rule |
| --- | --- | --- | --- |
| Citation or claim support | Check artifact paths, ledgers, and reproducibility of evidence capture. | Check claim scope, venue expectations, and reader interpretation. | Narrow or mark the claim as a gap unless both evidence traceability and rhetorical scope are acceptable. |
| Method design | Check implementation feasibility, data requirements, and validation commands. | Check construct validity, alternative explanations, and reviewer objections. | Block until both executable method and defensible research design are present. |
| Code change | Check tests, diff scope, and rollback path. | Check whether the code still answers the research question and explains limitations. | Accept only when command evidence and research rationale agree. |
| Manuscript revision | Check required outputs, references, and contract conformance. | Check coherence, tone, reviewer empathy, and contribution clarity. | Revise until the artifact is both contract-valid and publication-facing. |
| Residual uncertainty | Preserve unresolved technical risk. | Preserve unresolved scholarly risk. | Escalate to `--triad` or human adjudication if the risk changes conclusions or downstream decisions. |

Do not average conflicting judgments. Record the concrete issue, cite the artifact or evidence reference, assign risk level, and choose accept, revise, block, or escalate.
