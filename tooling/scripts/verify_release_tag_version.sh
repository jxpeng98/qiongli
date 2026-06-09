#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
TAG=""

usage() {
  cat <<'EOF'
Usage:
  ./scripts/verify_release_tag_version.sh [--root <dir>] --tag <tag>

Description:
  Verify that a Git release tag matches the package and workflow versions in the repo.

Options:
  --root <dir>  Repository root to verify. Defaults to this script's checkout.
  --tag <tag>   Required release tag.
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --root)
      [[ $# -ge 2 ]] || { echo "[verify-release-tag] missing value for --root" >&2; exit 2; }
      ROOT_DIR="$(cd "$2" && pwd)"
      shift 2
      ;;
    --tag)
      [[ $# -ge 2 ]] || { echo "[verify-release-tag] missing value for --tag" >&2; exit 2; }
      TAG="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "[verify-release-tag] unknown option: $1" >&2
      usage
      exit 2
      ;;
  esac
done

[[ -n "$TAG" ]] || { echo "[verify-release-tag] --tag is required" >&2; usage; exit 2; }

cd "$ROOT_DIR"

expected_repo_tag="$(python3 scripts/sync_versions.py "$TAG" --print-field repo_version)"
expected_package_version="$(python3 scripts/sync_versions.py "$TAG" --print-field package_version)"
expected_skill_version="$(python3 scripts/sync_versions.py "$TAG" --print-field skill_version)"
expected_npm_version="$(python3 scripts/sync_versions.py "$TAG" --print-field npm_version)"

if [[ "$expected_repo_tag" != "$TAG" ]]; then
  echo "[verify-release-tag] normalized tag mismatch: expected $expected_repo_tag from input $TAG" >&2
  exit 1
fi

actual_package_version="$(python3 - <<'PY'
import re
from pathlib import Path

content = Path("pyproject.toml").read_text(encoding="utf-8")
match = re.search(r'^version = "([^"]+)"$', content, re.MULTILINE)
if not match:
    raise SystemExit("missing version in pyproject.toml")
print(match.group(1))
PY
)"

actual_init_version="$(python3 - <<'PY'
import re
from pathlib import Path

content = Path("packages/python-qiongli/src/qiongli/__init__.py").read_text(encoding="utf-8")
match = re.search(r'^__version__ = "([^"]+)"$', content, re.MULTILINE)
if not match:
    raise SystemExit("missing __version__ in packages/python-qiongli/src/qiongli/__init__.py")
print(match.group(1))
PY
)"

actual_skill_version="$(python3 - <<'PY'
import re
from pathlib import Path

content = Path("content/skills/registry.yaml").read_text(encoding="utf-8")
versions = set(re.findall(r'^\s*version:\s*"?([^"\n]+)"?$', content, re.MULTILINE))
if not versions:
    raise SystemExit("missing version in content/skills/registry.yaml")
if len(versions) != 1:
    raise SystemExit(f"mixed versions in content/skills/registry.yaml: {sorted(versions)}")
print(versions.pop())
PY
)"

actual_workflow_version="$(tr -d '\r\n' < content/workflow/VERSION)"
actual_python_payload_workflow_version="$(tr -d '\r\n' < packages/python-qiongli/src/qiongli/payload/qiongli-workflow/VERSION)"
actual_python_payload_workflow_registry_version="$(python3 - <<'PY'
import re
from pathlib import Path

path = Path("packages/python-qiongli/src/qiongli/payload/qiongli-workflow/skills/registry.yaml")
if not path.exists():
    raise SystemExit("missing packages/python-qiongli/src/qiongli/payload/qiongli-workflow/skills/registry.yaml")
content = path.read_text(encoding="utf-8")
versions = set(re.findall(r'^\s*version:\s*"?([^"\n]+)"?$', content, re.MULTILINE))
if not versions:
    raise SystemExit(f"missing version in {path}")
if len(versions) != 1:
    raise SystemExit(f"mixed versions in {path}: {sorted(versions)}")
