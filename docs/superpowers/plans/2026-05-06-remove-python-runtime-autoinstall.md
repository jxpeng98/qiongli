# Remove Python Runtime Autoinstall Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the full research-skills install experience require an existing Python 3.12+ installation, while removing all automatic mise/Python installation behavior from bootstrap and installer flows.

**Architecture:** Installation profiles remain `partial` and `full`, but `full` becomes a readiness gate instead of a runtime provisioner. Bash, PowerShell, and Python installer surfaces should share the same user-facing rule: partial does not require Python; full requires Python 3.12+ and prints install options when missing or too old. Documentation and tests must stop promising automatic mise installation.

**Tech Stack:** Bash, PowerShell, Python stdlib `unittest`, Markdown documentation.

---

## File Structure

- `scripts/bootstrap_research_skill.sh`: remove `mise` bootstrap and Python installation; keep a focused `ensure_python_runtime` check for full profile.
- `scripts/bootstrap_research_skill.ps1`: remove `Find-Mise`, `Ensure-Mise`, `Ensure-NativePython312`, and Python winget installation; keep `Find-UsablePython` and make `Ensure-PythonRuntime` fail with install guidance.
- `research_skills/universal_installer.py`: update full-profile readiness hint text so it does not recommend mise as the preferred built-in route.
- `tests/test_bootstrap_research_skill.py`: add shell and PowerShell regression assertions that no automatic runtime install remains, and adjust existing PowerShell expectations.
- `tests/test_install_research_skill.py`: add installer hint assertions for full profile Python readiness text if needed.
- `README.md`, `docs/guide/install.md`, `docs/zh/guide/install.md`: replace automatic mise/Python install language with explicit prerequisites and install suggestions.

---

### Task 1: Lock Shell Bootstrap Behavior With Tests

**Files:**
- Modify: `tests/test_bootstrap_research_skill.py`
- Test: `tests/test_bootstrap_research_skill.py`

- [ ] **Step 1: Add failing assertions that shell bootstrap no longer contains runtime installers**

Add this test near the existing shell bootstrap tests:

```python
    def test_shell_bootstrap_does_not_install_python_runtime(self) -> None:
        content = BOOTSTRAP_SCRIPT.read_text(encoding="utf-8")

        self.assertNotIn("install_mise()", content)
        self.assertNotIn("mise install python@3.12", content)
        self.assertNotIn("mise use -g python@3.12", content)
        self.assertNotIn("curl https://mise.run", content)
        self.assertIn("Python 3.12+ is required for full profile", content)
        self.assertIn("python.org/downloads", content)
        self.assertIn("brew install python", content)
        self.assertIn("winget install -e --id Python.Python.3.12", content)
```

- [ ] **Step 2: Run the test and verify it fails**

Run:

```bash
python3 -m unittest tests.test_bootstrap_research_skill.BootstrapResearchSkillTests.test_shell_bootstrap_does_not_install_python_runtime -v
```

Expected: FAIL because `install_mise()` and `mise install python@3.12` still exist in `scripts/bootstrap_research_skill.sh`.

- [ ] **Step 3: Update existing full dry-run test expectations**

In `test_full_profile_dry_run_enables_cli_and_doctor`, add:

```python
        self.assertNotIn("mise", result.stdout.lower())
        self.assertNotIn("install via mise", result.stdout)
```

This keeps dry-run from presenting runtime installation as part of full mode.

- [ ] **Step 4: Run the bootstrap test file and keep the expected failures visible**

Run:

```bash
python3 -m unittest tests.test_bootstrap_research_skill -v
```

Expected before implementation: the new no-runtime-install test fails; unrelated tests should continue to pass.

---

### Task 2: Remove Bash Runtime Provisioning

**Files:**
- Modify: `scripts/bootstrap_research_skill.sh`
- Test: `tests/test_bootstrap_research_skill.py`

- [ ] **Step 1: Delete mise-specific state and helper functions**

Remove these definitions from `scripts/bootstrap_research_skill.sh`:

```bash
MISE_BIN="${HOME}/.local/bin/mise"
PYTHON_RUNTIME_MODE=""
```

Delete the functions:

```bash
resolve_mise_bin() { ... }
install_mise() { ... }
```

- [ ] **Step 2: Replace `ensure_python_runtime` with a check-only implementation**

Use this implementation:

