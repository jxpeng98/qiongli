# Tauri Prototype Hardening

## 1. Scope / Trigger

Use this contract when adding or upgrading a desktop dependency that executes
inside the Tauri WebView, especially a deferred visualization module. Tauri
freezes `Object.prototype` before application modules load; a browser-only test
does not prove compatibility with the packaged App.

## 2. Signatures

```json
{ "app": { "security": { "freezePrototype": true } } }
```

```bash
pnpm --dir packages/qiongli-desktop build
```

The production bundle check starts an isolated Node process that executes:

```js
Object.freeze(Object.prototype);
await import('cytoscape');
```

## 3. Contracts

- Keep Tauri prototype hardening enabled.
- Every deferred production dependency must import after
  `Object.prototype` is frozen without changing global security state.
- The Cytoscape patch copies selector methods with
  `Object.defineProperties(target, Object.getOwnPropertyDescriptors(source))`;
  it must not use `Object.assign` for an inherited frozen `toString` property.
- Dependency patches live in `patches/`, are registered in
  `pnpm-workspace.yaml`, and are locked by `pnpm-lock.yaml`.

## 4. Validation & Error Matrix

- hardened import exits non-zero -> production build fails;
- patch missing or stale -> frozen-prototype import fails;
- `freezePrototype` disabled -> security regression, reject the change;
- hardened import passes but packaged renderer falls back -> inspect the App
  manually and fix the next renderer-specific failure before acceptance.

## 5. Good / Base / Bad Cases

- Good: the hardened import passes and the packaged App exposes topology v2,
  zoom, fit, selection, and the synchronized table fallback.
- Base: a dependency does not touch inherited prototype members and imports
  without a patch.
- Bad: disabling Tauri hardening or accepting the deterministic fallback as
  proof that the interactive renderer works.

## 6. Tests Required

- Run `pnpm --dir packages/qiongli-desktop build`; assert the hardened child
  import and production bundle budgets pass.
- Run the Cytoscape component and adapter tests.
- For package-input changes, open the exact-source macOS package and assert the
  renderer reports topology v2 rather than the unavailable-renderer fallback.

## 7. Wrong vs Correct

Wrong:

```json
{ "app": { "security": { "freezePrototype": false } } }
```

Correct:

```js
Object.defineProperties(target, Object.getOwnPropertyDescriptors(source));
```

The compatible copy preserves an own `toString` method without weakening the
WebView's global prototype-pollution defense.
