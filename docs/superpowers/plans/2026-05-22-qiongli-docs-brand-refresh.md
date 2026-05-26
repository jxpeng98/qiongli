# Qiongli Docs Brand Refresh Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Refresh README and the VitePress docs around the current Qiongli capabilities, with a reusable SVG logo and clearer new-user/research-user entrypoints.

**Architecture:** Keep the documentation site in VitePress and reuse the existing `docs/public/mark.svg` path for the logo. Treat README as the repository front door, docs home as the task router, and quickstart/install pages as operator instructions.

**Tech Stack:** Markdown, VitePress config/theme CSS, SVG, existing npm docs build.

---

### Task 1: Brand Mark And README Front Door

**Files:**
- Modify: `docs/public/mark.svg`
- Modify: `README.md`
- Modify: `README_CN.md`

- [ ] Replace the existing mark with a compact Q-shaped SVG that includes an evidence path.
- [ ] Add a centered logo/title/introduction block to both README files.
- [ ] Reorder the top README content so installation, docs, and research workflow routes are visible before deep architecture.

### Task 2: VitePress Site Structure And Theme

**Files:**
- Modify: `docs/.vitepress/config.mjs`
- Modify: `docs/.vitepress/theme/custom.css`

- [ ] Tune nav/sidebar labels around Guide, Workflows, CLI, Architecture, Advanced, and Maintainer.
- [ ] Keep the existing bilingual route structure.
- [ ] Replace decorative orb-like gradients with restrained product-docs styling.
- [ ] Keep cards and UI surfaces at modest radius and readable spacing.

### Task 3: Docs Home And Getting Started Pages

**Files:**
- Modify: `docs/index.md`
- Modify: `docs/zh/index.md`
- Modify: `docs/quickstart.md`
- Modify: `docs/zh/quickstart.md`
- Modify: `docs/guide/install.md`
- Modify: `docs/zh/guide/install.md`

- [ ] Rewrite home pages as task routers for install, workflow selection, quality gates, literature diagnostics, and multi-agent usage.
- [ ] Tighten quickstart pages to stable default workflows.
- [ ] Keep beta/next material available but secondary.

### Task 4: Verification

**Files:**
- No source edits expected.

- [ ] Run `npm run docs:build`.
- [ ] Run `python3 scripts/audit_distribution_payloads.py`.
- [ ] Run focused tests if documentation/config changes indicate a contract risk.
