#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
TAG=""
DEV_PRERELEASE_BRANCH="dev"
REPO_SLUG=""
SKIP_REMOTE=0
SKIP_CI=0
CREATE_RELEASE=0
ACCEPTANCE_OUT=""
CI_STATUS="unknown"
BRANCH_REQUIRED_WORKFLOWS=("CI" "Checkout Install Check")
TAG_REQUIRED_WORKFLOWS=("Publish to PyPI" "Publish to npm")
WAIT_CI=0
CI_TIMEOUT_SECONDS=1800
CI_TIMEOUT_MODE="hard"
CI_POLL_INTERVAL_SECONDS=30
TEMP_RELEASE_NOTES=""
POSTFLIGHT_STAGING_DIR=""
ACCEPTANCE_EVIDENCE_FILE=""
UPLOAD_ASSETS_FILE=""

cleanup() {
  if [[ -n "$TEMP_RELEASE_NOTES" && -f "$TEMP_RELEASE_NOTES" ]]; then
    rm -f "$TEMP_RELEASE_NOTES"
  fi
  if [[ -n "$POSTFLIGHT_STAGING_DIR" && -d "$POSTFLIGHT_STAGING_DIR" ]]; then
    rm -rf "$POSTFLIGHT_STAGING_DIR"
  fi
  if [[ -n "$ACCEPTANCE_EVIDENCE_FILE" && -f "$ACCEPTANCE_EVIDENCE_FILE" ]]; then
    rm -f "$ACCEPTANCE_EVIDENCE_FILE"
  fi
  if [[ -n "$UPLOAD_ASSETS_FILE" && -f "$UPLOAD_ASSETS_FILE" ]]; then
    rm -f "$UPLOAD_ASSETS_FILE"
  fi
}

trap cleanup EXIT

usage() {
  cat <<'EOF'
Usage:
  ./scripts/release_postflight.sh --tag <tag> [options]

Description:
  Run standardized post-release checks:
    1) verify local/remote tag consistency
    2) verify release docs (stable uses CHANGELOG.md plus generated download guidance,
       prerelease uses tooling/release/<tag>.md) + rollback docs
    3) optionally check branch CI and tag publish status on GitHub Actions
    4) generate release acceptance receipt from template

Options:
  --tag <tag>           Required release tag (for example v0.1.0 or v0.1.1-beta.1).
  --repo <owner/repo>   Optional GitHub repo slug. Auto-derived from origin if omitted.
  --skip-remote         Skip remote ref checks.
  --skip-ci-status      Skip GitHub Actions status check.
  --wait-ci             Wait for branch checks and tag publish workflows to complete successfully.
  --ci-timeout-seconds <n>
                        Max time to wait for CI when --wait-ci is enabled (default: 1800).
  --ci-timeout-mode <hard|soft>
                        hard fails on unresolved CI after timeout; soft records pending/skipped and continues (default: hard).
  --ci-poll-interval-seconds <n>
                        Poll interval for CI checks when --wait-ci is enabled (default: 30).
  --create-release      Create GitHub release if missing and gh auth is available.
  --acceptance-out <p>  Output path for acceptance receipt (default: tooling/release/acceptance/<tag>-receipt.md).
  -h, --help            Show this message.
EOF
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

detect_primary_branch() {
  local candidate
  for candidate in main master; do
    if detect_branch_ref "$candidate"; then
      return 0
    fi
  done
  return 1
}

detect_branch_ref() {
  local branch="$1"
  if git show-ref --verify --quiet "refs/heads/$branch"; then
    printf '%s\t%s\n' "$branch" "$branch"
    return 0
  fi
  if git show-ref --verify --quiet "refs/remotes/origin/$branch"; then
    printf '%s\trefs/remotes/origin/%s\n' "$branch" "$branch"
    return 0
  fi
  return 1
}

