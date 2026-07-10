#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
MODE="${1:-}"
DEV_PRERELEASE_BRANCH="dev"
BRANCH_REQUIRED_WORKFLOWS=("CI" "Checkout Install Check")
shift || true

normalize_field() {
  local version="$1"
  local field="$2"
  python3 "${ROOT_DIR}/scripts/sync_versions.py" "$version" --print-field "$field"
}

is_prerelease_tag() {
  [[ "$1" == *beta* || "$1" =~ b[0-9]+ ]]
}

detect_primary_branch() {
  if git show-ref --verify --quiet "refs/heads/main"; then
    printf 'main\n'
    return 0
  fi
  if git show-ref --verify --quiet "refs/heads/master"; then
    printf 'master\n'
    return 0
  fi
  git rev-parse --abbrev-ref HEAD
}

ensure_git_identity() {
  if git config user.name >/dev/null 2>&1 && git config user.email >/dev/null 2>&1; then
    return 0
  fi

  if [[ "${GITHUB_ACTIONS:-}" == "true" ]]; then
    git config user.name "github-actions[bot]"
    git config user.email "41898282+github-actions[bot]@users.noreply.github.com"
    return 0
  fi

  echo "[release-automation] git user.name and user.email must be configured before publish mode can create a commit" >&2
  exit 1
}

ensure_clean_resume_worktree() {
  if [[ -n "$(git status --porcelain --untracked-files=normal)" ]]; then
    echo "[release-automation] --resume-after-ready requires a clean working tree; commit or discard release-prep changes first" >&2
    exit 1
  fi
}

derive_repo_slug() {
  local remote_url
  remote_url="$(git remote get-url origin 2>/dev/null || true)"
  if [[ "$remote_url" =~ github\.com[:/]([^/]+)/([^/.]+)(\.git)?$ ]]; then
    printf '%s/%s\n' "${BASH_REMATCH[1]}" "${BASH_REMATCH[2]}"
    return 0
  fi
  return 1
}

resolve_local_tag_target() {
  local tag="$1"
  git rev-parse -q --verify "refs/tags/${tag}^{}" 2>/dev/null || true
}

resolve_remote_tag_target() {
  local remote="$1"
  local tag="$2"
  local target

  target="$(git ls-remote --exit-code --tags "$remote" "${tag}^{}" 2>/dev/null | awk 'NR == 1 { print $1 }' || true)"
  if [[ -z "$target" ]]; then
    target="$(git ls-remote --exit-code --tags "$remote" "$tag" 2>/dev/null | awk 'NR == 1 { print $1 }' || true)"
  fi
  printf '%s\n' "$target"
}

ensure_tag_matches_release_commit() {
  local repo_tag="$1"
  local release_commit="$2"
  local push_remote="$3"
  local local_tag_target remote_tag_target

  local_tag_target="$(resolve_local_tag_target "$repo_tag")"
  if [[ -n "$local_tag_target" && "$local_tag_target" != "$release_commit" ]]; then
    echo "[release-automation] local tag $repo_tag points at $local_tag_target, expected $release_commit" >&2
    exit 1
  fi

  remote_tag_target="$(resolve_remote_tag_target "$push_remote" "$repo_tag")"
  if [[ -n "$remote_tag_target" && "$remote_tag_target" != "$release_commit" ]]; then
    echo "[release-automation] remote tag $repo_tag on $push_remote points at $remote_tag_target, expected $release_commit" >&2
    exit 1
  fi
}

fetch_actions_runs() {
  local repo_slug="$1"
  local ref_name="$2"
  local api_url ci_json
  local -a curl_cmd

  if command -v gh >/dev/null 2>&1; then
    if ci_json="$(gh api "repos/${repo_slug}/actions/runs?branch=${ref_name}&per_page=20" 2>/dev/null)"; then
      printf '%s' "$ci_json"
      return 0
    fi
  fi

  api_url="https://api.github.com/repos/${repo_slug}/actions/runs?branch=${ref_name}&per_page=20"
  curl_cmd=(curl -fsSL "$api_url")
  if [[ -n "${GH_TOKEN:-}" ]]; then
    curl_cmd=(curl -fsSL -H "Authorization: Bearer ${GH_TOKEN}" "$api_url")
  fi

  "${curl_cmd[@]}"
}

