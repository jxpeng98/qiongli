# Worker Orchestration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a platform-neutral worker orchestration contract and explicit `task-run` execution path that can later map to Codex subagents or Claude cowork while defaulting to a generic prompt adapter.

**Architecture:** Add canonical worker contracts and templates under `content/`, wire sparse task config through `mcp-agent-capability-map.yaml`, then implement explicit `task-run` flags and mocked generic worker execution in the Python orchestrator. Keep worker mode disabled by default and preserve current `task-run` behavior unless the user opts in.

**Tech Stack:** Python 3.12, `unittest`, `yaml`, JSON templates, existing `ModelOrchestrator`, existing `BridgeResponse` / `CollaborationResult` types.

---

## File Structure

- Create `content/standards/worker-orchestration-contract.yaml`: canonical worker orchestration enums and required fields.
- Create `content/templates/worker-run-packet.json`: safe default worker packet shape.
- Create `content/templates/worker-review-packet.md`: reviewer-facing worker review template.
- Create `content/templates/worker-merge-report.md`: merge report template for controller synthesis.
- Create `tests/test_worker_orchestration_contract.py`: contract, template, and capability-map validation tests.
- Create `tests/test_worker_orchestration_runtime.py`: mocked orchestrator tests for CLI flags, worker plan construction, generic execution, barrier behavior, and disabled-by-default behavior.
- Modify `content/standards/mcp-agent-capability-map.yaml`: add sparse `worker_orchestration_config` for `B1` and `H3`.
- Modify `packages/python-qiongli/src/qiongli/bridges/orchestrator.py`: add worker choices, parser args, `task_run` parameters, worker helper methods, generic adapter execution, and result rendering.
- Modify `docs/guide/multi-agent.md`: document worker orchestration below runtime orchestration.
- Modify `docs/advanced/controller-modes.md`: clarify runtime accountability versus worker delegation.
- Modify `docs/advanced/agent-skill-collaboration.md`: update the standard chain with worker-plan steps.
- Modify `content/workflow/references/platform-routing.md`: document adapter mapping and generic fallback.

## Task 1: Add Worker Contract And Templates

**Files:**
- Create: `tests/test_worker_orchestration_contract.py`
- Create: `content/standards/worker-orchestration-contract.yaml`
- Create: `content/templates/worker-run-packet.json`
- Create: `content/templates/worker-review-packet.md`
- Create: `content/templates/worker-merge-report.md`

- [ ] **Step 1: Write failing contract and template tests**

Create `tests/test_worker_orchestration_contract.py` with:

```python
from __future__ import annotations

import json
import unittest
from pathlib import Path

import yaml

from qiongli.source_layout import RepoLayout


REPO_ROOT = Path(__file__).resolve().parents[1]
LAYOUT = RepoLayout(REPO_ROOT)

CONTRACT_PATH = LAYOUT.standards / "worker-orchestration-contract.yaml"
RUN_PACKET_PATH = LAYOUT.templates / "worker-run-packet.json"
REVIEW_PACKET_PATH = LAYOUT.templates / "worker-review-packet.md"
MERGE_REPORT_PATH = LAYOUT.templates / "worker-merge-report.md"
CAPABILITY_MAP_PATH = LAYOUT.standards / "mcp-agent-capability-map.yaml"

ORCHESTRATION_MODES = {"none", "delegated_workers", "review_swarm"}
PLATFORM_ADAPTERS = {"generic_prompt", "codex_subagent", "claude_cowork"}
WORKER_STATUSES = {"planned", "running", "passed", "failed", "blocked", "skipped"}
MERGE_POLICIES = {
    "synthesize_with_conflict_matrix",
    "consensus_then_gaps",
    "controller_adjudication",
}

REQUIRED_WORKER_PLAN_FIELDS = {
    "orchestration_mode",
    "controller_runtime",
    "platform_adapter",
    "task_id",
    "paper_type",
    "topic",
    "workers",
    "merge",
    "final_review",
}

REQUIRED_WORKER_FIELDS = {
    "id",
    "goal",
    "functional_role",
    "required_skills",
    "allowed_artifacts",
    "forbidden_artifacts",
    "review_required",
    "stop_conditions",
}

REQUIRED_MERGE_FIELDS = {
    "agent",
    "policy",
    "output_artifacts",
}

REQUIRED_FINAL_REVIEW_FIELDS = {
    "reviewer",
    "gate",
}

REQUIRED_RUN_PACKET_FIELDS = {
    "run_id",
    "worker_id",
    "controller_runtime",
    "platform_adapter",
    "task_id",
    "paper_type",
    "topic",
    "goal",
    "functional_role",
    "required_skills",
    "required_mcp",
    "allowed_artifacts",
    "forbidden_artifacts",
    "artifacts_read",
    "artifacts_written",
    "warnings",
    "blocking_issues",
    "status",
    "confidence",
}


class WorkerOrchestrationContractTests(unittest.TestCase):
    def test_contract_defines_enums_and_required_fields(self) -> None:
        self.assertTrue(CONTRACT_PATH.exists(), f"Missing {CONTRACT_PATH}")
        contract = yaml.safe_load(CONTRACT_PATH.read_text(encoding="utf-8")) or {}

        self.assertEqual("1.0.0", contract.get("contract_version"))
        self.assertEqual(ORCHESTRATION_MODES, set(contract.get("orchestration_modes", [])))
        self.assertEqual(PLATFORM_ADAPTERS, set(contract.get("platform_adapters", [])))
        self.assertEqual(WORKER_STATUSES, set(contract.get("worker_statuses", [])))
        self.assertEqual(MERGE_POLICIES, set(contract.get("merge_policies", [])))
        self.assertEqual(REQUIRED_WORKER_PLAN_FIELDS, set(contract.get("required_worker_plan_fields", [])))
        self.assertEqual(REQUIRED_WORKER_FIELDS, set(contract.get("required_worker_fields", [])))
        self.assertEqual(REQUIRED_MERGE_FIELDS, set(contract.get("required_merge_fields", [])))
        self.assertEqual(REQUIRED_FINAL_REVIEW_FIELDS, set(contract.get("required_final_review_fields", [])))

    def test_worker_run_packet_has_safe_defaults(self) -> None:
        self.assertTrue(RUN_PACKET_PATH.exists(), f"Missing {RUN_PACKET_PATH}")
        packet = json.loads(RUN_PACKET_PATH.read_text(encoding="utf-8"))

        self.assertEqual(REQUIRED_RUN_PACKET_FIELDS, set(packet))
        self.assertEqual("planned", packet["status"])
        self.assertEqual("generic_prompt", packet["platform_adapter"])
        self.assertEqual(0.0, packet["confidence"])
        for key in ("required_skills", "required_mcp", "allowed_artifacts", "forbidden_artifacts", "artifacts_read", "artifacts_written", "warnings", "blocking_issues"):
            self.assertEqual([], packet[key], key)

    def test_worker_review_template_has_required_headings(self) -> None:
        self.assertTrue(REVIEW_PACKET_PATH.exists(), f"Missing {REVIEW_PACKET_PATH}")
        text = REVIEW_PACKET_PATH.read_text(encoding="utf-8")
        for heading in (
            "# Worker Review Packet",
            "## Review Metadata",
            "## Findings",
            "## Blocking Issues",
            "## Required Revisions",
            "## Verification Evidence",
            "## Verdict",
        ):
            with self.subTest(heading=heading):
                self.assertIn(heading, text)

    def test_worker_merge_template_has_required_headings(self) -> None:
        self.assertTrue(MERGE_REPORT_PATH.exists(), f"Missing {MERGE_REPORT_PATH}")
        text = MERGE_REPORT_PATH.read_text(encoding="utf-8")
        for heading in (
            "# Worker Merge Report",
            "## Worker Status Table",
            "## Accepted Worker Outputs",
            "## Rejected Or Blocked Worker Outputs",
            "## Conflict Summary",
            "## Gap Summary",
            "## Controller Adjudication",
            "## Canonical Output Update Plan",
            "## Final Review Request",
        ):
            with self.subTest(heading=heading):
                self.assertIn(heading, text)

    def test_worker_orchestration_config_references_valid_contract_values(self) -> None:
        capability_map = yaml.safe_load(CAPABILITY_MAP_PATH.read_text(encoding="utf-8")) or {}
        config = capability_map.get("worker_orchestration_config", {})
        self.assertEqual({"B1", "H3"}, set(config))

        for task_id, block in config.items():
            with self.subTest(task_id=task_id):
                self.assertIn(block["default_mode"], ORCHESTRATION_MODES)
                self.assertIn(block["merge_policy"], MERGE_POLICIES)
                self.assertGreaterEqual(int(block["max_workers"]), 1)
                self.assertIn(block["barrier_rules"]["on_failure"], {"degrade", "block", "retry"})
                self.assertGreaterEqual(float(block["barrier_rules"]["min_success_ratio"]), 0.0)
                self.assertLessEqual(float(block["barrier_rules"]["min_success_ratio"]), 1.0)
                for adapter in block["adapter_preference"].values():
                    self.assertIn(adapter, PLATFORM_ADAPTERS)
                for worker in block["worker_pool"]:
                    self.assertTrue(str(worker).strip())


if __name__ == "__main__":
    unittest.main()
```

