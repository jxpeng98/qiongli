# Qiongli 2 R0 Native CI Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the rolling Qiongli 2 migration PR use fast Rust-native required checks while keeping frozen 1.x compatibility suites available only as manual diagnostics.

**Architecture:** A dedicated `Native CI` workflow owns `2.x` pushes and pull requests. A dependency-free Bash boundary guard rejects edits to frozen 1.x product/oracle and accepted architecture paths, while a three-platform Rust matrix runs format, check, Clippy, and workspace tests. Existing Python/Node and checkout workflows continue automatically for legacy branches and remain manually dispatchable against `2.x`.

**Tech Stack:** GitHub Actions, Bash 3.2-compatible scripting, Python `unittest` for focused repository-policy tests, Cargo/Rust 1.97.0, GitHub repository rulesets.

---

## File Map

| File | Responsibility |
|---|---|
| `.github/workflows/native-ci.yml` | Automatic required CI for `2.x`; runs the frozen-boundary guard and Tier 1 Rust workspace matrix only |
| `.github/workflows/ci.yml` | Legacy Python/Node compatibility CI for `main`, `master`, and `dev`, with manual dispatch available on any ref |
| `.github/workflows/install-check.yml` | Legacy checkout installation diagnostics for `main`, `master`, and `dev`, with manual dispatch available on any ref |
| `scripts/check_2x_native_change_boundary.sh` | Reject changes to frozen 1.x implementation/oracle and accepted 2.x architecture anchors without starting Python or Node |
| `tests/test_native_change_boundary.py` | Behavioral tests for allowed native changes and rejected frozen-path changes |
| `tests/test_branch_policy.py` | Static workflow routing, native command, platform matrix, and no-legacy-runtime assertions |
| `docs/maintainer/release-branch-policy.md` | English source of truth for R0 workflow and required-check policy |
| `docs/zh/maintainer/release-branch-policy.md` | Chinese mirror of the R0 workflow and required-check policy |
| `docs/superpowers/roadmaps/2026-07-13-qiongli-2-accelerated-rust-migration-roadmap.md` | R0 execution status and evidence receipt |

### Task 1: Add Failing R0 Policy Tests

**Files:**
- Create: `tests/test_native_change_boundary.py`
- Modify: `tests/test_branch_policy.py:15-25`
- Modify: `tests/test_branch_policy.py:142-190`

- [x] **Step 1: Create the boundary-guard behavioral tests**

Create `tests/test_native_change_boundary.py` with this complete test harness:

```python
from __future__ import annotations

import subprocess
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
SCRIPT = REPO_ROOT / "scripts" / "check_2x_native_change_boundary.sh"


class NativeChangeBoundaryTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp_dir = tempfile.TemporaryDirectory()
        self.repo = Path(self.temp_dir.name)
        self.run_git("init", "-b", "main")
        self.run_git("config", "user.name", "Qiongli Test")
        self.run_git("config", "user.email", "qiongli-test@example.invalid")
        self.write("README.md", "baseline\n")
        self.run_git("add", "README.md")
        self.run_git("commit", "-m", "baseline")
        self.base_ref = self.run_git("rev-parse", "HEAD").stdout.strip()

    def tearDown(self) -> None:
        self.temp_dir.cleanup()

    def run_git(self, *args: str) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            ["git", "-C", str(self.repo), *args],
            check=True,
            capture_output=True,
            text=True,
        )

    def write(self, relative_path: str, content: str) -> None:
        path = self.repo / relative_path
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content, encoding="utf-8")

    def commit_paths(self, *relative_paths: str) -> None:
        self.run_git("add", *relative_paths)
        self.run_git("commit", "-m", "test change")

    def run_guard(self) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [
                "bash",
                str(SCRIPT),
                "--repo-root",
                str(self.repo),
                "--base-ref",
                self.base_ref,
            ],
            check=False,
            capture_output=True,
            text=True,
        )

    def test_allows_native_workspace_changes(self) -> None:
        path = "packages/qiongli-native/apps/qiongli/src/main.rs"
        self.write(path, "fn main() {}\n")
        self.commit_paths(path)

        result = self.run_guard()

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("Native 2.x change boundary passed", result.stdout)

    def test_rejects_frozen_legacy_and_architecture_paths(self) -> None:
        paths = (
            "packages/python-qiongli/src/qiongli/__init__.py",
            "packages/qiongli-literature-mcpb/src/index.js",
            "tooling/migration/baselines/v1.19.0-beta.1/manifest.json",
            "tooling/migration/2x-branch-point.json",
            "docs/architecture/decisions/0201-native-executable-and-resource-architecture.md",
        )
        for path in paths:
            self.write(path, "changed\n")
        self.commit_paths(*paths)

        result = self.run_guard()

        self.assertEqual(result.returncode, 1)
        for path in paths:
            self.assertIn(path, result.stderr)
        self.assertIn("Frozen 1.x or accepted architecture paths changed", result.stderr)


if __name__ == "__main__":
    unittest.main()
```

