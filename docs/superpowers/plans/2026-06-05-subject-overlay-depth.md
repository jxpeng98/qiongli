# Subject Overlay Depth Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Deepen Qiongli subject overlays so every active subject gives discipline-specific execution instructions rather than short descriptive notes.

**Architecture:** Keep the existing subject materializer unchanged. Add an audit-level depth rule for subject overlays, then expand the existing Markdown overlays under `content/subjects/*/overlays/skills/` so materialized subject packages receive richer instructions through the current append/replace overlay mechanism.

**Tech Stack:** Python `unittest`, existing `tooling/scripts/audit_subject_specialization.py`, Markdown subject overlays, existing subject materializer.

---

## File Structure

- Modify: `tooling/scripts/audit_subject_specialization.py`
  - Add overlay-depth checks for append and replace-section overlays.
  - Reuse existing `SubjectSpecializationFinding` output style.
- Modify: `tests/test_subject_specialization_audit.py`
  - Add tests proving a short overlay reports `thin-overlay-instructions`.
  - Keep current audit pass test as the full integration acceptance target.
- Modify: `content/subjects/economics/overlays/skills/*.md`
  - Expand append overlays and keep stats replacement headings.
- Modify: `content/subjects/accounting/overlays/skills/*.md`
  - Expand append overlays and keep stats replacement headings.
- Modify: `content/subjects/business/overlays/skills/*.md`
  - Expand append overlays and keep stats replacement headings.
- Modify: `content/subjects/finance/overlays/skills/*.md`
  - Expand append overlays and keep stats replacement headings.
- Modify: `content/subjects/political-economy/overlays/skills/*.md`
  - Expand all append overlays, including `stats-engine`.
- Modify: `content/subjects/geoeconomics/overlays/skills/*.md`
  - Expand all append overlays.
- Modify: `content/subjects/economics-accounting/overlays/skills/*.md`
  - Expand composite append overlay and keep stats replacement headings.

## Task 1: Add Overlay Depth Audit Test

**Files:**
- Modify: `tests/test_subject_specialization_audit.py`

- [ ] **Step 1: Write the failing test**

Add this test after `test_missing_overlay_term_is_reported`:

```python
    def test_short_append_overlay_is_reported(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            temp_root = Path(tmp_dir)
            self._copy_minimal_repo(temp_root)
            overlay = temp_root / "subjects" / "economics" / "overlays" / "skills" / "manuscript-architect.md"
            overlay.write_text(
                "## Economics Overlay\n\n- Mention identification and robustness.\n",
                encoding="utf-8",
            )

            findings = audit_subject_specialization(temp_root, subjects=["economics"])

        self.assertTrue(
            any(finding.code == "thin-overlay-instructions" for finding in findings),
            [f"{finding.subject}: {finding.code}: {finding.message}" for finding in findings],
        )
```

- [ ] **Step 2: Run the test to verify it fails**

Run:

```bash
python3 -m unittest tests.test_subject_specialization_audit.SubjectSpecializationAuditTests.test_short_append_overlay_is_reported -v
```

Expected: FAIL because `thin-overlay-instructions` is not implemented yet.

## Task 2: Implement Overlay Depth Audit

**Files:**
- Modify: `tooling/scripts/audit_subject_specialization.py`
- Test: `tests/test_subject_specialization_audit.py`

- [ ] **Step 1: Add constants near `SUBJECT_TERMS`**

```python
APPEND_OVERLAY_REQUIRED_SECTIONS = {
    "activation",
    "required context",
    "subject-specific procedure",
    "reviewer-risk checks",
    "output requirements",
    "blocked conditions",
}
MIN_APPEND_OVERLAY_SECTIONS = 4
REPLACE_OVERLAY_REQUIRED_SECTIONS = {"quality bar", "common pitfalls"}
MIN_REPLACE_OVERLAY_ITEMS = 8
```

- [ ] **Step 2: Wire depth audit into `_audit_materialized_outputs`**

Change the return block to:

```python
        findings = _audit_focused_domain_profiles(focused, subject)
        findings.extend(_audit_overlay_subject_terms(root, subject))
        findings.extend(_audit_overlay_instruction_depth(root, subject))
        findings.extend(_audit_materialized_subject_terms(complete, subject))
        return findings
```

- [ ] **Step 3: Add helper functions before `_subject_layer_skill_paths`**