```bash
print_python_install_hints() {
  cat >&2 <<'EOF'
Python 3.12+ is required for full profile.
Install Python with any method you prefer, then rerun this command.

Common options:
  macOS:
    - python.org/downloads
    - brew install python
    - pyenv install 3.12 && pyenv global 3.12
    - mise install python@3.12 && mise use -g python@3.12
  Windows:
    - python.org/downloads/windows
    - winget install -e --id Python.Python.3.12 --source winget
    - Microsoft Store Python 3.12
    - pyenv-win
  Linux:
    - distro package manager, for example apt/dnf/pacman
    - pyenv install 3.12 && pyenv global 3.12
    - mise install python@3.12 && mise use -g python@3.12
EOF
}

ensure_python_runtime() {
  local current_python
  local current_version

  if current_python="$(command -v python3 2>/dev/null)"; then
    current_version="$("$current_python" -c 'import sys; print(f"{sys.version_info.major}.{sys.version_info.minor}")' 2>/dev/null || true)"
    if [[ "$current_version" =~ ^([0-9]+)\.([0-9]+)$ ]]; then
      if (( BASH_REMATCH[1] > 3 || (BASH_REMATCH[1] == 3 && BASH_REMATCH[2] >= 12) )); then
        info "python:  $current_python ($current_version)"
        return 0
      fi
    fi
    err "python3 found at $current_python but version is below 3.12."
  else
    err "python3 not found."
  fi

  print_python_install_hints
  exit 1
}
```

- [ ] **Step 3: Remove mise execution wrapper**

Replace:

```bash
if [[ "$PROFILE" == "full" && "$DRY_RUN" -ne 1 && "$PYTHON_RUNTIME_MODE" == "mise" ]]; then
  "$MISE_BIN" exec python@3.12 -- "${cmd[@]}"
else
  "${cmd[@]}"
fi
```

with:

```bash
"${cmd[@]}"
```

- [ ] **Step 4: Update shell profile description text**

Replace the full profile bullet:

```bash
- Ensures Python 3.12 is available via mise if missing
```

with:

```bash
- Requires an existing Python 3.12+ installation
```

Update help notes so `--doctor` says full profile requires Python 3.12+, not that bootstrap installs it.

- [ ] **Step 5: Run shell tests**

Run:

```bash
bash -n scripts/bootstrap_research_skill.sh
python3 -m unittest tests.test_bootstrap_research_skill -v
```

Expected: all bootstrap shell tests pass.

- [ ] **Step 6: Commit Task 2**

```bash
git add scripts/bootstrap_research_skill.sh tests/test_bootstrap_research_skill.py
git commit -m "fix: require existing python for shell full profile"
```

---

### Task 3: Remove PowerShell Runtime Provisioning

**Files:**
- Modify: `scripts/bootstrap_research_skill.ps1`
- Modify: `tests/test_bootstrap_research_skill.py`
- Test: `tests/test_bootstrap_research_skill.py`

- [ ] **Step 1: Replace old PowerShell assertions with no-autoinstall assertions**

In `test_powershell_bootstrap_is_manifest_driven`, remove assertions for:

```python
        self.assertIn("Microsoft\\WinGet\\Links\\mise.exe", content)
        self.assertIn("Python.Python.3.12", content)
        self.assertIn("Ensure-NativePython312", content)
```

Add:

```python
        self.assertNotIn("Ensure-Mise", content)
        self.assertNotIn("Find-Mise", content)
        self.assertNotIn("Ensure-NativePython312", content)
        self.assertNotIn("winget install jdx.mise", content)
        self.assertNotIn("mise install python@3.12", content)
        self.assertIn("Python 3.12+ is required for full profile", content)
        self.assertIn("python.org/downloads/windows", content)
        self.assertIn("winget install -e --id Python.Python.3.12", content)
```

- [ ] **Step 2: Run the PowerShell text test and verify it fails**

Run:

```bash
python3 -m unittest tests.test_bootstrap_research_skill.BootstrapResearchSkillTests.test_powershell_bootstrap_is_manifest_driven -v
```

Expected: FAIL because PowerShell bootstrap still includes mise and Python installation helpers.

- [ ] **Step 3: Delete PowerShell mise and Python installers**

Remove these functions from `scripts/bootstrap_research_skill.ps1`:

```powershell
function Find-Mise { ... }
function Ensure-Mise { ... }
function Ensure-NativePython312 { ... }
```

- [ ] **Step 4: Replace `Ensure-PythonRuntime` with check-only behavior**

Use this implementation:

```powershell
function Write-PythonInstallHints {
    Write-Error "Python 3.12+ is required for full profile."
    Write-Host "Install Python with any method you prefer, then rerun this command."
    Write-Host ""
    Write-Host "Common options:"
    Write-Host "  Windows:"
    Write-Host "    - python.org/downloads/windows"
    Write-Host "    - winget install -e --id Python.Python.3.12 --source winget"
    Write-Host "    - Microsoft Store Python 3.12"
    Write-Host "    - pyenv-win"
    Write-Host "  macOS:"
    Write-Host "    - python.org/downloads"
    Write-Host "    - brew install python"
    Write-Host "    - pyenv install 3.12 && pyenv global 3.12"
    Write-Host "    - mise install python@3.12 && mise use -g python@3.12"
    Write-Host "  Linux:"
    Write-Host "    - distro package manager, for example apt/dnf/pacman"
    Write-Host "    - pyenv install 3.12 && pyenv global 3.12"
    Write-Host "    - mise install python@3.12 && mise use -g python@3.12"
}

function Ensure-PythonRuntime {
    $python = Find-UsablePython
    if ($python -and ($python.Major -gt 3 -or ($python.Major -eq 3 -and $python.Minor -ge 12))) {
        Write-Info "python:  $($python.Path) ($($python.Version))"
        return @{
            Mode = "direct"
            Python = $python.Path
            Mise = $null
        }
    }

    if ($python) {
        Write-Error "python exists but is below 3.12: $($python.Path) ($($python.Version))"
    }
    else {
        Write-Error "python not found."
    }
    Write-PythonInstallHints
    throw "Install Python 3.12+ before using --profile full."
}
```

- [ ] **Step 5: Remove any PowerShell mise execution branch**

Search:

```bash
rg -n "Mise|mise|PythonRuntime.Mode" scripts/bootstrap_research_skill.ps1
```

Expected after edit: no `mise` runtime installation branch remains. If `$PythonRuntime.Mode -eq "mise"` is still present near command execution, replace that branch with direct invocation of the installer command.

- [ ] **Step 6: Run tests**

Run:

```bash
python3 -m unittest tests.test_bootstrap_research_skill -v
```

Expected: all bootstrap tests pass.

- [ ] **Step 7: Commit Task 3**

```bash
git add scripts/bootstrap_research_skill.ps1 tests/test_bootstrap_research_skill.py
git commit -m "fix: require existing python for powershell full profile"
```

---

### Task 4: Align Python Installer Readiness Text

**Files:**
- Modify: `research_skills/universal_installer.py`
- Modify: `tests/test_install_research_skill.py`
- Test: `tests/test_install_research_skill.py`

- [ ] **Step 1: Add a text-level regression test**

Add this test to `InstallResearchSkillTests`:

```python
    def test_full_profile_python_hint_does_not_prefer_mise(self) -> None:
        content = (REPO_ROOT / "research_skills" / "universal_installer.py").read_text(encoding="utf-8")

        self.assertIn("install Python >= 3.12", content)
        self.assertIn("python.org/downloads", content)
        self.assertIn("winget install -e --id Python.Python.3.12", content)
        self.assertNotIn("preferably with mise", content)
```

- [ ] **Step 2: Run the test and verify it fails**

Run:

```bash
python3 -m unittest tests.test_install_research_skill.InstallResearchSkillTests.test_full_profile_python_hint_does_not_prefer_mise -v
```

Expected: FAIL because `_print_full_readiness` currently prints `preferably with mise`.

- [ ] **Step 3: Update `_print_full_readiness` hint**

Replace:

```python
print("          Hint: install Python >= 3.12, preferably with mise")
```

with:

```python
print("          Hint: install Python >= 3.12 using python.org/downloads, your OS package manager, pyenv, mise, winget, or another method you prefer")
```

- [ ] **Step 4: Run tests**

Run:

```bash
python3 -m unittest tests.test_install_research_skill -v
```

Expected: all installer tests pass.

- [ ] **Step 5: Commit Task 4**

```bash
git add research_skills/universal_installer.py tests/test_install_research_skill.py
git commit -m "docs: make python readiness hints installer-neutral"
```

---

### Task 5: Rewrite Install Documentation

**Files:**
- Modify: `README.md`
- Modify: `docs/guide/install.md`
- Modify: `docs/zh/guide/install.md`
- Test: `tests/test_bootstrap_research_skill.py`

- [ ] **Step 1: Update profile table language**

In each documentation file, replace the full profile description:

```markdown
`partial` + shell CLI + Python 3.12 when needed + `doctor`
```

or Chinese equivalent with:

```markdown
`partial` + shell CLI + requires existing Python 3.12+ + `doctor`
```

Chinese wording:

```markdown
`partial` + shell CLI + 要求本机已有 Python 3.12+ + `doctor`
```

