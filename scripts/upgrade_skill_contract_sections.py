#!/usr/bin/env python3
from __future__ import annotations

import argparse
import re
from pathlib import Path

import yaml


REQUIRED_SECTION_PATTERN = re.compile(r"^##\s+(.+?)\s*$", re.MULTILINE)


def _split_frontmatter(text: str) -> tuple[dict[str, object], str]:
    if not text.startswith("---\n"):
        return {}, text
    end = text.find("\n---\n", 4)
    if end == -1:
        return {}, text
    raw = text[4:end]
    data = yaml.safe_load(raw) or {}
    if not isinstance(data, dict):
        data = {}
    return data, text[end + 5 :]


def _section_names(text: str) -> set[str]:
    return {match.group(1).strip() for match in REQUIRED_SECTION_PATTERN.finditer(text)}


def _insert_before_heading(text: str, heading: str, block: str) -> str:
    pattern = re.compile(rf"^##\s+{re.escape(heading)}\s*$", re.MULTILINE)
    match = pattern.search(text)
    if not match:
        return text.rstrip() + "\n\n" + block.rstrip() + "\n"
    return text[: match.start()].rstrip() + "\n\n" + block.rstrip() + "\n\n" + text[match.start() :]


def _append_to_section(text: str, heading: str, lines: list[str]) -> str:
    pattern = re.compile(rf"^##\s+{re.escape(heading)}(?:\s+\(.+?\))?\s*$", re.MULTILINE)
    match = pattern.search(text)
    if not match:
        return text
    next_match = REQUIRED_SECTION_PATTERN.search(text, match.end())
    insert_at = next_match.start() if next_match else len(text)
    addition = "\n" + "\n".join(lines) + "\n"
    return text[:insert_at].rstrip() + addition + "\n" + text[insert_at:].lstrip()


def _artifact_lines(outputs: object) -> list[str]:
    lines: list[str] = []
    if isinstance(outputs, list):
        for item in outputs:
            if isinstance(item, dict):
                artifact = item.get("artifact")
                output_type = item.get("type", "Artifact")
                if isinstance(artifact, str) and artifact:
                    lines.append(f"- `{output_type}`: write `RESEARCH/[topic]/{artifact}`.")
            elif isinstance(item, str):
                lines.append(f"- `{item}`: write the registry-aligned artifact for this output.")
    return lines


def _input_lines(inputs: object) -> list[str]:
    lines: list[str] = []
    if isinstance(inputs, list):
        for item in inputs:
            if isinstance(item, dict):
                input_type = item.get("type", "Input")
                description = item.get("description", "Required upstream input")
                lines.append(f"- `{input_type}`: {description}")
            elif isinstance(item, str):
                lines.append(f"- `{item}`")
    return lines


def _inputs_block(frontmatter: dict[str, object]) -> str:
    lines = _input_lines(frontmatter.get("inputs"))
    if not lines:
        lines = ["- Use the upstream artifacts required by the workflow contract for this stage."]
    lines.extend(
        [
            "- If a required input is missing or insufficient, write a gap note under `RESEARCH/[topic]/context/gap_notes.md` and ask for the missing artifact instead of inventing content.",
            "- Treat literature, data, citations, and project files as evidence sources; keep unsupported assumptions visibly marked.",
        ]
    )
    return "## Inputs\n\n" + "\n".join(lines)


def _output_contract_block(frontmatter: dict[str, object]) -> str:
    lines = _artifact_lines(frontmatter.get("outputs"))
    if not lines:
        lines = ["- Write the stage-appropriate artifact path defined in `references/workflow-contract.md`."]
    lines.extend(
        [
            "- Separate finding, interpretation, and implication in the final artifact.",
            "- Do not invent citations, data, sample sizes, statistical results, or reviewer comments.",
            "- Apply `references/academic-output-rubric.md` before finalizing scholarly prose or review artifacts.",
        ]
    )
    return "## Output Contract\n\n" + "\n".join(lines)


def upgrade_skill_file(path: Path) -> bool:
    original = path.read_text(encoding="utf-8")
    frontmatter, _body = _split_frontmatter(original)
    sections = _section_names(original)
    updated = original
    changed = False

    if "Inputs" not in sections:
        updated = _insert_before_heading(updated, "Process", _inputs_block(frontmatter))
        changed = True
    elif not re.search(r"missing input|missing inputs|insufficient|gap note|if inputs are missing", updated, re.IGNORECASE):
        updated = _append_to_section(
            updated,
            "Inputs",
            [
                "- If a required input is missing or insufficient, write a gap note under `RESEARCH/[topic]/context/gap_notes.md` and ask for the missing artifact instead of inventing content.",
            ],
        )
        changed = True

    if "Output Contract" not in sections:
        updated = _insert_before_heading(updated, "Quality Bar", _output_contract_block(frontmatter))
        changed = True
    else:
        output_lines: list[str] = []
        lowered = updated.lower()
        if not all(term in lowered for term in ("finding", "interpretation", "implication")):
            output_lines.append("- Separate finding, interpretation, and implication in the final artifact.")
        if not re.search(r"do not invent|do not fabricate|no hallucinated|never invent|not invent|不编造", updated, re.IGNORECASE):
            output_lines.append("- Do not invent citations, data, sample sizes, statistical results, or reviewer comments.")
        if "academic-output-rubric.md" not in updated:
            output_lines.append("- Apply `references/academic-output-rubric.md` before finalizing scholarly prose or review artifacts.")
        if output_lines:
            updated = _append_to_section(updated, "Output Contract", output_lines)
            changed = True

    if changed:
        path.write_text(updated.rstrip() + "\n", encoding="utf-8")
    return changed


def _skill_files(root: Path, stages: set[str] | None) -> list[Path]:
    paths = sorted((root / "skills").rglob("*.md"))
    if stages is None:
        return paths
    return [path for path in paths if path.relative_to(root / "skills").parts[0] in stages]


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Mechanically add baseline skill quality contract sections.")
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1], help="Repository root")
    parser.add_argument("--stages", help="Comma-separated stage directories to upgrade")
    args = parser.parse_args(argv)

    stages = {part.strip() for part in args.stages.split(",") if part.strip()} if args.stages else None
    changed = 0
    for path in _skill_files(args.root.resolve(), stages):
        if upgrade_skill_file(path):
            changed += 1
            print(f"[upgrade-skill-contract] updated {path.relative_to(args.root.resolve())}")
    print(f"[upgrade-skill-contract] changed {changed} files")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