```python
def _audit_overlay_instruction_depth(root: Path, subject: SubjectDefinition) -> list[SubjectSpecializationFinding]:
    findings: list[SubjectSpecializationFinding] = []
    overlay_root = RepoLayout(root).subjects / subject.id
    for override in subject.skill_overrides:
        overlay_rel = override.get("overlay")
        if not isinstance(overlay_rel, str) or not overlay_rel.strip():
            continue
        overlay_path = overlay_root / overlay_rel
        if not overlay_path.is_file():
            continue
        text = overlay_path.read_text(encoding="utf-8")
        mode = str(override.get("mode") or "append")
        if mode == "append":
            present = _markdown_subsections(text) & APPEND_OVERLAY_REQUIRED_SECTIONS
            if len(present) < MIN_APPEND_OVERLAY_SECTIONS:
                rel_path = overlay_path.relative_to(root)
                findings.append(
                    SubjectSpecializationFinding(
                        subject=subject.id,
                        code="thin-overlay-instructions",
                        message=(
                            f"{rel_path} has {len(present)} overlay instruction sections; "
                            f"expected at least {MIN_APPEND_OVERLAY_SECTIONS}"
                        ),
                    )
                )
        elif mode == "replace_sections":
            present = _markdown_sections(text)
            missing = sorted(REPLACE_OVERLAY_REQUIRED_SECTIONS - present)
            item_count = _instruction_item_count(text)
            if missing or item_count < MIN_REPLACE_OVERLAY_ITEMS:
                rel_path = overlay_path.relative_to(root)
                detail = (
                    f"missing sections: {', '.join(missing)}"
                    if missing
                    else f"has {item_count} checklist/table items; expected at least {MIN_REPLACE_OVERLAY_ITEMS}"
                )
                findings.append(
                    SubjectSpecializationFinding(
                        subject=subject.id,
                        code="thin-overlay-instructions",
                        message=f"{rel_path} replacement overlay is too thin: {detail}",
                    )
                )
    return findings


def _markdown_sections(text: str) -> set[str]:
    return {
        _normalize_heading(match.group("title"))
        for match in re.finditer(r"(?m)^##\s+(?P<title>.+?)\s*$", text)
    }


def _markdown_subsections(text: str) -> set[str]:
    return {
        _normalize_heading(match.group("title"))
        for match in re.finditer(r"(?m)^###\s+(?P<title>.+?)\s*$", text)
    }


def _instruction_item_count(text: str) -> int:
    count = 0
    for line in text.splitlines():
        stripped = line.strip()
        if stripped.startswith(("- ", "- [ ] ", "|")) and not set(stripped) <= {"|", "-", " "}:
            count += 1
    return count
```

- [ ] **Step 4: Add missing import**

Add `import re` near the top of `tooling/scripts/audit_subject_specialization.py`.

- [ ] **Step 5: Run focused test**

Run:

```bash
python3 -m unittest tests.test_subject_specialization_audit.SubjectSpecializationAuditTests.test_short_append_overlay_is_reported -v
```

Expected: PASS.

- [ ] **Step 6: Run full subject audit test and verify red baseline**

Run:

```bash
python3 -m unittest tests.test_subject_specialization_audit -v
```

Expected: FAIL in `test_current_subjects_pass_depth_audit` because current overlays are still thin.

## Task 3: Expand Subject Append Overlays

**Files:**
- Modify append overlays under `content/subjects/*/overlays/skills/*.md`
- Do not change `mode: replace_sections` headings in stats overlays.

- [ ] **Step 1: Replace each append overlay with the required section shape**

For each append overlay, use this structure with subject-specific content:

```markdown
## <Subject> Overlay

### Activation
Apply this overlay when the active subject is `<subject>` and this generic skill is used for a subject package.

### Required Context
- Require the active research question, paper type, target venue or venue family, data/evidence source, and current artifact path.
- Require the subject-specific construct, mechanism, or claim type before writing or evaluating output.

### Subject-Specific Procedure
1. Classify the claim or task using the subject's disciplinary categories.
2. Map the claim to the required evidence units named in the subject design spec.
3. Check the strongest discipline-specific validity or reviewer-risk issue.
4. Narrow, block, or revise claims when the required evidence is missing.

### Reviewer-Risk Checks
- Check the highest-risk objection a strong reviewer in this subject would raise.
- Check whether the artifact confuses description, mechanism, identification, measurement, or interpretation.

### Output Requirements
- State the subject-specific claim classification.
- Name the required evidence, diagnostic, or mechanism map.
- Record missing evidence as a gap note or blocked condition instead of inventing support.

### Blocked Conditions
- Block or narrow the output when the required subject-specific context is absent.
- Do not upgrade descriptive evidence into causal, mechanism, measurement, or policy claims.
```