print(versions.pop())
PY
)"
actual_python_payload_registry_version="$(python3 - <<'PY'
import re
from pathlib import Path

path = Path("packages/python-qiongli/src/qiongli/payload/skills/registry.yaml")
if not path.exists():
    raise SystemExit("missing packages/python-qiongli/src/qiongli/payload/skills/registry.yaml")
content = path.read_text(encoding="utf-8")
versions = set(re.findall(r'^\s*version:\s*"?([^"\n]+)"?$', content, re.MULTILINE))
if not versions:
    raise SystemExit(f"missing version in {path}")
if len(versions) != 1:
    raise SystemExit(f"mixed versions in {path}: {sorted(versions)}")
print(versions.pop())
PY
)"
actual_bundled_workflow_version="$(tr -d '\r\n' < packages/npm-qiongli/payload/qiongli-workflow/VERSION)"
actual_bundled_workflow_registry_version="$(python3 - <<'PY'
import re
from pathlib import Path

path = Path("packages/npm-qiongli/payload/qiongli-workflow/skills/registry.yaml")
if not path.exists():
    raise SystemExit("missing packages/npm-qiongli/payload/qiongli-workflow/skills/registry.yaml")
content = path.read_text(encoding="utf-8")
versions = set(re.findall(r'^\s*version:\s*"?([^"\n]+)"?$', content, re.MULTILINE))
if not versions:
    raise SystemExit(f"missing version in {path}")
if len(versions) != 1:
    raise SystemExit(f"mixed versions in {path}: {sorted(versions)}")
print(versions.pop())
PY
)"
actual_plugin_workflow_version="$(tr -d '\r\n' < plugins/qiongli/skills/qiongli-workflow/VERSION)"
actual_plugin_workflow_registry_version="$(python3 - <<'PY'
import re
from pathlib import Path

path = Path("plugins/qiongli/skills/qiongli-workflow/skills/registry.yaml")
if not path.exists():
    raise SystemExit("missing plugins/qiongli/skills/qiongli-workflow/skills/registry.yaml")
content = path.read_text(encoding="utf-8")
versions = set(re.findall(r'^\s*version:\s*"?([^"\n]+)"?$', content, re.MULTILINE))
if not versions:
    raise SystemExit(f"missing version in {path}")
if len(versions) != 1:
    raise SystemExit(f"mixed versions in {path}: {sorted(versions)}")
print(versions.pop())
PY
)"
actual_next_plugin_workflow_version="$(tr -d '\r\n' < packages/qiongli-next-plugin/skills/qiongli-workflow/VERSION)"
actual_next_plugin_workflow_registry_version="$(python3 - <<'PY'
import re
from pathlib import Path

path = Path("packages/qiongli-next-plugin/skills/qiongli-workflow/skills/registry.yaml")
if not path.exists():
    raise SystemExit("missing packages/qiongli-next-plugin/skills/qiongli-workflow/skills/registry.yaml")
content = path.read_text(encoding="utf-8")
versions = set(re.findall(r'^\s*version:\s*"?([^"\n]+)"?$', content, re.MULTILINE))
if not versions:
    raise SystemExit(f"missing version in {path}")
if len(versions) != 1:
    raise SystemExit(f"mixed versions in {path}: {sorted(versions)}")
print(versions.pop())
PY
)"

actual_npm_version="$(python3 - <<'PY'
import json
from pathlib import Path

path = Path("packages/npm-qiongli/package.json")
if not path.exists():
    raise SystemExit("missing packages/npm-qiongli/package.json")
print(json.loads(path.read_text(encoding="utf-8"))["version"])
PY
)"

actual_npm_lock_version="$(python3 - <<'PY'
import json
from pathlib import Path

path = Path("package-lock.json")
if not path.exists():
    raise SystemExit("missing package-lock.json")
data = json.loads(path.read_text(encoding="utf-8"))
try:
    print(data["packages"]["packages/npm-qiongli"]["version"])
except KeyError as exc:
    raise SystemExit(f"missing package-lock workspace version: {exc}") from exc
PY
)"

actual_bundled_init_version="$(python3 - <<'PY'
import re
from pathlib import Path

