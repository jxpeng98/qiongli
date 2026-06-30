# Runtime Subject Refinement Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build adaptive runtime subject refinement so every Qiongli install starts as the full core agent, infers subject needs during use, borrows narrow audited method lenses for boundary cases, and only persists a subject when the evidence or user choice supports it.

**Architecture:** Add a standards-backed `SubjectResolver` that produces a stable `subject_refinement` packet from task context plus `.qiongli/guidance_manifest.yaml`. Keep installed packages core-first and adaptive; subject catalog, overlays, profiles, and venue metadata are runtime resources, not install-time choices. Integrate the packet into MCP previews, real task runs, guidance proposals, and materialized skill/plugin packages while preserving existing `project_subject`, `domain`, and manifest compatibility fields.

**Tech Stack:** Python 3.12+, `dataclasses`, `re`, `pathlib`, `PyYAML`, existing `unittest` test style, Qiongli materializer/installers, Markdown skill content.

---

## File Structure

- Create: `content/standards/subject-refinement-contract.yaml`
  - Declares decision classes, persistence modes, loading levels, and initial economics/finance signal rules.
- Create: `packages/python-qiongli/src/qiongli/bridges/subject_refinement.py`
  - Owns signal scoring, edge-subject classification, resource loading metadata, and packet serialization.
- Modify: `packages/python-qiongli/src/qiongli/bridges/project_manifest.py`
  - Adds `subject_mode: auto | suggested | confirmed | locked` while preserving older manifests.
- Modify: `packages/python-qiongli/src/qiongli/bridges/project_inference.py`
  - Keeps the legacy manifest-suggestion API by delegating to `subject_refinement`.
- Modify: `packages/python-qiongli/src/qiongli/bridges/subject_runtime.py`
  - Keeps `ProjectSubjectState` stable and exposes summary wording for refinement-aware callers.
- Modify: `packages/python-qiongli/src/qiongli/bridges/mcp_tool_handlers.py`
  - Adds `subject_refinement` to `qiongli_task_run` preview and task packet output.
- Modify: `packages/python-qiongli/src/qiongli/bridges/orchestrator.py`
  - Adds the same packet to real `task_run` packets and agent prompts.
- Modify: `packages/python-qiongli/src/qiongli/bridges/guidance_runtime.py`
  - Writes `subject_refinement.json`, renders refinement-aware proposals, and only applies structured manifest changes for promoted subjects.
- Modify: `packages/python-qiongli/src/qiongli/subject_materializer.py`
  - Ships adaptive subject-refinement resources in full packages and a compact index in desktop packages.
- Modify: `packages/python-qiongli/src/qiongli/local_plugin_installer.py`
  - Uses the adaptive materialized core package for Codex, Claude, and Antigravity plugin installs.
- Modify: `packages/python-qiongli/src/qiongli/universal_installer.py`
  - Prints install output as adaptive core and keeps `--subject` as an advanced override.
- Modify: `packages/python-qiongli/src/qiongli/cli.py`
  - Updates install and upgrade help text to explain runtime subject refinement.
- Modify: `packages/python-qiongli/src/qiongli/bridges/mcp_cli.py`
  - Updates MCP install help text to match CLI semantics.
- Modify: `content/workflow/SKILL.md`
  - Documents the adaptive refinement contract for skill-only clients.
- Modify: `content/skills-core.md`
  - Documents domain profiles, borrowed lenses, and subject refinement guardrails.
- Test: `tests/test_subject_refinement.py`
  - New resolver tests for borrow/suggest/locked behavior.
- Test: `tests/test_project_manifest.py`
  - Manifest mode normalization and backward compatibility.
- Test: `tests/test_project_inference.py`
  - Legacy API compatibility after the resolver move.
- Test: `tests/test_guidance_runtime.py`
  - Trace/proposal/apply behavior for refinement packets.
- Test: `tests/test_mcp_tool_handlers.py`
  - Preview packet and task packet behavior.
- Test: `tests/test_subject_materializer.py`
  - Full and desktop adaptive package contents.
- Test: `tests/test_local_plugin_installer.py`
  - Marketplace/client plugin adaptive package contents.
- Test: `tests/test_universal_installer.py`
  - Default install output/help semantics.

---

## Decision Contract

Use these fields consistently across implementation and tests:

```python
DecisionClass = Literal[
    "no_subject",
    "borrow_lens",
    "suggest_subject",
    "confirm_subject",
    "lock_subject",
]

SubjectMode = Literal["auto", "suggested", "confirmed", "locked"]

PersistenceStatus = Literal["temporary", "proposed", "applied", "locked"]

ResourceLevel = Literal[
    "core_only",
    "method_pack_only",
    "skill_overlay",
    "subject_skill",
    "venue_profile",
    "project_guidance",
]
```

Classification rules:

- `no_subject`: no candidate has method, data, outcome, venue, or literature evidence.
- `borrow_lens`: exactly one subject has method-only evidence.
- `suggest_subject`: a subject has method evidence plus at least one of data, outcome, venue, or literature.
- `confirm_subject`: current manifest is `subject_mode: confirmed`, or a non-auto subject exists in an older manifest.
- `lock_subject`: current manifest is `subject_mode: locked`.

Promotion rules:

- A borrowed lens never updates `active_subject`.
- A suggested subject can be written to a proposal.
- A confirmed or locked subject can be used as the active project subject.
- A locked subject cannot be replaced by inference; it can still receive borrowed lenses from another subject.

---

### Task 1: Add Resolver Acceptance Tests

**Files:**
- Create: `tests/test_subject_refinement.py`

- [ ] **Step 1: Write the failing tests**

Create `tests/test_subject_refinement.py`:

```python
from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from bridges.project_manifest import MANIFEST_REL, ProjectManifest, ProjectManifestState
from bridges.subject_refinement import infer_subject_refinement


def _manifest_state(
    root: Path,
    *,
    exists: bool = False,
    active_subject: str = "auto",
    subject_mode: str = "auto",
    method_lenses: list[str] | None = None,
) -> ProjectManifestState:
    return ProjectManifestState(
        exists=exists,
        path=root / MANIFEST_REL,
        project_root=root,
        manifest=ProjectManifest(
            active_subject=active_subject,
            subject_mode=subject_mode,
            method_lenses=method_lenses or [],
        ).normalized(),
        warnings=[],
    )


class SubjectRefinementTests(unittest.TestCase):
    def test_single_finance_method_signal_borrows_lens_without_subject_switch(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            packet = infer_subject_refinement(
                {
                    "task_id": "C1",
                    "paper_type": "empirical",
                    "topic": "management disclosure study",
                    "context": "Use an event study around the disclosure date.",
                },
                manifest_state=_manifest_state(root),
                draft_content="",
                review_content="",
                merged_analysis="",
            ).to_packet()

        self.assertEqual(packet["decision"], "borrow_lens")
        self.assertEqual(packet["mode"], "auto")
        self.assertEqual(packet["active_subject"], "auto")
        self.assertIsNone(packet["primary_subject"])
        self.assertEqual(packet["method_lenses"], ["event-study"])
        self.assertEqual(
            packet["borrowed_lenses"],
            [
                {
                    "source_subject": "finance",
                    "lens": "event-study",
                    "resource_level": "method_pack_only",
                    "reason": (
                        "Finance event-study diagnostics are relevant, but the task has "
                        "method-only evidence and should not switch the whole project to finance."
                    ),
                }
            ],
        )
        self.assertEqual(packet["loaded_resources"]["levels"], ["method_pack_only"])
        self.assertEqual(packet["persistence"]["status"], "temporary")

    def test_finance_method_data_outcome_and_venue_suggests_primary_subject(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            packet = infer_subject_refinement(
                {
                    "task_id": "C1",
                    "paper_type": "empirical",
                    "topic": "stock market reaction to earnings announcements",
                    "context": (
                        "CRSP abnormal returns event study with a Journal of Finance "
                        "contribution framing."
                    ),
                },
                manifest_state=_manifest_state(root),
                draft_content="The design estimates cumulative abnormal returns.",
                review_content="",
                merged_analysis="",
            ).to_packet()

        self.assertEqual(packet["decision"], "suggest_subject")
        self.assertEqual(packet["mode"], "suggested")
        self.assertEqual(packet["active_subject"], "auto")
        self.assertEqual(packet["primary_subject"], "finance")
        self.assertGreaterEqual(packet["candidate_subjects"][0]["confidence"], 0.75)
        self.assertEqual(
            packet["candidate_subjects"][0]["matched_dimensions"],
            ["method", "data", "outcome", "venue"],
        )
        self.assertIn("skill_overlay", packet["loaded_resources"]["levels"])
        self.assertIn("subject_skill", packet["loaded_resources"]["levels"])
        self.assertIn("finance-identification-risk-auditor", packet["loaded_resources"]["subject_skills"])
        self.assertEqual(packet["persistence"]["status"], "proposed")

    def test_locked_manifest_prevents_auto_switch_but_keeps_borrowed_lens(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            packet = infer_subject_refinement(
                {
                    "task_id": "C1",
                    "paper_type": "empirical",
                    "topic": "public policy project",
                    "context": "Use abnormal returns in an event window as one robustness check.",
                },
                manifest_state=_manifest_state(
                    root,
                    exists=True,
                    active_subject="economics",
                    subject_mode="locked",
                ),
                draft_content="",
                review_content="",
                merged_analysis="",
            ).to_packet()

        self.assertEqual(packet["decision"], "lock_subject")
        self.assertEqual(packet["mode"], "locked")
        self.assertEqual(packet["active_subject"], "economics")
        self.assertEqual(packet["primary_subject"], "economics")
        self.assertEqual(packet["borrowed_lenses"][0]["source_subject"], "finance")
        self.assertEqual(packet["borrowed_lenses"][0]["lens"], "event-study")
        self.assertEqual(packet["persistence"]["status"], "locked")

    def test_confirmed_manifest_controls_active_subject_when_context_is_weak(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            packet = infer_subject_refinement(
                {
                    "task_id": "F3",
                    "paper_type": "empirical",
                    "topic": "revise discussion",
                    "context": "Tighten the contribution paragraph.",
                },
                manifest_state=_manifest_state(
                    root,
                    exists=True,
                    active_subject="finance",
                    subject_mode="confirmed",
                    method_lenses=["asset-pricing"],
                ),
                draft_content="",
                review_content="",
                merged_analysis="",
            ).to_packet()

        self.assertEqual(packet["decision"], "confirm_subject")
        self.assertEqual(packet["mode"], "confirmed")
        self.assertEqual(packet["active_subject"], "finance")
        self.assertEqual(packet["primary_subject"], "finance")
        self.assertEqual(packet["method_lenses"], ["asset-pricing"])
        self.assertEqual(packet["persistence"]["status"], "applied")

    def test_no_subject_signal_keeps_core_only_resources(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            packet = infer_subject_refinement(
                {
                    "task_id": "F1",
                    "paper_type": "theory",
                    "topic": "revise introduction",
                    "context": "Make paragraph transitions clearer.",
                },
                manifest_state=_manifest_state(root),
                draft_content="",
                review_content="",
                merged_analysis="",
            ).to_packet()

        self.assertEqual(packet["decision"], "no_subject")
        self.assertEqual(packet["mode"], "auto")
        self.assertEqual(packet["active_subject"], "auto")
        self.assertEqual(packet["loaded_resources"]["levels"], ["core_only"])
        self.assertEqual(packet["candidate_subjects"], [])


if __name__ == "__main__":
    unittest.main()
```

