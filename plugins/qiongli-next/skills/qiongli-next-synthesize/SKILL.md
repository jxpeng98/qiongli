---
name: qiongli-next-synthesize
description: Use when the user asks Qiongli for evidence synthesis, meta-analysis, qualitative synthesis, effect sizes, quality assessment, or /synthesize routing.
---

# Qiongli Next Synthesize

This Codex wrapper mirrors the cross-platform `/synthesize` workflow entrypoint.

## Canonical Route

- Use `$qiongli-next` as the main Qiongli skill for trigger rules, project guidance, and subject overlays.
- Follow `../qiongli-workflow/workflows/synthesize.md` as the source of truth for task order, artifacts, and quality gates.
- Keep behavior aligned with Claude Code `/synthesize` and Antigravity workflow routing.

Do not duplicate or reinterpret workflow logic in this wrapper. If this wrapper and the canonical workflow disagree, the canonical workflow wins.