path = Path("packages/npm-qiongli/python-runtime/qiongli/__init__.py")
if not path.exists():
    raise SystemExit("missing packages/npm-qiongli/python-runtime/qiongli/__init__.py")
content = path.read_text(encoding="utf-8")
match = re.search(r'^__version__ = "([^"]+)"$', content, re.MULTILINE)
if not match:
    raise SystemExit("missing __version__ in packages/npm-qiongli/python-runtime/qiongli/__init__.py")
print(match.group(1))
PY
)"

actual_bundled_registry_version="$(python3 - <<'PY'
import re
from pathlib import Path

path = Path("packages/npm-qiongli/python-runtime/skills/registry.yaml")
if not path.exists():
    raise SystemExit("missing packages/npm-qiongli/python-runtime/skills/registry.yaml")
content = path.read_text(encoding="utf-8")
versions = set(re.findall(r'^\s*version:\s*"?([^"\n]+)"?$', content, re.MULTILINE))
if not versions:
    raise SystemExit("missing version in packages/npm-qiongli/python-runtime/skills/registry.yaml")
if len(versions) != 1:
    raise SystemExit(f"mixed versions in {path}: {sorted(versions)}")
print(versions.pop())
PY
)"

actual_plugin_versions="$(python3 - <<'PY'
import json
from pathlib import Path

paths = [
    Path("packages/qiongli-plugin/.codex-plugin/plugin.json"),
    Path("packages/qiongli-plugin/.claude-plugin/plugin.json"),
    Path("packages/qiongli-plugin/gemini-extension.json"),
    Path("packages/qiongli-next-plugin/.codex-plugin/plugin.json"),
]
for path in paths:
    data = json.loads(path.read_text(encoding="utf-8"))
    versions = []

    def visit(value):
        if isinstance(value, dict):
            for key, item in value.items():
                if key == "version":
                    versions.append(item)
                else:
                    visit(item)
        elif isinstance(value, list):
            for item in value:
                visit(item)

    visit(data)
    if not versions:
        raise SystemExit(f"missing version in {path}")
    print(f"{path}:{','.join(versions)}")
PY
)"

[[ "$actual_package_version" == "$expected_package_version" ]] || {
  echo "[verify-release-tag] pyproject version mismatch: tag=$TAG expects $expected_package_version, found $actual_package_version" >&2
  exit 1
}

[[ "$actual_init_version" == "$expected_package_version" ]] || {
  echo "[verify-release-tag] packages/python-qiongli/src/qiongli/__init__.py mismatch: tag=$TAG expects $expected_package_version, found $actual_init_version" >&2
  exit 1
}

[[ "$actual_skill_version" == "$expected_skill_version" ]] || {
  echo "[verify-release-tag] content/skills/registry.yaml mismatch: tag=$TAG expects $expected_skill_version, found $actual_skill_version" >&2
  exit 1
}

[[ "$actual_workflow_version" == "$expected_repo_tag" ]] || {
  echo "[verify-release-tag] content/workflow/VERSION mismatch: tag=$TAG expects $expected_repo_tag, found $actual_workflow_version" >&2
  exit 1
}

[[ "$actual_python_payload_workflow_version" == "$expected_repo_tag" ]] || {
  echo "[verify-release-tag] packages/python-qiongli/src/qiongli/payload/qiongli-workflow/VERSION mismatch: tag=$TAG expects $expected_repo_tag, found $actual_python_payload_workflow_version" >&2
  exit 1
}

[[ "$actual_python_payload_workflow_registry_version" == "$expected_skill_version" ]] || {
  echo "[verify-release-tag] packages/python-qiongli/src/qiongli/payload/qiongli-workflow/skills/registry.yaml mismatch: tag=$TAG expects $expected_skill_version, found $actual_python_payload_workflow_registry_version" >&2
  exit 1
}

[[ "$actual_python_payload_registry_version" == "$expected_skill_version" ]] || {
  echo "[verify-release-tag] packages/python-qiongli/src/qiongli/payload/skills/registry.yaml mismatch: tag=$TAG expects $expected_skill_version, found $actual_python_payload_registry_version" >&2
  exit 1
}