- [ ] **Step 2: Run tests to verify they fail**

Run:

```bash
PYTHONPATH=packages/python-qiongli/src/qiongli:packages/python-qiongli/src uv run pytest tests/test_subject_refinement.py -q
```

Expected: FAIL with `ModuleNotFoundError: No module named 'bridges.subject_refinement'`.

- [ ] **Step 3: Commit failing tests**

```bash
git add tests/test_subject_refinement.py
git commit -m "test(subjects): add runtime refinement acceptance cases"
```

---

### Task 2: Extend Project Manifest With Subject Mode

**Files:**
- Modify: `packages/python-qiongli/src/qiongli/bridges/project_manifest.py`
- Modify: `tests/test_project_manifest.py`

- [ ] **Step 1: Add failing manifest mode tests**

Append these tests to `tests/test_project_manifest.py` inside the existing test class:

```python
    def test_project_manifest_defaults_to_auto_subject_mode(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            state = load_project_manifest(Path(tmp_dir))

        self.assertEqual(state.manifest.subject_mode, "auto")
        self.assertEqual(state.to_packet()["manifest"]["subject_mode"], "auto")

    def test_project_manifest_reads_subject_mode(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            (root / ".qiongli").mkdir()
            (root / ".qiongli" / "guidance_manifest.yaml").write_text(
                "\n".join(
                    [
                        "active_subject: finance",
                        "subject_mode: locked",
                        "method_lenses:",
                        "  - event-study",
                    ]
                ),
                encoding="utf-8",
            )

            state = load_project_manifest(root)

        self.assertEqual(state.manifest.active_subject, "finance")
        self.assertEqual(state.manifest.subject_mode, "locked")
        self.assertEqual(state.manifest.method_lenses, ["event-study"])

    def test_legacy_non_auto_manifest_is_treated_as_confirmed(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            (root / ".qiongli").mkdir()
            (root / ".qiongli" / "guidance_manifest.yaml").write_text(
                "active_subject: economics\n",
                encoding="utf-8",
            )

            state = load_project_manifest(root)

        self.assertEqual(state.manifest.active_subject, "economics")
        self.assertEqual(state.manifest.subject_mode, "confirmed")

    def test_update_project_manifest_can_lock_subject(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            state = update_project_manifest(
                root,
                active_subject="finance",
                subject_mode="locked",
                method_lenses=["event-study"],
            )

            payload = state.to_packet()["manifest"]

        self.assertEqual(payload["active_subject"], "finance")
        self.assertEqual(payload["subject_mode"], "locked")
        self.assertEqual(payload["method_lenses"], ["event-study"])
```

- [ ] **Step 2: Run tests to verify they fail**

Run:

```bash
PYTHONPATH=packages/python-qiongli/src/qiongli:packages/python-qiongli/src uv run pytest tests/test_project_manifest.py -q
```

Expected: FAIL with `AttributeError: 'ProjectManifest' object has no attribute 'subject_mode'`.

- [ ] **Step 3: Add `subject_mode` support**

Patch `packages/python-qiongli/src/qiongli/bridges/project_manifest.py`:

```python
SUBJECT_MODE_CHOICES = ("auto", "suggested", "confirmed", "locked")
KNOWN_FIELDS = {
    "active_subject",
    "subject_mode",
    "secondary_subjects",
    "venue_profiles",
    "method_lenses",
    "strictness",
}


@dataclass(frozen=True)
class ProjectManifest:
    active_subject: str = "auto"
    subject_mode: str = "auto"
    secondary_subjects: list[str] | None = None
    venue_profiles: list[str] | None = None
    method_lenses: list[str] | None = None
    strictness: str = "standard"

    def normalized(self) -> ProjectManifest:
        subject = _validate_subject(self.active_subject, field="active_subject")
        return ProjectManifest(
            active_subject=subject,
            subject_mode=_validate_subject_mode(self.subject_mode, active_subject=subject),
            secondary_subjects=_validate_subject_list(
                self.secondary_subjects,
                field="secondary_subjects",
            ),
            venue_profiles=_validate_rel_path_list(self.venue_profiles, field="venue_profiles"),
            method_lenses=_validate_rel_path_list(self.method_lenses, field="method_lenses"),
            strictness=_validate_strictness(self.strictness),
        )

    def to_dict(self) -> dict[str, Any]:
        manifest = self.normalized()
        return {
            "active_subject": manifest.active_subject,
            "subject_mode": manifest.subject_mode,
            "secondary_subjects": list(manifest.secondary_subjects or []),
            "venue_profiles": list(manifest.venue_profiles or []),
            "method_lenses": list(manifest.method_lenses or []),
            "strictness": manifest.strictness,
        }
```

Update `update_project_manifest()` signature and construction:

```python
def update_project_manifest(
    project_root: Path,
    *,
    active_subject: str | None = None,
    subject_mode: str | None = None,
    secondary_subjects: Sequence[str] | None = None,
    venue_profiles: Sequence[str] | None = None,
    method_lenses: Sequence[str] | None = None,
    strictness: str | None = None,
) -> ProjectManifestState:
    current = load_project_manifest(project_root)
    manifest = ProjectManifest(
        active_subject=active_subject if active_subject is not None else current.manifest.active_subject,
        subject_mode=subject_mode if subject_mode is not None else current.manifest.subject_mode,
        secondary_subjects=(
            list(secondary_subjects)
            if secondary_subjects is not None
            else list(current.manifest.secondary_subjects or [])
        ),
        venue_profiles=(
            list(venue_profiles)
            if venue_profiles is not None
            else list(current.manifest.venue_profiles or [])
        ),
        method_lenses=(
            list(method_lenses)
            if method_lenses is not None
            else list(current.manifest.method_lenses or [])
        ),
        strictness=strictness if strictness is not None else current.manifest.strictness,
    ).normalized()
```

Update `manifest_to_guidance_section()` rows:

```python
        f"- active_subject: {manifest.active_subject}",
        f"- subject_mode: {manifest.subject_mode}",
        f"- secondary_subjects: {_display_list(manifest.secondary_subjects)}",
```

Update `_manifest_from_mapping()`:

```python
def _manifest_from_mapping(payload: Mapping[Any, Any]) -> tuple[ProjectManifest, list[str]]:
    warnings = [
        f"Ignored unsupported manifest field: {key}"
        for key in sorted(str(key) for key in payload.keys() if str(key) not in KNOWN_FIELDS)
    ]
    active_subject = payload.get("active_subject", "auto")
    raw_mode = payload.get("subject_mode")
    subject_mode = (
        raw_mode
        if raw_mode is not None
        else ("confirmed" if str(active_subject).strip() not in {"auto", "core"} else "auto")
    )
    manifest = ProjectManifest(
        active_subject=active_subject,
        subject_mode=subject_mode,
        secondary_subjects=payload.get("secondary_subjects"),
        venue_profiles=payload.get("venue_profiles"),
        method_lenses=payload.get("method_lenses"),
        strictness=payload.get("strictness", "standard"),
    ).normalized()
    return manifest, warnings
```

Add validator:

```python
def _validate_subject_mode(value: Any, *, active_subject: str) -> str:
    if not isinstance(value, str):
        raise ProjectManifestError(f"Unsupported subject_mode: {value!r}")
    normalized = value.strip()
    if normalized not in SUBJECT_MODE_CHOICES:
        raise ProjectManifestError(f"Unsupported subject_mode: {normalized}")
    if normalized in {"confirmed", "locked"} and active_subject in {"auto", "core"}:
        raise ProjectManifestError(f"subject_mode {normalized} requires a non-auto active_subject")
    return normalized
```

- [ ] **Step 4: Run manifest tests**

Run:

```bash
PYTHONPATH=packages/python-qiongli/src/qiongli:packages/python-qiongli/src uv run pytest tests/test_project_manifest.py -q
```

Expected: PASS.

- [ ] **Step 5: Commit manifest mode support**

```bash
git add packages/python-qiongli/src/qiongli/bridges/project_manifest.py tests/test_project_manifest.py
git commit -m "feat(subjects): track project subject mode"
```

---

### Task 3: Implement Subject Refinement Contract And Resolver

**Files:**
- Create: `content/standards/subject-refinement-contract.yaml`
- Create: `packages/python-qiongli/src/qiongli/bridges/subject_refinement.py`
- Modify: `packages/python-qiongli/src/qiongli/bridges/project_inference.py`
- Modify: `tests/test_project_inference.py`

- [ ] **Step 1: Add the contract YAML**

Create `content/standards/subject-refinement-contract.yaml`:

