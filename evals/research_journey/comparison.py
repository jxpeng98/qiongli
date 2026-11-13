"""Prepare frozen three-arm calibration materials; never run or score a model."""
from __future__ import annotations

import argparse
import json
from pathlib import Path, PurePosixPath
import random

from evals.research_journey.writing_observation import (
    _bytes, bind_files, freeze_packet, verify_files,
)

ROOT = Path(__file__).resolve().parents[2]
MATERIALS = Path(__file__).resolve().parent / "three-arm"


def selected_path(root, name):
    if (not isinstance(name, str) or not name or "\\" in name
            or PurePosixPath(name).is_absolute()
            or any(part in ("", ".", "..") for part in name.split("/"))):
        raise ValueError("Material path must be a relative, nonescaping file")
    path = root / name
    if not path.resolve().is_relative_to(root.resolve()):
        raise ValueError("Material path escapes the selected root")
    return path


def check_materials(root=MATERIALS):
    """Check this explicit calibration selection, without scanning a workspace."""
    root = Path(root)
    protocol = json.loads(_bytes(root / "protocol.json"))
    manifest = json.loads(_bytes(root / "fixture-manifest.json"))
    if (protocol["kind"] != "qiongli-three-arm-protocol/v1"
            or protocol["status"] != "calibration-materials-only"
            or protocol["arms"] != ["A", "B", "C"]
            or type(protocol["calibration_repeats"]) is not int
            or protocol["calibration_repeats"] != 1
            or manifest["kind"] != "qiongli-three-arm-materials/v1"):
        raise ValueError("Expected the frozen three-arm calibration protocol")
    cases = protocol["calibration_cases"]
    if [case["id"] for case in cases] != ["reading", "continuity"]:
        raise ValueError("Calibration cases must remain separate from main tasks")
    names = ["protocol.json", *protocol["shared_files"]]
    for case, checkpoints in zip(cases, [["initial"], ["initial", "followup"]]):
        if [item["id"] for item in case["checkpoints"]] != checkpoints:
            raise ValueError("Missing or reordered calibration checkpoint")
        for item in case["checkpoints"]:
            prefix = f"calibration/{case['id']}/{item['id']}/"
            files = item["files"]
            if (not isinstance(files, list) or prefix + "task.md" not in files
                    or len(files) < 2 or any(not f.startswith(prefix) for f in files)):
                raise ValueError("Producer inputs must stay in their own checkpoint")
            names.extend(files)
        for field, prefix in [("coordinator_files", "coordinator/"), ("review_files", "review/")]:
            files = case[field]
            if not isinstance(files, list) or any(not f.startswith(prefix) for f in files):
                raise ValueError("Coordinator/review materials must stay separate")
            names.extend(files)
        if not case["review_files"]:
            raise ValueError("Calibration requires separate source-specific review criteria")
    if len(names) != len(set(names)) or set(manifest["files"]) != set(names):
        raise ValueError("Duplicate, missing or unselected frozen material")
    files = {name: selected_path(root, name) for name in names}
    verify_files(files, manifest["files"])
    return protocol, files


def schedule(protocol, seed):
    """Shuffle complete calibration units; keep dependent sessions together."""
    if type(seed) is not int or not 0 <= seed < 2**32:
        raise ValueError("Schedule seed must be an unsigned 32-bit integer")
    units = []
    for case in protocol["calibration_cases"]:
        for arm in protocol["arms"]:
            units.append({
                "unit_id": f"cal-{case['id']}-{arm}-1", "task_id": case["id"],
                "arm": arm, "repeat": 1, "status": "not-started",
                "checkpoints": [{"id": item["id"], "status": "not-started",
                                 "material_files": item["files"]}
                                for item in case["checkpoints"]],
            })
    random.Random(seed).shuffle(units)
    for order, unit in enumerate(units, 1):
        unit["order"] = order
    return {"kind": "qiongli-calibration-schedule/v1", "seed": seed,
            "scope": "Run order only; never use this seed as a blind-review identity key",
            "planned_units": len(units),
            "planned_sessions": sum(len(unit["checkpoints"]) for unit in units),
            "units": units}


def prepare(destination, seed, root=MATERIALS):
    """Copy one verified public selection to a new external preparation directory."""
    destination = Path(destination)
    if (not destination.is_absolute() or destination.exists() or destination.is_symlink()
            or any(p.is_symlink() for p in destination.parents)
            or destination.resolve().is_relative_to(ROOT)):
        raise ValueError("Preparation needs a new absolute directory outside the checkout")
    protocol, files = check_materials(root)
    order = schedule(protocol, seed)
    files = {**files, "fixture-manifest.json": Path(root) / "fixture-manifest.json"}
    before = bind_files(files)
    destination.mkdir(mode=0o700)
    copied = {}
    for name, path in files.items():
        target = destination / "materials" / name
        target.parent.mkdir(parents=True, exist_ok=True)
        with target.open("xb") as stream:
            stream.write(_bytes(path))
        copied["materials/" + name] = target
    verify_files(files, before)
    verify_files({name: copied["materials/" + name] for name in files}, before)
    records = {
        "schedule.json": order,
        "preparation.json": {
            "kind": "qiongli-calibration-preparation/v1", "status": "materials-only",
            "model_calls_executed": 0, "runtime_identity": None,
            "runtime_policy_verified": False, "main_cohort_ready": False,
            "producer_access": "Grant only the current checkpoint and this arm's own work; never this root",
        },
    }
    for name, value in records.items():
        path = destination / name
        with path.open("x", encoding="utf-8") as stream:
            json.dump(value, stream, ensure_ascii=False, indent=2)
            stream.write("\n")
        copied[name] = path
    receipt = freeze_packet(copied, destination / "packet.json")
    return {**receipt, "status": "materials-only", "planned_units": order["planned_units"],
            "planned_sessions": order["planned_sessions"], "model_calls_executed": 0}


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)
    sub.add_parser("check")
    prep = sub.add_parser("prepare")
    prep.add_argument("destination", type=Path)
    prep.add_argument("--seed", type=int, required=True)
    args = parser.parse_args(argv)
    if args.command == "check":
        _, files = check_materials()
        print(json.dumps({"status": "frozen-materials-current", "files": len(files),
                          "scope": "Input identity only; no Host or outcome validation"}))
    else:
        print(json.dumps(prepare(args.destination, args.seed), indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
