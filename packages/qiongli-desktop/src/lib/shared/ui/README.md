# Qiongli UI

Qiongli UI is the Desktop application's design-system boundary. It combines:

1. **Bits UI primitives** for keyboard interaction, ARIA semantics, focus
   management, and overlays.
2. **Qiongli components** for the public API, semantic variants, layout, and
   application-specific patterns.
3. **CSS design tokens** for light/dark themes and component styling.
4. **A replaceable material layer** for restrained glass on shell and overlay
   surfaces.

Routes must import public components from `$lib/shared/ui`. They should not
import `bits-ui` directly or create new page-local loading, empty, metric, tab,
or dialog patterns.

## Material rules

- `solid`: default for content, data, forms, and dense reading surfaces.
- `glass`: reserved for persistent shell or contextual navigation.
- `glass-strong`: reserved for blocking overlays and transient top-level
  feedback.

Material classes are assembled through `materialClass` or `surfaceClass` so a
future renderer can replace CSS backdrop filtering without changing routes.
Dark mode deliberately uses higher opacity, lower saturation, softer
highlights, and less blur than light mode.

## Component rules

- Prefer semantic variants such as `warning` or `danger`; never encode a
  component contract as a raw color.
- Keep visual states in shared component tokens and accessibility behavior in
  Bits UI primitives.
- Extend the open-code wrapper in this directory when a pattern repeats. Do
  not add a second UI framework for a single component.
- Native elements remain appropriate for simple controls. Use a Bits UI
  primitive when focus management, roving keyboard navigation, disclosure, or
  overlay behavior would otherwise be reimplemented by a route.