- [x] **Step 2: Replace the branch-routing policy test**

Replace `test_ci_workflows_cover_legacy_and_native_development_branches` in
`tests/test_branch_policy.py` with:

```python
    def test_ci_routes_legacy_and_native_branches_to_separate_workflows(self) -> None:
        legacy_ci = read(".github/workflows/ci.yml")
        native_ci = read(".github/workflows/native-ci.yml")
        install_check = read(".github/workflows/install-check.yml")

        legacy_filter = 'branches: ["main", "master", "dev"]'
        native_filter = 'branches: ["2.x"]'
        old_filter = 'branches: ["main", "master", "dev", "2.x"]'

        self.assertEqual(legacy_ci.count(legacy_filter), 2)
        self.assertEqual(install_check.count(legacy_filter), 2)
        self.assertEqual(native_ci.count(native_filter), 2)
        self.assertNotIn(old_filter, legacy_ci)
        self.assertNotIn(old_filter, install_check)
        self.assertIn("workflow_dispatch:", legacy_ci)
        self.assertIn("workflow_dispatch:", install_check)
        self.assertIn("workflow_dispatch:", native_ci)
        self.assertIn("tooling/release/acceptance/**", legacy_ci)
        self.assertIn("tooling/release/acceptance/**", native_ci)
```

- [x] **Step 3: Point the native matrix policy test at the new workflow**

Replace `test_ci_has_independent_three_platform_native_rust_foundation_gate`
with:

```python
    def test_2x_native_ci_has_independent_three_platform_rust_gate(self) -> None:
        content = read(".github/workflows/native-ci.yml")
        start = content.index("  rust-native-foundation:")
        job = content[start:]

        self.assertIn("name: Rust native foundation (${{ matrix.platform }})", job)
        self.assertIn("fail-fast: false", job)
        for platform, runner in (
            ("Linux", "ubuntu-latest"),
            ("macOS", "macos-latest"),
            ("Windows", "windows-latest"),
        ):
            with self.subTest(platform=platform):
                self.assertIn(
                    f"          - platform: {platform}\n            os: {runner}", job
                )
        self.assertIn("uses: dtolnay/rust-toolchain@1.97.0", job)
        self.assertIn("components: rustfmt, clippy", job)
        self.assertIn("Reject injected target-specific Rust flags", job)
        self.assertIn("CARGO_TARGET_*_RUSTFLAGS", job)
        self.assertEqual(job.count("CARGO_HOME:"), 4)
        self.assertIn('CARGO_ENCODED_RUSTFLAGS: ""', job)
        self.assertIn('RUSTC_WRAPPER: ""', job)
        self.assertIn('RUSTFLAGS: ""', job)
        commands = (
            "cargo fmt --manifest-path packages/qiongli-native/Cargo.toml --all -- --check",
            "cargo check --manifest-path packages/qiongli-native/Cargo.toml --workspace --all-targets --all-features --locked",
            "cargo clippy --manifest-path packages/qiongli-native/Cargo.toml --workspace --all-targets --all-features --locked -- -D warnings",
            "cargo test --manifest-path packages/qiongli-native/Cargo.toml --workspace --all-targets --all-features --locked",
        )
        for command in commands:
            self.assertIn(command, job)
        self.assertEqual(
            [job.index(command) for command in commands],
            sorted(job.index(command) for command in commands),
        )
        self.assertNotIn("continue-on-error", job)
        self.assertNotRegex(job, r"(?m)^\s+if:")
        self.assertNotIn("cache:", job)

    def test_2x_native_ci_does_not_start_legacy_language_runtimes(self) -> None:
        content = read(".github/workflows/native-ci.yml")
        forbidden = (
            "actions/setup-python",
            "actions/setup-node",
            "python -m",
            "python3 ",
            "npm ",
            "packages/qiongli-lite-mcp",
            "packages/qiongli-literature-mcpb",
            "cross-platform-tests",
            "shell-release-gates",
            "bootstrap_qiongli",
        )
        for marker in forbidden:
            with self.subTest(marker=marker):
                self.assertNotIn(marker, content)
```

