# Qiongli Wordmark Design

## Context

The current `docs/public/mark.svg` logo is too busy for small display contexts. It combines a
Q-shaped mark, book, prism, audit path, and multiple nodes, which weakens the brand at README,
navigation, hero, and favicon sizes.

The approved direction is to remove the logo for now and use a stronger text-only brand treatment.

## Approved Direction

Use a wordmark-first design:

- Remove the visual logo from the documentation hero.
- Remove the logo from the VitePress navigation title.
- Remove the SVG favicon link rather than replacing it with another icon.
- Remove the README header image in both English and Chinese READMEs.
- Style the documentation brand text with an academic serif font stack.

## Typography

Use this local/system font stack for brand display text:

```css
"Iowan Old Style", "Palatino Linotype", Palatino, "Songti SC", "STSong", Georgia, serif
```

This keeps the design dependency-free while giving Qiongli a more scholarly editorial tone. The base
documentation font remains the current sans-serif stack for readability.

## Implementation Scope

Update only source files:

- `docs/.vitepress/config.mjs`
- `docs/.vitepress/theme/custom.css`
- `docs/index.md`
- `docs/zh/index.md`
- `README.md`
- `README_CN.md`
- `.gitignore` for local `.superpowers/` mockup artifacts

Do not edit generated `docs/.vitepress/dist` files.

## Validation

Run the VitePress build or the repository validation command if available. Also inspect the docs home
page visually to confirm:

- no broken logo image remains;
- the hero layout still feels balanced without an image;
- the wordmark uses the approved serif treatment;
- README headers are clean text-only.
