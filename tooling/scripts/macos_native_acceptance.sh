#!/bin/bash
set -euo pipefail

umask 077

usage() {
  cat <<'EOF'
Usage: macos_native_acceptance.sh \
  --artifact-dir ABSOLUTE_PATH \
  --expected-source-commit HEX \
  --expected-package-sha256 HEX \
  --output ABSOLUTE_NEW_JSON_PATH \
  [--launchservices-preflight]

Verify one exact Qiongli 2 alpha macOS package, run its packaged launcher with
an isolated HOME and empty PATH, and write a path-redacted non-publishing
acceptance receipt. The optional LaunchServices preflight opens the bundle with
the fixed internal --startup-check; it does not claim that a window was shown.

This command never signs, notarizes, publishes, installs, or changes client
configuration. Clean-machine, displayed-window, scale, VoiceOver, contrast,
and production-signing gates remain explicitly open.
EOF
}

fail() {
  printf 'macOS native acceptance failed: %s\n' "$1" >&2
  exit 1
}

valid_absolute_path() {
  case "$1" in
    /*) ;;
    *) return 1 ;;
  esac
  case "/$1/" in
    */../*|*/./*) return 1 ;;
  esac
}

valid_lower_hex() {
  local value="$1"
  local expected_length="$2"
  [[ "${#value}" -eq "$expected_length" ]] || return 1
  case "$value" in
    *[!0-9a-f]*) return 1 ;;
  esac
}

sha256_file() {
  /usr/bin/shasum -a 256 -- "$1" | /usr/bin/awk '{print $1}'
}

plist_raw() {
  local key="$1"
  local expected_type="$2"
  local file="$3"
  /usr/bin/plutil -extract "$key" raw -expect "$expected_type" "$file" 2>/dev/null
}

insert_string() {
  local file="$1"
  local key="$2"
  local value="$3"
  /usr/bin/plutil -insert "$key" -string "$value" -s "$file"
}

artifact_dir=""
expected_source_commit=""
expected_package_sha256=""
output=""
launchservices_preflight="false"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --artifact-dir)
      [[ $# -ge 2 ]] || { usage >&2; exit 2; }
      artifact_dir="$2"
      shift 2
      ;;
    --expected-source-commit)
      [[ $# -ge 2 ]] || { usage >&2; exit 2; }
      expected_source_commit="$2"
      shift 2
      ;;
    --expected-package-sha256)
      [[ $# -ge 2 ]] || { usage >&2; exit 2; }
      expected_package_sha256="$2"
      shift 2
      ;;
    --output)
      [[ $# -ge 2 ]] || { usage >&2; exit 2; }
      output="$2"
      shift 2
      ;;
    --launchservices-preflight)
      launchservices_preflight="true"
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      usage >&2
      exit 2
      ;;
  esac
done

[[ "$(/usr/bin/uname -s)" == "Darwin" ]] || fail "unsupported-host"
[[ -n "$artifact_dir" && -n "$expected_source_commit" && -n "$expected_package_sha256" && -n "$output" ]] || {
  usage >&2
  exit 2
}
valid_absolute_path "$artifact_dir" || fail "artifact-directory-invalid"
valid_absolute_path "$output" || fail "output-path-invalid"
[[ -d "$artifact_dir" && ! -L "$artifact_dir" ]] || fail "artifact-directory-invalid"
[[ ! -e "$output" && ! -L "$output" ]] || fail "output-path-exists"
output_parent="$(/usr/bin/dirname "$output")"
[[ -d "$output_parent" && ! -L "$output_parent" ]] || fail "output-parent-invalid"
case "${#expected_source_commit}" in
  40|64) valid_lower_hex "$expected_source_commit" "${#expected_source_commit}" || fail "source-commit-invalid" ;;
  *) fail "source-commit-invalid" ;;
esac
valid_lower_hex "$expected_package_sha256" 64 || fail "expected-package-digest-invalid"

manifest="$artifact_dir/qiongli-desktop-package.manifest.json"
package_receipt="$artifact_dir/qiongli-desktop-package.receipt.json"
[[ -f "$manifest" && ! -L "$manifest" ]] || fail "package-manifest-invalid"
[[ -f "$package_receipt" && ! -L "$package_receipt" ]] || fail "package-receipt-invalid"

[[ "$(plist_raw schema_version integer "$package_receipt")" == "2" ]] || fail "package-receipt-schema-invalid"
[[ "$(plist_raw status string "$package_receipt")" == "assembled-unpublished" ]] || fail "package-status-invalid"
[[ "$(plist_raw product_source_commit string "$package_receipt")" == "$expected_source_commit" ]] || fail "package-source-mismatch"
[[ "$(plist_raw package_manifest_file string "$package_receipt")" == "qiongli-desktop-package.manifest.json" ]] || fail "package-manifest-file-invalid"
package_file="$(plist_raw package_file string "$package_receipt")"
product_version="$(plist_raw artifact.version string "$manifest")"
[[ "$product_version" =~ ^2\.[0-9]+\.[0-9]+-alpha\.[0-9]+$ ]] ||
  fail "manifest-version-invalid"
[[ "$package_file" == "Qiongli-$product_version-macOS-arm64.source.zip" ]] ||
  fail "package-file-invalid"
archive="$artifact_dir/$package_file"
[[ -f "$archive" && ! -L "$archive" ]] || fail "package-archive-invalid"

actual_package_sha256="$(sha256_file "$archive")"
valid_lower_hex "$actual_package_sha256" 64 || fail "package-digest-invalid"
[[ "$actual_package_sha256" == "$expected_package_sha256" ]] || fail "package-digest-mismatch"
[[ "$(plist_raw package_sha256 string "$package_receipt")" == "$actual_package_sha256" ]] || fail "package-receipt-digest-mismatch"
actual_package_size="$(/usr/bin/stat -f '%z' "$archive")"
[[ "$(plist_raw package_size_bytes integer "$package_receipt")" == "$actual_package_size" ]] || fail "package-receipt-size-mismatch"

actual_manifest_sha256="$(sha256_file "$manifest")"
valid_lower_hex "$actual_manifest_sha256" 64 || fail "manifest-digest-invalid"
[[ "$(plist_raw package_manifest_sha256 string "$package_receipt")" == "$actual_manifest_sha256" ]] || fail "package-manifest-digest-mismatch"
[[ "$(plist_raw product_source_commit string "$manifest")" == "$expected_source_commit" ]] || fail "manifest-source-mismatch"
[[ "$(plist_raw status string "$manifest")" == "assembled-unpublished" ]] || fail "manifest-status-invalid"
[[ "$(plist_raw package_kind string "$manifest")" == "macos-application-zip" ]] || fail "manifest-kind-invalid"
[[ "$(plist_raw artifact.product string "$manifest")" == "qiongli" ]] || fail "manifest-product-invalid"
[[ "$(plist_raw artifact.version string "$manifest")" == "$product_version" ]] || fail "manifest-version-invalid"
[[ "$(plist_raw artifact.channel string "$manifest")" == "alpha" ]] || fail "manifest-channel-invalid"
[[ "$(plist_raw artifact.profile string "$manifest")" == "lite" ]] || fail "manifest-profile-invalid"
[[ "$(plist_raw artifact.os string "$manifest")" == "macos" ]] || fail "manifest-os-invalid"
[[ "$(plist_raw artifact.arch string "$manifest")" == "aarch64" ]] || fail "manifest-architecture-invalid"
[[ "$(plist_raw artifact.installer_kind string "$manifest")" == "native-installer" ]] || fail "manifest-installer-invalid"
[[ "$(plist_raw source_artifact.product string "$manifest")" == "qiongli" ]] || fail "source-artifact-product-invalid"
[[ "$(plist_raw source_artifact.version string "$manifest")" == "$product_version" ]] || fail "source-artifact-version-invalid"
[[ "$(plist_raw source_artifact.channel string "$manifest")" == "alpha" ]] || fail "source-artifact-channel-invalid"
[[ "$(plist_raw source_artifact.profile string "$manifest")" == "lite" ]] || fail "source-artifact-profile-invalid"
[[ "$(plist_raw source_artifact.os string "$manifest")" == "macos" ]] || fail "source-artifact-os-invalid"
[[ "$(plist_raw source_artifact.arch string "$manifest")" == "aarch64" ]] || fail "source-artifact-architecture-invalid"
[[ "$(plist_raw source_artifact.installer_kind string "$manifest")" == "portable-archive" ]] || fail "source-artifact-installer-invalid"
[[ "$(plist_raw application.application_identifier string "$manifest")" == "io.github.jxpeng98.qiongli" ]] || fail "manifest-application-invalid"

host_arch="$(/usr/bin/uname -m)"
[[ "$host_arch" == "arm64" ]] || fail "host-architecture-unsupported"

stage="$(/usr/bin/mktemp -d "$output_parent/.qiongli-macos-acceptance.XXXXXX")"
cleanup() {
  /bin/rm -rf -- "$stage"
}
trap cleanup EXIT HUP INT TERM
/bin/mkdir -m 700 "$stage/extracted" "$stage/home"
/usr/bin/ditto -x -k "$archive" "$stage/extracted"

app="$stage/extracted/Qiongli.app"
launcher="$app/Contents/MacOS/Qiongli"
canonical="$app/Contents/MacOS/qiongli-cli"
update_helper="$app/Contents/MacOS/qiongli-update-helper"
info_plist="$app/Contents/Info.plist"
internal_manifest="$app/Contents/Resources/.qiongli-desktop-package.json"
[[ -d "$app" && ! -L "$app" ]] || fail "application-bundle-invalid"
[[ -f "$launcher" && -x "$launcher" && ! -L "$launcher" ]] || fail "application-launcher-invalid"
[[ -f "$canonical" && -x "$canonical" && ! -L "$canonical" ]] || fail "application-canonical-binary-invalid"
[[ -f "$update_helper" && -x "$update_helper" && ! -L "$update_helper" ]] || fail "application-update-helper-invalid"
[[ -f "$info_plist" && ! -L "$info_plist" ]] || fail "application-info-invalid"
[[ -f "$internal_manifest" && ! -L "$internal_manifest" ]] || fail "application-manifest-invalid"
/usr/bin/cmp -s "$manifest" "$internal_manifest" || fail "application-manifest-mismatch"
[[ "$(sha256_file "$launcher")" == "$(plist_raw launcher_sha256 string "$manifest")" ]] || fail "application-launcher-digest-mismatch"
[[ "$(sha256_file "$canonical")" == "$(plist_raw canonical_binary_sha256 string "$manifest")" ]] || fail "application-canonical-digest-mismatch"
[[ "$(sha256_file "$update_helper")" == "$(plist_raw update_helper_sha256 string "$manifest")" ]] || fail "application-update-helper-digest-mismatch"
[[ "$(plist_raw CFBundleIdentifier string "$info_plist")" == "io.github.jxpeng98.qiongli" ]] || fail "application-bundle-identifier-invalid"
[[ "$(plist_raw QiongliProductVersion string "$info_plist")" == "$product_version" ]] || fail "application-product-version-invalid"

HOME="$stage/home" PATH="" "$launcher" --startup-check >"$stage/startup.stdout" 2>"$stage/startup.stderr" || fail "empty-path-startup-failed"
[[ "$(/usr/bin/stat -f '%z' "$stage/startup.stdout")" -le 65536 ]] || fail "startup-output-too-large"
[[ "$(/usr/bin/stat -f '%z' "$stage/startup.stderr")" -le 65536 ]] || fail "startup-output-too-large"

launchservices_status="not-run"
if [[ "$launchservices_preflight" == "true" ]]; then
  /usr/bin/open -n \
    --env "HOME=$stage/home" \
    --env "PATH=" \
    --stdout "$stage/launchservices.stdout" \
    --stderr "$stage/launchservices.stderr" \
    "$app" --args --startup-check || fail "launchservices-preflight-failed"
  /bin/sleep 2
  [[ "$(/usr/bin/stat -f '%z' "$stage/launchservices.stdout")" -le 65536 ]] || fail "launchservices-output-too-large"
  [[ "$(/usr/bin/stat -f '%z' "$stage/launchservices.stderr")" -le 65536 ]] || fail "launchservices-output-too-large"
  launchservices_status="request-accepted"
fi

receipt="$stage/acceptance-receipt.json"
/usr/bin/plutil -create xml1 "$receipt"
/usr/bin/plutil -insert schema_version -integer 1 -s "$receipt"
insert_string "$receipt" record_type "qiongli-macos-native-acceptance"
insert_string "$receipt" product_version "$product_version"
insert_string "$receipt" status "accepted-nonpublishing-automated-evidence"
/usr/bin/plutil -insert publication_allowed -bool false -s "$receipt"
/usr/bin/plutil -insert artifact -dictionary -s "$receipt"
insert_string "$receipt" artifact.product_source_commit "$expected_source_commit"
insert_string "$receipt" artifact.package_file "$package_file"
/usr/bin/plutil -insert artifact.package_size_bytes -integer "$actual_package_size" -s "$receipt"
insert_string "$receipt" artifact.package_sha256 "$actual_package_sha256"
insert_string "$receipt" artifact.package_manifest_sha256 "$actual_manifest_sha256"
insert_string "$receipt" artifact.target "macos-aarch64"
/usr/bin/plutil -insert checks -dictionary -s "$receipt"
insert_string "$receipt" checks.receipt_binding "passed"
insert_string "$receipt" checks.manifest_binding "passed"
insert_string "$receipt" checks.extracted_bundle_layout "passed"
insert_string "$receipt" checks.empty_path_startup "passed"
insert_string "$receipt" checks.launchservices_request "$launchservices_status"
/usr/bin/plutil -insert open_gates -dictionary -s "$receipt"
insert_string "$receipt" open_gates.clean_machine "not-asserted"
insert_string "$receipt" open_gates.displayed_window "not-observed"
insert_string "$receipt" open_gates.manual_scale "not-run"
insert_string "$receipt" open_gates.voiceover "not-run"
insert_string "$receipt" open_gates.contrast "not-run"
insert_string "$receipt" open_gates.production_signing "not-run"
insert_string "$receipt" reason "automated-package-preflight-is-not-human-or-clean-machine-acceptance"
/usr/bin/plutil -convert json -r -o "$stage/final-receipt.json" "$receipt"
/bin/chmod 600 "$stage/final-receipt.json"
/bin/mv "$stage/final-receipt.json" "$output"

printf 'macOS native acceptance: passed (%s, %s)\n' "$product_version" "$launchservices_status"