- [ ] **Step 2: Run the tests and verify they fail**

Run:

```bash
python3 -m unittest tests.test_worker_orchestration_contract -v
```

Expected: fail because the worker contract, templates, and capability-map config do not exist.

- [ ] **Step 3: Add the worker contract**

Create `content/standards/worker-orchestration-contract.yaml`:

```yaml
contract_version: "1.0.0"
orchestration_modes:
  - none
  - delegated_workers
  - review_swarm
platform_adapters:
  - generic_prompt
  - codex_subagent
  - claude_cowork
worker_statuses:
  - planned
  - running
  - passed
  - failed
  - blocked
  - skipped
merge_policies:
  - synthesize_with_conflict_matrix
  - consensus_then_gaps
  - controller_adjudication
required_worker_plan_fields:
  - orchestration_mode
  - controller_runtime
  - platform_adapter
  - task_id
  - paper_type
  - topic
  - workers
  - merge
  - final_review
required_worker_fields:
  - id
  - goal
  - functional_role
  - required_skills
  - allowed_artifacts
  - forbidden_artifacts
  - review_required
  - stop_conditions
required_merge_fields:
  - agent
  - policy
  - output_artifacts
required_final_review_fields:
  - reviewer
  - gate
```

- [ ] **Step 4: Add worker packet template**

Create `content/templates/worker-run-packet.json`:

```json
{
  "run_id": "",
  "worker_id": "",
  "controller_runtime": "",
  "platform_adapter": "generic_prompt",
  "task_id": "",
  "paper_type": "",
  "topic": "",
  "goal": "",
  "functional_role": "",
  "required_skills": [],
  "required_mcp": [],
  "allowed_artifacts": [],
  "forbidden_artifacts": [],
  "artifacts_read": [],
  "artifacts_written": [],
  "warnings": [],
  "blocking_issues": [],
  "status": "planned",
  "confidence": 0.0
}
```

- [ ] **Step 5: Add review and merge templates**

Create `content/templates/worker-review-packet.md`:

```markdown
# Worker Review Packet

## Review Metadata

- reviewer:
- worker_plan_run_id:
- reviewed_merge_artifact:
- review_status:

## Findings

- No findings recorded.

## Blocking Issues

- No blocking issues recorded.

## Required Revisions

- No required revisions recorded.

## Verification Evidence

- No verification evidence recorded.

## Verdict

- status: BLOCK
- confidence: 0.0
```

Create `content/templates/worker-merge-report.md`:

```markdown
# Worker Merge Report

## Worker Status Table

| Worker | Status | Confidence | Notes |
| --- | --- | ---: | --- |

## Accepted Worker Outputs

- No accepted worker outputs recorded.

## Rejected Or Blocked Worker Outputs

- No rejected or blocked worker outputs recorded.

## Conflict Summary

- No conflicts recorded.

## Gap Summary

- No gaps recorded.

## Controller Adjudication

- No adjudication recorded.

## Canonical Output Update Plan

- No canonical output updates planned.

## Final Review Request

- No final review requested.
```

- [ ] **Step 6: Run focused contract tests**

Run:

```bash
python3 -m unittest tests.test_worker_orchestration_contract.WorkerOrchestrationContractTests.test_contract_defines_enums_and_required_fields tests.test_worker_orchestration_contract.WorkerOrchestrationContractTests.test_worker_run_packet_has_safe_defaults tests.test_worker_orchestration_contract.WorkerOrchestrationContractTests.test_worker_review_template_has_required_headings tests.test_worker_orchestration_contract.WorkerOrchestrationContractTests.test_worker_merge_template_has_required_headings -v
```

Expected: pass for contract and templates; capability-map config test still fails.

- [ ] **Step 7: Commit contract and templates**

Run:

```bash
git add content/standards/worker-orchestration-contract.yaml content/templates/worker-run-packet.json content/templates/worker-review-packet.md content/templates/worker-merge-report.md tests/test_worker_orchestration_contract.py
git commit -m "feat(orchestration): add worker orchestration contract"
```

Expected: commit succeeds with only Task 1 files.

## Task 2: Add Worker Capability Map Config

**Files:**
- Modify: `content/standards/mcp-agent-capability-map.yaml`
- Test: `tests/test_worker_orchestration_contract.py`

- [ ] **Step 1: Add worker orchestration config after `team_run_config`**

Append this block after the existing `team_run_config` section in `content/standards/mcp-agent-capability-map.yaml`:

```yaml

# -- Worker Orchestration Configuration --------------------------------------
# Worker orchestration describes how one controller runtime can split a Task ID
# into scoped in-platform workers before merge/review. It is disabled by
# default in task-run and only activates when worker-mode flags request it.
worker_orchestration_config:
  B1:
    default_mode: delegated_workers
    adapter_preference:
      codex: codex_subagent
      claude: claude_cowork
      gemini: generic_prompt
    partition_strategy: by_search_facet
    max_workers: 4
    worker_pool:
      - literature_search_worker
      - screening_worker
      - extraction_worker
    merge_policy: synthesize_with_conflict_matrix
    barrier_rules:
      min_success_ratio: 0.6
      on_failure: degrade

  H3:
    default_mode: review_swarm
    adapter_preference:
      codex: codex_subagent
      claude: claude_cowork
      gemini: generic_prompt
    partition_strategy: by_reviewer_persona
    max_workers: 3
    worker_pool:
      - methodologist
      - domain_expert
      - reviewer_2
    merge_policy: controller_adjudication
    barrier_rules:
      min_success_ratio: 1.0
      on_failure: block
```

- [ ] **Step 2: Run capability-map worker config test**

Run:

```bash
python3 -m unittest tests.test_worker_orchestration_contract.WorkerOrchestrationContractTests.test_worker_orchestration_config_references_valid_contract_values -v
```

Expected: pass.

- [ ] **Step 3: Run strict standard validation**

Run:

```bash
python3 scripts/validate_research_standard.py --strict
```

Expected: pass. If it fails because the validator rejects unknown top-level keys, add `worker_orchestration_config` to the validator's allowed standard shape and add a validator regression test in the same task.

- [ ] **Step 4: Commit capability map config**

Run:

```bash
git add content/standards/mcp-agent-capability-map.yaml tests/test_worker_orchestration_contract.py
git commit -m "feat(orchestration): configure worker orchestration tasks"
```

Expected: commit succeeds.

## Task 3: Add CLI Flags And Disabled-By-Default Plumbing

**Files:**
- Modify: `packages/python-qiongli/src/qiongli/bridges/orchestrator.py`
- Create: `tests/test_worker_orchestration_runtime.py`

- [ ] **Step 1: Write failing parser and default behavior tests**

Create `tests/test_worker_orchestration_runtime.py`:

```python
from __future__ import annotations

import argparse
import unittest
from pathlib import Path
from typing import Any

from bridges.base_bridge import BridgeResponse
from bridges.mcp_connectors import MCPEvidence
from bridges.orchestrator import (
    ModelOrchestrator,
    _add_worker_orchestration_task_run_args,
)
from qiongli.source_layout import RepoLayout


REPO_ROOT = Path(__file__).resolve().parents[1]


class WorkerCaptureOrchestrator(ModelOrchestrator):
    def __init__(self) -> None:
        super().__init__(standards_dir=RepoLayout(REPO_ROOT).standards)
        self.runtime_calls: list[dict[str, Any]] = []

    def _runtime_preflight_error(
        self,
        agent_name: str,
        cwd: Path,
        runtime_options: dict[str, Any] | None = None,
    ) -> str | None:
        return None

    def _execute_runtime_agent(
        self,
        agent_name: str,
        prompt: str,
        cwd: Path,
        runtime_options: dict[str, Any] | None = None,
        profile_directive: str | None = None,
    ) -> BridgeResponse:
        self.runtime_calls.append(
            {
                "agent": agent_name,
                "prompt": prompt,
                "runtime_options": dict(runtime_options or {}),
                "profile_directive": profile_directive or "",
            }
        )
        return BridgeResponse(success=True, model=agent_name, content=f"{agent_name} ok")

    def _collect_mcp_evidence(
        self,
        task_packet: dict[str, Any],
        cwd: Path,
        strict: bool = False,
    ) -> tuple[list[MCPEvidence], list[str]]:
        return [MCPEvidence(provider="filesystem", status="ok", summary="mock")], []


class WorkerOrchestrationRuntimeTests(unittest.TestCase):
    def test_parser_accepts_worker_orchestration_flags(self) -> None:
        parser = argparse.ArgumentParser()
        _add_worker_orchestration_task_run_args(parser)

        args = parser.parse_args(
            [
                "--worker-mode",
                "delegated-workers",
                "--worker-adapter",
                "generic-prompt",
                "--max-workers",
                "2",
            ]
        )

        self.assertEqual("delegated-workers", args.worker_mode)
        self.assertEqual("generic-prompt", args.worker_adapter)
        self.assertEqual(2, args.max_workers)

    def test_task_run_defaults_to_no_worker_plan(self) -> None:
        orchestrator = WorkerCaptureOrchestrator()

        result = orchestrator.task_run(
            task_id="B1",
            paper_type="systematic-review",
            topic="worker-default",
            cwd=REPO_ROOT,
            skip_validation=True,
        )

        packet = result.data["task_packet"]
        self.assertEqual("none", packet["worker_orchestration"]["mode"])
        self.assertEqual("disabled", packet["worker_orchestration"]["status"])
        self.assertNotIn("Worker Orchestration", result.merged_analysis)


if __name__ == "__main__":
    unittest.main()
```

- [ ] **Step 2: Run tests and verify they fail**

Run:

```bash
python3 -m unittest tests.test_worker_orchestration_runtime -v
```

Expected: import failure for `_add_worker_orchestration_task_run_args` or `task_run` missing worker fields.

- [ ] **Step 3: Add worker choice constants and parser helper**

In `packages/python-qiongli/src/qiongli/bridges/orchestrator.py`, after `SOLO_ROLE_GATE_CHOICES`, add:

```python
WORKER_MODE_CHOICES = ("none", "auto", "delegated-workers", "review-swarm")
WORKER_ADAPTER_CHOICES = ("auto", "generic-prompt", "codex-subagent", "claude-cowork")
```