- [x] **Step 4: Run the focused tests and verify the red state**

Run:

```bash
python3 -m unittest tests.test_native_change_boundary tests.test_branch_policy -v
```

Expected: FAIL because `.github/workflows/native-ci.yml` and
`scripts/check_2x_native_change_boundary.sh` do not exist yet.

### Task 2: Implement the Frozen Native Change Boundary

**Files:**
- Create: `scripts/check_2x_native_change_boundary.sh`
- Test: `tests/test_native_change_boundary.py`

- [x] **Step 1: Add the dependency-free boundary guard**

Create `scripts/check_2x_native_change_boundary.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: check_2x_native_change_boundary.sh --base-ref REF [--head-ref REF] [--repo-root PATH]

Reject changes to the frozen 1.x product/oracle and accepted 2.x architecture
anchors while allowing Rust-native migration work elsewhere in the repository.
EOF
}

repo_root="$(pwd)"
base_ref=""
head_ref="HEAD"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --base-ref)
      [[ $# -ge 2 ]] || { usage >&2; exit 2; }
      base_ref="$2"
      shift 2
      ;;
    --head-ref)
      [[ $# -ge 2 ]] || { usage >&2; exit 2; }
      head_ref="$2"
      shift 2
      ;;
    --repo-root)
      [[ $# -ge 2 ]] || { usage >&2; exit 2; }
      repo_root="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

if [[ -z "$base_ref" ]]; then
  echo "--base-ref is required" >&2
  usage >&2
  exit 2
fi

git -C "$repo_root" rev-parse --verify "$base_ref^{commit}" >/dev/null
git -C "$repo_root" rev-parse --verify "$head_ref^{commit}" >/dev/null
git -C "$repo_root" merge-base "$base_ref" "$head_ref" >/dev/null

violations=()
while IFS= read -r -d '' path; do
  case "$path" in
    packages/python-qiongli/*|\
    packages/qiongli-literature-mcpb/*|\
    tooling/migration/baselines/v1.19.0-beta.1/*|\
    tooling/migration/qiongli-1x-baseline-plan.json|\
    tooling/migration/baseline-plan.schema.json|\
    tooling/migration/baseline-manifest.schema.json|\
    tooling/migration/oracle-fixture.schema.json|\
    tooling/migration/2x-branch-point.json|\
    tooling/migration/2x-branch-point.schema.json|\
    tooling/architecture/arc-201-decisions.json|\
    docs/architecture/decisions/020[1-7]-*)
      violations+=("$path")
      ;;
  esac
done < <(
  git -C "$repo_root" diff \
    --no-renames \
    --name-only \
    -z \
    "$base_ref...$head_ref" \
    --
)

if [[ ${#violations[@]} -gt 0 ]]; then
  echo "Frozen 1.x or accepted architecture paths changed:" >&2
  for path in "${violations[@]}"; do
    printf '  %s\n' "$path" >&2
  done
  echo "Use the critical-fix maintenance line or a superseding ADR instead." >&2
  exit 1
fi

echo "Native 2.x change boundary passed."
```

- [x] **Step 2: Make the guard executable**

Run:

```bash
chmod +x scripts/check_2x_native_change_boundary.sh
```

- [x] **Step 3: Run the behavioral tests**

Run:

```bash
python3 -m unittest tests.test_native_change_boundary -v
```

Expected: 2 tests pass.

### Task 3: Route 2.x Through Dedicated Native CI

**Files:**
- Create: `.github/workflows/native-ci.yml`
- Modify: `.github/workflows/ci.yml:1-10`
- Modify: `.github/workflows/install-check.yml:1-11`
- Test: `tests/test_branch_policy.py`

- [x] **Step 1: Add the dedicated native workflow**

Create `.github/workflows/native-ci.yml` with two jobs:

```yaml
name: Native CI

on:
  push:
    branches: ["2.x"]
    paths-ignore:
      - "tooling/release/acceptance/**"
  pull_request:
    branches: ["2.x"]
  workflow_dispatch:

concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true

permissions:
  contents: read

jobs:
  native-change-boundary:
    name: Native 2.x change boundary
    runs-on: ubuntu-latest
    timeout-minutes: 5
    steps:
      - name: Checkout
        uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Resolve comparison base
        id: comparison-base
        shell: bash
        env:
          PULL_REQUEST_BASE: ${{ github.base_ref }}
          PUSH_BEFORE: ${{ github.event.before }}
        run: |
          set -euo pipefail
          zero_sha="0000000000000000000000000000000000000000"

          if [[ -n "$PULL_REQUEST_BASE" ]]; then
            base_ref="origin/$PULL_REQUEST_BASE"
          elif [[ -n "$PUSH_BEFORE" && "$PUSH_BEFORE" != "$zero_sha" ]] &&
               git cat-file -e "$PUSH_BEFORE^{commit}" 2>/dev/null; then
            base_ref="$PUSH_BEFORE"
          elif git rev-parse --verify HEAD^ >/dev/null 2>&1; then
            base_ref="HEAD^"
          else
            base_ref="$(git rev-list --max-parents=0 HEAD)"
          fi

          echo "base-ref=$base_ref" >> "$GITHUB_OUTPUT"

      - name: Protect frozen legacy and architecture boundaries
        run: >-
          ./scripts/check_2x_native_change_boundary.sh
          --base-ref "${{ steps.comparison-base.outputs.base-ref }}"

  rust-native-foundation:
    name: Rust native foundation (${{ matrix.platform }})
    runs-on: ${{ matrix.os }}
    timeout-minutes: 30
    strategy:
      fail-fast: false
      matrix:
        include:
          - platform: Linux
            os: ubuntu-latest
          - platform: macOS
            os: macos-latest
          - platform: Windows
            os: windows-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@1.97.0
        with:
          components: rustfmt, clippy

      - name: Reject injected target-specific Rust flags
        shell: bash
        run: |
          injected=0
          while IFS='=' read -r name value; do
            case "$name" in
              CARGO_TARGET_*_RUSTFLAGS)
                if [[ -n "$value" ]]; then
                  echo "Unexpected Rust compiler override: $name" >&2
                  injected=1
                fi
                ;;
            esac
          done < <(env)
          [[ "$injected" -eq 0 ]]

      - name: Check native Rust formatting
        env:
          CARGO_HOME: ${{ runner.temp }}/qiongli-cargo-home
          CARGO_BUILD_RUSTC_WRAPPER: ""
          CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER: ""
          CARGO_BUILD_RUSTFLAGS: ""
          CARGO_ENCODED_RUSTFLAGS: ""
          RUSTC_WORKSPACE_WRAPPER: ""
          RUSTC_WRAPPER: ""
          RUSTFLAGS: ""
        run: cargo fmt --manifest-path packages/qiongli-native/Cargo.toml --all -- --check

      - name: Check native Rust workspace
        env:
          CARGO_HOME: ${{ runner.temp }}/qiongli-cargo-home
          CARGO_BUILD_RUSTC_WRAPPER: ""
          CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER: ""
          CARGO_BUILD_RUSTFLAGS: ""
          CARGO_ENCODED_RUSTFLAGS: ""
          RUSTC_WORKSPACE_WRAPPER: ""
          RUSTC_WRAPPER: ""
          RUSTFLAGS: ""
        run: cargo check --manifest-path packages/qiongli-native/Cargo.toml --workspace --all-targets --all-features --locked

      - name: Run native Rust clippy
        env:
          CARGO_HOME: ${{ runner.temp }}/qiongli-cargo-home
          CARGO_BUILD_RUSTC_WRAPPER: ""
          CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER: ""
          CARGO_BUILD_RUSTFLAGS: ""
          CARGO_ENCODED_RUSTFLAGS: ""
          RUSTC_WORKSPACE_WRAPPER: ""
          RUSTC_WRAPPER: ""
          RUSTFLAGS: ""
        run: cargo clippy --manifest-path packages/qiongli-native/Cargo.toml --workspace --all-targets --all-features --locked -- -D warnings

      - name: Run native Rust tests
        env:
          CARGO_HOME: ${{ runner.temp }}/qiongli-cargo-home
          CARGO_BUILD_RUSTC_WRAPPER: ""
          CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER: ""
          CARGO_BUILD_RUSTFLAGS: ""
          CARGO_ENCODED_RUSTFLAGS: ""
          RUSTC_WORKSPACE_WRAPPER: ""
          RUSTC_WRAPPER: ""
          RUSTFLAGS: ""
        run: cargo test --manifest-path packages/qiongli-native/Cargo.toml --workspace --all-targets --all-features --locked
```

