---
name: qiongli-ethics-check
description: Use when the user asks Qiongli for ethics, IRB text, consent, deidentification, data governance, disclosure, or /ethics-check routing.
---

# Qiongli Ethics Check

This Codex wrapper mirrors the cross-platform `/ethics-check` workflow entrypoint.

## Canonical Route

- Use `$qiongli` as the main Qiongli skill for trigger rules, project guidance, and subject overlays.
- Follow `../qiongli-workflow/workflows/ethics-check.md` as the source of truth for task order, artifacts, and quality gates.
- Keep behavior aligned with Claude Code `/ethics-check` and Antigravity workflow routing.

Do not duplicate or reinterpret workflow logic in this wrapper. If this wrapper and the canonical workflow disagree, the canonical workflow wins.
