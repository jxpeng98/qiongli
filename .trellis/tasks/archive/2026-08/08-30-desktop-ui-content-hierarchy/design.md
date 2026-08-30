# Desktop UI content hierarchy design

## Context

The Desktop UI already has a sound layering model:

1. `src/app.css` owns Qiongli color, spacing, type, radius, and state tokens.
2. `src/lib/styles/shadcn.css` maps those tokens into Tailwind/shadcn roles.
3. `src/lib/components/ui/` owns low-level controls.
4. `src/lib/components/app/` owns recurring product composition.
5. Routes and feature components own domain-specific layout and copy.

The implementation keeps this boundary. The current discomfort comes from the
compact values and route-local overrides inside it, not from Svelte, Tailwind,
or shadcn-svelte. The green baseline (249 tests, one skip, zero check warnings)
also makes a framework migration a higher-risk solution to the wrong problem.

## Chosen direction

Use one neutral, comfortable productivity density for all users. Keep Geist and
the current monochrome visual language, but restore a readable semantic scale
and make content importance determine spacing and surface treatment.

### Typography roles

Define and consume shared roles rather than spreading new literal sizes:

| Role | Initial value | Use |
| --- | ---: | --- |
| Page title | existing `clamp(24px, 2.25vw, 32px)` | one title per route |
| Section title | `18px / 1.3` | primary page sections |
| Component title | `14px / 1.4`, semibold | cards, rows, alerts |
| Body | `14px / 1.5` | ordinary content and controls |
| Supporting | `13px / 1.5` | descriptions and secondary context |
| Label | `12px / 1.35`, medium | field labels, tabs, metadata headings |
| Micro | `11px / 1.35` | timestamps, hashes, terse technical metadata only |

The implementation may make small optical adjustments for Chinese glyphs and
data tables, but ordinary descriptive prose must not use the micro role.

### Spacing and rhythm

- Move shared page padding toward `24px` top, `clamp(16px, 2vw, 32px)` inline,
  and `36px` bottom.
- Use `12px` as the normal section gap, `8px` for tightly related content, and
  `12-16px` panel padding.
- Preserve 32px desktop and 44px coarse-pointer minimum interaction targets.
- Keep the current responsive grid collapse and one-scroll-owner rules.

These values deliberately sit between the older Rhea baseline and the compact
Nova baseline; they improve reading comfort without turning a productivity UI
into a spacious marketing page.

### Surface hierarchy

- A card or border represents a real grouping, state, or interactive boundary.
- Nested `InfoGrid` and `DescriptionGrid` content uses spacing and subtle surface
  contrast; it does not add a new visible box around every fact.
- Status color remains semantic and sparse. Neutral content stays neutral.
- Primary actions remain filled; repeated secondary actions become outline,
  ghost, text, or existing overflow-menu actions according to importance.
- Technical identifiers use the mono stack and tabular figures where useful.

### Content hierarchy

Every route follows the same reading order:

1. Where am I?
2. What is the current state or task?
3. What is the primary action?
4. What supporting evidence is needed now?
5. What technical detail is available on demand?

Use existing `DescriptionTip`, native `<details>`, tabs, and responsive data
views for progressive disclosure. Safety, failure, recovery, and destructive
consequences remain visible; they are never hidden merely to make a page tidy.

Representative adjustments:

- Overview: strengthen the product summary and flatten repeated status cards.
- Research Library: make project selection/current-project work visually
  dominant; keep the required portfolio topology available without letting its
  detailed map compete with the project task; isolate destructive unregister.
- Project workspaces and graph/capture/timeline pages: separate controls from
  results, keep current focus/status prominent, and disclose long diagnostics.
- Client Integrations and About: show readiness and next action first; keep
  versions, paths, digests, and diagnostic evidence in existing detail surfaces.

## Component changes

Prefer changes in this order:

1. `app.css` semantic type and density tokens.
2. Shared app primitives: `PageHeader`, `SectionHeader`, `StatePanel`,
   `MetricCard`, `InfoGrid`, `DescriptionGrid`, `ProjectWorkspaceBar`, and
   `DescriptionTip`.
3. Low-level card/button/tab sizing only where shared primitives cannot express
   the selected scale.
4. Route-local cleanup for the highest-density routes, replacing literal sizes
   with roles and changing layout only when token changes are insufficient.

Do not create a second design-system layer. Existing components remain the
owners, and a new component is justified only when at least two current callers
need the same behavior.

## Data and behavior contracts

No App API, native intent, snapshot, persistence, routing, or i18n-loading
contract changes. UI state remains where it is today. Progressive disclosure is
presentational local state or native HTML behavior and must not affect product
authority.

Chinese and English retain the same semantic structure. Copy may be shortened
or regrouped only when meaning, safety guidance, and action consequences remain
unchanged in both catalogs.

## Alternatives considered

### Revert the compact Nova commit wholesale

Useful as calibration evidence but rejected as the implementation. It would
restore larger numbers while also reviving older page composition and visible
copy decisions that later responsiveness work intentionally changed.

### Add a compact/comfortable density switch

Rejected for this task. It adds persistence, settings UI, dual visual testing,
and long-term maintenance before a second density has a demonstrated user need.

### Replace Geist or use platform-only fonts

Rejected. Geist is already loaded locally and paired with system/CJK fallbacks.
The issue is scale and hierarchy, not glyph quality; changing the font would add
visual churn without fixing the dense content model.

### Migrate framework or component library

Rejected. Svelte 5, Tailwind 4, Bits UI, and shadcn-svelte currently pass the
full Desktop test/check baseline and already expose the required primitives.
Migration would touch behavior and accessibility while leaving the content
hierarchy problem to solve afterward.

### Further direction if this pass is insufficient

The next useful change is information architecture within the current stack:
master-detail workspaces, task-focused overview sections, and stronger
progressive disclosure for diagnostics. A platform-native shell or framework
migration is considered only if measured interaction or accessibility limits
cannot be solved with existing primitives.

## Compatibility and rollback

- Preserve light/dark contrast contracts and all current responsive breakpoints.
- Preserve accessibility roles, keyboard focus, loading, empty, error, and
  disabled states.
- Make changes in reviewable layers: tokens/shared primitives first, then one
  representative page at a time.
- Each route cleanup can be reverted independently. If the shared scale causes
  clipping, revert the owning token/shared-component commit before touching
  product behavior.

