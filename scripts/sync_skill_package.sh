#!/usr/bin/env bash
# sync_skill_package.sh — populate distributable research skill packages
# Run before install/release to ensure the skill package is self-contained.
#
# Usage:
#   ./scripts/sync_skill_package.sh [--target pkg|plugin|all] [--dry-run]
#
# Copies canonical source directories into qiongli-workflow/, then mirrors
# that portable package into plugins/qiongli/skills/qiongli-workflow/.
#
# These paths are .gitignore'd — they are generated artifacts, not source of truth.
# The canonical source of truth remains the repo-root directories.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
PKG_DIR="$ROOT_DIR/qiongli-workflow"
PLUGIN_PKG_DIR="$ROOT_DIR/plugins/qiongli/skills/qiongli-workflow"

DRY_RUN=0
TARGET="pkg"
while [[ $# -gt 0 ]]; do
  case "$1" in
    --dry-run)
      DRY_RUN=1
      shift
      ;;
    --target)
      if [[ $# -lt 2 ]]; then
        echo "Missing value for --target" >&2
        exit 1
      fi
      TARGET="$2"
      shift 2
      ;;
    --target=*)
      TARGET="${1#--target=}"
      shift
      ;;
    *)
      echo "Unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

case "$TARGET" in
  pkg|plugin|all) ;;
  *) echo "Unknown target: $TARGET (expected pkg, plugin, or all)" >&2; exit 1 ;;
esac

# ── Sync targets ─────────────────────────────────────────────────────────────

SYNC_DIRS=(
  "skills"
  "templates"
  "standards"
  "roles"
)

SYNC_FILES=(
  "skills-core.md"
  "skills-summary.md"
)

# Exclude files that are project-bootstrap templates, not research output templates
EXCLUDE_FILES=(
  "CLAUDE.project.md"
)

# ── Helpers ──────────────────────────────────────────────────────────────────

sync_dir() {
  local package_dir="$1"
  local rel="$2"
  local src="$ROOT_DIR/$rel"
  local dest="$package_dir/$rel"
  if [[ ! -d "$src" ]]; then
    echo "  [skip] $rel (source not found)"
    return
  fi
  if [[ "$DRY_RUN" -eq 1 ]]; then
    echo "  [dry-run] sync $src/ → $dest/"
    return
  fi
  # Clean copy: remove stale files or symlinks, copy fresh, prune excludes.
  rm -rf "$dest"
  mkdir -p "$(dirname "$dest")"
  cp -a "$src" "$dest"
  for excl in "${EXCLUDE_FILES[@]}"; do
    find "$dest" -name "$excl" -delete 2>/dev/null || true
  done
  find "$dest" -name '.DS_Store' -delete 2>/dev/null || true
  find "$dest" -name '__pycache__' -type d -exec rm -rf {} + 2>/dev/null || true
  echo "  [ok] $rel/"
}

sync_file() {
  local package_dir="$1"
  local rel="$2"
  local src="$ROOT_DIR/$rel"
  local dest="$package_dir/$rel"
  if [[ ! -f "$src" ]]; then
    echo "  [skip] $rel (source not found)"
    return
  fi
  if [[ "$DRY_RUN" -eq 1 ]]; then
    echo "  [dry-run] cp $src → $dest"
    return
  fi
  mkdir -p "$(dirname "$dest")"
  cp "$src" "$dest"
  echo "  [ok] $rel"
}

sync_package() {
  local package_dir="$1"
  if [[ "$DRY_RUN" -eq 1 ]]; then
    echo "Syncing skill package: $package_dir"
  else
    mkdir -p "$package_dir"
    echo "Syncing skill package: $package_dir"
  fi
  for dir in "${SYNC_DIRS[@]}"; do
    sync_dir "$package_dir" "$dir"
  done
  for file in "${SYNC_FILES[@]}"; do
    sync_file "$package_dir" "$file"
  done
}

sync_plugin_package() {
  if [[ "$DRY_RUN" -eq 1 ]]; then
    echo "Syncing skill package: $PLUGIN_PKG_DIR"
    echo "  [dry-run] mirror $PKG_DIR/ → $PLUGIN_PKG_DIR/"
    return
  fi
  echo "Syncing skill package: $PLUGIN_PKG_DIR"
  rm -rf "$PLUGIN_PKG_DIR"
  mkdir -p "$(dirname "$PLUGIN_PKG_DIR")"
  cp -a "$PKG_DIR" "$PLUGIN_PKG_DIR"
  find "$PLUGIN_PKG_DIR" -name '.DS_Store' -delete 2>/dev/null || true
  find "$PLUGIN_PKG_DIR" -name '__pycache__' -type d -exec rm -rf {} + 2>/dev/null || true
  echo "  [ok] mirrored portable package"
}

# ── Main ─────────────────────────────────────────────────────────────────────

if [[ "$TARGET" == "pkg" || "$TARGET" == "all" ]]; then
  sync_package "$PKG_DIR"
fi

if [[ "$TARGET" == "plugin" || "$TARGET" == "all" ]]; then
  sync_plugin_package
fi

echo "[done] Skill package is self-contained."