- [ ] **Step 2: Add subject-specific details to each overlay**

Use these required terms and checks:

```text
economics: estimand, identifying variation, comparison group, treatment timing, causal claim, robustness, standard errors
accounting: construct, proxy, disclosure, accrual, fiscal timing, sample filters, measurement validity
business: theory contribution, construct, level of analysis, empirical setting, doctoral-level journal, rival framing
finance: asset pricing, risk-adjusted benchmark, return construction, event window, look-ahead bias, survivorship
political-economy: political mechanism, actors, institutions, incentives, distributional conflict, policy outcome
geoeconomics: statecraft, sanctions, sender, target, instrument, supply chain exposure, strategic competition
economics-accounting: identification, disclosure, measurement, archival proxy, fiscal timing, capital-market outcome
```

- [ ] **Step 3: Run subject audit**

Run:

```bash
python3 scripts/audit_subject_specialization.py
```

Expected: PASS or only replacement-overlay depth findings that Task 4 addresses.

## Task 4: Deepen Replacement Stats Overlays

**Files:**
- Modify: `content/subjects/economics/overlays/skills/stats-engine.md`
- Modify: `content/subjects/accounting/overlays/skills/stats-engine.md`
- Modify: `content/subjects/business/overlays/skills/stats-engine.md`
- Modify: `content/subjects/finance/overlays/skills/stats-engine.md`
- Modify: `content/subjects/economics-accounting/overlays/skills/stats-engine.md`

- [ ] **Step 1: Preserve exact replacement headings**

Each file must keep:

```markdown
## Quality Bar
- [ ] The overlay-specific statistical quality checks are listed here.

## Common Pitfalls
| Pitfall | Impact | Fix |
|---|---|---|
| Subject-specific statistical failure | The result is overclaimed or underdiagnosed | Add the required diagnostic, robustness check, or claim narrowing |
```

- [ ] **Step 2: Add at least eight concrete checklist/table items**

Each stats overlay must include field-specific items matching its subject:

```text
economics: DID, IV, RD, clustering, estimand, pretrend, robustness, specification search
accounting: accrual model, fiscal timing, winsorization, Compustat/CRSP linking, fixed effects, clustering, sample filters, proxy validity
business: construct validity, level of analysis, qualitative transparency, sampling, common-method bias, model fit, robustness, theory-to-evidence alignment
finance: factor model, abnormal return, event window, delisting returns, survivorship, look-ahead, overlapping observations, Fama-MacBeth/Newey-West
economics-accounting: disclosure event timing, archival proxy, causal estimand, fiscal timing, capital-market outcome, matched sample, clustered errors, measurement robustness
```

- [ ] **Step 3: Run replacement overlay audit**

Run:

```bash
python3 scripts/audit_subject_specialization.py
```

Expected: PASS.

## Task 5: Verify Materialization And Eval Cases

**Files:**
- No new implementation files.

- [ ] **Step 1: Run focused unit tests**

Run:

```bash
python3 -m unittest tests.test_subject_specialization_audit -v
python3 -m unittest tests.test_subject_materializer -v
python3 -m unittest tests.test_subject_eval_cases -v
```

Expected: all PASS.

- [ ] **Step 2: Run CLI audits**

Run:

```bash
python3 scripts/audit_subject_specialization.py
python3 scripts/audit_subject_eval_cases.py
```

Expected: both commands exit 0 with no findings.

- [ ] **Step 3: Inspect git diff**

Run:

```bash
git diff --stat
git diff -- content/subjects tooling/scripts/audit_subject_specialization.py tests/test_subject_specialization_audit.py
```

Expected: diff is limited to audit/test code and subject overlay Markdown files.

## Task 6: Commit The Implementation

**Files:**
- Commit files changed by Tasks 1-5.

- [ ] **Step 1: Stage implementation files**

Run:

```bash
git add tooling/scripts/audit_subject_specialization.py tests/test_subject_specialization_audit.py content/subjects
```

- [ ] **Step 2: Commit**

Run:

```bash
git commit -m "feat: deepen subject overlays"
```

- [ ] **Step 3: Confirm final status**

Run:

```bash
git status --short --branch
```

Expected: clean worktree on `dev`, with local branch ahead only by the implementation commit if it has not been pushed.
