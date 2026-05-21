# Skill Set Optimization Scorecard

## Baseline

- Canonical registered skills: 71
- Current gate model: Q1-Q4 labels exist in workflow contracts and task routing.
- Current domain model: domain profiles are injected at runtime.
- Current eval model: offline academic quality fixtures score broad quality dimensions.

## Next Optimization Targets

| Target | Baseline | Next Bar |
|--------|----------|----------|
| Executable Q1-Q4 semantic gates | Gate labels and prompt instructions | Contract-backed gate reports audited offline |
| Economics and finance method packs | Method names and general diagnostics | Method templates with assumptions, required diagnostics, artifacts, and failure modes |
| Offline eval expansion | 6 broad cases | Gate-failure and domain-method cases included |
| Skill routing precision | Registry inputs/outputs plus task routing | Gate and domain-pack requirements visible in task prompts |
| Release confidence | Structural validation | Structural + semantic contract validation |

## Measurement Commands

Current baseline commands:

```bash
uv run python scripts/validate_research_standard.py --strict
uv run python scripts/audit_skill_sections.py --strict
uv run python scripts/run_academic_quality_evals.py evals/academic_quality/cases
```

Post-optimization commands, available after Task 1/2 create the audit scripts:

```bash
uv run python scripts/audit_quality_gates.py --strict
uv run python scripts/audit_domain_method_packs.py --strict
```
