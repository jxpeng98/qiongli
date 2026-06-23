# B Literature Skill Precision Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Convert Stage B literature skills into precise, auditable execution contracts backed by a Stage B semantic audit.

**Architecture:** Add a focused audit script under `tooling/scripts/` with a top-level wrapper under `scripts/`, then rewrite canonical B skill markdown to satisfy the semantic checks. Keep the source of truth in `content/`; update `content/skills-core.md` and `content/skills/registry.yaml` only for B-stage summaries and triggers.

**Tech Stack:** Python standard library, `unittest`, repository wrapper script pattern, Markdown skill cards, YAML registry.

---

## Baseline

Run validation with the project environment, not system Python:

```bash
uv run python -m unittest tests.test_audit_skill_sections -v
uv run python -m unittest tests.test_literature_search_quality_audit -v
uv run python -m unittest tests.test_skill_set_scorecard -v
```

Expected baseline: all pass. System `python3` is not sufficient on this machine because it lacks `PyYAML`.

## File Structure

- Create `tooling/scripts/audit_b_literature_skill_precision.py`: implements Stage B semantic checks and CLI reporting.
- Create `scripts/audit_b_literature_skill_precision.py`: wrapper matching existing `scripts/audit_*.py` forwarding style.
- Create `tests/test_b_literature_skill_precision.py`: unit tests for fixture failures and canonical repository checks.
- Modify `content/skills/B_literature/academic-searcher.md`: provider-owned search and diagnostics contract.
- Modify `content/skills/B_literature/paper-screener.md`: diagnostics-aware screening contract.
- Modify `content/skills/B_literature/fulltext-fetcher.md`: retrieval manifest and resolver boundary.
- Modify `content/skills/B_literature/reference-manager-bridge.md`: local Zotero companion and import fallback contract.
- Modify `content/skills/B_literature/concept-extractor.md`: B1_5 concept bucket and seed recall contract.
- Modify `content/skills/B_literature/citation-snowballer.md`: citation-graph, seed rationale, saturation, and append contract.
- Modify `content/skills/B_literature/paper-extractor.md`: source anchor and evidence limit contract.
- Modify `content/skills/B_literature/literature-mapper.md`: cluster evidence and non-chronological taxonomy contract.
- Modify `content/skills/B_literature/citation-formatter.md`: metadata integrity and citekey conflict contract.
- Modify `content/skills-core.md`: concise Stage B summaries without direct API fallback defaults.
- Modify `content/skills/registry.yaml`: B-stage trigger metadata only.

## Task 1: Stage B Semantic Audit

**Files:**
- Create: `tooling/scripts/audit_b_literature_skill_precision.py`
- Create: `scripts/audit_b_literature_skill_precision.py`
- Create: `tests/test_b_literature_skill_precision.py`

- [ ] **Step 1: Write failing tests**

Add tests with these concrete expectations:

```python
def test_fixture_flags_direct_api_defaults_and_missing_evidence_limits(self) -> None:
    result = audit_b_literature_skill_precision(root)
    self.assertTrue(result.has_gaps)
    self.assertIn("provider ownership", report)
    self.assertIn("evidence limits", report)

def test_canonical_repository_initially_has_precision_gaps(self) -> None:
    result = audit_b_literature_skill_precision(REPO_ROOT)
    paths = {item.path.as_posix() for item in result.skill_results if item.issue_count}
    self.assertIn("content/skills/B_literature/academic-searcher.md", paths)
    self.assertIn("content/skills/B_literature/reference-manager-bridge.md", paths)
```

- [ ] **Step 2: Run tests to verify RED**

Run:

```bash
uv run python -m unittest tests.test_b_literature_skill_precision -v
```

Expected: fail because `tests.test_b_literature_skill_precision` or `audit_b_literature_skill_precision` does not exist.

- [ ] **Step 3: Implement minimal audit script**

Create dataclasses similar to `tooling/scripts/audit_skill_sections.py`:

```python
@dataclass
class SkillPrecisionResult:
    path: Path
    checks: dict[str, bool]

    @property
    def missing_checks(self) -> list[str]:
        return [name for name, passed in self.checks.items() if not passed]

    @property
    def issue_count(self) -> int:
        return len(self.missing_checks)
```

Implement checks for provider ownership, artifact paths, review-grade blockers, evidence limits, full-text statuses, Zotero write safety, compact provider references, and `skills-core.md` direct API defaults.

- [ ] **Step 4: Run tests to verify audit sees current gaps**

Run:

```bash
uv run python -m unittest tests.test_b_literature_skill_precision -v
```

Expected: pass, with fixture and canonical current-gap tests confirming existing Stage B gaps.

- [ ] **Step 5: Commit**

```bash
git add tests/test_b_literature_skill_precision.py tooling/scripts/audit_b_literature_skill_precision.py scripts/audit_b_literature_skill_precision.py
git commit -m "test(skills): add B literature precision audit"
```

## Task 2: Live MCP And Zotero Skill Rewrites

**Files:**
- Modify: `content/skills/B_literature/academic-searcher.md`
- Modify: `content/skills/B_literature/paper-screener.md`
- Modify: `content/skills/B_literature/fulltext-fetcher.md`
- Modify: `content/skills/B_literature/reference-manager-bridge.md`

- [ ] **Step 1: Rewrite live-path skill contracts**

For each file, keep required frontmatter and sections, then replace broad API-manual prose with precise contracts:

```markdown
## Blocking Conditions

- Review-grade work blocks when `search_diagnostics.md` is missing.
- Review-grade work blocks when fewer than two productive providers are recorded.
- Triage may continue only when the limitation is copied into the downstream artifact.
```

Use the exact contract details from `docs/superpowers/specs/2026-06-23-b-literature-skill-precision-design.md`.

- [ ] **Step 2: Run semantic audit**

Run:

```bash
uv run python scripts/audit_b_literature_skill_precision.py
```

Expected: remaining gaps only in the five non-live B skills and core/registry checks.

- [ ] **Step 3: Run structural audit**

Run:

```bash
uv run python scripts/audit_skill_sections.py --strict
```

Expected: pass.

- [ ] **Step 4: Commit**

```bash
git add content/skills/B_literature/academic-searcher.md content/skills/B_literature/paper-screener.md content/skills/B_literature/fulltext-fetcher.md content/skills/B_literature/reference-manager-bridge.md
git commit -m "docs(skills): sharpen B literature MCP contracts"
```

## Task 3: Remaining B Skill Rewrites

**Files:**
- Modify: `content/skills/B_literature/concept-extractor.md`
- Modify: `content/skills/B_literature/citation-snowballer.md`
- Modify: `content/skills/B_literature/paper-extractor.md`
- Modify: `content/skills/B_literature/literature-mapper.md`
- Modify: `content/skills/B_literature/citation-formatter.md`

- [ ] **Step 1: Rewrite remaining skill contracts**

Apply these file-specific requirements:

```text
concept-extractor: concept buckets, excluded ambiguous terms, seed recall test
citation-snowballer: seed rationale, citation-graph owner, saturation status
paper-extractor: source_anchor, evidence_limit, unsupported_gap
literature-mapper: clustering basis, representative papers, evidence limits
citation-formatter: canonical bibliography.bib, DOI normalization, citekey conflicts
```

- [ ] **Step 2: Run semantic audit**

Run:

```bash
uv run python scripts/audit_b_literature_skill_precision.py
```

Expected: no B skill file gaps; possible remaining `skills-core.md` or registry gaps.

- [ ] **Step 3: Run structural audit**

Run:

```bash
uv run python scripts/audit_skill_sections.py --strict
```

Expected: pass.

- [ ] **Step 4: Commit**

```bash
git add content/skills/B_literature/concept-extractor.md content/skills/B_literature/citation-snowballer.md content/skills/B_literature/paper-extractor.md content/skills/B_literature/literature-mapper.md content/skills/B_literature/citation-formatter.md
git commit -m "docs(skills): sharpen remaining B literature contracts"
```

## Task 4: Core Summary And Registry Sync

**Files:**
- Modify: `content/skills-core.md`
- Modify: `content/skills/registry.yaml`
- Modify: `tests/test_b_literature_skill_precision.py` if canonical expectations need to switch from current-gap mode to strict-pass mode.

- [ ] **Step 1: Update concise core summaries**

Replace direct API fallback defaults with provider ownership. The `academic-searcher` core entry must mention `scholarly-search`, `search_diagnostics.md`, and review-grade blockers. The `reference-manager-bridge` core entry must mention Zotero dry-run and import fallback.

- [ ] **Step 2: Update B registry triggers**

Change only B-stage `summary` and `when_to_use` strings. Keep YAML structure and version values unchanged.

- [ ] **Step 3: Flip canonical audit test to strict pass**

Change the canonical repository test to:

```python
def test_canonical_repository_passes_stage_b_precision_audit(self) -> None:
    result = audit_b_literature_skill_precision(REPO_ROOT)
    self.assertFalse(result.has_gaps, render_markdown_report(result))
```

- [ ] **Step 4: Run B semantic tests and CLI**

Run:

```bash
uv run python -m unittest tests.test_b_literature_skill_precision -v
uv run python scripts/audit_b_literature_skill_precision.py --strict
```

Expected: pass.

- [ ] **Step 5: Commit**

```bash
git add content/skills-core.md content/skills/registry.yaml tests/test_b_literature_skill_precision.py
git commit -m "docs(skills): sync B literature triggers"
```

## Task 5: Final Validation And Boundary Review

**Files:**
- Modify only if a validation failure reveals a necessary fix.

- [ ] **Step 1: Run full targeted validation**

Run:

```bash
uv run python -m unittest tests.test_audit_skill_sections -v
uv run python -m unittest tests.test_literature_search_quality_audit -v
uv run python -m unittest tests.test_skill_set_scorecard -v
uv run python -m unittest tests.test_b_literature_skill_precision -v
uv run python scripts/audit_skill_sections.py --strict
uv run python scripts/audit_b_literature_skill_precision.py --strict
```

Expected: all pass.

- [ ] **Step 2: Review repository boundary**

Run:

```bash
git status --short
git diff --stat HEAD
git diff --name-only HEAD
```

Expected: no generated package mirrors, release ZIPs, secrets, local config, or unrelated files.

- [ ] **Step 3: Commit validation fixes only if needed**

If Step 1 or Step 2 required edits, inspect the exact file list first:

```bash
git diff --name-only HEAD
git add tests/test_b_literature_skill_precision.py tooling/scripts/audit_b_literature_skill_precision.py scripts/audit_b_literature_skill_precision.py content/skills/B_literature content/skills-core.md content/skills/registry.yaml
git commit -m "fix(skills): satisfy B literature precision validation"
```

Before running that `git add`, remove any path from the command that does not
appear in `git diff --name-only HEAD`. If no edits are needed, do not create an
empty validation commit.