After `_add_controller_agnostic_task_run_args`, add:

```python
def _add_worker_orchestration_task_run_args(parser: argparse.ArgumentParser) -> None:
    """Add optional worker orchestration flags for task-run."""
    parser.add_argument(
        "--worker-mode",
        choices=WORKER_MODE_CHOICES,
        default="none",
        help="Worker orchestration mode: none, auto, delegated-workers, or review-swarm.",
    )
    parser.add_argument(
        "--worker-adapter",
        choices=WORKER_ADAPTER_CHOICES,
        default="auto",
        help="Worker adapter: auto, generic-prompt, codex-subagent, or claude-cowork.",
    )
    parser.add_argument(
        "--max-workers",
        type=int,
        help="Maximum worker units for worker orchestration.",
    )
```

- [ ] **Step 4: Add disabled worker state helper**

Inside `ModelOrchestrator`, near `_controller_runtime_overrides`, add:

```python
    @staticmethod
    def _build_disabled_worker_orchestration() -> dict[str, Any]:
        return {
            "mode": "none",
            "status": "disabled",
            "adapter": "none",
            "workers": [],
            "notes": [],
        }
```

- [ ] **Step 5: Add `task_run` parameters and packet default**

Update `task_run` signature with:

```python
        worker_mode: str = "none",
        worker_adapter: str = "auto",
        max_workers: int | None = None,
```

After `packet.update(...)` for runtime plan and self-critique loop, add:

```python
                "worker_orchestration": self._build_disabled_worker_orchestration(),
```

Use `max_workers` only in Task 4 to avoid unused behavior. It is acceptable for the parameter to be passed through before execution is implemented.

- [ ] **Step 6: Wire CLI flags into task-run parser and call**

In `main()`, after `_add_controller_agnostic_task_run_args(task_run)`, add:

```python
    _add_worker_orchestration_task_run_args(task_run)
```

In the `orchestrator.task_run(...)` call, add:

```python
            worker_mode=getattr(args, "worker_mode", "none"),
            worker_adapter=getattr(args, "worker_adapter", "auto"),
            max_workers=getattr(args, "max_workers", None),
```

- [ ] **Step 7: Run parser/default tests**

Run:

```bash
python3 -m unittest tests.test_worker_orchestration_runtime -v
```

Expected: pass.

- [ ] **Step 8: Run existing controller tests**

Run:

```bash
python3 -m unittest tests.test_controller_agnostic_orchestration -v
```

Expected: pass.

- [ ] **Step 9: Commit disabled-by-default plumbing**

Run:

```bash
git add packages/python-qiongli/src/qiongli/bridges/orchestrator.py tests/test_worker_orchestration_runtime.py
git commit -m "feat(orchestration): add worker orchestration task flags"
```

Expected: commit succeeds.

## Task 4: Build Worker Plan And Generic Adapter Execution

**Files:**
- Modify: `packages/python-qiongli/src/qiongli/bridges/orchestrator.py`
- Modify: `tests/test_worker_orchestration_runtime.py`

- [ ] **Step 1: Add failing worker-plan execution tests**

Append these tests to `WorkerOrchestrationRuntimeTests`:

```python
    def test_task_run_builds_and_executes_generic_worker_plan_when_enabled(self) -> None:
        orchestrator = WorkerCaptureOrchestrator()

        result = orchestrator.task_run(
            task_id="B1",
            paper_type="systematic-review",
            topic="worker-enabled",
            cwd=REPO_ROOT,
            skip_validation=True,
            execution_mode="solo",
            controller="codex",
            primary_agent="codex",
            worker_mode="delegated-workers",
            worker_adapter="generic-prompt",
            max_workers=2,
        )

        packet = result.data["task_packet"]
        worker_state = packet["worker_orchestration"]
        self.assertEqual("delegated_workers", worker_state["mode"])
        self.assertEqual("generic_prompt", worker_state["adapter"])
        self.assertEqual("ok", worker_state["barrier_status"])
        self.assertEqual(2, len(worker_state["workers"]))
        self.assertIn("## Worker Orchestration", result.merged_analysis)
        self.assertIn("Worker barrier status: ok", result.merged_analysis)

        worker_prompts = [
            call["prompt"]
            for call in orchestrator.runtime_calls
            if "Worker packet (JSON):" in call["prompt"]
        ]
        self.assertEqual(2, len(worker_prompts))
        self.assertIn("forbidden_artifacts", worker_prompts[0])

    def test_worker_barrier_degrades_when_allowed(self) -> None:
        class OneWorkerFailsOrchestrator(WorkerCaptureOrchestrator):
            def _execute_runtime_agent(
                self,
                agent_name: str,
                prompt: str,
                cwd: Path,
                runtime_options: dict[str, Any] | None = None,
                profile_directive: str | None = None,
            ) -> BridgeResponse:
                self.runtime_calls.append(
                    {
                        "agent": agent_name,
                        "prompt": prompt,
                        "runtime_options": dict(runtime_options or {}),
                        "profile_directive": profile_directive or "",
                    }
                )
                if "screening_worker" in prompt:
                    return BridgeResponse.from_error(agent_name, "worker failed")
                return BridgeResponse(success=True, model=agent_name, content=f"{agent_name} ok")

        orchestrator = OneWorkerFailsOrchestrator()
        result = orchestrator.task_run(
            task_id="B1",
            paper_type="systematic-review",
            topic="worker-degraded",
            cwd=REPO_ROOT,
            skip_validation=True,
            execution_mode="solo",
            controller="codex",
            primary_agent="codex",
            worker_mode="delegated-workers",
            worker_adapter="generic-prompt",
            max_workers=3,
        )

        worker_state = result.data["task_packet"]["worker_orchestration"]
        self.assertEqual("degraded", worker_state["barrier_status"])
        self.assertIn("Worker screening_worker failed", "\n".join(worker_state["notes"]))
        self.assertIn("Worker barrier status: degraded", result.merged_analysis)
```

- [ ] **Step 2: Run tests and verify they fail**

Run:

```bash
python3 -m unittest tests.test_worker_orchestration_runtime -v
```

Expected: worker-enabled tests fail because worker config loading and execution are not implemented.

- [ ] **Step 3: Add worker normalization helpers**

Inside `ModelOrchestrator`, add:

```python
    @staticmethod
    def _normalize_worker_mode(value: str | None) -> str:
        normalized = (value or "none").strip().lower().replace("-", "_")
        if normalized == "delegated_workers":
            return "delegated_workers"
        if normalized == "review_swarm":
            return "review_swarm"
        if normalized in {"none", "auto"}:
            return normalized
        raise ValueError("worker_mode must be one of: none, auto, delegated-workers, review-swarm.")

    @staticmethod
    def _normalize_worker_adapter(value: str | None) -> str:
        normalized = (value or "auto").strip().lower().replace("-", "_")
        if normalized in {"auto", "generic_prompt", "codex_subagent", "claude_cowork"}:
            return normalized
        raise ValueError("worker_adapter must be one of: auto, generic-prompt, codex-subagent, claude-cowork.")
```

- [ ] **Step 4: Add worker config loader**

Inside `ModelOrchestrator`, add:

```python
    def _load_worker_orchestration_config(self, task_id: str) -> dict[str, Any] | None:
        capability_map = yaml.safe_load(
            (self.standards_dir / "mcp-agent-capability-map.yaml").read_text(encoding="utf-8")
        ) or {}
        config = capability_map.get("worker_orchestration_config", {})
        if not isinstance(config, dict):
            return None
        block = config.get(task_id)
        return dict(block) if isinstance(block, dict) else None
```

- [ ] **Step 5: Add adapter resolution helper**

Inside `ModelOrchestrator`, add:

```python
    def _resolve_worker_adapter(
        self,
        controller_runtime: str,
        requested_adapter: str,
        worker_config: dict[str, Any],
    ) -> tuple[str, list[str]]:
        notes: list[str] = []
        requested = self._normalize_worker_adapter(requested_adapter)
        if requested != "auto":
            if requested in {"codex_subagent", "claude_cowork"}:
                notes.append(
                    f"Worker adapter '{requested}' requested but native dispatch is not implemented; using generic_prompt."
                )
                return "generic_prompt", notes
            return requested, notes
        preferences = worker_config.get("adapter_preference", {})
        preferred = ""
        if isinstance(preferences, dict):
            preferred = str(preferences.get(controller_runtime, "")).strip()
        if preferred in {"codex_subagent", "claude_cowork"}:
            notes.append(
                f"Worker adapter '{preferred}' preferred for {controller_runtime} but native dispatch is not implemented; using generic_prompt."
            )
            return "generic_prompt", notes
        if preferred == "generic_prompt":
            return "generic_prompt", notes
        return "generic_prompt", notes
```

- [ ] **Step 6: Add worker plan builder**

Inside `ModelOrchestrator`, add:

```python
    def _build_worker_plan(
        self,
        task_packet: dict[str, Any],
        worker_config: dict[str, Any],
        *,
        run_id: str,
        controller_runtime: str,
        adapter: str,
        requested_mode: str,
        max_workers: int | None,
    ) -> dict[str, Any]:
        configured_mode = str(worker_config.get("default_mode", "delegated_workers")).strip()
        mode = configured_mode if requested_mode == "auto" else requested_mode
        worker_ids = [str(item).strip() for item in worker_config.get("worker_pool", []) if str(item).strip()]
        if max_workers is not None:
            worker_ids = worker_ids[: max(1, int(max_workers))]
        topic = str(task_packet.get("topic", "")).strip()
        artifact_root = str(task_packet.get("artifact_root", "RESEARCH/[topic]/")).replace("[topic]", topic)
        workers = []
        for worker_id in worker_ids:
            worker_root = f"{artifact_root}runs/{run_id}/workers/{worker_id}/"
            workers.append(
                {
                    "id": worker_id,
                    "goal": f"Execute scoped worker objective for {task_packet.get('task_id')} as {worker_id}.",
                    "functional_role": str(task_packet.get("functional_owner", "research-orchestrator")),
                    "required_skills": list(task_packet.get("required_skills", [])),
                    "required_mcp": list(task_packet.get("required_mcp", [])),
                    "allowed_artifacts": [worker_root + "**"],
                    "forbidden_artifacts": list(task_packet.get("required_outputs", [])),
                    "review_required": True,
                    "stop_conditions": ["required_mcp_unavailable", "evidence_provenance_missing"],
                    "worker_root": worker_root,
                }
            )
        return {
            "orchestration_mode": mode,
            "controller_runtime": controller_runtime,
            "platform_adapter": adapter,
            "task_id": str(task_packet.get("task_id", "")),
            "paper_type": str(task_packet.get("paper_type", "")),
            "topic": topic,
            "workers": workers,
            "merge": {
                "agent": "controller",
                "policy": str(worker_config.get("merge_policy", "synthesize_with_conflict_matrix")),
                "output_artifacts": [f"{artifact_root}runs/{run_id}/worker-merge-report.md"],
            },
            "final_review": {
                "reviewer": "independent_runtime_or_worker",
                "gate": "accept_revise_block",
            },
            "barrier_rules": dict(worker_config.get("barrier_rules", {})),
            "notes": [],
        }
```

- [ ] **Step 7: Add generic worker prompt and execution helpers**

Inside `ModelOrchestrator`, add:

```python
    def _build_worker_prompt(
        self,
        worker_packet: dict[str, Any],
        task_packet: dict[str, Any],
        mcp_evidence: list[MCPEvidence],
        skill_cards: list[dict[str, Any]],
    ) -> str:
        return f"""Execute one scoped worker unit for a Qiongli task.

Worker packet (JSON):
{json.dumps(worker_packet, ensure_ascii=False, indent=2)}

Task packet (JSON):
{json.dumps(task_packet, ensure_ascii=False, indent=2)}

MCP evidence snapshot:
{self._format_mcp_evidence(mcp_evidence)}

Required skill cards:
{self._format_skill_context(skill_cards)}

Rules:
1. Work only on this worker goal.
2. Write or describe outputs only under allowed_artifacts.
3. Do not write forbidden_artifacts directly.
4. Preserve evidence provenance for every claim.

Return sections:
- Worker Status
- Artifacts Read
- Artifacts Written
- Findings
- Blocking Issues
- Confidence
"""

    def _execute_worker_plan(
        self,
        worker_plan: dict[str, Any],
        task_packet: dict[str, Any],
        mcp_evidence: list[MCPEvidence],
        skill_cards: list[dict[str, Any]],
        cwd: Path,
        profile_cfg: dict[str, Any],
        profile_name: str,
    ) -> list[dict[str, Any]]:
        results: list[dict[str, Any]] = []
        runtime = str(worker_plan.get("controller_runtime", "codex"))
        adapter = str(worker_plan.get("platform_adapter", "generic_prompt"))
        for worker in worker_plan.get("workers", []):
            worker_packet = {
                "run_id": str(task_packet.get("run_id", "")),
                "worker_id": str(worker.get("id", "")),
                "controller_runtime": runtime,
                "platform_adapter": adapter,
                "task_id": str(task_packet.get("task_id", "")),
                "paper_type": str(task_packet.get("paper_type", "")),
                "topic": str(task_packet.get("topic", "")),
                "goal": str(worker.get("goal", "")),
                "functional_role": str(worker.get("functional_role", "")),
                "required_skills": list(worker.get("required_skills", [])),
                "required_mcp": list(worker.get("required_mcp", [])),
                "allowed_artifacts": list(worker.get("allowed_artifacts", [])),
                "forbidden_artifacts": list(worker.get("forbidden_artifacts", [])),
                "artifacts_read": [],
                "artifacts_written": [],
                "warnings": [],
                "blocking_issues": [],
                "status": "planned",
                "confidence": 0.0,
            }
            prompt = self._build_worker_prompt(worker_packet, task_packet, mcp_evidence, skill_cards)
            resp = self._execute_runtime_agent(
                runtime,
                prompt,
                cwd,
                self._profile_runtime_options(profile_cfg, runtime),
                self._build_profile_directive(profile_name, profile_cfg, stage="draft"),
            )
            results.append(
                {
                    "worker_id": worker_packet["worker_id"],
                    "agent": runtime,
                    "success": resp.success,
                    "status": "passed" if resp.success else "failed",
                    "content": resp.content if resp.success else "",
                    "error": resp.error if not resp.success else "",
                    "confidence": 0.75 if resp.success else 0.0,
                }
            )
        return results
```