[[ "$actual_bundled_workflow_version" == "$expected_repo_tag" ]] || {
  echo "[verify-release-tag] packages/npm-qiongli/payload/qiongli-workflow/VERSION mismatch: tag=$TAG expects $expected_repo_tag, found $actual_bundled_workflow_version" >&2
  exit 1
}

[[ "$actual_bundled_workflow_registry_version" == "$expected_skill_version" ]] || {
  echo "[verify-release-tag] packages/npm-qiongli/payload/qiongli-workflow/skills/registry.yaml mismatch: tag=$TAG expects $expected_skill_version, found $actual_bundled_workflow_registry_version" >&2
  exit 1
}

[[ "$actual_plugin_workflow_version" == "$expected_repo_tag" ]] || {
  echo "[verify-release-tag] plugins/qiongli/skills/qiongli-workflow/VERSION mismatch: tag=$TAG expects $expected_repo_tag, found $actual_plugin_workflow_version" >&2
  exit 1
}

[[ "$actual_plugin_workflow_registry_version" == "$expected_skill_version" ]] || {
  echo "[verify-release-tag] plugins/qiongli/skills/qiongli-workflow/skills/registry.yaml mismatch: tag=$TAG expects $expected_skill_version, found $actual_plugin_workflow_registry_version" >&2
  exit 1
}

[[ "$actual_next_plugin_workflow_version" == "$expected_repo_tag" ]] || {
  echo "[verify-release-tag] packages/qiongli-next-plugin/skills/qiongli-workflow/VERSION mismatch: tag=$TAG expects $expected_repo_tag, found $actual_next_plugin_workflow_version" >&2
  exit 1
}

[[ "$actual_next_plugin_workflow_registry_version" == "$expected_skill_version" ]] || {
  echo "[verify-release-tag] packages/qiongli-next-plugin/skills/qiongli-workflow/skills/registry.yaml mismatch: tag=$TAG expects $expected_skill_version, found $actual_next_plugin_workflow_registry_version" >&2
  exit 1
}

[[ "$actual_npm_version" == "$expected_npm_version" ]] || {
  echo "[verify-release-tag] packages/npm-qiongli/package.json mismatch: tag=$TAG expects $expected_npm_version, found $actual_npm_version" >&2
  exit 1
}

[[ "$actual_npm_lock_version" == "$expected_npm_version" ]] || {
  echo "[verify-release-tag] package-lock.json mismatch: tag=$TAG expects $expected_npm_version, found $actual_npm_lock_version" >&2
  exit 1
}

[[ "$actual_bundled_init_version" == "$expected_package_version" ]] || {
  echo "[verify-release-tag] packages/npm-qiongli/python-runtime/qiongli/__init__.py mismatch: tag=$TAG expects $expected_package_version, found $actual_bundled_init_version" >&2
  exit 1
}

[[ "$actual_bundled_registry_version" == "$expected_skill_version" ]] || {
  echo "[verify-release-tag] packages/npm-qiongli/python-runtime/skills/registry.yaml mismatch: tag=$TAG expects $expected_skill_version, found $actual_bundled_registry_version" >&2
  exit 1
}

while IFS= read -r plugin_line; do
  [[ -n "$plugin_line" ]] || continue
  plugin_path="${plugin_line%%:*}"
  plugin_versions="${plugin_line#*:}"
  IFS=',' read -r -a version_items <<< "$plugin_versions"
  for actual_plugin_version in "${version_items[@]}"; do
    [[ "$actual_plugin_version" == "$expected_skill_version" ]] || {
      echo "[verify-release-tag] ${plugin_path} mismatch: tag=$TAG expects $expected_skill_version, found $actual_plugin_version" >&2
      exit 1
    }
  done
done <<< "$actual_plugin_versions"

python3 scripts/audit_distribution_payloads.py --root "$ROOT_DIR"

echo "[verify-release-tag] tag and repo versions are aligned: $TAG"
