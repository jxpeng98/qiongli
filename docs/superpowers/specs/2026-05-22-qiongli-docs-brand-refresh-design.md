# Qiongli Docs Brand Refresh Design

## Goal

Refresh README and the VitePress documentation site so they explain the current stable Qiongli system through two primary reader paths:

- new users who need to understand, install, and start quickly
- research users who need to see the workflow depth, quality gates, literature diagnostics, multi-agent modes, and auditable artifacts

The site should be based on the capabilities implemented by the current release, not on advertising the release number.

## Scope

- Add a reusable SVG mark for README, docs navbar, and plugin-facing surfaces.
- Rework the top of `README.md` and `README_CN.md` into a clear brand and entrypoint section.
- Improve VitePress information architecture and visual system.
- Rewrite the English and Chinese docs home pages around installation, workflow, quality gates, and research task routes.
- Tighten quickstart and install pages so stable default paths are clear, while beta/next remains secondary.

## Design

The visual identity uses a compact Q-shaped mark with an internal evidence path. The mark should work at small sizes, use no bitmap dependencies, and retain enough contrast on light and dark backgrounds.

The documentation homepage should avoid a marketing-only hero. It should provide immediate usable routes: install, run a workflow, choose a paper type, inspect quality gates, and understand when to use the orchestrator.

The README should be a project front door, not an exhaustive manual at the top. Detailed architecture and maintainer material can remain later in the file or in docs.

## Acceptance Criteria

- `docs/public/mark.svg` renders as a polished project logo.
- README top section includes the logo, current stable installation path, research workflow positioning, and direct links to docs.
- VitePress homepage and Chinese homepage present current Qiongli capabilities in an operator-friendly order.
- Visual theme is restrained, professional, and readable; no large decorative gradient orb background.
- `npm run docs:build` succeeds.
