#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: check_2x_native_change_boundary.sh --base-ref REF [--head-ref REF] [--repo-root PATH]

Reject changes to the frozen 1.x product/oracle and accepted 2.x architecture
anchors while allowing Rust-native migration work elsewhere in the repository.
EOF
}

repo_root="$(pwd)"
base_ref=""
head_ref="HEAD"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --base-ref)
      [[ $# -ge 2 ]] || { usage >&2; exit 2; }
      base_ref="$2"
      shift 2
      ;;
    --head-ref)
      [[ $# -ge 2 ]] || { usage >&2; exit 2; }
      head_ref="$2"
      shift 2
      ;;
    --repo-root)
      [[ $# -ge 2 ]] || { usage >&2; exit 2; }
      repo_root="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

if [[ -z "$base_ref" ]]; then
  echo "--base-ref is required" >&2
  usage >&2
  exit 2
fi

git -C "$repo_root" rev-parse --verify "$base_ref^{commit}" >/dev/null
git -C "$repo_root" rev-parse --verify "$head_ref^{commit}" >/dev/null
git -C "$repo_root" merge-base "$base_ref" "$head_ref" >/dev/null

violations=()
while IFS= read -r -d '' path; do
  case "$path" in
    packages/python-qiongli/*|\
    packages/qiongli-literature-mcpb/*|\
    tooling/migration/baselines/v1.19.0-beta.1/*|\
    tooling/migration/qiongli-1x-baseline-plan.json|\
    tooling/migration/baseline-plan.schema.json|\
    tooling/migration/baseline-manifest.schema.json|\
    tooling/migration/oracle-fixture.schema.json|\
    tooling/migration/2x-branch-point.json|\
    tooling/migration/2x-branch-point.schema.json|\
    tooling/architecture/arc-201-decisions.json|\
    docs/architecture/decisions/020[1-7]-*)
      violations+=("$path")
      ;;
  esac
done < <(
  git -C "$repo_root" diff \
    --no-renames \
    --name-only \
    -z \
    "$base_ref...$head_ref" \
    --
)

if [[ ${#violations[@]} -gt 0 ]]; then
  echo "Frozen 1.x or accepted architecture paths changed:" >&2
  for path in "${violations[@]}"; do
    printf '  %s\n' "$path" >&2
  done
  echo "Use the critical-fix maintenance line or a superseding ADR instead." >&2
  exit 1
fi

echo "Native 2.x change boundary passed."