- [ ] **Step 2: Remove automatic mise statements**

Delete or rewrite lines that say:

```markdown
If Python is missing or too old, bootstrap installs `mise`, then installs `python@3.12`.
If `full` mode installs `mise` automatically...
```

Chinese equivalents should also be removed:

```markdown
如果 Python 缺失或版本过低，bootstrap 会先安装 `mise`，再安装 `python@3.12`。
`full` 模式如果自动安装 `mise`...
```

- [ ] **Step 3: Add neutral Python installation options**

Add an English section:

```markdown
### Python prerequisite for `full`

`full` mode requires Python 3.12+ to already be available on PATH. The installer does not install Python or `mise` for you. Install Python using any method you prefer:

- macOS: python.org installer, `brew install python`, `pyenv`, or `mise`
- Windows: python.org installer, `winget install -e --id Python.Python.3.12 --source winget`, Microsoft Store, or pyenv-win
- Linux: distro package manager, `pyenv`, or `mise`

Verify before running `full`:

```bash
python3 --version
```
```

Add a Chinese section:

```markdown
### `full` 模式的 Python 前提

`full` 模式要求机器上已经有 Python 3.12+，并且能在 PATH 中找到。安装器不会再自动安装 Python 或 `mise`。你可以用任何方式安装 Python：

- macOS：python.org 安装包、`brew install python`、`pyenv` 或 `mise`
- Windows：python.org 安装包、`winget install -e --id Python.Python.3.12 --source winget`、Microsoft Store 或 pyenv-win
- Linux：系统包管理器、`pyenv` 或 `mise`

运行 `full` 前先确认：

```bash
python3 --version
```
```

- [ ] **Step 4: Update docs tests if text assertions break**

Run:

```bash
python3 -m unittest tests.test_bootstrap_research_skill -v
```

If `test_shell_bootstrap_documents_beta_channel` remains green, no doc test update is needed. If a README/install-doc-specific assertion exists elsewhere, update it to assert that automatic mise installation is not documented.

- [ ] **Step 5: Commit Task 5**

```bash
git add README.md docs/guide/install.md docs/zh/guide/install.md tests/test_bootstrap_research_skill.py
git commit -m "docs: document python prerequisite for full install"
```

---

### Task 6: Final Release-Flow Verification

**Files:**
- Verify: all changed files
- Optional generated evidence: `release/v0.6.0-beta.2.md`

- [ ] **Step 1: Search for stale automatic install claims**

Run:

```bash
rg -n "installs `mise`|install via mise|Installing mise|Ensure-Mise|Find-Mise|Ensure-NativePython312|mise install python@3.12|mise use -g python@3.12|缺失.*mise|自动安装 `mise`" README.md docs scripts tests research_skills
```

Expected: no stale claims or runtime installer functions. Mentions of `mise` are allowed only as one neutral user-chosen Python installation option or unrelated CLI shim detection.

- [ ] **Step 2: Run targeted tests**

Run:

```bash
python3 -m unittest tests.test_bootstrap_research_skill tests.test_install_research_skill -v
```

Expected: all tests pass.

- [ ] **Step 3: Run release unit test command**

Run:

```bash
python3 -m unittest discover -s tests -v
```

Expected: all tests pass.

- [ ] **Step 4: Run shell syntax checks**

Run:

```bash
bash -n scripts/bootstrap_research_skill.sh
bash -n scripts/install_research_skill.sh
bash -n scripts/release_preflight.sh
bash -n scripts/release_ready.sh
```

Expected: no syntax errors.

- [ ] **Step 5: Run release-ready dry run after committing code changes**

Run after all implementation commits are made and only release-note evidence may be dirty:

```bash
./scripts/release_ready.sh --version 0.6.0b2 --skip-bump
```

Expected: validator, unit tests, smoke, package build, `twine check`, and install smoke pass. If the script updates `release/v0.6.0-beta.2.md`, keep that evidence for the publish commit.

- [ ] **Step 6: Commit final evidence if appropriate**

If `release/v0.6.0-beta.2.md` was updated by `release_ready`, include it in the release-prep commit created by publish mode. Do not manually tag here.

---

## Self-Review

- Spec coverage: The plan removes automatic mise/Python provisioning from Bash and PowerShell, updates Python readiness messaging in the Python installer, and rewrites English/Chinese docs.
- Placeholder scan: No `TBD`, `TODO`, or unspecified test commands remain.
- Type and command consistency: Commands use existing `unittest`, `bash -n`, and `release_ready` patterns from this repository.
- Scope check: This is one coherent installer behavior change; no unrelated refactor is included.