```yaml
contract_version: "1.0.0"
name: "subject-refinement-contract"
default_mode: "auto"

subject_modes:
  - "auto"
  - "suggested"
  - "confirmed"
  - "locked"

decision_classes:
  no_subject:
    persistence_status: "temporary"
    levels: ["core_only"]
  borrow_lens:
    persistence_status: "temporary"
    levels: ["method_pack_only"]
  suggest_subject:
    persistence_status: "proposed"
    levels: ["skill_overlay", "subject_skill"]
  confirm_subject:
    persistence_status: "applied"
    levels: ["skill_overlay", "subject_skill", "project_guidance"]
  lock_subject:
    persistence_status: "locked"
    levels: ["skill_overlay", "subject_skill", "project_guidance"]

resource_levels:
  core_only: "Use canonical core workflow only."
  method_pack_only: "Borrow narrow audited method diagnostics without switching the project subject."
  skill_overlay: "Apply subject overlays to active core skills."
  subject_skill: "Expose subject-specific auditor skills."
  venue_profile: "Load venue profile only when venue evidence is present."
  project_guidance: "Persist confirmed project-local guidance."

subjects:
  finance:
    domain_profile: "skills/domain-profiles/finance.yaml"
    subject_skills:
      - "finance-identification-risk-auditor"
    overlays:
      - "subjects/finance/overlays/skills/study-designer.md"
      - "subjects/finance/overlays/skills/stats-engine.md"
      - "subjects/finance/overlays/skills/manuscript-architect.md"
    venue_profiles:
      journal-of-finance:
        patterns:
          - "journal of finance"
          - "\\bjf\\b"
      review-of-financial-studies:
        patterns:
          - "review of financial studies"
          - "\\brfs\\b"
      journal-of-financial-economics:
        patterns:
          - "journal of financial economics"
          - "\\bjfe\\b"
    lenses:
      event-study:
        method:
          - "event study"
          - "event window"
          - "announcement window"
        data:
          - "crsp"
          - "compustat"
          - "earnings announcement"
          - "disclosure date"
        outcome:
          - "abnormal return"
          - "cumulative abnormal return"
          - "stock return"
          - "market reaction"
        literature:
          - "corporate finance"
          - "market microstructure"
      asset-pricing:
        method:
          - "asset pricing"
          - "factor model"
          - "factor exposure"
        data:
          - "portfolio sort"
          - "ff3"
          - "ff5"
          - "fama french"
        outcome:
          - "expected return"
          - "return predictability"
          - "portfolio return"
        literature:
          - "asset pricing"
          - "cross-section of returns"

  economics:
    domain_profile: "skills/domain-profiles/economics.yaml"
    subject_skills:
      - "econ-identification-auditor"
      - "econ-replication-package-auditor"
    overlays:
      - "subjects/economics/overlays/skills/study-designer.md"
      - "subjects/economics/overlays/skills/robustness-planner.md"
      - "subjects/economics/overlays/skills/stats-engine.md"
      - "subjects/economics/overlays/skills/manuscript-architect.md"
    venue_profiles:
      aer:
        patterns:
          - "american economic review"
          - "\\baer\\b"
      qje:
        patterns:
          - "quarterly journal of economics"
          - "\\bqje\\b"
      econometrica:
        patterns:
          - "econometrica"
    lenses:
      did:
        method:
          - "difference-in-differences"
          - "difference in differences"
          - "\\bdid\\b"
          - "parallel trends"
          - "pre-trends"
        data:
          - "panel data"
          - "administrative data"
          - "policy shock"
        outcome:
          - "treatment effect"
          - "policy outcome"
          - "welfare effect"
        literature:
          - "applied microeconomics"
          - "labor economics"
      causal-identification:
        method:
          - "instrumental variable"
          - "regression discontinuity"
          - "causal identification"
          - "\\brdd?\\b"
        data:
          - "natural experiment"
          - "quasi-experiment"
        outcome:
          - "causal effect"
          - "treatment effect"
        literature:
          - "econometrics"
          - "causal inference"
```

- [ ] **Step 2: Implement the resolver module**

Create `packages/python-qiongli/src/qiongli/bridges/subject_refinement.py`:

```python
from __future__ import annotations

import re
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Any, Mapping

import yaml

from .project_manifest import ProjectManifestState


DECISION_ORDER = ("no_subject", "borrow_lens", "suggest_subject", "confirm_subject", "lock_subject")
DIMENSION_ORDER = ("method", "data", "outcome", "venue", "literature")
SUBJECT_TO_DOMAIN = {
    "auto": "auto",
    "core": "auto",
    "economics": "economics",
    "finance": "finance",
    "accounting": "accounting",
    "business": "business-management",
    "political-economy": "political-economy",
    "geoeconomics": "geoeconomics",
    "economics-accounting": "economics",
}


@dataclass(frozen=True)
class SubjectCandidate:
    subject: str
    confidence: float
    evidence: list[str]
    matched_dimensions: list[str]
    method_lenses: list[str]

    def to_packet(self) -> dict[str, Any]:
        return asdict(self)


@dataclass(frozen=True)
class BorrowedLens:
    source_subject: str
    lens: str
    resource_level: str
    reason: str

    def to_packet(self) -> dict[str, Any]:
        return asdict(self)


@dataclass(frozen=True)
class SubjectRefinementPacket:
    decision: str
    mode: str
    active_subject: str
    primary_subject: str | None
    secondary_subjects: list[str]
    candidate_subjects: list[SubjectCandidate]
    method_lenses: list[str]
    borrowed_lenses: list[BorrowedLens]
    loaded_resources: dict[str, Any]
    persistence: dict[str, Any]
    summary: str

    @property
    def domain(self) -> str:
        return SUBJECT_TO_DOMAIN.get(self.primary_subject or self.active_subject, "auto")

    def to_packet(self) -> dict[str, Any]:
        return {
            "decision": self.decision,
            "mode": self.mode,
            "active_subject": self.active_subject,
            "primary_subject": self.primary_subject,
            "secondary_subjects": list(self.secondary_subjects),
            "candidate_subjects": [candidate.to_packet() for candidate in self.candidate_subjects],
            "method_lenses": list(self.method_lenses),
            "borrowed_lenses": [lens.to_packet() for lens in self.borrowed_lenses],
            "loaded_resources": dict(self.loaded_resources),
            "persistence": dict(self.persistence),
            "summary": self.summary,
            "domain": self.domain,
        }


def infer_subject_refinement(
    task_packet: dict[str, Any],
    *,
    manifest_state: ProjectManifestState,
    draft_content: str = "",
    review_content: str = "",
    merged_analysis: str = "",
    standards_dir: Path | None = None,
) -> SubjectRefinementPacket:
    contract = _load_contract(standards_dir)
    text = _joined_text(task_packet, draft_content, review_content, merged_analysis)
    candidates = _rank_candidates(contract, text)
    manifest = manifest_state.manifest.normalized()
    manifest_lenses = list(manifest.method_lenses or [])

    if manifest.subject_mode == "locked":
        borrowed = _borrowed_lenses(candidates, active_subject=manifest.active_subject)
        method_lenses = _dedupe([*manifest_lenses, *(lens.lens for lens in borrowed)])
        return _packet(
            contract,
            decision="lock_subject",
            mode="locked",
            active_subject=manifest.active_subject,
            primary_subject=manifest.active_subject,
            secondary_subjects=list(manifest.secondary_subjects or []),
            candidates=candidates,
            method_lenses=method_lenses,
            borrowed_lenses=borrowed,
            subject_for_resources=manifest.active_subject,
        )

    if manifest.subject_mode == "confirmed":
        borrowed = _borrowed_lenses(candidates, active_subject=manifest.active_subject)
        method_lenses = _dedupe([*manifest_lenses, *(lens.lens for lens in borrowed)])
        return _packet(
            contract,
            decision="confirm_subject",
            mode="confirmed",
            active_subject=manifest.active_subject,
            primary_subject=manifest.active_subject,
            secondary_subjects=list(manifest.secondary_subjects or []),
            candidates=candidates,
            method_lenses=method_lenses,
            borrowed_lenses=borrowed,
            subject_for_resources=manifest.active_subject,
        )

    best = candidates[0] if candidates else None
    if best is None:
        return _packet(
            contract,
            decision="no_subject",
            mode="auto",
            active_subject=manifest.active_subject,
            primary_subject=None,
            secondary_subjects=list(manifest.secondary_subjects or []),
            candidates=[],
            method_lenses=manifest_lenses,
            borrowed_lenses=[],
            subject_for_resources=None,
        )

    if best.matched_dimensions == ["method"]:
        borrowed = _borrowed_lenses([best], active_subject=manifest.active_subject)
        return _packet(
            contract,
            decision="borrow_lens",
            mode="auto",
            active_subject=manifest.active_subject,
            primary_subject=None,
            secondary_subjects=list(manifest.secondary_subjects or []),
            candidates=candidates,
            method_lenses=_dedupe([*manifest_lenses, *best.method_lenses]),
            borrowed_lenses=borrowed,
            subject_for_resources=best.subject,
        )

    return _packet(
        contract,
        decision="suggest_subject",
        mode="suggested",
        active_subject=manifest.active_subject,
        primary_subject=best.subject,
        secondary_subjects=_dedupe([*list(manifest.secondary_subjects or []), best.subject]),
        candidates=candidates,
        method_lenses=_dedupe([*manifest_lenses, *best.method_lenses]),
        borrowed_lenses=[],
        subject_for_resources=best.subject,
    )


def legacy_manifest_suggestion(packet: SubjectRefinementPacket) -> dict[str, Any]:
    payload = packet.to_packet()
    active_subject = "auto"
    if packet.decision in {"suggest_subject", "confirm_subject", "lock_subject"} and packet.primary_subject:
        active_subject = packet.primary_subject
    return {
        "active_subject": active_subject,
        "subject_mode": packet.mode,
        "method_lenses": list(packet.method_lenses),
        "confidence": (
            packet.candidate_subjects[0].confidence
            if packet.candidate_subjects
            else (1.0 if packet.decision in {"confirm_subject", "lock_subject"} else 0.0)
        ),
        "evidence": payload["candidate_subjects"][0]["evidence"] if payload["candidate_subjects"] else [],
        "subject_refinement": payload,
    }


def _load_contract(standards_dir: Path | None) -> dict[str, Any]:
    candidates: list[Path] = []
    if standards_dir is not None:
        candidates.append(Path(standards_dir) / "subject-refinement-contract.yaml")
    current = Path(__file__).resolve()
    for parent in current.parents:
        candidates.extend(
            [
                parent / "content" / "standards" / "subject-refinement-contract.yaml",
                parent / "standards" / "subject-refinement-contract.yaml",
                parent / "payload" / "qiongli-workflow" / "standards" / "subject-refinement-contract.yaml",
            ]
        )
    for path in candidates:
        if path.is_file():
            loaded = yaml.safe_load(path.read_text(encoding="utf-8")) or {}
            if isinstance(loaded, dict):
                return loaded
    raise FileNotFoundError("subject-refinement-contract.yaml was not found")


def _rank_candidates(contract: Mapping[str, Any], text: str) -> list[SubjectCandidate]:
    raw_subjects = contract.get("subjects", {})
    if not isinstance(raw_subjects, Mapping):
        return []
    candidates: list[SubjectCandidate] = []
    for subject, raw_subject in raw_subjects.items():
        if not isinstance(raw_subject, Mapping):
            continue
        evidence: list[str] = []
        dimensions: list[str] = []
        lenses: list[str] = []
        raw_lenses = raw_subject.get("lenses", {})
        if isinstance(raw_lenses, Mapping):
            for lens, raw_lens in raw_lenses.items():
                if not isinstance(raw_lens, Mapping):
                    continue
                lens_dimensions = _matched_lens_dimensions(raw_lens, text, evidence)
                if lens_dimensions and "method" in lens_dimensions:
                    lenses.append(str(lens))
                dimensions.extend(lens_dimensions)
        venue_hit = _matched_venue(raw_subject, text, evidence)
        if venue_hit:
            dimensions.append("venue")
        dimensions = [dimension for dimension in DIMENSION_ORDER if dimension in set(dimensions)]
        if not dimensions:
            continue
        confidence = _confidence(dimensions)
        candidates.append(
            SubjectCandidate(
                subject=str(subject),
                confidence=confidence,
                evidence=_dedupe(evidence)[:5],
                matched_dimensions=dimensions,
                method_lenses=_dedupe(lenses),
            )
        )
    return sorted(candidates, key=lambda item: (item.confidence, len(item.matched_dimensions)), reverse=True)


def _matched_lens_dimensions(raw_lens: Mapping[str, Any], text: str, evidence: list[str]) -> list[str]:
    dimensions: list[str] = []
    for dimension in ("method", "data", "outcome", "literature"):
        patterns = raw_lens.get(dimension, [])
        if _any_pattern_matches(patterns, text, evidence):
            dimensions.append(dimension)
    return dimensions


def _matched_venue(raw_subject: Mapping[str, Any], text: str, evidence: list[str]) -> bool:
    raw_profiles = raw_subject.get("venue_profiles", {})
    if not isinstance(raw_profiles, Mapping):
        return False
    matched = False
    for raw_profile in raw_profiles.values():
        if isinstance(raw_profile, Mapping) and _any_pattern_matches(raw_profile.get("patterns", []), text, evidence):
            matched = True
    return matched


def _any_pattern_matches(patterns: Any, text: str, evidence: list[str]) -> bool:
    if not isinstance(patterns, list):
        return False
    for raw in patterns:
        pattern = str(raw)
        match = re.search(pattern, text, flags=re.IGNORECASE)
        if match:
            evidence.append(_snippet(text, match.start(), match.end()))
            return True
    return False


def _confidence(dimensions: list[str]) -> float:
    weights = {"method": 0.45, "data": 0.12, "outcome": 0.12, "venue": 0.16, "literature": 0.10}
    return min(0.95, round(sum(weights[dimension] for dimension in dimensions), 2))


def _borrowed_lenses(candidates: list[SubjectCandidate], *, active_subject: str) -> list[BorrowedLens]:
    borrowed: list[BorrowedLens] = []
    for candidate in candidates:
        if candidate.subject == active_subject:
            continue
        if "method" not in candidate.matched_dimensions:
            continue
        for lens in candidate.method_lenses:
            borrowed.append(
                BorrowedLens(
                    source_subject=candidate.subject,
                    lens=lens,
                    resource_level="method_pack_only",
                    reason=(
                        f"{candidate.subject.title()} {lens.replace('-', ' ')} diagnostics are relevant, "
                        f"but the task has method-only evidence and should not switch the whole project "
                        f"to {candidate.subject}."
                    ),
                )
            )
    return borrowed


def _packet(
    contract: Mapping[str, Any],
    *,
    decision: str,
    mode: str,
    active_subject: str,
    primary_subject: str | None,
    secondary_subjects: list[str],
    candidates: list[SubjectCandidate],
    method_lenses: list[str],
    borrowed_lenses: list[BorrowedLens],
    subject_for_resources: str | None,
) -> SubjectRefinementPacket:
    loaded = _loaded_resources(contract, decision=decision, subject=subject_for_resources, method_lenses=method_lenses)
    persistence_status = str(
        contract.get("decision_classes", {})
        .get(decision, {})
        .get("persistence_status", "temporary")
    )
    return SubjectRefinementPacket(
        decision=decision,
        mode=mode,
        active_subject=active_subject,
        primary_subject=primary_subject,
        secondary_subjects=_dedupe(secondary_subjects),
        candidate_subjects=candidates,
        method_lenses=_dedupe(method_lenses),
        borrowed_lenses=borrowed_lenses,
        loaded_resources=loaded,
        persistence={"status": persistence_status, "proposal_path": ""},
        summary=_summary(decision, active_subject, primary_subject, borrowed_lenses),
    )


def _loaded_resources(
    contract: Mapping[str, Any],
    *,
    decision: str,
    subject: str | None,
    method_lenses: list[str],
) -> dict[str, Any]:
    levels = list(contract.get("decision_classes", {}).get(decision, {}).get("levels", ["core_only"]))
    result: dict[str, Any] = {
        "levels": levels,
        "domain_profiles": [],
        "overlays": [],
        "subject_skills": [],
        "venue_profiles": [],
        "method_lenses": list(method_lenses),
    }
    if not subject:
        return result
    raw_subject = contract.get("subjects", {}).get(subject, {})
    if not isinstance(raw_subject, Mapping):
        return result
    profile = raw_subject.get("domain_profile")
    if isinstance(profile, str):
        result["domain_profiles"].append(profile)
    if decision in {"suggest_subject", "confirm_subject", "lock_subject"}:
        result["overlays"].extend(str(item) for item in raw_subject.get("overlays", []) or [])
        result["subject_skills"].extend(str(item) for item in raw_subject.get("subject_skills", []) or [])
    return result


def _joined_text(
    task_packet: Mapping[str, Any],
    draft_content: str,
    review_content: str,
    merged_analysis: str,
) -> str:
    fields = [
        task_packet.get("task_id", ""),
        task_packet.get("paper_type", ""),
        task_packet.get("topic", ""),
        task_packet.get("context", ""),
        task_packet.get("venue", ""),
        draft_content,
        review_content,
        merged_analysis,
    ]
    return " ".join(str(item) for item in fields if str(item).strip())


def _summary(
    decision: str,
    active_subject: str,
    primary_subject: str | None,
    borrowed_lenses: list[BorrowedLens],
) -> str:
    if decision == "no_subject":
        return "Subject refinement: core workflow only; no subject-specific evidence was strong enough."
    if decision == "borrow_lens":
        borrowed = ", ".join(f"{item.source_subject}:{item.lens}" for item in borrowed_lenses)
        return f"Subject refinement: keep active_subject={active_subject}; borrow {borrowed}."
    if decision == "suggest_subject":
        return f"Subject refinement: suggest primary_subject={primary_subject} while active_subject remains {active_subject}."
    if decision == "confirm_subject":
        return f"Subject refinement: confirmed active_subject={active_subject}."
    return f"Subject refinement: locked active_subject={active_subject}; inference cannot replace it."


def _snippet(text: str, start: int, end: int) -> str:
    left = max(0, start - 36)
    right = min(len(text), end + 36)
    return " ".join(text[left:right].split())


def _dedupe(values: list[str] | Any) -> list[str]:
    result: list[str] = []
    for value in values:
        item = str(value).strip()
        if item and item not in result:
            result.append(item)
    return result
```

- [ ] **Step 3: Update legacy project inference wrapper**

Replace `packages/python-qiongli/src/qiongli/bridges/project_inference.py` with:

```python
from __future__ import annotations

from pathlib import Path
from typing import Any

from .project_manifest import ProjectManifest, ProjectManifestState
from .subject_refinement import infer_subject_refinement, legacy_manifest_suggestion


def infer_project_manifest_suggestion(
    task_packet: dict[str, Any],
    *,
    draft_content: str,
    review_content: str,
    merged_analysis: str,
) -> dict[str, Any]:
    root = Path.cwd()
    manifest_state = ProjectManifestState(
        exists=False,
        path=root / ".qiongli" / "guidance_manifest.yaml",
        project_root=root,
        manifest=ProjectManifest().normalized(),
        warnings=[],
    )
    packet = infer_subject_refinement(
        task_packet,
        manifest_state=manifest_state,
        draft_content=draft_content,
        review_content=review_content,
        merged_analysis=merged_analysis,
    )
    return legacy_manifest_suggestion(packet)
```

- [ ] **Step 4: Update project inference tests for boundary behavior**

In `tests/test_project_inference.py`, update the event-study test so method-only evidence does not force a manifest subject:

```python
    def test_method_only_finance_event_study_returns_borrowed_lens_without_manifest_switch(self) -> None:
        suggestion = infer_project_manifest_suggestion(
            {
                "topic": "management disclosure study",
                "context": "event study around disclosure date",
            },
            draft_content="Use an event window.",
            review_content="",
            merged_analysis="",
        )

        self.assertEqual(suggestion["active_subject"], "auto")
        self.assertIn("event-study", suggestion["method_lenses"])
        self.assertEqual(suggestion["subject_refinement"]["decision"], "borrow_lens")
```

Add a stronger finance test:

```python
    def test_finance_event_study_with_returns_data_suggests_manifest_subject(self) -> None:
        suggestion = infer_project_manifest_suggestion(
            {
                "topic": "earnings announcement stock market reaction",
                "context": "CRSP abnormal returns event study for Journal of Finance framing",
            },
            draft_content="Estimate cumulative abnormal returns.",
            review_content="",
            merged_analysis="",
        )

        self.assertEqual(suggestion["active_subject"], "finance")
        self.assertEqual(suggestion["subject_mode"], "suggested")
        self.assertIn("event-study", suggestion["method_lenses"])
        self.assertEqual(suggestion["subject_refinement"]["decision"], "suggest_subject")
```

