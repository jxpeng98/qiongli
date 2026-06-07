from __future__ import annotations

from pathlib import Path

import yaml


def scaffold_custom_subject(out: Path, *, base_subject: str, name: str, force: bool = False) -> None:
    if out.exists() and any(out.iterdir()) and not force:
        raise FileExistsError(f"custom subject directory is not empty: {out}")

    out.mkdir(parents=True, exist_ok=True)
    overlays_skills = out / "overlays" / "skills"
    skills = out / "skills"
    domain_profiles = out / "domain-profiles"
    venue_profiles = out / "venue-profiles"

    overlays_skills.mkdir(parents=True, exist_ok=True)
    skills.mkdir(parents=True, exist_ok=True)
    domain_profiles.mkdir(parents=True, exist_ok=True)
    venue_profiles.mkdir(parents=True, exist_ok=True)

    subject_payload = {
        "name": name,
        "base_subject": base_subject,
        "skill_refs": [],
        "domain_profiles": [],
        "venue_profiles": [],
        "skill_overrides": [
            {
                "skill": "manuscript-architect",
                "overlay": "overlays/skills/manuscript-architect.md",
                "mode": "append",
            }
        ],
    }
    (out / "subject.yaml").write_text(
        yaml.safe_dump(subject_payload, sort_keys=False, allow_unicode=True),
        encoding="utf-8",
    )
    (skills / "registry.yaml").write_text("skills: []\n", encoding="utf-8")
    overlays_skills.joinpath("README.md").write_text(
        "\n".join(
            [
                "# Skill Overlays",
                "",
                "Add local skill overlay markdown here. Materialize the effective package with:",
                "",
                "```bash",
                "python3 scripts/materialize_subject_package.py --subject "
                f"{base_subject} --custom-dir path/to/custom --source . --out /tmp/qiongli-workflow",
                "```",
                "",
                "Local overlays affect generated output only and do not mutate canonical Qiongli source files.",
                "",
            ]
        ),
        encoding="utf-8",
    )
    (out / "README.md").write_text(
        "\n".join(
            [
                f"# {name}",
                "",
                f"This is a custom Qiongli subject layer for `{base_subject}`.",
                "",
                "Use this directory for local overlays, skills, domain profiles, and venue profiles.",
                "These customizations affect generated output only and do not mutate canonical Qiongli source files.",
                "",
            ]
        ),
        encoding="utf-8",
    )
