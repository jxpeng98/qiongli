---
name: qiongli-proofread
description: Use when the user asks Qiongli to proofread, polish academic prose, reduce AI-like wording, check tone, final copyedit, or /proofread routing.
---

# Qiongli Proofread

This Codex wrapper mirrors the cross-platform `/proofread` workflow entrypoint.

## Canonical Route

- Use `$qiongli` as the main Qiongli skill for trigger rules, project guidance, and subject overlays.
- Follow `../qiongli-workflow/workflows/proofread.md` as the source of truth for task order, artifacts, and quality gates.
- Keep behavior aligned with Claude Code `/proofread` and Antigravity workflow routing.

Do not duplicate or reinterpret workflow logic in this wrapper. If this wrapper and the canonical workflow disagree, the canonical workflow wins.
