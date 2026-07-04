---
name: qiongli-next-compliance-check
description: Use when the user asks Qiongli for reporting compliance, PRISMA, CONSORT, STROBE, checklist review, citation risk, or /compliance-check routing.
---

# Qiongli Next Compliance Check

This Codex wrapper mirrors the cross-platform `/compliance-check` workflow entrypoint.

## Canonical Route

- Use `$qiongli-next` as the main Qiongli skill for trigger rules, project guidance, and subject overlays.
- Follow `../qiongli-workflow/workflows/compliance-check.md` as the source of truth for task order, artifacts, and quality gates.
- Keep behavior aligned with Claude Code `/compliance-check` and Antigravity workflow routing.

Do not duplicate or reinterpret workflow logic in this wrapper. If this wrapper and the canonical workflow disagree, the canonical workflow wins.