query_actions_status() {
  local repo_slug="$1"
  local ref_name="$2"
  local commit="$3"
  shift 3
  local ci_json ci_json_file
  local -a required_workflows=("$@")

  if [[ -z "$repo_slug" ]]; then
    printf 'skipped:no-repo-slug\n'
    return 0
  fi

  if ! ci_json="$(fetch_actions_runs "$repo_slug" "$ref_name" 2>/dev/null)"; then
    printf 'skipped:request-failed\n'
    return 0
  fi

  ci_json_file="$(mktemp)"
  printf '%s' "$ci_json" >"$ci_json_file"

  set +e
  python3 - "$ci_json_file" "$commit" "${required_workflows[@]}" <<'PY'
import json
import sys

payload_path = sys.argv[1]
commit = sys.argv[2]
required = sys.argv[3:]
with open(payload_path, "r", encoding="utf-8") as fh:
    raw = fh.read().strip()
if not raw:
    print("skipped:empty-response")
    raise SystemExit(0)

try:
    payload = json.loads(raw)
except json.JSONDecodeError:
    print("skipped:invalid-json")
    raise SystemExit(0)

runs = payload.get("workflow_runs", [])
observed = sorted({r.get("name") or "unknown" for r in runs if r.get("head_sha") == commit})
results = []
pending = []
failed = []
missing = []

for workflow_name in required:
    matches = [r for r in runs if r.get("head_sha") == commit and r.get("name") == workflow_name]
    if not matches:
        missing.append(workflow_name)
        continue
    latest = sorted(matches, key=lambda r: r.get("created_at", ""), reverse=True)[0]
    status = latest.get("status") or "unknown"
    conclusion = latest.get("conclusion") or "unknown"
    html_url = latest.get("html_url") or ""
    results.append(f"{workflow_name}={status}/{conclusion}:{html_url}")
    if conclusion == "success":
        continue
    if status != "completed":
        pending.append(workflow_name)
        continue
    failed.append(workflow_name)

if failed:
    print("failed:" + "; ".join(results))
    raise SystemExit(1)
if pending or missing:
    labels = []
    if pending:
        labels.append("pending=" + ",".join(sorted(pending)))
    if missing:
        labels.append("missing=" + ",".join(sorted(missing)))
        labels.append("observed=" + ",".join(observed))
    print("pending:" + "; ".join(labels + results))
    raise SystemExit(0)
print("success:" + "; ".join(results))
raise SystemExit(0)
PY
  local status=$?
  set -e
  rm -f "$ci_json_file"
  return "$status"
}

wait_for_required_workflows() {
  local repo_slug="$1"
  local ref_name="$2"
  local commit="$3"
  local timeout_seconds="$4"
  local poll_interval_seconds="$5"
  shift 5
  local result status wait_deadline
  local -a required_workflows=("$@")

  if [[ -z "$repo_slug" ]]; then
    echo "[release-automation] cannot derive GitHub repo slug; refusing to create release tag before branch checks pass" >&2
    exit 1
  fi

  wait_deadline=$((SECONDS + timeout_seconds))
  while true; do
    set +e
    result="$(query_actions_status "$repo_slug" "$ref_name" "$commit" "${required_workflows[@]}")"
    status=$?
    set -e

    if [[ "$status" -ne 0 ]]; then
      echo "[release-automation] branch checks failed before tag creation: $result" >&2
      exit 1
    fi
    if [[ "$result" == success:* ]]; then
      echo "[release-automation] branch checks passed before tag creation: $result"
      return 0
    fi
    if [[ "$result" == skipped:* ]]; then
      echo "[release-automation] unable to verify branch checks before tag creation: $result" >&2
      exit 1
    fi
    if (( SECONDS >= wait_deadline )); then
      echo "[release-automation] timed out waiting for branch checks before tag creation: $result" >&2
      exit 1
    fi

    echo "[release-automation] waiting for branch checks before tag creation: $result"
    sleep "$poll_interval_seconds"
  done
}