- [ ] **Step 8: Add barrier helper**

Inside `ModelOrchestrator`, add:

```python
    def _apply_worker_barrier(
        self,
        worker_results: list[dict[str, Any]],
        barrier_rules: dict[str, Any],
    ) -> tuple[str, list[dict[str, Any]], list[str]]:
        total = len(worker_results)
        successes = [item for item in worker_results if item.get("success")]
        failures = [item for item in worker_results if not item.get("success")]
        notes = [
            f"Worker {item.get('worker_id')} failed ({item.get('agent', '?')}): {item.get('error', 'unknown')}"
            for item in failures
        ]
        if not total:
            return "blocked", [], ["No worker units were dispatched."]
        if not failures:
            return "ok", successes, notes
        policy = str(barrier_rules.get("on_failure", "degrade"))
        min_ratio = float(barrier_rules.get("min_success_ratio", 0.6))
        success_ratio = len(successes) / total
        if policy == "block":
            notes.append("Worker barrier policy=block: halting because a worker failed.")
            return "blocked", [], notes
        if success_ratio >= min_ratio:
            notes.append(
                f"Worker barrier policy=degrade: {len(successes)}/{total} succeeded; proceeding."
            )
            return "degraded", successes, notes
        notes.append(
            f"Worker barrier policy=degrade: {len(successes)}/{total} succeeded below {min_ratio}; blocked."
        )
        return "blocked", [], notes
```

- [ ] **Step 9: Integrate worker execution in `task_run`**

Inside `task_run`, after MCP evidence collection and before draft runtime resolution, add:

```python
        normalized_worker_mode = self._normalize_worker_mode(worker_mode)
        normalized_worker_adapter = self._normalize_worker_adapter(worker_adapter)
        worker_merge_content = ""
        if normalized_worker_mode != "none":
            worker_config = self._load_worker_orchestration_config(normalized_task)
            if not worker_config:
                packet["worker_orchestration"] = {
                    "mode": normalized_worker_mode,
                    "status": "skipped",
                    "adapter": "none",
                    "workers": [],
                    "notes": [f"No worker orchestration config for task {normalized_task}."],
                }
            else:
                controller_runtime = controller_metadata.get("controller") or effective_runtime_plan["primary_agent"]
                adapter, adapter_notes = self._resolve_worker_adapter(
                    controller_runtime,
                    normalized_worker_adapter,
                    worker_config,
                )
                run_id = str(packet.get("run_id") or f"{normalized_task.lower()}-{normalized_topic}")
                worker_plan = self._build_worker_plan(
                    packet,
                    worker_config,
                    run_id=run_id,
                    controller_runtime=controller_runtime,
                    adapter=adapter,
                    requested_mode=normalized_worker_mode,
                    max_workers=max_workers,
                )
                worker_results = self._execute_worker_plan(
                    worker_plan,
                    packet,
                    mcp_evidence,
                    packet.get("required_skill_cards", []),
                    cwd,
                    draft_profile_cfg,
                    selected_profiles["draft"],
                )
                barrier_status, successful_workers, worker_notes = self._apply_worker_barrier(
                    worker_results,
                    worker_plan.get("barrier_rules", {}),
                )
                notes = [*adapter_notes, *worker_notes]
                packet["worker_orchestration"] = {
                    "mode": worker_plan["orchestration_mode"],
                    "status": "completed" if barrier_status in {"ok", "degraded"} else "blocked",
                    "adapter": adapter,
                    "barrier_status": barrier_status,
                    "workers": worker_results,
                    "successful_workers": [item["worker_id"] for item in successful_workers],
                    "merge": worker_plan["merge"],
                    "final_review": worker_plan["final_review"],
                    "notes": notes,
                }
                routing_notes.append(
                    f"Worker orchestration: mode={worker_plan['orchestration_mode']}, adapter={adapter}, barrier={barrier_status}."
                )
                routing_notes.extend(notes)
                worker_merge_content = "\\n".join(
                    [
                        f"## Worker Orchestration",
                        f"- Worker mode: {worker_plan['orchestration_mode']}",
                        f"- Worker adapter: {adapter}",
                        f"- Worker barrier status: {barrier_status}",
                        f"- Workers: {len(worker_results)}",
                    ]
                )
```

In `merged_parts`, after routing notes, add:

```python
        if worker_merge_content:
            merged_parts.extend([worker_merge_content, ""])
```

- [ ] **Step 10: Run worker runtime tests**

Run:

```bash
python3 -m unittest tests.test_worker_orchestration_runtime -v
```

Expected: pass.

- [ ] **Step 11: Run controller regression tests**

Run:

```bash
python3 -m unittest tests.test_controller_agnostic_orchestration -v
```

Expected: pass.

- [ ] **Step 12: Commit generic adapter execution**

Run:

```bash
git add packages/python-qiongli/src/qiongli/bridges/orchestrator.py tests/test_worker_orchestration_runtime.py
git commit -m "feat(orchestration): execute generic worker plans"
```

Expected: commit succeeds.

## Task 5: Add Merge And Final Review Prompts