- [x] **Step 2: Restrict legacy CI automatic triggers**

Change the top of `.github/workflows/ci.yml` to:

```yaml
name: Legacy Compatibility CI

on:
  push:
    branches: ["main", "master", "dev"]
    paths-ignore:
      - "tooling/release/acceptance/**"
  pull_request:
    branches: ["main", "master", "dev"]
  workflow_dispatch:
```

Keep all existing legacy jobs unchanged so maintainers can dispatch the
workflow manually against `2.x` for a named compatibility investigation.

- [x] **Step 3: Restrict checkout-install automatic triggers**

Change the top of `.github/workflows/install-check.yml` to:

```yaml
name: Legacy Checkout Install Check

on:
  push:
    branches: ["main", "master", "dev"]
    paths-ignore:
      - "tooling/release/acceptance/**"
  pull_request:
    branches: ["main", "master", "dev"]
  workflow_dispatch:
```

- [x] **Step 4: Run the complete focused R0 policy suite**

Run:

```bash
python3 -m unittest tests.test_native_change_boundary tests.test_branch_policy -v
```

Expected: all focused tests pass.

- [x] **Step 5: Commit the workflow checkpoint**

```bash
git add .github/workflows/native-ci.yml .github/workflows/ci.yml \
  .github/workflows/install-check.yml scripts/check_2x_native_change_boundary.sh \
  tests/test_native_change_boundary.py tests/test_branch_policy.py
git commit -m "ci(native): route 2.x through Rust-only gates"
```

### Task 4: Update R0 Governance And Status

**Files:**
- Modify: `docs/maintainer/release-branch-policy.md:54-89`
- Modify: `docs/maintainer/release-branch-policy.md:171-190`
- Modify: `docs/zh/maintainer/release-branch-policy.md:47-77`
- Modify: `docs/zh/maintainer/release-branch-policy.md:111-132`
- Modify: `docs/superpowers/roadmaps/2026-07-13-qiongli-2-accelerated-rust-migration-roadmap.md:143-164`

- [x] **Step 1: Replace the obsolete 2.x workflow policy in English**

Document these exact rules under `2.x Native Branch Governance` and
`Development Flow`:

```markdown
`Native CI` is the only automatic workflow for pushes to `2.x` and pull
requests targeting it. Its required checks are `Native 2.x change boundary`
plus `Rust native foundation` on Linux, macOS, and Windows. The native matrix
runs format, check, Clippy, and workspace tests from the same commit without
starting Python or Node.

`Legacy Compatibility CI` and `Legacy Checkout Install Check` continue to run
automatically for `main`, `master`, and `dev`. Both remain manually
dispatchable against a named `2.x` ref when a specific compatibility question
requires the frozen Python, Node, Rust Lite, distribution, or checkout oracle.
Their results are diagnostic and are not required checks for native 2.x work.
```

Record ruleset `18800504` as the enforcement source and list the four required
contexts. Replace the development-flow instruction to run old CI with the
native gate commands and state that legacy workflows are manual diagnostics.

- [x] **Step 2: Mirror the policy in Chinese**

Add the equivalent rules:

```markdown
`Native CI` 是 `2.x` push 与以 `2.x` 为目标的 pull request 唯一自动运行的
workflow。必需检查为 `Native 2.x change boundary`，以及 Linux、macOS、
Windows 三个平台的 `Rust native foundation`。同一 commit 必须依次通过
format、check、Clippy 和 workspace tests；该 workflow 不启动 Python 或
Node。

`Legacy Compatibility CI` 与 `Legacy Checkout Install Check` 只对 `main`、
`master`、`dev` 自动运行。需要核查某个明确的兼容性问题时，维护者仍可对
指定的 `2.x` ref 手动触发它们；其结果是诊断证据，不是 2.x 原生开发的
required checks。
```