- [ ] **Step 5: Run resolver and legacy inference tests**

Run:

```bash
PYTHONPATH=packages/python-qiongli/src/qiongli:packages/python-qiongli/src uv run pytest tests/test_subject_refinement.py tests/test_project_inference.py -q
```

Expected: PASS.

- [ ] **Step 6: Commit resolver**

```bash
git add content/standards/subject-refinement-contract.yaml packages/python-qiongli/src/qiongli/bridges/subject_refinement.py packages/python-qiongli/src/qiongli/bridges/project_inference.py tests/test_subject_refinement.py tests/test_project_inference.py
git commit -m "feat(subjects): infer runtime refinement packets"
```

---

### Task 4: Integrate Refinement Into MCP Preview Packets

**Files:**
- Modify: `packages/python-qiongli/src/qiongli/bridges/mcp_tool_handlers.py`
- Modify: `tests/test_mcp_tool_handlers.py`

- [ ] **Step 1: Add failing MCP preview tests**

Append these tests to `tests/test_mcp_tool_handlers.py`:

```python
    def test_task_run_preview_exposes_borrowed_subject_refinement_without_domain_switch(self) -> None:
        class StubResult:
            mode = "task-plan"
            confidence = 0.8
            merged_analysis = "preview"
            recommendations: list[str] = []
            data = {
                "task_id": "C1",
                "paper_type": "empirical",
                "topic": "management disclosure study",
                "artifact_root": "RESEARCH/[topic]/",
                "runtime_plan": {"primary_agent": "codex"},
            }

        class StubOrchestrator:
            def task_plan(self, **_kwargs: object) -> StubResult:
                return StubResult()

            def _build_controller_metadata(self, **_kwargs: object) -> dict[str, str]:
                return {
                    "execution_mode": "duo",
                    "controller": "codex",
                    "primary_agent": "",
                    "review_agent": "",
                    "verifier_agent": "",
                    "solo_role_gates": "standard",
                }

            def _controller_runtime_overrides(self, _metadata: dict[str, str]) -> dict[str, str]:
                return {}

            def _load_domain_profile_context(self, domain: str) -> dict[str, str]:
                return {"requested_domain": domain, "domain": domain, "status": "auto"}

            def _build_domain_packet_fields(self, domain_context: dict[str, str]) -> dict[str, str]:
                return {"domain": domain_context["domain"], "requested_domain": domain_context["requested_domain"]}

        with mock.patch.object(tool_handlers, "ModelOrchestrator", return_value=StubOrchestrator()):
            result = call_qiongli_tool(
                "qiongli_task_run",
                {
                    "task_id": "C1",
                    "paper_type": "empirical",
                    "topic": "management disclosure study",
                    "context": "Use an event study around the disclosure date.",
                    "cwd": ".",
                },
            )

        task_packet = result["structuredContent"]["data"]["task_packet"]
        preview = result["structuredContent"]["data"]["task_run_preview"]
        self.assertEqual(preview["subject_refinement"]["decision"], "borrow_lens")
        self.assertEqual(preview["effective_domain"], "auto")
        self.assertEqual(task_packet["domain"], "auto")
        self.assertEqual(task_packet["subject_refinement"]["decision"], "borrow_lens")
        self.assertEqual(task_packet["subject_refinement"]["borrowed_lenses"][0]["source_subject"], "finance")

    def test_task_run_preview_uses_suggested_subject_for_temporary_domain_context(self) -> None:
        class StubResult:
            mode = "task-plan"
            confidence = 0.8
            merged_analysis = "preview"
            recommendations: list[str] = []
            data = {
                "task_id": "C1",
                "paper_type": "empirical",
                "topic": "earnings announcement stock market reaction",
                "artifact_root": "RESEARCH/[topic]/",
                "runtime_plan": {"primary_agent": "codex"},
            }

        class StubOrchestrator:
            loaded_domain = ""

            def task_plan(self, **_kwargs: object) -> StubResult:
                return StubResult()

            def _build_controller_metadata(self, **_kwargs: object) -> dict[str, str]:
                return {
                    "execution_mode": "duo",
                    "controller": "codex",
                    "primary_agent": "",
                    "review_agent": "",
                    "verifier_agent": "",
                    "solo_role_gates": "standard",
                }

            def _controller_runtime_overrides(self, _metadata: dict[str, str]) -> dict[str, str]:
                return {}

            def _load_domain_profile_context(self, domain: str) -> dict[str, str]:
                self.loaded_domain = domain
                return {
                    "requested_domain": domain,
                    "domain": domain,
                    "status": "loaded",
                    "display_name": domain.title(),
                }

            def _build_domain_packet_fields(self, domain_context: dict[str, str]) -> dict[str, str]:
                return {
                    "domain": domain_context["domain"],
                    "requested_domain": domain_context["requested_domain"],
                    "domain_profile_status": domain_context["status"],
                }

        stub = StubOrchestrator()
        with mock.patch.object(tool_handlers, "ModelOrchestrator", return_value=stub):
            result = call_qiongli_tool(
                "qiongli_task_run",
                {
                    "task_id": "C1",
                    "paper_type": "empirical",
                    "topic": "earnings announcement stock market reaction",
                    "context": "CRSP abnormal returns event study for Journal of Finance framing.",
                    "cwd": ".",
                },
            )

        task_packet = result["structuredContent"]["data"]["task_packet"]
        preview = result["structuredContent"]["data"]["task_run_preview"]
        self.assertEqual(preview["subject_refinement"]["decision"], "suggest_subject")
        self.assertEqual(preview["effective_domain"], "finance")
        self.assertEqual(stub.loaded_domain, "finance")
        self.assertEqual(task_packet["domain"], "finance")
        self.assertEqual(task_packet["subject_refinement"]["primary_subject"], "finance")
```

- [ ] **Step 2: Run tests to verify they fail**

Run:

```bash
PYTHONPATH=packages/python-qiongli/src/qiongli:packages/python-qiongli/src uv run pytest tests/test_mcp_tool_handlers.py -q
```

Expected: FAIL because `subject_refinement` is absent from preview and task packet.

- [ ] **Step 3: Compute refinement in preview helpers**

Patch `packages/python-qiongli/src/qiongli/bridges/mcp_tool_handlers.py` imports:

```python
from bridges.subject_refinement import infer_subject_refinement
```

Add helper:

```python
def _task_run_preview_subject_refinement(
    task_run_kwargs: dict[str, Any],
    *,
    manifest_state: Any,
) -> dict[str, Any]:
    task_packet = {
        "task_id": task_run_kwargs["task_id"],
        "paper_type": task_run_kwargs["paper_type"],
        "topic": task_run_kwargs["topic"],
        "context": task_run_kwargs.get("context") or "",
        "venue": task_run_kwargs.get("venue") or "",
    }
    packet = infer_subject_refinement(
        task_packet,
        manifest_state=manifest_state,
    )
    return packet.to_packet()
```

Update `_task_run_preview()` after `project_manifest_state`:

```python
    subject_refinement = _task_run_preview_subject_refinement(
        task_run_kwargs,
        manifest_state=project_manifest_state,
    )
    project_subject = _task_run_preview_project_subject(
        task_run_kwargs,
        manifest_state=project_manifest_state,
    )
    effective_domain = _task_run_preview_effective_domain(
        task_run_kwargs,
        project_subject,
        subject_refinement=subject_refinement,
    )
```

Add returned field:

```python
        "subject_refinement": subject_refinement,
```

Update `_tool_task_run()` when building the preview task packet:

```python
            subject_refinement = preview["subject_refinement"]
            task_packet["subject_refinement"] = subject_refinement
```

Update `_task_run_preview_domain_fields()` signature and calls:

```python
def _task_run_preview_domain_fields(
    orchestrator: Any,
    task_run_kwargs: dict[str, Any],
    *,
    project_subject: dict[str, Any] | None = None,
    subject_refinement: dict[str, Any] | None = None,
) -> dict[str, Any]:
```

Call it with:

```python
                    subject_refinement=subject_refinement,
```

Update effective-domain helper:

```python
def _task_run_preview_effective_domain(
    task_run_kwargs: dict[str, Any],
    project_subject: dict[str, Any] | None,
    *,
    subject_refinement: dict[str, Any] | None = None,
) -> str:
    requested_domain = str(task_run_kwargs.get("domain") or "auto").strip() or "auto"
    if requested_domain.lower() != "auto":
        return requested_domain
    if isinstance(subject_refinement, dict):
        decision = str(subject_refinement.get("decision", ""))
        if decision in {"suggest_subject", "confirm_subject", "lock_subject"}:
            refined_domain = str(subject_refinement.get("domain", "")).strip()
            if refined_domain:
                return refined_domain
    if isinstance(project_subject, dict):
        subject_domain = str(project_subject.get("domain", "")).strip()
        if subject_domain:
            return subject_domain
    return "auto"
```

- [ ] **Step 4: Run MCP tests**

Run:

```bash
PYTHONPATH=packages/python-qiongli/src/qiongli:packages/python-qiongli/src uv run pytest tests/test_mcp_tool_handlers.py -q
```

Expected: PASS.

- [ ] **Step 5: Commit MCP preview integration**

```bash
git add packages/python-qiongli/src/qiongli/bridges/mcp_tool_handlers.py tests/test_mcp_tool_handlers.py
git commit -m "feat(mcp): expose subject refinement in task previews"
```

---

### Task 5: Integrate Refinement Into Real Orchestrator Task Runs

**Files:**
- Modify: `packages/python-qiongli/src/qiongli/bridges/orchestrator.py`
- Create: `tests/test_orchestrator_subject_refinement.py`

- [ ] **Step 1: Add a focused orchestrator test**

Create `tests/test_orchestrator_subject_refinement.py`:

```python
from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from unittest import mock

from bridges.base_bridge import BridgeResponse, CollaborationResult
from bridges.orchestrator import ModelOrchestrator


class OrchestratorSubjectRefinementTests(unittest.TestCase):
    def test_builds_subject_refinement_for_real_task_run_packet(self) -> None:
        orchestrator = ModelOrchestrator()

        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            with mock.patch.object(orchestrator, "task_plan") as task_plan:
                task_plan.return_value = CollaborationResult(
                    mode="task-plan",
                    task_description="C1 empirical earnings announcement stock market reaction",
                    confidence=0.8,
                    merged_analysis="plan",
                    recommendations=[],
                    data={
                        "functional_handoff_trace": [],
                        "functional_owner_chain": [],
                    },
                )
                with mock.patch.object(orchestrator, "_execute_runtime_agent") as execute:
                    execute.side_effect = [
                        BridgeResponse(success=True, model="codex", content="draft"),
                        BridgeResponse(success=True, model="claude", content="PASS\n\nRecommendations:\n- ok"),
                    ]
                    result = orchestrator.task_run(
                        task_id="C1",
                        paper_type="empirical",
                        topic="earnings announcement stock market reaction",
                        context="CRSP abnormal returns event study for Journal of Finance framing.",
                        cwd=root,
                        skip_validation=True,
                        max_revision_rounds=0,
                    )

        packet = result.data["task_packet"]
        self.assertEqual(packet["subject_refinement"]["decision"], "suggest_subject")
        self.assertEqual(packet["subject_refinement"]["primary_subject"], "finance")
        self.assertEqual(packet["domain"], "finance")
```

- [ ] **Step 2: Run the focused test to verify it fails**

Run:

```bash
PYTHONPATH=packages/python-qiongli/src/qiongli:packages/python-qiongli/src uv run pytest tests/test_orchestrator_subject_refinement.py -q
```

Expected: FAIL because `task_packet["subject_refinement"]` is absent.

- [ ] **Step 3: Add refinement to orchestrator task packets**

Patch `packages/python-qiongli/src/qiongli/bridges/orchestrator.py` imports:

```python
from .subject_refinement import infer_subject_refinement
```

In `task_run()`, replace the current project subject/effective domain block with:

```python
        manifest_state = (
            implicit_project_manifest_state(cwd)
            if guidance_state.mode == "off"
            else load_project_manifest(cwd)
        )
        subject_refinement = infer_subject_refinement(
            {
                **packet,
                "context": context or "",
                "venue": venue or "",
            },
            manifest_state=manifest_state,
            standards_dir=self.standards_dir,
        )
        subject_refinement_packet = subject_refinement.to_packet()
        project_subject = resolve_project_subject(manifest_state, requested_domain=domain)
        requested_domain = str(domain or "auto").strip().lower()
        effective_domain = (
            domain
            if requested_domain != "auto"
            else (
                subject_refinement_packet.get("domain")
                if subject_refinement_packet.get("decision")
                in {"suggest_subject", "confirm_subject", "lock_subject"}
                else project_subject.domain
            )
        )
        domain_context = self._load_domain_profile_context(str(effective_domain or "auto"))
        packet["local_guidance"] = guidance_state.to_packet()
        packet.update(self._build_domain_packet_fields(domain_context))
        packet["project_subject"] = project_subject.to_packet()
        packet["subject_refinement"] = subject_refinement_packet
        packet["domain"] = (
            str(domain_context.get("domain", effective_domain or "auto")).strip()
            or "auto"
        )
        routing_notes.append(project_subject.summary)
        routing_notes.append(subject_refinement_packet["summary"])
```

Add refinement context to draft and review prompt construction in both prompt builders:

```python
        subject_refinement = task_packet.get("subject_refinement", {})
        subject_refinement_section = ""
        if isinstance(subject_refinement, dict) and subject_refinement:
            subject_refinement_section = (
                "\nRuntime subject refinement:\n"
                + str(subject_refinement.get("summary", "")).strip()
                + "\n"
                + "Decision: "
                + str(subject_refinement.get("decision", ""))
                + "; loaded_resources: "
                + str(subject_refinement.get("loaded_resources", {}))
                + "\n"
            )
```

Insert `{subject_refinement_section}` immediately after `{project_subject_section}` in both prompt templates.

- [ ] **Step 4: Run the focused orchestrator test**

Run:

```bash
PYTHONPATH=packages/python-qiongli/src/qiongli:packages/python-qiongli/src uv run pytest tests/test_orchestrator_subject_refinement.py -q
```

Expected: PASS.

- [ ] **Step 5: Run the broader MCP/orchestrator-related tests**

Run:

```bash
PYTHONPATH=packages/python-qiongli/src/qiongli:packages/python-qiongli/src uv run pytest tests/test_mcp_tool_handlers.py tests/test_mcp_stdio_server.py tests/test_orchestrator_subject_refinement.py -q
```

Expected: PASS.

- [ ] **Step 6: Commit orchestrator integration**

```bash
git add packages/python-qiongli/src/qiongli/bridges/orchestrator.py tests/test_orchestrator_subject_refinement.py
git commit -m "feat(orchestrator): route task runs through subject refinement"
```

---

### Task 6: Make Guidance Proposals Refinement-Aware

**Files:**
- Modify: `packages/python-qiongli/src/qiongli/bridges/guidance_runtime.py`
- Modify: `tests/test_guidance_runtime.py`

- [ ] **Step 1: Add failing guidance tests**

Append these tests to `tests/test_guidance_runtime.py`:

```python
    def test_guidance_trace_writes_subject_refinement_packet(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            init_project_guidance(root)
            state = effective_guidance(root, mode="propose", run_id="subject-refinement-run")

            trace = write_guidance_trace(
                project_root=root,
                guidance_state=state,
                task_packet={
                    "task_id": "C1",
                    "paper_type": "empirical",
                    "topic": "management disclosure study",
                    "context": "Use an event study around disclosure date.",
                },
                draft_content="",
                review_content="",
                merged_analysis="",
                validator_gate={"passed": True, "found": [], "missing": [], "checked": 0},
                applied=False,
            )

            run_dir = root / ".qiongli" / "trace" / "runs" / "subject-refinement-run"
            packet = json.loads((run_dir / "subject_refinement.json").read_text(encoding="utf-8"))

        self.assertEqual(packet["decision"], "borrow_lens")
        self.assertEqual(trace["subject_refinement"]["decision"], "borrow_lens")
        self.assertEqual(trace["subject_refinement"]["borrowed_lenses"][0]["source_subject"], "finance")

    def test_borrowed_lens_guidance_proposal_does_not_write_manifest_subject(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            init_project_guidance(root)
            state = effective_guidance(root, mode="propose", run_id="borrowed-lens-run")

            write_guidance_trace(
                project_root=root,
                guidance_state=state,
                task_packet={
                    "task_id": "C1",
                    "paper_type": "empirical",
                    "topic": "management disclosure study",
                    "context": "Use an event study around disclosure date.",
                },
                draft_content="",
                review_content="",
                merged_analysis="",
                validator_gate={"passed": True, "found": [], "missing": [], "checked": 0},
                applied=False,
            )

            text = (
                root
                / ".qiongli"
                / "trace"
                / "runs"
                / "borrowed-lens-run"
                / "guidance_update_proposal.md"
            ).read_text(encoding="utf-8")

        self.assertIn("## Subject Refinement Decision", text)
        self.assertIn("borrow_lens", text)
        self.assertIn("No structured manifest change proposed.", text)
        self.assertNotIn("active_subject: finance", text)

    def test_suggested_subject_guidance_proposal_includes_subject_mode(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            init_project_guidance(root)
            state = effective_guidance(root, mode="propose", run_id="suggest-finance-run")

            write_guidance_trace(
                project_root=root,
                guidance_state=state,
                task_packet={
                    "task_id": "C1",
                    "paper_type": "empirical",
                    "topic": "earnings announcement stock market reaction",
                    "context": "CRSP abnormal returns event study for Journal of Finance framing.",
                },
                draft_content="Estimate cumulative abnormal returns.",
                review_content="",
                merged_analysis="",
                validator_gate={"passed": True, "found": [], "missing": [], "checked": 0},
                applied=False,
            )

            text = (
                root
                / ".qiongli"
                / "trace"
                / "runs"
                / "suggest-finance-run"
                / "guidance_update_proposal.md"
            ).read_text(encoding="utf-8")

        self.assertIn("active_subject: finance", text)
        self.assertIn("subject_mode: suggested", text)
        self.assertIn("method_lenses:", text)
        self.assertIn("event-study", text)
```

- [ ] **Step 2: Run guidance tests to verify they fail**

Run:

```bash
PYTHONPATH=packages/python-qiongli/src/qiongli:packages/python-qiongli/src uv run pytest tests/test_guidance_runtime.py -q
```

Expected: FAIL because `subject_refinement.json` is missing and proposal text uses legacy manifest suggestion only.

- [ ] **Step 3: Update guidance trace writing**

Patch imports in `guidance_runtime.py`:

```python
from .subject_refinement import infer_subject_refinement, legacy_manifest_suggestion
```

Add `subject_mode` to `MANIFEST_PROPOSAL_FIELDS`:

```python
MANIFEST_PROPOSAL_FIELDS = {
    "active_subject",
    "subject_mode",
    "secondary_subjects",
    "venue_profiles",
    "method_lenses",
    "strictness",
}
```

In `write_guidance_trace()`, replace legacy suggestion creation with:

```python
    manifest_state = load_project_manifest(paths.project_root)
    subject_refinement = infer_subject_refinement(
        task_packet,
        manifest_state=manifest_state,
        draft_content=draft_content,
        review_content=review_content,
        merged_analysis=merged_analysis,
    )
    subject_refinement_packet = subject_refinement.to_packet()
    _write_json(run_dir / "subject_refinement.json", subject_refinement_packet)
    suggestion = legacy_manifest_suggestion(subject_refinement)
    (run_dir / "guidance_update_proposal.md").write_text(
        _proposal_text(
            task_packet,
            validator_gate,
            applied,
            suggestion,
            subject_refinement=subject_refinement_packet,
        ),
        encoding="utf-8",
    )
```

Add to index record:

```python
        "subject_refinement": subject_refinement_packet,
```

Update `_proposal_text()` signature and body:

```python
def _proposal_text(
    task_packet: dict[str, Any],
    validator_gate: dict[str, Any],
    applied: bool,
    manifest_suggestion: dict[str, Any],
    *,
    subject_refinement: dict[str, Any] | None = None,
) -> str:
```