usage() {
  cat <<'EOF'
Usage:
  ./scripts/release_automation.sh <pre|post|publish> [options]

Examples:
  ./scripts/release_automation.sh pre --tag v0.1.0 --from-tag v0.1.0-beta.6
  ./scripts/release_automation.sh pre --tag v0.1.1-beta.1 --from-tag v0.1.0
  ./scripts/release_automation.sh post --tag v0.1.0
  ./scripts/release_automation.sh post --tag v0.1.0 --create-release
  ./scripts/release_automation.sh publish --version 0.1.0 --from-tag v0.1.0-beta.6
  ./scripts/release_automation.sh publish --tag v0.1.0 --from-tag v0.1.0-beta.6
  ./scripts/release_automation.sh publish --tag v0.1.0 --resume-after-ready

Notes:
  - pre  -> runs scripts/release_preflight.sh
  - post -> runs scripts/release_postflight.sh
  - publish is the canonical release entrypoint. Use pre/post only for diagnostics or recovery.
  - pre supports pass-through flags such as --from-tag, --skip-note-gen, --note-overwrite, --skip-smoke, --maintainer-smoke, and --no-strict.
  - publish -> runs release_ready with staged distribution checks (including pypi_preflight.sh and npm_preflight.sh), commits and pushes release-prep files, waits for branch CI/checks, creates/pushes the tag, waits for tag publish workflows, then runs postflight with release-page creation.
  - publish --resume-after-ready starts after a completed release_ready/release-prep commit and continues branch push, branch CI gate, tag push, postflight, and acceptance receipt.
  - publish always uses hard CI gates before tag creation and release-page creation; use post mode for diagnostic soft waits only.
  - publish stable releases from the primary branch; publish prerelease/beta tags from dev or the primary branch.
EOF
}

[[ -n "$MODE" ]] || { usage; exit 2; }

cd "$ROOT_DIR"