- [x] **Step 3: Mark R0 repository work implemented but server verification pending**

In the accelerated roadmap, change the R0 status to state that workflow routing,
the boundary guard, and focused tests are implemented. Keep the R0 exit gate
open until the pushed exact head produces all four required contexts and
ruleset `18800504` is verified with only those contexts.

- [x] **Step 4: Run documentation and patch checks**

Run:

```bash
git diff --check
python3 -m unittest tests.test_branch_policy -v
```

Expected: no whitespace errors and the branch policy suite passes.

- [x] **Step 5: Commit the governance checkpoint**

```bash
git add docs/maintainer/release-branch-policy.md \
  docs/zh/maintainer/release-branch-policy.md \
  docs/superpowers/roadmaps/2026-07-13-qiongli-2-accelerated-rust-migration-roadmap.md
git commit -m "docs(native): record the R0 CI control plane"
```

### Task 5: Run The Local Native Gate

**Files:**
- Verify only; no expected source edits

- [x] **Step 1: Check formatting**

```bash
cargo fmt --manifest-path packages/qiongli-native/Cargo.toml --all -- --check
```

Expected: exit 0.

- [x] **Step 2: Check all workspace targets**

```bash
cargo check --manifest-path packages/qiongli-native/Cargo.toml \
  --workspace --all-targets --all-features --locked
```

Expected: exit 0.

- [x] **Step 3: Run Clippy**

```bash
cargo clippy --manifest-path packages/qiongli-native/Cargo.toml \
  --workspace --all-targets --all-features --locked -- -D warnings
```

Expected: exit 0 with no warnings.

- [x] **Step 4: Run all native workspace tests**

```bash
cargo test --manifest-path packages/qiongli-native/Cargo.toml \
  --workspace --all-targets --all-features --locked
```

Expected: exit 0 with all tests passing.

### Task 6: Publish R0 And Narrow The Server Ruleset

**Files:**
- External update: GitHub ruleset `18800504`
- External update: Draft PR #63 body

- [x] **Step 1: Push the same rolling branch**

```bash
git push origin feat/2x-native-alpha1
```

- [x] **Step 2: Verify workflow routing on the exact head**

Run:

```bash
gh pr checks 63
gh run list --branch feat/2x-native-alpha1 --limit 10
```

Expected: `Native CI` produces `Native 2.x change boundary` and three
`Rust native foundation` checks. `Legacy Compatibility CI` and `Legacy Checkout
Install Check` do not start automatically for the PR head.

- [x] **Step 3: Update ruleset 18800504**

Submit a full ruleset update preserving pull-request, deletion, and
non-fast-forward protection while replacing the old eleven required contexts
with:

```json
[
  {"context": "Native 2.x change boundary", "integration_id": 15368},
  {"context": "Rust native foundation (Linux)", "integration_id": 15368},
  {"context": "Rust native foundation (macOS)", "integration_id": 15368},
  {"context": "Rust native foundation (Windows)", "integration_id": 15368}
]
```

Keep `strict_required_status_checks_policy=true`, no bypass actors, zero
required approving reviews, stale-review dismissal, review-thread resolution,
and merge/squash/rebase methods exactly as currently configured.

- [x] **Step 4: Verify the live ruleset and exact-head checks**

```bash
gh api repos/jxpeng98/qiongli/rulesets/18800504
gh pr checks 63 --required
```

Expected: the live ruleset contains exactly four required contexts and all four
are successful on the current PR head.

- [x] **Step 5: Update the Draft PR ledger without overstating R1**

Record:

- current capability: R0 native CI control plane;
- checkpoint commits and exact-head evidence;
- next batch: FND-202E atomic materialization, then FND-202F embedding/drift;
- nonclaims: no materializer, config service, Lite MCP integration, UI,
  installer, packaging, signing, or alpha clean-machine acceptance yet.

## Completion Gate

R0 is complete only when all of the following are true on the same pushed head:

- `Native CI` is the only automatic workflow for the 2.x PR;
- the native boundary guard and Linux/macOS/Windows Rust checks pass;
- no Python or Node setup appears in the required workflow;
- legacy compatibility and checkout workflows remain manually dispatchable;
- ruleset `18800504` requires exactly the four native contexts;
- the bilingual policy and rolling PR ledger report the current state without
  claiming R1-R3 functionality.
