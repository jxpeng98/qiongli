#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Build a local macOS Qiongli.app for development.

Usage:
  pnpm desktop:macos [-- --open]

Options:
  --open     Open the App after it has been built.
  -h, --help Show this help.

The generated App uses Cargo's release profile and Qiongli's custom-protocol
feature so Tauri serves the embedded frontend instead of its development URL.
This command does not build Windows or Linux packages, run release acceptance,
or add production signing, notarization, update, or packaged-product authority.
EOF
}

profile="release"
open_after_build="false"
for argument in "$@"; do
  case "$argument" in
    --open)
      open_after_build="true"
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      printf 'Unknown option: %s\n\n' "$argument" >&2
      usage >&2
      exit 2
      ;;
  esac
done

if [[ "$(/usr/bin/uname -s)" != "Darwin" ]]; then
  printf 'desktop:macos can only build on macOS.\n' >&2
  exit 1
fi

script_dir="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd -P)"
repo_root="$(CDPATH= cd -- "$script_dir/.." && pwd -P)"
native_manifest="$repo_root/packages/qiongli-native/Cargo.toml"
native_target="$repo_root/packages/qiongli-native/target/$profile/qiongli"
frontend_root="$repo_root/packages/qiongli-desktop"
vite_entry="$frontend_root/node_modules/vite/bin/vite.js"
output_parent="$repo_root/dist/macos"
app="$output_parent/Qiongli.app"

if [[ ! -f "$vite_entry" ]]; then
  printf 'Desktop dependencies are missing; run pnpm install --frozen-lockfile first.\n' >&2
  exit 1
fi

mkdir -p "$output_parent"
stage_parent="$(/usr/bin/mktemp -d "$output_parent/.Qiongli.local.XXXXXX")"
stage="$stage_parent/Qiongli.app"
cleanup() {
  rm -rf -- "$stage_parent"
}
trap cleanup EXIT

printf 'Building the Svelte desktop assets...\n'
(
  cd "$frontend_root"
  node "$vite_entry" build
)

printf 'Building the %s macOS native executable...\n' "$profile"
source_commit=""
if git -C "$repo_root" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  if [[ -z "$(git -C "$repo_root" status --short)" ]]; then
    source_commit="$(git -C "$repo_root" rev-parse --verify HEAD)"
    if [[ ! "$source_commit" =~ ^[0-9a-f]{40}$ ]]; then
      printf 'Could not resolve a valid clean source commit.\n' >&2
      exit 1
    fi
  else
    printf 'Note: the dirty source App will not claim an embedded source commit.\n'
  fi
else
  printf 'Note: the exported source App has no Git commit binding.\n'
fi
cargo_arguments=(
  build
  --manifest-path "$native_manifest"
  --package qiongli
  --bin qiongli
  --locked
  --release
  --features custom-protocol
)
if [[ -n "$source_commit" ]]; then
  QIONGLI_NATIVE_SOURCE_COMMIT="$source_commit" cargo "${cargo_arguments[@]}"
else
  /usr/bin/env -u QIONGLI_NATIVE_SOURCE_COMMIT cargo "${cargo_arguments[@]}"
fi

if [[ ! -f "$native_target" || -L "$native_target" ]]; then
  printf 'Expected native executable was not produced: %s\n' "$native_target" >&2
  exit 1
fi

contents="$stage/Contents"
macos="$contents/MacOS"
resources="$contents/Resources"
mkdir -p "$macos" "$resources"
cp "$native_target" "$macos/Qiongli"
cp "$repo_root/LICENSE" "$resources/LICENSE"
chmod 0755 "$macos/Qiongli"
chmod 0644 "$resources/LICENSE"

product_version="$(/usr/bin/sed -n '/^\[workspace.package\]/,/^\[/s/^version = "\([^"]*\)"/\1/p' "$native_manifest" | /usr/bin/head -n 1)"
bundle_version="${product_version%%-*}"
if [[ -z "$product_version" || ! "$bundle_version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  printf 'Could not resolve a valid workspace package version.\n' >&2
  exit 1
fi

info_plist="$contents/Info.plist"
/usr/bin/plutil -create xml1 "$info_plist"
/usr/bin/plutil -insert CFBundleDevelopmentRegion -string en "$info_plist"
/usr/bin/plutil -insert CFBundleDisplayName -string Qiongli "$info_plist"
/usr/bin/plutil -insert CFBundleExecutable -string Qiongli "$info_plist"
/usr/bin/plutil -insert CFBundleIdentifier -string io.github.jxpeng98.qiongli.local "$info_plist"
/usr/bin/plutil -insert CFBundleInfoDictionaryVersion -string 6.0 "$info_plist"
/usr/bin/plutil -insert CFBundleName -string Qiongli "$info_plist"
/usr/bin/plutil -insert CFBundlePackageType -string APPL "$info_plist"
/usr/bin/plutil -insert CFBundleShortVersionString -string "$bundle_version" "$info_plist"
/usr/bin/plutil -insert CFBundleVersion -string "$bundle_version" "$info_plist"
/usr/bin/plutil -insert NSHighResolutionCapable -bool true "$info_plist"
/usr/bin/plutil -insert QiongliBuildProfile -string "$profile" "$info_plist"
/usr/bin/plutil -insert QiongliProductVersion -string "$product_version" "$info_plist"

# Ad-hoc signing is intentionally local-only. It makes the generated bundle
# launchable without introducing release credentials or notarization state.
/usr/bin/codesign --force --sign - "$stage"
/usr/bin/codesign --verify --deep --strict "$stage"
"$macos/Qiongli" ui --startup-check >/dev/null
content_inventory="$("$macos/Qiongli" content list)"
if [[ "$content_inventory" != *'"pack_id": "qiongli-core"'* \
  || "$content_inventory" != *'"id": "marketplace-lite"'* \
  || "$content_inventory" != *'"id": "full"'* ]]; then
  printf 'The generated App does not contain the expected Qiongli content profiles.\n' >&2
  exit 1
fi

previous="$output_parent/.Qiongli.previous.$$"
if [[ -e "$app" || -L "$app" ]]; then
  mv "$app" "$previous"
fi
if mv "$stage" "$app"; then
  rm -rf -- "$previous"
else
  if [[ -e "$previous" || -L "$previous" ]]; then
    mv "$previous" "$app"
  fi
  exit 1
fi

printf '\nBuilt local macOS App:\n  %s\n' "$app"
printf 'Run it with: open %q\n' "$app"
printf 'Profile: %s; signing: ad-hoc; packaged-product authority: none\n' "$profile"

if [[ "$open_after_build" == "true" ]]; then
  /usr/bin/open "$app"
fi