case "$MODE" in
  pre)
    ./scripts/release_preflight.sh "$@"
    ;;
  post)
    ./scripts/release_postflight.sh "$@"
    ;;
  publish)
    version=""
    version_from_tag=""
    from_tag=""
    allow_dirty=0
    skip_bump=0
    commit_message=""
    push_remote="origin"
    push_branch=""
    tag_message="qiongli release"
    create_release=1
    skip_remote=0
    skip_ci_status=0
    wait_ci=1
    ci_timeout_seconds=1800
    ci_timeout_mode="hard"
    ci_poll_interval_seconds=30
    resume_after_ready=0
    ready_args=()
    post_args=()

    while [[ $# -gt 0 ]]; do
      case "$1" in
        --version)
          [[ $# -ge 2 ]] || { echo "[release-automation] missing value for --version" >&2; exit 2; }
          version="$2"
          shift 2
          ;;
        --tag)
          [[ $# -ge 2 ]] || { echo "[release-automation] missing value for --tag" >&2; exit 2; }
          version_from_tag="$2"
          shift 2
          ;;
        --from-tag)
          [[ $# -ge 2 ]] || { echo "[release-automation] missing value for --from-tag" >&2; exit 2; }
          from_tag="$2"
          ready_args+=("$1" "$2")
          shift 2
          ;;
        --skip-bump)
          skip_bump=1
          ready_args+=("$1")
          shift
          ;;
        --allow-dirty)
          allow_dirty=1
          ready_args+=("$1")
          shift
          ;;
        --resume-after-ready)
          resume_after_ready=1
          shift
          ;;
        --skip-smoke|--maintainer-smoke|--no-strict|--skip-note-gen|--note-overwrite|--no-build|--no-install-smoke|--keep-dist)
          ready_args+=("$1")
          shift
          ;;
        --staging-dir)
          [[ $# -ge 2 ]] || { echo "[release-automation] missing value for --staging-dir" >&2; exit 2; }
          ready_args+=("$1" "$2")
          shift 2
          ;;
        --commit-message)
          [[ $# -ge 2 ]] || { echo "[release-automation] missing value for --commit-message" >&2; exit 2; }
          commit_message="$2"
          shift 2
          ;;
        --push-remote)
          [[ $# -ge 2 ]] || { echo "[release-automation] missing value for --push-remote" >&2; exit 2; }
          push_remote="$2"
          shift 2
          ;;
        --push-branch)
          [[ $# -ge 2 ]] || { echo "[release-automation] missing value for --push-branch" >&2; exit 2; }
          push_branch="$2"
          shift 2
          ;;
        --tag-message)
          [[ $# -ge 2 ]] || { echo "[release-automation] missing value for --tag-message" >&2; exit 2; }
          tag_message="$2"
          shift 2
          ;;
        --no-create-release)
          create_release=0
          shift
          ;;
        --skip-remote)
          skip_remote=1
          post_args+=("$1")
          shift
          ;;
        --skip-ci-status)
          echo "[release-automation] publish mode cannot skip CI status checks before tag creation" >&2
          exit 2
          ;;
        --wait-ci)
          wait_ci=1
          shift
          ;;
        --no-wait-ci)
          echo "[release-automation] publish mode cannot disable CI waiting before tag creation" >&2
          exit 2
          ;;
        --ci-timeout-seconds)
          [[ $# -ge 2 ]] || { echo "[release-automation] missing value for --ci-timeout-seconds" >&2; exit 2; }
          ci_timeout_seconds="$2"
          shift 2
          ;;
        --ci-timeout-mode)
          [[ $# -ge 2 ]] || { echo "[release-automation] missing value for --ci-timeout-mode" >&2; exit 2; }
          ci_timeout_mode="$2"
          [[ "$ci_timeout_mode" == "hard" || "$ci_timeout_mode" == "soft" ]] || {
            echo "[release-automation] --ci-timeout-mode must be hard or soft" >&2
            exit 2
          }
          if [[ "$ci_timeout_mode" != "hard" ]]; then
            echo "[release-automation] publish mode requires hard CI timeout mode before tag creation" >&2
            exit 2
          fi
          shift 2
          ;;
        --ci-poll-interval-seconds)
          [[ $# -ge 2 ]] || { echo "[release-automation] missing value for --ci-poll-interval-seconds" >&2; exit 2; }
          ci_poll_interval_seconds="$2"
          shift 2
          ;;
        *)
          echo "[release-automation] unknown option for publish mode: $1" >&2
          usage
          exit 2
          ;;
      esac
    done

    [[ -n "$version" || -n "$version_from_tag" ]] || { echo "[release-automation] publish mode requires --version or --tag" >&2; exit 2; }

    if [[ -n "$version" && -n "$version_from_tag" ]]; then
      repo_tag_from_version="$(normalize_field "$version" repo_version)"
      repo_tag_from_tag="$(normalize_field "$version_from_tag" repo_version)"
      if [[ -n "$version" && -n "$version_from_tag" && "$repo_tag_from_version" != "$repo_tag_from_tag" ]]; then
        echo "[release-automation] --version and --tag point to different releases: ${repo_tag_from_version} != ${repo_tag_from_tag}" >&2
        exit 2
      fi
      version_input="$version"
      repo_tag="$repo_tag_from_version"
    elif [[ -n "$version" ]]; then
      version_input="$version"
      repo_tag="$(normalize_field "$version_input" repo_version)"
    else
      version_input="$version_from_tag"
      repo_tag="$(normalize_field "$version_input" repo_version)"
    fi
    package_version="$(normalize_field "$version_input" package_version)"

    primary_branch="$(detect_primary_branch)"
    current_branch="$(git rev-parse --abbrev-ref HEAD)"
    if [[ -z "$push_branch" ]]; then
      push_branch="$current_branch"
    fi

    release_branch="$primary_branch"
    if is_prerelease_tag "$repo_tag" && [[ "$current_branch" == "$DEV_PRERELEASE_BRANCH" ]]; then
      release_branch="$DEV_PRERELEASE_BRANCH"
    fi
    if [[ "$current_branch" != "$release_branch" || "$push_branch" != "$release_branch" ]]; then
      echo "[release-automation] publish mode must run from the release branch. Stable releases use primary branch ($primary_branch); prerelease releases may run from $DEV_PRERELEASE_BRANCH. Current branch: $current_branch; push branch: $push_branch; expected release branch: $release_branch" >&2
      exit 1
    fi

    if [[ -z "$commit_message" ]]; then
      commit_message="chore: prepare release ${package_version}"
    fi

    ensure_git_identity

    if [[ "$resume_after_ready" -eq 0 ]]; then
      release_ready_args=(--version "$version_input")
      release_ready_args+=("${ready_args[@]}")
      ./scripts/release_ready.sh "${release_ready_args[@]}"

      git add \
        pyproject.toml \
        docs/reference/skills.md \
        docs/zh/reference/skills.md \
        packages/python-qiongli/src/qiongli/__init__.py \
        content/workflow/SKILL.md \
        content/workflow/VERSION \
        content/skills/registry.yaml \
        content/distribution/plugins.yaml \
        README.md \
        README_CN.md \
        docs/index.md \
        docs/zh/index.md \
        docs/guide/install.md \
        docs/zh/guide/install.md \
        package-lock.json \
        uv.lock \
        packages/npm-qiongli \
        tooling/scripts/build_plugin_artifacts.py \
        tooling/scripts/materialize_distribution_payloads.py
      if is_prerelease_tag "$repo_tag"; then
        git add "tooling/release/${repo_tag}.md"
      else
        git add CHANGELOG.md
      fi
      if ! git diff --cached --quiet; then
        git commit -m "$commit_message"
      else
        echo "[release-automation] no staged release-prep changes to commit; continuing"
      fi
    else
      echo "[release-automation] resuming after release_ready; skipping preflight and release-prep commit"
      ensure_clean_resume_worktree
    fi
    release_commit="$(git rev-parse HEAD)"
    ensure_tag_matches_release_commit "$repo_tag" "$release_commit" "$push_remote"

    repo_slug="$(derive_repo_slug || true)"
    git push "$push_remote" "$push_branch"
    wait_for_required_workflows "$repo_slug" "$push_branch" "$release_commit" "$ci_timeout_seconds" "$ci_poll_interval_seconds" "${BRANCH_REQUIRED_WORKFLOWS[@]}"

    local_tag_target="$(resolve_local_tag_target "$repo_tag")"
    if [[ -z "$local_tag_target" ]]; then
      git tag -a "$repo_tag" -m "$tag_message"
    fi

    remote_tag_target="$(resolve_remote_tag_target "$push_remote" "$repo_tag")"
    if [[ -z "$remote_tag_target" ]]; then
      git push "$push_remote" "$repo_tag"
    else
      echo "[release-automation] tag already exists remotely at release commit; skipping tag push"
    fi

    post_args+=(--wait-ci --ci-timeout-seconds "$ci_timeout_seconds" --ci-timeout-mode hard --ci-poll-interval-seconds "$ci_poll_interval_seconds")
    if [[ "$create_release" -eq 1 ]]; then
      post_args+=(--create-release)
    fi

    acceptance_out="tooling/release/acceptance/${repo_tag}-receipt.md"
    ./scripts/release_postflight.sh --tag "$repo_tag" --acceptance-out "$acceptance_out" "${post_args[@]}"

    if [[ -f "$acceptance_out" ]]; then
      git add "$acceptance_out"
      if ! git diff --cached --quiet -- "$acceptance_out"; then
        git commit -m "chore: record release ${repo_tag} acceptance"
        git push "$push_remote" "$push_branch"
      fi
    fi
    ;;
  -h|--help|help)
    usage
    ;;
  *)
    echo "[release-automation] unknown mode: $MODE" >&2
    usage
    exit 2
    ;;
esac
