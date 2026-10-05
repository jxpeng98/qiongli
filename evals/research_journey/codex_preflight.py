"""Check Codex sandbox prerequisites without credentials or model invocations.

This checks public guidance-file access in a fresh profile. It does not qualify
Plugin policy, model guidance use, MCP calls or research quality.
"""
from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import shutil
import sys

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT))
from evals.research_journey.antigravity_observation import run_process  # noqa: E402
from evals.research_journey import plugin_baseline as baseline  # noqa: E402

PROFILE = "qiongli_public_read"


def check(codex: Path, guidance: list[Path], output: Path, timeout_seconds: int = 20):
    if type(timeout_seconds) is not int or not 0 < timeout_seconds <= 30:
        raise ValueError("Preflight timeout must be 1–30 seconds per command")
    if not guidance or len(guidance) > 8:
        raise ValueError("Supply 1–8 public installed guidance files")
    codex = codex.resolve(strict=True)
    expected = {}
    for path in guidance:
        if path.is_symlink() or not path.is_file():
            raise ValueError("Guidance must be a regular public file")
        path = path.resolve(strict=True)
        data = path.read_bytes()
        if not data or len(data) > 512_000:
            raise ValueError("Guidance must be nonempty and bounded")
        data.decode("utf-8")
        expected[path] = data
    if output.exists() or output.is_symlink():
        raise ValueError("Preflight output must be new")
    output = output.resolve()
    if output == ROOT or ROOT in output.parents:
        raise ValueError("Preflight output must be outside the checkout")
    output.mkdir(mode=0o700, parents=True)
    home, profile, work = (output / name for name in ("home", "codex-home", "work"))
    for path in (home, profile, work):
        path.mkdir(mode=0o700)
    # Codex re-executes its own binary inside the sandbox. An installation
    # outside the minimal system paths needs this exact runtime file readable.
    # Do not grant access to its parent (which may be a user's home or `/`).
    readable = dict.fromkeys([codex, *expected])
    config = (f'default_permissions = "{PROFILE}"\napproval_policy = "never"\n'
              f'[permissions.{PROFILE}.filesystem]\n":minimal" = "read"\n'
              + ''.join(f'{json.dumps(str(path))} = "read"\n' for path in readable)
              + f'[permissions.{PROFILE}.network]\nenabled = false\n')
    (profile / "config.toml").write_text(config, encoding="utf-8")
    env = {key: value for key, value in os.environ.items()
           if key in ("PATH", "LANG", "LC_ALL", "TERM", "SYSTEMROOT", "WINDIR", "TEMP", "TMP")}
    env.update(HOME=str(home), USERPROFILE=str(home), CODEX_HOME=str(profile),
               XDG_CONFIG_HOME=str(home / "config"))
    receipt = {"kind": "qiongli-codex-sandbox-preflight/v1", "status": "blocked",
               "scope": "sandbox execution and public-file access only",
               "effective_plugin_policy": "not-checked", "model_calls": 0,
               "codex_sha256": baseline.probe.digest(codex),
               "config_sha256": baseline.probe.sha(config),
               "guidance": {str(p): baseline.probe.sha(b.decode("utf-8")) for p, b in expected.items()},
               "checks": []}
    sandbox = [str(codex), "sandbox", "--permission-profile", PROFILE, "--cd", str(work), "--"]
    commands = [("version", [str(codex), "--version"], None),
                ("sandbox", sandbox + ["/usr/bin/true"], b"")]
    commands.extend((f"guidance-{i}", sandbox + ["/usr/bin/cat", "--", str(path)], data)
                    for i, (path, data) in enumerate(expected.items()))
    for name, command, wanted in commands:
        spool = output / name
        spool.mkdir()
        observed = run_process(command, work, spool, timeout_seconds, env=env)
        cleanup = observed.get("process_cleanup") or {}
        actual = (spool / "events.jsonl").read_bytes()
        success = (observed["termination_reason"] == "completed"
                   and observed["driver_error"] is None
                   and type(cleanup.get("exit_code")) is int and cleanup["exit_code"] == 0
                   and cleanup.get("reaped") is True and cleanup.get("group_gone") is True
                   and (bool(actual.strip()) if wanted is None else actual == wanted))
        receipt["checks"].append({"name": name, "command": command, "passed": success, **observed})
        if not success:
            receipt["blocker"] = name
            break
    else:
        receipt["status"] = "passed"
    try:
        unchanged = all(path.is_file() and not path.is_symlink() and path.read_bytes() == data
                        for path, data in expected.items())
    except OSError:
        unchanged = False
    config_path = profile / "config.toml"
    try:
        config_unchanged = (config_path.is_file() and not config_path.is_symlink()
                            and config_path.read_bytes() == config.encode("utf-8"))
    except OSError:
        config_unchanged = False
    auth = profile / "auth.json"
    receipt.update(guidance_unchanged=unchanged, config_unchanged=config_unchanged,
                   authentication_path_absent=not auth.exists() and not auth.is_symlink())
    if not unchanged or not config_unchanged or not receipt["authentication_path_absent"]:
        receipt.update(status="blocked", blocker="profile-or-guidance-changed")
    baseline.exclusive_json(output / "preflight.json", receipt)
    return receipt


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("output", type=Path)
    parser.add_argument("--guidance-file", type=Path, action="append", required=True)
    parser.add_argument("--codex", type=Path, default=shutil.which("codex"))
    args = parser.parse_args(argv)
    if not args.codex:
        parser.error("Codex executable is required")
    result = check(Path(args.codex), args.guidance_file, args.output)
    print(json.dumps(result, indent=2))
    return 0 if result["status"] == "passed" else 1


if __name__ == "__main__":
    raise SystemExit(main())
