#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
MODE="${1:-}"
DEV_PRERELEASE_BRANCH="dev"
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

sync_generated_distribution_payloads() {
  local repo_tag="$1"

  echo "[release-automation] materialize distribution payloads"
  python3 scripts/materialize_distribution_payloads.py --target all --in-place

  echo "[release-automation] verify release tag version"
  bash scripts/verify_release_tag_version.sh --tag "$repo_tag"
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

Notes:
  - pre  -> runs scripts/release_preflight.sh
  - post -> runs scripts/release_postflight.sh
  - publish is the canonical release entrypoint. Use pre/post only for diagnostics or recovery.
  - pre supports pass-through flags such as --from-tag, --skip-note-gen, --note-overwrite, --skip-smoke, --maintainer-smoke, and --no-strict.
  - publish -> runs release_ready (including pypi_preflight.sh and npm_preflight.sh), syncs generated distribution payloads, commits release-prep files, creates/pushes the tag, waits for branch CI and tag publish workflows, then runs postflight with release-page creation.
  - publish supports --ci-timeout-mode hard|soft. Use soft for beta releases when CI may exceed the local wait window; keep hard for stable releases.
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
        --skip-smoke|--maintainer-smoke|--no-strict|--skip-note-gen|--note-overwrite|--no-build|--no-install-smoke|--keep-dist)
          ready_args+=("$1")
          shift
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
          skip_ci_status=1
          wait_ci=0
          post_args+=("$1")
          shift
          ;;
        --wait-ci)
          wait_ci=1
          shift
          ;;
        --no-wait-ci)
          wait_ci=0
          shift
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

    ./scripts/release_ready.sh --version "$version_input" "${ready_args[@]}"

    sync_generated_distribution_payloads "$repo_tag"

    ensure_git_identity

    git add \
      pyproject.toml \
      docs/reference/skills.md \
      docs/zh/reference/skills.md \
      qiongli/__init__.py \
      qiongli-workflow/VERSION \
      skills/registry.yaml \
      package-lock.json \
      packages/npm-qiongli \
      plugins/qiongli/.codex-plugin/plugin.json \
      plugins/qiongli/.claude-plugin/plugin.json \
      plugins/qiongli/gemini-extension.json \
      skills
    if is_prerelease_tag "$repo_tag"; then
      git add "release/${repo_tag}.md"
    else
      git add CHANGELOG.md
    fi
    if ! git diff --cached --quiet; then
      git commit -m "$commit_message"
    else
      echo "[release-automation] no staged release-prep changes to commit; continuing"
    fi

    if git rev-parse -q --verify "refs/tags/$repo_tag" >/dev/null; then
      echo "[release-automation] tag already exists locally: $repo_tag" >&2
      exit 1
    fi

    git tag -a "$repo_tag" -m "$tag_message"
    git push "$push_remote" "$push_branch" "$repo_tag"

    if [[ "$wait_ci" -eq 1 ]]; then
      post_args+=(--wait-ci --ci-timeout-seconds "$ci_timeout_seconds" --ci-timeout-mode "$ci_timeout_mode" --ci-poll-interval-seconds "$ci_poll_interval_seconds")
    fi
    if [[ "$create_release" -eq 1 ]]; then
      post_args+=(--create-release)
    fi

    acceptance_out="release/acceptance/${repo_tag}-receipt.md"
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