Insert before manifest proposal:

```python
    if subject_refinement:
        lines.extend(_subject_refinement_section(subject_refinement))
```

Add section renderer:

```python
def _subject_refinement_section(subject_refinement: dict[str, Any]) -> list[str]:
    borrowed = list(subject_refinement.get("borrowed_lenses", []) or [])
    lines = [
        "",
        "## Subject Refinement Decision",
        "",
        f"- decision: `{subject_refinement.get('decision', '')}`",
        f"- mode: `{subject_refinement.get('mode', '')}`",
        f"- active_subject: `{subject_refinement.get('active_subject', '')}`",
        f"- primary_subject: `{subject_refinement.get('primary_subject', None)}`",
        f"- summary: {subject_refinement.get('summary', '')}",
    ]
    if borrowed:
        lines.append("- borrowed_lenses:")
        for item in borrowed:
            if not isinstance(item, Mapping):
                continue
            lines.append(
                "  - "
                + str(item.get("source_subject", ""))
                + ":"
                + str(item.get("lens", ""))
                + " ("
                + str(item.get("resource_level", ""))
                + ")"
            )
    return lines
```

Update `_manifest_proposal_section()`:

```python
def _manifest_proposal_section(manifest_suggestion: dict[str, Any]) -> list[str]:
    subject_refinement = manifest_suggestion.get("subject_refinement", {})
    decision = (
        str(subject_refinement.get("decision", ""))
        if isinstance(subject_refinement, Mapping)
        else ""
    )
    active_subject = str(manifest_suggestion.get("active_subject", "auto"))
    subject_mode = str(manifest_suggestion.get("subject_mode", "auto"))
    method_lenses = [str(item) for item in list(manifest_suggestion.get("method_lenses", []) or [])]
    confidence = float(manifest_suggestion.get("confidence", 0.0) or 0.0)
    evidence = [str(item) for item in list(manifest_suggestion.get("evidence", []) or [])]
    lines = ["", "## Proposed Manifest Changes", ""]
    if active_subject == "auto" or decision == "borrow_lens" or confidence < 0.6:
        lines.append("No structured manifest change proposed.")
    else:
        lines.extend(["```yaml", f"active_subject: {active_subject}", f"subject_mode: {subject_mode}"])
        if method_lenses:
            lines.append("method_lenses:")
            lines.extend(f"  - {method}" for method in method_lenses)
        lines.append("```")
    lines.extend(["", "## Manifest Evidence", "", f"- confidence: {confidence:g}"])
    if evidence:
        lines.extend(f"- evidence: {item}" for item in evidence)
    else:
        lines.append("- evidence: none")
    return lines
