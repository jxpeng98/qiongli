# Qiongli Wordmark Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove the current Qiongli icon from public-facing docs surfaces and replace it with a dependency-free academic wordmark treatment.

**Architecture:** Keep the documentation source text-only for brand surfaces. VitePress config removes logo and favicon references, page frontmatter removes hero images, and CSS styles VitePress title/hero brand text with a local serif font stack while leaving body copy unchanged.

**Tech Stack:** VitePress, Markdown frontmatter, CSS, GitHub README HTML.

---

### Task 1: Remove Visible Icon References

**Files:**
- Modify: `docs/.vitepress/config.mjs`
- Modify: `docs/index.md`
- Modify: `docs/zh/index.md`
- Modify: `README.md`
- Modify: `README_CN.md`
- Delete: `docs/public/mark.svg`

- [ ] **Step 1: Remove VitePress favicon and logo config**

In `docs/.vitepress/config.mjs`, remove this item from `commonHead`:

```js
['link', { rel: 'icon', type: 'image/svg+xml', href: '/mark.svg' }]
```

Then replace the root `themeConfig` logo object:

```js
themeConfig: {
  logo: { src: '/mark.svg', alt: 'Qiongli' }
}
```

with:

```js
themeConfig: {}
```

- [ ] **Step 2: Remove hero image frontmatter**

In both `docs/index.md` and `docs/zh/index.md`, delete:

```yaml
  image:
    src: /mark.svg
    alt: Qiongli logo
```

- [ ] **Step 3: Remove README header image**

In both `README.md` and `README_CN.md`, delete:

```html
  <img src="docs/public/mark.svg" alt="Qiongli logo" width="104" height="104">
```

- [ ] **Step 4: Verify no source reference remains**

Delete `docs/public/mark.svg`.

Run:

```bash
rg -n "mark\\.svg|Qiongli logo|logo: \\{ src" README.md README_CN.md docs/index.md docs/zh/index.md docs/.vitepress/config.mjs docs/public
```

Expected: no matches.

### Task 2: Apply Academic Wordmark Typography

**Files:**
- Modify: `docs/.vitepress/theme/custom.css`

- [ ] **Step 1: Add brand font variable**

Add this variable to `:root` in `docs/.vitepress/theme/custom.css`:

```css
--qiongli-font-family-brand: "Iowan Old Style", "Palatino Linotype", Palatino, "Songti SC", "STSong", Georgia, serif;
```

- [ ] **Step 2: Replace logo-only styling with wordmark styling**

Remove:

```css
.VPNavBarTitle .logo {
  border-radius: 6px;
}
```

Add:

```css
.VPNavBarTitle .title,
.VPHero .name {
  font-family: var(--qiongli-font-family-brand);
  font-weight: 600;
  letter-spacing: 0;
}

.VPNavBarTitle .title {
  color: #12344d;
}
```

- [ ] **Step 3: Confirm CSS selectors are present**

Run:

```bash
rg -n "qiongli-font-family-brand|VPNavBarTitle \\.title|VPHero \\.name|VPNavBarTitle \\.logo" docs/.vitepress/theme/custom.css
```

Expected: matches for `qiongli-font-family-brand`, `.VPNavBarTitle .title`, and `.VPHero .name`; no match for `.VPNavBarTitle .logo`.

### Task 3: Ignore Local Mockup Artifacts

**Files:**
- Modify: `.gitignore`

- [ ] **Step 1: Ignore visual companion artifacts**

Add this line near other local/generated tool artifacts:

```gitignore
.superpowers/
```

- [ ] **Step 2: Verify mockup artifacts are ignored**

Run:

```bash
git status --short --ignored .superpowers
```

Expected: `.superpowers/` appears only as ignored output.

### Task 4: Build And Review

**Files:**
- Review: all modified files

- [ ] **Step 1: Build docs**

Run:

```bash
npm run docs:build
```

Expected: VitePress build exits with status 0.

- [ ] **Step 2: Check final source references**

Run:

```bash
rg -n "mark\\.svg|Qiongli logo|VPNavBarTitle \\.logo" README.md README_CN.md docs docs/.vitepress/config.mjs
```

Expected: no source matches except generated or ignored files if they are explicitly excluded.

- [ ] **Step 3: Review git diff**

Run:

```bash
git diff -- README.md README_CN.md docs/.vitepress/config.mjs docs/.vitepress/theme/custom.css docs/index.md docs/zh/index.md docs/public/mark.svg .gitignore
```

Expected: diff only removes logo usage and the old public logo asset, adds wordmark typography, and ignores `.superpowers/`.