refresh_branch_ref() {
  local branch="$1"
  local ref="$2"
  local fetch_ref=""

  if [[ "$ref" == refs/remotes/origin/* ]]; then
    fetch_ref="+refs/heads/${branch}:refs/remotes/origin/${branch}"
  elif [[ "$ref" == "$branch" ]]; then
    fetch_ref="+refs/heads/${branch}:refs/heads/${branch}"
  else
    return 0
  fi

  if git fetch --force --no-tags origin "$fetch_ref" >/dev/null 2>&1; then
    echo "[postflight] refreshed $branch ref from origin"
    return 0
  fi

  echo "[postflight] warning: unable to refresh $branch ref from origin; using existing local ref"
  return 0
}

is_prerelease_tag() {
  [[ "$1" == *beta* || "$1" =~ b[0-9]+ ]]
}

select_release_branch_ref() {
  local tag="$1"
  local commit="$2"
  local branch_ref=""
  local branch=""
  local ref=""

  if is_prerelease_tag "$tag" && branch_ref="$(detect_branch_ref "$DEV_PRERELEASE_BRANCH")"; then
    branch="${branch_ref%%$'\t'*}"
    ref="${branch_ref#*$'\t'}"
    refresh_branch_ref "$branch" "$ref" >/dev/null
    if git merge-base --is-ancestor "$commit" "$ref"; then
      printf '%s\n' "$branch_ref"
      return 0
    fi
  fi

  detect_primary_branch
}

prepare_release_notes_file() {
  local tag="$1"
  local version="${tag#v}"

  if is_prerelease_tag "$tag"; then
    RELEASE_NOTES_FILE="tooling/release/${tag}.md"
    RELEASE_NOTES_LABEL="$RELEASE_NOTES_FILE"
    [[ -f "$RELEASE_NOTES_FILE" ]] || {
      echo "[postflight] missing prerelease notes: $RELEASE_NOTES_FILE" >&2
      exit 1
    }
    return 0
  fi

  [[ -f "CHANGELOG.md" ]] || {
    echo "[postflight] missing stable release changelog: CHANGELOG.md" >&2
    exit 1
  }

  TEMP_RELEASE_NOTES="$(mktemp -t qiongli-release-notes.XXXXXX.md)"
  python3 scripts/generate_stable_release_notes.py \
    --tag "$tag" \
    --repo "${REPO_SLUG:-jxpeng98/qiongli}" \
    --output "$TEMP_RELEASE_NOTES"
  RELEASE_NOTES_FILE="$TEMP_RELEASE_NOTES"
  RELEASE_NOTES_LABEL="stable notes: CHANGELOG.md [${version}] + download guide"
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

prepare_platform_dist_source() {
  local channel="$1"
  local slug="$2"
  local source_dir="$3"
  local out_root="$4"
  local platform_source="$out_root/$channel/$slug"

  if [[ ! -d "$source_dir" ]]; then
    echo "[postflight] missing $channel dist payload: $source_dir" >&2
    exit 1
  fi

  mkdir -p "$(dirname "$platform_source")"
  cp -R "$source_dir" "$platform_source"

  if [[ "$channel" == "codex" ]]; then
    rm -rf "$platform_source/.claude-plugin"
  elif [[ "$channel" == "claude" ]]; then
    rm -rf "$platform_source/.codex-plugin" "$platform_source/.mcp.json"
    if [[ -d "$platform_source/skills" ]]; then
      find "$platform_source/skills" -mindepth 1 -maxdepth 1 -type d -name "${slug}-*" ! -name "qiongli-workflow" -exec rm -rf {} +
    fi
  else
    echo "[postflight] unsupported dist ref channel: $channel" >&2
    exit 1
  fi

  printf '%s\n' "$platform_source"
}

publish_platform_dist_ref() {
  local channel="$1"
  local platform_slug="$2"
  local platform_source="$3"

  echo "[postflight] publishing $channel dist ref: $channel/${TAG}"
  node scripts/publish-codex-dist-ref.mjs \
    --channel "$channel" \
    --version "${TAG#v}" \
    --slug "$platform_slug" \
    --source "$platform_source"
}

publish_plugin_dist_refs() {
  local tag="$1"
  local codex_slug="qiongli"
  local claude_slug="qiongli"
  local platform_work_root="$POSTFLIGHT_STAGING_DIR/.platform-dist-refs"
  local platform_slug platform_source

  if is_prerelease_tag "$tag"; then
    codex_slug="qiongli-next"
    claude_slug="qiongli-next"
  fi

  platform_slug="$codex_slug"
  platform_source="$(prepare_platform_dist_source codex "$platform_slug" "$POSTFLIGHT_STAGING_DIR/plugins/$platform_slug" "$platform_work_root")"
  publish_platform_dist_ref codex "$platform_slug" "$platform_source"

  platform_slug="$claude_slug"
  platform_source="$(prepare_platform_dist_source claude "$platform_slug" "$POSTFLIGHT_STAGING_DIR/plugins/$platform_slug" "$platform_work_root")"
  publish_platform_dist_ref claude "$platform_slug" "$platform_source"
}

native_plugin_dist_ref_policy() {
  local tag="$1"
  local plugin_slug="qiongli"
  local identity_path

  if is_prerelease_tag "$tag"; then
    plugin_slug="qiongli-next"
  fi
  identity_path="$POSTFLIGHT_STAGING_DIR/plugins/$plugin_slug/bin/qiongli-literature-provider.target.json"
  python3 - "$identity_path" <<'PY'
import json
from pathlib import Path
import sys

identity_path = Path(sys.argv[1])
if not identity_path.is_file():
    raise SystemExit(f"missing native target identity: {identity_path}")
identity = json.loads(identity_path.read_text(encoding="utf-8"))
policy = identity.get("target_policy")
if not isinstance(policy, str) or not policy:
    raise SystemExit(f"native target identity has no target_policy: {identity_path}")
print(policy)
PY
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --tag)
      [[ $# -ge 2 ]] || { echo "[postflight] missing value for --tag" >&2; exit 2; }
      TAG="$2"
      shift 2
      ;;
    --repo)
      [[ $# -ge 2 ]] || { echo "[postflight] missing value for --repo" >&2; exit 2; }
      REPO_SLUG="$2"
      shift 2
      ;;
    --skip-remote)
      SKIP_REMOTE=1
      shift
      ;;
    --skip-ci-status)
      SKIP_CI=1
      shift
      ;;
    --wait-ci)
      WAIT_CI=1
      shift
      ;;
    --ci-timeout-seconds)
      [[ $# -ge 2 ]] || { echo "[postflight] missing value for --ci-timeout-seconds" >&2; exit 2; }
      CI_TIMEOUT_SECONDS="$2"
      shift 2
      ;;
    --ci-timeout-mode)
      [[ $# -ge 2 ]] || { echo "[postflight] missing value for --ci-timeout-mode" >&2; exit 2; }
      CI_TIMEOUT_MODE="$2"
      [[ "$CI_TIMEOUT_MODE" == "hard" || "$CI_TIMEOUT_MODE" == "soft" ]] || {
        echo "[postflight] --ci-timeout-mode must be hard or soft" >&2
        exit 2
      }
      shift 2
      ;;
    --ci-poll-interval-seconds)
      [[ $# -ge 2 ]] || { echo "[postflight] missing value for --ci-poll-interval-seconds" >&2; exit 2; }
      CI_POLL_INTERVAL_SECONDS="$2"
      shift 2
      ;;
    --create-release)
      CREATE_RELEASE=1
      shift
      ;;
    --acceptance-out)
      [[ $# -ge 2 ]] || { echo "[postflight] missing value for --acceptance-out" >&2; exit 2; }
      ACCEPTANCE_OUT="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "[postflight] unknown option: $1" >&2
      usage
      exit 2
      ;;
  esac
done

[[ -n "$TAG" ]] || { echo "[postflight] --tag is required" >&2; usage; exit 2; }

cd "$ROOT_DIR"

if ! LOCAL_TAG_COMMIT="$(git rev-parse "$TAG^{}" 2>/dev/null)"; then
  echo "[postflight] local tag not found: $TAG" >&2
  exit 1
fi
echo "[postflight] local tag commit: $LOCAL_TAG_COMMIT"

POSTFLIGHT_STAGING_DIR="$(mktemp -d "${TMPDIR:-/tmp}/qiongli-postflight.XXXXXX")"
echo "[postflight] materialize distribution payloads"
python3 scripts/materialize_distribution_payloads.py --target all --out "$POSTFLIGHT_STAGING_DIR" --force

bash ./scripts/verify_release_tag_version.sh --root "$POSTFLIGHT_STAGING_DIR" --tag "$TAG"

if ! release_branch_record="$(select_release_branch_ref "$TAG" "$LOCAL_TAG_COMMIT")"; then
  echo "[postflight] unable to detect release branch (expected dev for reachable prerelease tags, or main/master locally or under origin/)" >&2
  exit 1
fi
RELEASE_BRANCH="${release_branch_record%%$'\t'*}"
RELEASE_BRANCH_REF="${release_branch_record#*$'\t'}"

refresh_branch_ref "$RELEASE_BRANCH" "$RELEASE_BRANCH_REF"

if git merge-base --is-ancestor "$LOCAL_TAG_COMMIT" "$RELEASE_BRANCH_REF"; then
  echo "[postflight] tag commit is reachable from $RELEASE_BRANCH"
else
  echo "[postflight] tag commit is not reachable from $RELEASE_BRANCH" >&2
  exit 1
fi

ROLLBACK_PATH="tooling/release/rollback.md"
TEMPLATE_PATH="tooling/release/templates/beta-acceptance-template.md"

[[ -f "$ROLLBACK_PATH" ]] || { echo "[postflight] missing rollback doc: $ROLLBACK_PATH" >&2; exit 1; }
[[ -f "$TEMPLATE_PATH" ]] || { echo "[postflight] missing acceptance template: $TEMPLATE_PATH" >&2; exit 1; }
if [[ -z "$REPO_SLUG" ]]; then
  REPO_SLUG="$(derive_repo_slug || true)"
fi
prepare_release_notes_file "$TAG"
echo "[postflight] release docs present: $RELEASE_NOTES_LABEL"

if [[ "$SKIP_REMOTE" -eq 0 ]]; then
  if REMOTE_BRANCH="$(git ls-remote --heads origin "$RELEASE_BRANCH" 2>/dev/null | awk '{print $1}' | head -n 1)" \
    && REMOTE_TAG="$(git ls-remote --tags origin "${TAG}^{}" 2>/dev/null | awk '{print $1}' | head -n 1)"; then
    [[ -n "$REMOTE_BRANCH" ]] || { echo "[postflight] remote branch not found: $RELEASE_BRANCH" >&2; exit 1; }
    [[ -n "$REMOTE_TAG" ]] || { echo "[postflight] remote tag not found: $TAG" >&2; exit 1; }
    [[ "$REMOTE_TAG" == "$LOCAL_TAG_COMMIT" ]] || {
      echo "[postflight] remote tag commit mismatch: local=$LOCAL_TAG_COMMIT remote=$REMOTE_TAG" >&2
      exit 1
    }
    echo "[postflight] remote refs verified (branch=$RELEASE_BRANCH, tag=$TAG)"
  else
    echo "[postflight] warning: remote check skipped (network/auth unavailable)"
    SKIP_REMOTE=1
  fi
else
  echo "[postflight] remote check skipped by flag"
fi

if [[ -z "$REPO_SLUG" ]]; then
  REPO_SLUG="$(derive_repo_slug || true)"
fi

if [[ "$SKIP_CI" -eq 0 ]]; then
  if [[ -z "$REPO_SLUG" ]]; then
    if [[ "$WAIT_CI" -eq 1 ]]; then
      if [[ "$CI_TIMEOUT_MODE" == "soft" ]]; then
        echo "[postflight] warning: cannot derive repo slug while --wait-ci is enabled; continuing because --ci-timeout-mode=soft"
        CI_STATUS="skipped:query-unavailable"
      else
        echo "[postflight] cannot derive repo slug while --wait-ci is enabled" >&2
        exit 1
      fi
    else
      echo "[postflight] warning: cannot derive repo slug, skip GitHub Actions status check"
      CI_STATUS="skipped"
    fi
  else
    if [[ "$WAIT_CI" -eq 1 ]]; then
      wait_deadline=$((SECONDS + CI_TIMEOUT_SECONDS))
      while true; do
        set +e
        BRANCH_CI_RESULT="$(query_actions_status "$REPO_SLUG" "$RELEASE_BRANCH" "$LOCAL_TAG_COMMIT" "${BRANCH_REQUIRED_WORKFLOWS[@]}")"
        BRANCH_CI_EXIT=$?
        TAG_PUBLISH_RESULT="$(query_actions_status "$REPO_SLUG" "$TAG" "$LOCAL_TAG_COMMIT" "${TAG_REQUIRED_WORKFLOWS[@]}")"
        TAG_PUBLISH_EXIT=$?
        set -e

        if [[ "$BRANCH_CI_EXIT" -ne 0 ]]; then
          CI_STATUS="failed"
          echo "[postflight] branch checks failed for release commit: $BRANCH_CI_RESULT" >&2
          exit 1
        fi
        if [[ "$TAG_PUBLISH_EXIT" -ne 0 ]]; then
          CI_STATUS="failed"
          echo "[postflight] tag publish workflows failed for release commit: $TAG_PUBLISH_RESULT" >&2
          exit 1
        fi

        if [[ "$BRANCH_CI_RESULT" == success:* && "$TAG_PUBLISH_RESULT" == success:* ]]; then
          CI_STATUS="success:branch-and-tag"
          echo "[postflight] branch checks: $BRANCH_CI_RESULT"
          echo "[postflight] tag publish workflows: $TAG_PUBLISH_RESULT"
          break
        fi

        if [[ "$BRANCH_CI_RESULT" == skipped:* || "$TAG_PUBLISH_RESULT" == skipped:* ]]; then
          if [[ "$CI_TIMEOUT_MODE" == "soft" ]]; then
            CI_STATUS="skipped:query-unavailable"
            echo "[postflight] warning: unable to query GitHub Actions status while --wait-ci is enabled; continuing because --ci-timeout-mode=soft: branch=$BRANCH_CI_RESULT tag=$TAG_PUBLISH_RESULT"
            break
          fi
          CI_STATUS="skipped"
          echo "[postflight] unable to query GitHub Actions status while --wait-ci is enabled: branch=$BRANCH_CI_RESULT tag=$TAG_PUBLISH_RESULT" >&2
          exit 1
        fi

        CI_STATUS="pending"
        if (( SECONDS >= wait_deadline )); then
          CI_STATUS="pending:timeout-after-${CI_TIMEOUT_SECONDS}s"
          if [[ "$CI_TIMEOUT_MODE" == "soft" ]]; then
            echo "[postflight] warning: timed out waiting for branch checks and tag publish workflows; continuing because --ci-timeout-mode=soft: branch=$BRANCH_CI_RESULT tag=$TAG_PUBLISH_RESULT"
            break
          fi
          echo "[postflight] timed out waiting for branch checks and tag publish workflows: branch=$BRANCH_CI_RESULT tag=$TAG_PUBLISH_RESULT" >&2
          exit 1
        fi
        echo "[postflight] waiting for branch checks and tag publish workflows: branch=$BRANCH_CI_RESULT tag=$TAG_PUBLISH_RESULT"
        sleep "$CI_POLL_INTERVAL_SECONDS"
      done
    else
      set +e
      BRANCH_CI_RESULT="$(query_actions_status "$REPO_SLUG" "$RELEASE_BRANCH" "$LOCAL_TAG_COMMIT" "${BRANCH_REQUIRED_WORKFLOWS[@]}")"
      BRANCH_CI_EXIT=$?
      TAG_PUBLISH_RESULT="$(query_actions_status "$REPO_SLUG" "$TAG" "$LOCAL_TAG_COMMIT" "${TAG_REQUIRED_WORKFLOWS[@]}")"
      TAG_PUBLISH_EXIT=$?
      set -e

      if [[ "$BRANCH_CI_EXIT" -eq 0 && "$TAG_PUBLISH_EXIT" -eq 0 ]]; then
        if [[ "$BRANCH_CI_RESULT" == success:* && "$TAG_PUBLISH_RESULT" == success:* ]]; then
          CI_STATUS="success:branch-and-tag"
          echo "[postflight] branch checks: $BRANCH_CI_RESULT"
          echo "[postflight] tag publish workflows: $TAG_PUBLISH_RESULT"
        elif [[ "$BRANCH_CI_RESULT" == skipped:* || "$TAG_PUBLISH_RESULT" == skipped:* ]]; then
          CI_STATUS="skipped"
          echo "[postflight] warning: unable to query GitHub Actions status (branch=$BRANCH_CI_RESULT tag=$TAG_PUBLISH_RESULT)"
        else
          CI_STATUS="pending"
          echo "[postflight] warning: GitHub Actions status unresolved: branch=$BRANCH_CI_RESULT tag=$TAG_PUBLISH_RESULT"
        fi
      else
        CI_STATUS="failed"
        echo "[postflight] GitHub Actions failed for release commit: branch=$BRANCH_CI_RESULT tag=$TAG_PUBLISH_RESULT" >&2
        exit 1
      fi
    fi
  fi
else
  CI_STATUS="skipped"
  echo "[postflight] GitHub Actions status check skipped by flag"
fi

python3 scripts/build_plugin_artifacts.py --root "$POSTFLIGHT_STAGING_DIR" --tag "$TAG" --dist-dir dist
python3 scripts/build_literature_mcpb.py --dist-dir dist >/dev/null
python3 scripts/build_zotero_companion.py --dist-dir dist >/dev/null
python3 scripts/generate_release_downloads.py --tag "$TAG" --out-dir dist
UPLOAD_ASSETS_FILE="$(mktemp -t qiongli-upload-assets.XXXXXX.txt)"
python3 scripts/release_upload_assets.py --tag "$TAG" --dist-dir dist >"$UPLOAD_ASSETS_FILE"
mapfile -t PLUGIN_ARTIFACTS <"$UPLOAD_ASSETS_FILE"

NATIVE_PLUGIN_DIST_REF_POLICY="$(native_plugin_dist_ref_policy "$TAG")"
if [[ "$NATIVE_PLUGIN_DIST_REF_POLICY" == "multi-target" ]]; then
  publish_plugin_dist_refs "$TAG"
else
  echo "[postflight] generic plugin dist refs skipped: native policy is $NATIVE_PLUGIN_DIST_REF_POLICY"
  echo "[postflight] target-identified release assets remain available from the GitHub release"
fi

if ! command -v gh >/dev/null 2>&1 || ! gh auth status >/dev/null 2>&1; then
  echo "[postflight] gh auth is required to verify or create the GitHub release page" >&2
  exit 1
fi

if [[ -z "$REPO_SLUG" ]]; then
  echo "[postflight] unable to derive GitHub repo slug for release-page checks" >&2
  exit 1
fi

IS_PRERELEASE=0
if is_prerelease_tag "$TAG"; then
  IS_PRERELEASE=1
fi

if release_json="$(gh release view "$TAG" --repo "$REPO_SLUG" --json isDraft,isPrerelease,url 2>/dev/null)"; then
  set +e
  release_state="$(RELEASE_JSON="$release_json" python3 - "$IS_PRERELEASE" <<'PY'
import json
import os
import sys

expected_prerelease = sys.argv[1] == "1"
payload = json.loads(os.environ["RELEASE_JSON"])
is_draft = bool(payload.get("isDraft"))
is_prerelease = bool(payload.get("isPrerelease"))
url = payload.get("url") or ""
if is_draft:
    print(f"draft:{url}")
    raise SystemExit(1)
if is_prerelease != expected_prerelease:
    kind = "prerelease" if is_prerelease else "stable"
    expected = "prerelease" if expected_prerelease else "stable"
    print(f"mismatch:{kind}:{expected}:{url}")
    raise SystemExit(1)
print(f"ok:{url}")
PY
  )"
  release_state_exit=$?
  set -e
  if [[ "$release_state_exit" -ne 0 ]]; then
    echo "[postflight] invalid GitHub release state for $TAG: $release_state" >&2
    exit 1
  fi
  echo "[postflight] GitHub release exists: ${release_state#ok:}"
  gh release upload "$TAG" --repo "$REPO_SLUG" --clobber "${PLUGIN_ARTIFACTS[@]}"
  echo "[postflight] plugin artifacts uploaded to existing release"
elif [[ "$CREATE_RELEASE" -eq 1 ]]; then
  release_args=(
    "$TAG"
    --repo "$REPO_SLUG"
    --title "$TAG"
    --notes-file "$RELEASE_NOTES_FILE"
  )
  if [[ "$IS_PRERELEASE" -eq 1 ]]; then
    release_args+=(--prerelease)
  fi
  release_args+=("${PLUGIN_ARTIFACTS[@]}")
  gh release create "${release_args[@]}"
  if [[ "$IS_PRERELEASE" -eq 1 ]]; then
    echo "[postflight] GitHub prerelease created: $TAG"
  else
    echo "[postflight] GitHub release created: $TAG"
  fi
else
  echo "[postflight] missing GitHub release page for $TAG (rerun with --create-release)" >&2
  exit 1
fi

if [[ -z "$ACCEPTANCE_OUT" ]]; then
  ACCEPTANCE_OUT="tooling/release/acceptance/${TAG}-receipt.md"
fi
mkdir -p "$(dirname "$ACCEPTANCE_OUT")"

ACCEPTANCE_EVIDENCE_FILE="$(mktemp -t qiongli-acceptance-evidence.XXXXXX.md)"
python3 scripts/release_acceptance_evidence.py --root "$ROOT_DIR" --out "$ACCEPTANCE_EVIDENCE_FILE"

RELEASE_DATE="$(date +%F)"
DOWNLOAD_INDEX="dist/qiongli-downloads-${TAG}.json"
python3 - "$TEMPLATE_PATH" "$ACCEPTANCE_OUT" "$TAG" "$RELEASE_DATE" "$LOCAL_TAG_COMMIT" "$CI_STATUS" "$ACCEPTANCE_EVIDENCE_FILE" "$DOWNLOAD_INDEX" <<'PY'
import json
from pathlib import Path
import sys

template_path, out_path, tag, date, commit, ci_status, evidence_path, index_path = sys.argv[1:]
with open(template_path, "r", encoding="utf-8") as f:
    content = f.read()
evidence = Path(evidence_path)
subject_runtime_evidence = evidence.read_text(encoding="utf-8")
release_index = json.loads(Path(index_path).read_text(encoding="utf-8"))
components = release_index.get("component_versions")
if not isinstance(components, dict) or not components:
    raise SystemExit(f"release index has no component_versions: {index_path}")
component_version_map = "\n".join(
    [
        "| Component | Version | Runtime / target | Source |",
        "|---|---|---|---|",
        *(
            "| {name} | `{version}` | {runtime} | `{source}` |".format(
                name=name,
                version=entry.get("version", "unknown"),
                runtime=" / ".join(
                    value
                    for value in (
                        entry.get("runtime_profile"),
                        entry.get("runtime_implementation"),
                        entry.get("native_target"),
                    )
                    if isinstance(value, str) and value
                )
                or "not applicable",
                source=entry.get("source", "unknown"),
            )
            for name, entry in components.items()
            if isinstance(entry, dict)
        ),
    ]
)
content = (
    content.replace("{{TAG}}", tag)
    .replace("{{DATE}}", date)
    .replace("{{COMMIT}}", commit)
    .replace("{{CI_STATUS}}", ci_status)
    .replace("{{COMPONENT_VERSION_MAP}}", component_version_map)
    .replace("{{SUBJECT_RUNTIME_EVIDENCE}}", subject_runtime_evidence)
)
with open(out_path, "w", encoding="utf-8") as f:
    f.write(content)
PY
echo "[postflight] acceptance receipt generated: $ACCEPTANCE_OUT"
echo "[postflight] completed"