```

- [ ] **Step 4: Run guidance tests**

Run:

```bash
PYTHONPATH=packages/python-qiongli/src/qiongli:packages/python-qiongli/src uv run pytest tests/test_guidance_runtime.py -q
```

Expected: PASS.

- [ ] **Step 5: Commit guidance integration**

```bash
git add packages/python-qiongli/src/qiongli/bridges/guidance_runtime.py tests/test_guidance_runtime.py
git commit -m "feat(guidance): persist subject refinement proposals"
```

---

### Task 7: Ship Adaptive Resources In Materialized Packages

**Files:**
- Modify: `packages/python-qiongli/src/qiongli/subject_materializer.py`
- Modify: `packages/python-qiongli/src/qiongli/local_plugin_installer.py`
- Modify: `tests/test_subject_materializer.py`
- Modify: `tests/test_local_plugin_installer.py`

- [ ] **Step 1: Add failing materializer tests**

Append to `tests/test_subject_materializer.py`:

```python
    def test_core_full_package_contains_adaptive_subject_resources(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            out = Path(tmp_dir) / "qiongli-workflow"

            materialize_subject_package(
                MaterializeOptions(
                    source=REPO_ROOT,
                    out=out,
                    subject="core",
                    flavor="full",
                    coverage="complete",
                )
            )

            manifest = json.loads((out / "SUBJECT_MANIFEST.json").read_text(encoding="utf-8"))
            self.assertEqual(manifest["subject"], "core")
            self.assertTrue(manifest["adaptive_subject_refinement"]["enabled"])
            self.assertEqual(
                manifest["adaptive_subject_refinement"]["contract"],
                "standards/subject-refinement-contract.yaml",
            )
            self.assertTrue((out / "standards" / "subject-refinement-contract.yaml").exists())
            self.assertTrue((out / "subjects" / "catalog.yaml").exists())
            self.assertTrue((out / "subjects" / "finance" / "skills" / "finance-identification-risk-auditor.md").exists())
            self.assertTrue((out / "subjects" / "economics" / "skills" / "econ-identification-auditor.md").exists())

    def test_core_desktop_package_contains_compact_refinement_index_under_budget(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            out = Path(tmp_dir) / "qiongli"

            materialize_subject_package(
                MaterializeOptions(source=REPO_ROOT, out=out, subject="core", flavor="desktop")
            )

            files = [path for path in out.rglob("*") if path.is_file()]
            self.assertLessEqual(len(files), 180)
            self.assertTrue((out / "standards" / "subject-refinement-contract.yaml").exists())
            self.assertTrue((out / "subjects" / "refinement-index.yaml").exists())
            self.assertFalse((out / "subjects" / "finance" / "skills" / "finance-identification-risk-auditor.md").exists())
```

Append to `tests/test_local_plugin_installer.py`:

```python
    def test_install_codex_plugin_contains_adaptive_subject_resources(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            marketplace = root / "agents" / "marketplace.json"

            install_local_plugin(
                LocalPluginOptions(
                    repo_root=REPO_ROOT,
                    target="codex",
                    codex_marketplace_path=marketplace,
                )
            )

            skill_root = marketplace.parent / "plugins" / "qiongli" / "skills" / "qiongli-workflow"
            manifest = self._read_json(skill_root / "SUBJECT_MANIFEST.json")

        self.assertTrue(manifest["adaptive_subject_refinement"]["enabled"])
        self.assertTrue((skill_root / "standards" / "subject-refinement-contract.yaml").exists())
        self.assertTrue((skill_root / "subjects" / "catalog.yaml").exists())
```

- [ ] **Step 2: Run package tests to verify they fail**

Run:

```bash
PYTHONPATH=packages/python-qiongli/src/qiongli:packages/python-qiongli/src uv run pytest tests/test_subject_materializer.py tests/test_local_plugin_installer.py -q
```

Expected: FAIL because `subjects/` refinement resources are not written into materialized output.

- [ ] **Step 3: Materialize adaptive subject resources**

Patch `subject_materializer.py` after `_materialize_skills(...)` call in `materialize_subject_package()`:

```python
    _materialize_refinement_resources(
        source=source,
        out=out,
        subject=subject,
        flavor=options.flavor,
    )
```

Add helper functions:

```python
def _materialize_refinement_resources(
    *,
    source: Path,
    out: Path,
    subject: SubjectDefinition,
    flavor: str,
) -> None:
    layout = RepoLayout(source)
    subjects_root = layout.subjects
    if not subjects_root.exists():
        return
    dest_root = out / "subjects"
    if flavor == "desktop" and subject.id == "core":
        dest_root.mkdir(parents=True, exist_ok=True)
        _write_compact_refinement_index(subjects_root, dest_root / "refinement-index.yaml")
        return
    _copy_path(subjects_root, dest_root)


def _write_compact_refinement_index(subjects_root: Path, dest: Path) -> None:
    catalog_path = subjects_root / "catalog.yaml"
    payload = yaml.safe_load(catalog_path.read_text(encoding="utf-8")) if catalog_path.is_file() else {}
    subjects = payload.get("subjects", {}) if isinstance(payload, dict) else {}
    index: dict[str, Any] = {"subjects": {}}
    if isinstance(subjects, dict):
        for subject_id, raw_subject in subjects.items():
            if not isinstance(raw_subject, dict):
                continue
            index["subjects"][subject_id] = {
                "display_name": raw_subject.get("display_name", subject_id),
                "domain_profiles": raw_subject.get("domain_profiles", []),
                "venue_profiles": raw_subject.get("venue_profiles", []),
                "subject_specific_skill_refs": raw_subject.get("subject_specific_skill_refs", []),
                "skill_overrides": raw_subject.get("skill_overrides", []),
            }
    dest.parent.mkdir(parents=True, exist_ok=True)
    dest.write_text(yaml.safe_dump(index, sort_keys=True, allow_unicode=True), encoding="utf-8")
```

Patch `_write_subject_markers()` manifest payload:

```python
                "adaptive_subject_refinement": {
                    "enabled": subject.id == "core",
                    "default_mode": "auto",
                    "contract": "standards/subject-refinement-contract.yaml",
                    "resource_index": (
                        "subjects/refinement-index.yaml"
                        if flavor == "desktop" and subject.id == "core"
                        else "subjects/catalog.yaml"
                    ),
                },
```

- [ ] **Step 4: Run materializer and plugin tests**

Run:

```bash
PYTHONPATH=packages/python-qiongli/src/qiongli:packages/python-qiongli/src uv run pytest tests/test_subject_materializer.py tests/test_local_plugin_installer.py -q
```

Expected: PASS.

- [ ] **Step 5: Commit package resource changes**

```bash
git add packages/python-qiongli/src/qiongli/subject_materializer.py packages/python-qiongli/src/qiongli/local_plugin_installer.py tests/test_subject_materializer.py tests/test_local_plugin_installer.py
git commit -m "feat(install): ship adaptive subject resources"
```

---

### Task 8: Update CLI And Installer Semantics

**Files:**
- Modify: `packages/python-qiongli/src/qiongli/universal_installer.py`
- Modify: `packages/python-qiongli/src/qiongli/cli.py`
- Modify: `packages/python-qiongli/src/qiongli/bridges/mcp_cli.py`
- Modify: `tests/test_universal_installer.py`

- [ ] **Step 1: Add installer output test**

Append to `tests/test_universal_installer.py`:

```python
    def test_default_install_describes_adaptive_core_subject(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            temp_root = Path(tmp_dir)
            project_dir = temp_root / "project"
            project_dir.mkdir(parents=True)
            env = _isolated_qiongli_env(temp_root)
            env["PATH"] = ""

            stdout = io.StringIO()
            with mock.patch.dict(os.environ, env, clear=True):
                with contextlib.redirect_stdout(stdout):
                    result = install(
                        InstallOptions(
                            repo_root=REPO_ROOT,
                            project_dir=project_dir,
                            target="codex",
                            profile="partial",
                            dry_run=True,
                        )
                    )

        self.assertEqual(result, 0)
        rendered = stdout.getvalue()
        self.assertIn("subject: core (adaptive; active_subject defaults to auto)", rendered)
```

- [ ] **Step 2: Run installer test to verify it fails**

Run:

```bash
PYTHONPATH=packages/python-qiongli/src/qiongli:packages/python-qiongli/src uv run pytest tests/test_universal_installer.py::UniversalInstallerTests::test_default_install_describes_adaptive_core_subject -q
```

Expected: FAIL because the installer prints `subject: core`.

- [ ] **Step 3: Update installer output and help text**

Patch `universal_installer.py` print block:

```python
    subject_label = (
        "core (adaptive; active_subject defaults to auto)"
        if options.subject == "core"
        else f"{options.subject} (advanced override)"
    )
    print(f"  subject: {subject_label}")
```

Patch `cli.py` install parser help:

```python
    install_parser.add_argument(
        "--subject",
        default="core",
        help=(
            "Advanced override for pre-materialized subject packages. "
            "Default core installs adaptive runtime subject refinement."
        ),
    )
```

Patch `cli.py` upgrade parser help:

```python
    upgrade.add_argument(
        "--subject",
        default="core",
        help=(
            "Advanced override for pre-materialized subject packages. "
            "Default core keeps runtime subject refinement adaptive."
        ),
    )
```

Patch `bridges/mcp_cli.py` upgrade/install help text with the same wording.

- [ ] **Step 4: Run installer tests**

Run:

```bash
PYTHONPATH=packages/python-qiongli/src/qiongli:packages/python-qiongli/src uv run pytest tests/test_universal_installer.py tests/test_mcp_cli.py -q
```

Expected: PASS.

- [ ] **Step 5: Commit CLI semantics**

```bash
git add packages/python-qiongli/src/qiongli/universal_installer.py packages/python-qiongli/src/qiongli/cli.py packages/python-qiongli/src/qiongli/bridges/mcp_cli.py tests/test_universal_installer.py
git commit -m "docs(cli): clarify adaptive subject install semantics"
```

---

### Task 9: Update Skill-Level Runtime Guidance

**Files:**
- Modify: `content/workflow/SKILL.md`
- Modify: `content/skills-core.md`
- Modify: `packages/python-qiongli/src/qiongli/subject_materializer.py`
- Modify: `tests/test_subject_materializer.py`

- [ ] **Step 1: Add failing materialized skill text test**

Append to `tests/test_subject_materializer.py`:

```python
    def test_core_skill_entrypoint_explains_runtime_subject_refinement(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            out = Path(tmp_dir) / "qiongli-workflow"

            materialize_subject_package(
                MaterializeOptions(source=REPO_ROOT, out=out, subject="core", flavor="full")
            )

            text = (out / "SKILL.md").read_text(encoding="utf-8")

        self.assertIn("## Runtime Subject Refinement", text)
        self.assertIn("borrowed method lens", text)
        self.assertIn("Do not switch the whole project subject from a single method signal.", text)
```

- [ ] **Step 2: Run materializer test to verify it fails**

Run:

```bash
PYTHONPATH=packages/python-qiongli/src/qiongli:packages/python-qiongli/src uv run pytest tests/test_subject_materializer.py::SubjectMaterializerTests::test_core_skill_entrypoint_explains_runtime_subject_refinement -q
```

Expected: FAIL because the generated entrypoint does not include runtime subject refinement guidance.

- [ ] **Step 3: Update workflow and core skill docs**

Patch `content/workflow/SKILL.md` local guidance section to include:

```markdown
### Runtime Subject Refinement

Qiongli installs as an adaptive core workflow. Start from `active_subject: auto`
unless `.qiongli/guidance_manifest.yaml` says otherwise. During a task, infer
whether the request needs core-only guidance, a borrowed method lens, a suggested
subject, a confirmed subject, or a locked subject.

Do not switch the whole project subject from a single method signal. A management
paper that uses an event study borrows finance event-study diagnostics; it is not
automatically a finance paper. A political science paper that uses DID borrows
economics identification diagnostics; it is not automatically an economics paper.

Use `subject_refinement.borrowed_lenses` as temporary method guidance. Use
`subject_refinement.primary_subject` as the temporary subject only when the
decision is `suggest_subject`, `confirm_subject`, or `lock_subject`. Persist
changes only through project-local guidance proposals or an explicit
`subject_mode: confirmed` or `subject_mode: locked` manifest.
```

Patch `content/skills-core.md` domain profile area to include:

```markdown
**Runtime Subject Refinement:** The default installed package is adaptive core.
Use `standards/subject-refinement-contract.yaml` to distinguish core-only work,
borrowed method lenses, suggested subjects, confirmed subjects, and locked
subjects. Borrowed lenses load the narrow audited method pack without changing
`active_subject`.
```

- [ ] **Step 4: Update generated `SKILL.md` renderer**

Patch `_render_skill_md()` in `subject_materializer.py` before `## Skill Loading Strategy`:

```python
            "## Runtime Subject Refinement",
            "",
            "- Qiongli installs as an adaptive core workflow with `active_subject: auto` unless project guidance says otherwise.",
            "- Use `standards/subject-refinement-contract.yaml` to classify no-subject, borrowed-lens, suggested, confirmed, and locked subject states.",
            "- Treat a borrowed method lens as temporary method guidance. Do not switch the whole project subject from a single method signal.",
            "- Use `subject_refinement.primary_subject` as the temporary subject only for `suggest_subject`, `confirm_subject`, or `lock_subject` decisions.",
            "- Persist subject changes only through project-local guidance proposals, `subject_mode: confirmed`, or `subject_mode: locked`.",
            "",
            "## Skill Loading Strategy",
```

- [ ] **Step 5: Run materializer tests**

Run:

```bash
PYTHONPATH=packages/python-qiongli/src/qiongli:packages/python-qiongli/src uv run pytest tests/test_subject_materializer.py -q
```

Expected: PASS.

- [ ] **Step 6: Commit skill guidance changes**

```bash
git add content/workflow/SKILL.md content/skills-core.md packages/python-qiongli/src/qiongli/subject_materializer.py tests/test_subject_materializer.py
git commit -m "docs(skills): document runtime subject refinement"
```

---

### Task 10: Run Integrated Verification

**Files:**
- No new source files.

- [ ] **Step 1: Run focused subject/runtime suite**

Run:

```bash
PYTHONPATH=packages/python-qiongli/src/qiongli:packages/python-qiongli/src uv run pytest tests/test_subject_refinement.py tests/test_project_manifest.py tests/test_project_inference.py tests/test_guidance_runtime.py tests/test_mcp_tool_handlers.py tests/test_subject_materializer.py tests/test_local_plugin_installer.py tests/test_universal_installer.py -q
```

Expected: PASS.

- [ ] **Step 2: Run package validation suite**

Run:

```bash
PYTHONPATH=packages/python-qiongli/src/qiongli:packages/python-qiongli/src uv run pytest tests/test_subject_catalog.py tests/test_subject_specialization_audit.py tests/test_validate_project_artifacts.py -q
```

Expected: PASS.

- [ ] **Step 3: Run full Python test suite**

Run:

```bash
PYTHONPATH=packages/python-qiongli/src/qiongli:packages/python-qiongli/src uv run pytest -q
```

Expected: PASS.

- [ ] **Step 4: Run syntax and whitespace checks**

Run:

```bash
git diff --check
```

Expected: no output.

- [ ] **Step 5: Scan for plan-forbidden placeholders in changed implementation files**

Run:

```bash
rg -n "T[B]D|T[O]DO|implement lat[e]r|fill in detail[s]|add appropriat[e]|handle edge case[s]|Similar to Tas[k]" content packages tests docs/superpowers/plans/2026-06-30-runtime-subject-refinement.md
```

Expected: no matches in changed files. Existing unrelated matches outside changed files should be reviewed and ignored only when they predate this branch.

- [ ] **Step 6: Review repository boundary**

Run:

```bash
git diff --name-only HEAD~9..HEAD
```

Expected changed paths stay inside this repository's source, tests, content, and docs:

- `content/standards/subject-refinement-contract.yaml`
- `content/workflow/SKILL.md`
- `content/skills-core.md`
- `packages/python-qiongli/src/qiongli/...`
- `tests/...`
- `docs/superpowers/plans/...`

No marketplace catalog generated files, local absolute paths, secrets, copied third-party papers, or client cache files should appear.

- [ ] **Step 7: Commit final verification notes if any docs were adjusted during verification**

If verification required doc-only corrections, commit them:

```bash
git add content docs tests packages
git commit -m "docs(subjects): align refinement verification notes"
```

If no files changed, skip this commit.

---

## Acceptance Mapping

- Full installs default to adaptive core with `active_subject: auto`: Tasks 7 and 8.
- Runtime packets expose candidates, confidence, evidence, method lenses, borrowed lenses, loaded resources, and persistence status: Tasks 1, 3, 4, 5, and 6.
- Borderline projects borrow method-level audited resources without switching subject: Tasks 1, 3, 4, 6, and 9.
- Explicit or repeated subject evidence can promote to confirmed guidance: Tasks 2, 3, and 6.
- Locked user subjects are respected: Tasks 1, 2, and 3.
- CLI, marketplace, and client-native installs share the same packet contract: Tasks 4, 5, 7, 8, and 9.

## Handoff Notes

- Use subagent-driven execution by task. Each task has an isolated test target and a commit boundary.
- Start implementation from `dev` in a fresh worktree.
- Keep the old `project_subject`, `domain`, and manifest fields in all packets until a later major version removes compatibility shims.
- Do not make subject selection an installation prompt. `--subject` remains an advanced override for pre-materialized packages.
- Treat desktop/client-native file limits as package constraints only; they must not change the runtime packet contract.