**Files:**
- Modify: `packages/python-qiongli/src/qiongli/bridges/orchestrator.py`
- Modify: `tests/test_worker_orchestration_runtime.py`

- [ ] **Step 1: Add failing merge/review test**

Append this test:

```python
    def test_worker_execution_runs_merge_and_final_review(self) -> None:
        orchestrator = WorkerCaptureOrchestrator()

        result = orchestrator.task_run(
            task_id="B1",
            paper_type="systematic-review",
            topic="worker-merge",
            cwd=REPO_ROOT,
            skip_validation=True,
            execution_mode="duo",
            controller="codex",
            primary_agent="codex",
            review_agent="claude",
            worker_mode="delegated-workers",
            worker_adapter="generic-prompt",
            max_workers=2,
        )

        prompts = "\n\n".join(call["prompt"] for call in orchestrator.runtime_calls)
        self.assertIn("Merge worker results for this Qiongli task.", prompts)
        self.assertIn("Final-review the merged worker output.", prompts)
        worker_state = result.data["task_packet"]["worker_orchestration"]
        self.assertIn("merge_review_status", worker_state)
        self.assertEqual("passed", worker_state["merge_review_status"])
```

- [ ] **Step 2: Run test and verify it fails**

Run:

```bash
python3 -m unittest tests.test_worker_orchestration_runtime.WorkerOrchestrationRuntimeTests.test_worker_execution_runs_merge_and_final_review -v
```

Expected: fail because merge and final review prompts are not implemented.

- [ ] **Step 3: Add merge and final review prompt builders**

Inside `ModelOrchestrator`, add:

```python
    def _build_worker_merge_prompt(
        self,
        worker_results: list[dict[str, Any]],
        worker_plan: dict[str, Any],
        task_packet: dict[str, Any],
    ) -> str:
        return f"""Merge worker results for this Qiongli task.

Worker plan (JSON):
{json.dumps(worker_plan, ensure_ascii=False, indent=2)}

Task packet (JSON):
{json.dumps(task_packet, ensure_ascii=False, indent=2)}

Worker results (JSON):
{json.dumps(worker_results, ensure_ascii=False, indent=2)}

Rules:
1. Do not concatenate worker outputs.
2. Preserve disagreements in Conflict Summary.
3. Mark gaps that no worker covered.
4. Do not claim canonical outputs were updated unless the worker plan permits it.

Return sections:
- Worker Status Table
- Accepted Worker Outputs
- Rejected Or Blocked Worker Outputs
- Conflict Summary
- Gap Summary
- Controller Adjudication
- Canonical Output Update Plan
- Final Review Request
"""

    def _build_worker_final_review_prompt(
        self,
        merge_output: str,
        worker_plan: dict[str, Any],
        task_packet: dict[str, Any],
    ) -> str:
        return f"""Final-review the merged worker output.

Worker plan (JSON):
{json.dumps(worker_plan, ensure_ascii=False, indent=2)}

Task packet (JSON):
{json.dumps(task_packet, ensure_ascii=False, indent=2)}

Merged worker output:
{merge_output}

Review checklist:
1. Worker outputs stayed within allowed_artifacts.
2. Forbidden artifact writes were rejected or absent.
3. Conflicts and gaps are explicit.
4. Canonical output update plan is justified by worker evidence.

IMPORTANT: include one verdict line:
- Verdict: PASS
- Verdict: BLOCK

Return sections:
- Verdict
- Findings
- Blocking Issues
- Required Revisions
- Verification Evidence
- Confidence
"""
```

- [ ] **Step 4: Execute merge/review after successful barrier**

In the Task 4 integration block, after `_apply_worker_barrier`, add:

```python
                merge_resp = None
                final_review_resp = None
                merge_review_status = "skipped"
                if barrier_status in {"ok", "degraded"} and successful_workers:
                    merge_prompt = self._build_worker_merge_prompt(
                        successful_workers,
                        worker_plan,
                        packet,
                    )
                    merge_resp = self._execute_runtime_agent(
                        controller_runtime,
                        merge_prompt,
                        cwd,
                        self._profile_runtime_options(draft_profile_cfg, controller_runtime),
                        self._build_profile_directive(selected_profiles["draft"], draft_profile_cfg, stage="summary"),
                    )
                    if merge_resp.success:
                        final_review_prompt = self._build_worker_final_review_prompt(
                            merge_resp.content,
                            worker_plan,
                            packet,
                        )
                        review_runtime_for_workers = effective_runtime_plan.get("review_agent", controller_runtime)
                        final_review_resp = self._execute_runtime_agent(
                            review_runtime_for_workers,
                            final_review_prompt,
                            cwd,
                            self._profile_runtime_options(review_profile_cfg, review_runtime_for_workers),
                            self._build_profile_directive(selected_profiles["review"], review_profile_cfg, stage="review"),
                        )
                        merge_review_status = "passed" if final_review_resp.success else "failed"
                    else:
                        merge_review_status = "merge_failed"
```

Extend the `packet["worker_orchestration"]` dict with:

```python
                    "merge_status": "passed" if merge_resp and merge_resp.success else "skipped",
                    "merge_review_status": merge_review_status,
```

Extend `worker_merge_content` with:

```python
                        f"- Worker merge status: {packet['worker_orchestration']['merge_status']}",
                        f"- Worker final review status: {merge_review_status}",
```

- [ ] **Step 5: Run worker runtime tests**

Run:

```bash
python3 -m unittest tests.test_worker_orchestration_runtime -v
```

Expected: pass.

- [ ] **Step 6: Commit merge/review prompts**

Run:

```bash
git add packages/python-qiongli/src/qiongli/bridges/orchestrator.py tests/test_worker_orchestration_runtime.py
git commit -m "feat(orchestration): review merged worker outputs"
```

Expected: commit succeeds.

## Task 6: Document Worker Orchestration

**Files:**
- Modify: `docs/guide/multi-agent.md`
- Modify: `docs/advanced/controller-modes.md`
- Modify: `docs/advanced/agent-skill-collaboration.md`
- Modify: `content/workflow/references/platform-routing.md`

- [ ] **Step 1: Add focused doc assertions**

Add a test method to `tests/test_worker_orchestration_contract.py`:

```python
    def test_worker_orchestration_docs_describe_contract_and_fallbacks(self) -> None:
        docs = {
            "multi_agent": REPO_ROOT / "docs" / "guide" / "multi-agent.md",
            "controller_modes": REPO_ROOT / "docs" / "advanced" / "controller-modes.md",
            "collaboration": REPO_ROOT / "docs" / "advanced" / "agent-skill-collaboration.md",
            "platform_routing": LAYOUT.workflow / "references" / "platform-routing.md",
        }
        required_terms = {
            "worker_plan",
            "generic_prompt",
            "codex_subagent",
            "claude_cowork",
        }
        for name, path in docs.items():
            with self.subTest(name=name):
                text = path.read_text(encoding="utf-8")
                for term in required_terms:
                    self.assertIn(term, text)
```

- [ ] **Step 2: Run doc assertion and verify it fails**

Run:

```bash
python3 -m unittest tests.test_worker_orchestration_contract.WorkerOrchestrationContractTests.test_worker_orchestration_docs_describe_contract_and_fallbacks -v
```

Expected: fail because docs do not mention the new terms.

- [ ] **Step 3: Update multi-agent guide**

In `docs/guide/multi-agent.md`, after the `task-run` section, add:

```markdown
### Worker orchestration

Worker orchestration is a layer below runtime collaboration. Runtime flags such
as `--controller`, `--primary`, and `--reviewer` decide which model runtime owns
drafting and review. Worker flags decide whether that controller can split the
same Task ID into scoped in-platform workers before merge and final review.

Use:

```bash
python3 -m bridges.orchestrator task-run \
  --task-id B1 \
  --paper-type systematic-review \
  --topic ai-in-education \
  --cwd . \
  --execution-mode solo \
  --controller codex \
  --primary codex \
  --worker-mode delegated-workers \
  --worker-adapter generic-prompt \
  --max-workers 2
```

Adapter names are platform semantics:

- `generic_prompt`: portable fallback and CI baseline.
- `codex_subagent`: Codex subagent mapping when native dispatch is available.
- `claude_cowork`: Claude cowork mapping when native dispatch is available.

If native dispatch is unavailable, `codex_subagent` and `claude_cowork` degrade
to `generic_prompt` and record a routing note.
```

- [ ] **Step 4: Update controller modes doc**

In `docs/advanced/controller-modes.md`, after "Runtime Override Semantics", add:

```markdown
## Worker Delegation Semantics

Controller mode and worker orchestration are separate layers. `--controller`,
`--primary`, and `--reviewer` record runtime accountability and route the main
draft/review stages. `--worker-mode`, `--worker-adapter`, and `--max-workers`
optionally split one controller-owned task into a `worker_plan`.

The worker adapter names are `generic_prompt`, `codex_subagent`, and
`claude_cowork`. Native Codex subagent or Claude cowork execution is an adapter
detail. The Task ID, required skills, MCP evidence, artifact boundaries, and
quality gates remain unchanged.
```

- [ ] **Step 5: Update agent-skill collaboration doc**

In `docs/advanced/agent-skill-collaboration.md`, update the standard chain text to include:

```markdown
For worker-enabled runs, the chain becomes:

`plan -> mcp-evidence -> worker_plan -> worker-execute -> merge -> final-review -> validator-gate`

The `worker_plan` is platform-neutral. Codex can map it to `codex_subagent`,
Claude can map it to `claude_cowork`, and any runtime can fall back to
`generic_prompt` while preserving the same worker packets and merge report.
```

- [ ] **Step 6: Update platform routing reference**

In `content/workflow/references/platform-routing.md`, add:

```markdown
## Worker Adapter Routing

When `task-run` includes worker orchestration, use the canonical
`worker_plan` instead of inventing platform-specific delegation prose.

Adapter mapping:

- `generic_prompt`: execute worker packets as structured prompts.
- `codex_subagent`: map each worker packet to a Codex subagent task when native
  dispatch is available.
- `claude_cowork`: map each worker packet to Claude cowork when native dispatch
  is available.

If native dispatch is unavailable, record the degradation and run the same
packet through `generic_prompt`. Do not change Task IDs, required outputs,
quality gates, required skills, or MCP evidence requirements when switching
adapters.
```

- [ ] **Step 7: Run doc test**

Run:

```bash
python3 -m unittest tests.test_worker_orchestration_contract.WorkerOrchestrationContractTests.test_worker_orchestration_docs_describe_contract_and_fallbacks -v
```

Expected: pass.

- [ ] **Step 8: Commit docs**

Run:

```bash
git add docs/guide/multi-agent.md docs/advanced/controller-modes.md docs/advanced/agent-skill-collaboration.md content/workflow/references/platform-routing.md tests/test_worker_orchestration_contract.py
git commit -m "docs(orchestration): document worker delegation adapters"
```

Expected: commit succeeds.

## Task 7: Final Validation

**Files:**
- Verify all changed files.

- [ ] **Step 1: Run focused worker tests**

Run:

```bash
python3 -m unittest tests.test_worker_orchestration_contract tests.test_worker_orchestration_runtime -v
```

Expected: pass.

- [ ] **Step 2: Run orchestration regression tests**

Run:

```bash
python3 -m unittest tests.test_agent_run_contract tests.test_agent_routing_policy tests.test_controller_agnostic_orchestration -v
```

Expected: pass.

- [ ] **Step 3: Run strict standard validator**

Run:

```bash
python3 scripts/validate_research_standard.py --strict
```

Expected: pass.

- [ ] **Step 4: Check generated payload guard remains clean**

Run:

```bash
python3 -m unittest tests.test_generated_payload_guard tests.test_distribution_source_tree -v
```

Expected: pass. This verifies the implementation did not reintroduce tracked plugin payload edits.

- [ ] **Step 5: Check git status**

Run:

```bash
git status --short
```

Expected: no unstaged changes. If docs/superpowers plan file is intentionally untracked or ignored, stage it with `git add -f docs/superpowers/plans/2026-06-13-worker-orchestration.md` before the final plan commit.

- [ ] **Step 6: Confirm there is no cleanup commit needed**

Run:

```bash
git status --short
```

Expected: no output. If this command prints a changed file, return to the task
that owns that file, add a concrete checklist step for the cleanup, run that
task's focused tests again, and commit the named file with the task's commit
message pattern. Do not make a generic cleanup commit.

## Self-Review Notes

- Spec coverage: the plan covers canonical contract/templates, capability-map integration, explicit CLI controls, generic adapter fallback, worker barrier rules, merge/final-review prompts, docs, and validation. Native Codex subagent and Claude cowork dispatch remain documented adapter names and degrade to `generic_prompt`, matching the approved design.
- Scope control: the first implementation is explicit opt-in via `--worker-mode`; default `task-run` remains unchanged.
- Single-source boundary: the plan edits canonical `content/`, Python runtime source, tests, and docs only. It does not edit generated plugin payloads.
