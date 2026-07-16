#!/bin/bash
set -euo pipefail

umask 077

usage() {
  cat <<'EOF'
Usage: macos_alpha1_update_journey.sh \
  --signed-artifact-dir ABSOLUTE_PATH \
  --output ABSOLUTE_NEW_JSON_PATH

Exercise the packaged Qiongli macOS update helper with an ad-hoc signed
test-only application. The journey runs one successful atomic replacement and
one failed-health rollback with an isolated HOME and empty PATH, then writes a
path-redacted non-publishing receipt.

This command does not verify Developer ID, notarization, Gatekeeper,
publication readiness, version selection, network download, or stream
metadata. It never installs into a user application directory.
EOF
}

fail() {
  printf 'macOS Alpha.1 update journey failed: %s\n' "$1" >&2
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
  case "$1" in
    *$'\n'*|*$'\r'*|*'"'*|*\\*) return 1 ;;
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

signed_artifact_dir=""
output=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --signed-artifact-dir)
      [[ $# -ge 2 ]] || { usage >&2; exit 2; }
      signed_artifact_dir="$2"
      shift 2
      ;;
    --output)
      [[ $# -ge 2 ]] || { usage >&2; exit 2; }
      output="$2"
      shift 2
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
[[ -n "$signed_artifact_dir" && -n "$output" ]] || {
  usage >&2
  exit 2
}
valid_absolute_path "$signed_artifact_dir" || fail "signed-artifact-directory-invalid"
valid_absolute_path "$output" || fail "output-path-invalid"
[[ -d "$signed_artifact_dir" && ! -L "$signed_artifact_dir" ]] ||
  fail "signed-artifact-directory-invalid"
[[ ! -e "$output" && ! -L "$output" ]] || fail "output-path-exists"
output_parent="$(/usr/bin/dirname "$output")"
[[ -d "$output_parent" && ! -L "$output_parent" ]] || fail "output-parent-invalid"

signing_receipt="$signed_artifact_dir/qiongli-macos-alpha1-signing.receipt.json"
[[ -f "$signing_receipt" && ! -L "$signing_receipt" ]] || fail "signing-receipt-invalid"
[[ "$(plist_raw schema_version integer "$signing_receipt")" == "1" ]] ||
  fail "signing-receipt-schema-invalid"
[[ "$(plist_raw status string "$signing_receipt")" == "ad-hoc-signed-test-only" ]] ||
  fail "signing-receipt-status-invalid"
[[ "$(plist_raw signing.kind string "$signing_receipt")" == "ad-hoc-test" ]] ||
  fail "signing-kind-invalid"
[[ "$(plist_raw signing.verification string "$signing_receipt")" == "passed" ]] ||
  fail "signing-verification-invalid"

archive_name="$(plist_raw final_artifact.file string "$signing_receipt")"
[[ "$archive_name" == "qiongli-desktop-2.0.0-alpha.1-macos-aarch64.ad-hoc-test.app.zip" ]] ||
  fail "signed-archive-name-invalid"
archive="$signed_artifact_dir/$archive_name"
[[ -f "$archive" && ! -L "$archive" ]] || fail "signed-archive-invalid"
archive_sha256="$(sha256_file "$archive")"
valid_lower_hex "$archive_sha256" 64 || fail "signed-archive-digest-invalid"
[[ "$archive_sha256" == "$(plist_raw final_artifact.sha256 string "$signing_receipt")" ]] ||
  fail "signed-archive-digest-mismatch"

launcher_sha256="$(plist_raw final_artifact.launcher_sha256 string "$signing_receipt")"
canonical_sha256="$(plist_raw final_artifact.canonical_binary_sha256 string "$signing_receipt")"
helper_sha256="$(plist_raw final_artifact.update_helper_sha256 string "$signing_receipt")"
valid_lower_hex "$launcher_sha256" 64 || fail "launcher-digest-invalid"
valid_lower_hex "$canonical_sha256" 64 || fail "canonical-digest-invalid"
valid_lower_hex "$helper_sha256" 64 || fail "helper-digest-invalid"

stage="$(/usr/bin/mktemp -d "$output_parent/.qiongli-macos-update-journey.XXXXXX")"
cleanup() {
  /bin/rm -rf -- "$stage"
}
trap cleanup EXIT HUP INT TERM

/bin/mkdir -m 700 "$stage/source"
/usr/bin/ditto -x -k "$archive" "$stage/source"
source_app="$stage/source/Qiongli.app"
source_manifest="$source_app/Contents/Resources/.qiongli-desktop-package.json"
source_info="$source_app/Contents/Info.plist"
[[ -d "$source_app" && ! -L "$source_app" ]] || fail "signed-application-invalid"
[[ -f "$source_manifest" && ! -L "$source_manifest" ]] || fail "desktop-manifest-invalid"
[[ -f "$source_info" && ! -L "$source_info" ]] || fail "application-info-invalid"
/usr/bin/codesign --verify --deep --strict --verbose=2 "$source_app" >/dev/null 2>&1 ||
  fail "signed-application-verification-failed"

version="$(plist_raw QiongliProductVersion string "$source_info")"
resource_pack_sha256="$(plist_raw resource_pack_sha256 string "$source_manifest")"
[[ "$version" == "2.0.0-alpha.1" ]] || fail "application-version-invalid"
valid_lower_hex "$resource_pack_sha256" 64 || fail "resource-pack-digest-invalid"

old_archive_sha256="aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
old_resource_pack_sha256="bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
health_token="0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
transaction_id="update-0123456789abcdef0123456789abcdef"

write_state() {
  local state_file="$1"
  local phase="$2"
  /usr/bin/printf '%s\n' \
    '{' \
    '  "document_kind": "qiongli-update-state",' \
    '  "schema_version": 1,' \
    '  "revision": 1,' \
    '  "selected_stream": "beta",' \
    '  "last_accepted_generation": 1,' \
    '  "last_known_good": {' \
    "    \"version\": \"$version\"," \
    '    "channel": "alpha",' \
    '    "generation": 1,' \
    "    \"archive_sha256\": \"$old_archive_sha256\"," \
    "    \"resource_pack_sha256\": \"$old_resource_pack_sha256\"" \
    '  },' \
    '  "active_transaction": {' \
    "    \"transaction_id\": \"$transaction_id\"," \
    "    \"target_version\": \"$version\"," \
    "    \"phase\": \"$phase\"" \
    '  }' \
    '}' >"$state_file"
  /bin/chmod 600 "$state_file"
}

write_journal() {
  local journal_file="$1"
  local destination="$2"
  local staged="$3"
  local backup="$4"
  local health_token_sha256="$5"
  /usr/bin/printf '%s\n' \
    '{' \
    '  "document_kind": "qiongli-native-replacement",' \
    '  "schema_version": 1,' \
    "  \"transaction_id\": \"$transaction_id\"," \
    '  "parent_process_id": 2147483647,' \
    "  \"destination_application\": \"$destination\"," \
    "  \"staged_application\": \"$staged\"," \
    "  \"backup_application\": \"$backup\"," \
    "  \"target_version\": \"$version\"," \
    '  "target_channel": "alpha",' \
    '  "generation": 2,' \
    "  \"archive_sha256\": \"$archive_sha256\"," \
    "  \"resource_pack_sha256\": \"$resource_pack_sha256\"," \
    "  \"launcher_sha256\": \"$launcher_sha256\"," \
    "  \"canonical_binary_sha256\": \"$canonical_sha256\"," \
    "  \"update_helper_sha256\": \"$helper_sha256\"," \
    "  \"health_token_sha256\": \"$health_token_sha256\"," \
    '  "created_at_unix": 1' \
    '}' >"$journal_file"
  /bin/chmod 600 "$journal_file"
}

prepare_journey() {
  local name="$1"
  local token_digest="$2"
  journey_root="$stage/$name"
  journey_home="$journey_root/home"
  journey_config="$journey_root/config"
  journey_state_root="$journey_config/v2"
  journey_transaction_root="$journey_state_root/updates/staging/$transaction_id"
  journey_applications="$journey_root/Applications"
  journey_destination="$journey_applications/Qiongli.app"
  journey_staged="$journey_transaction_root/application/Qiongli.app"
  journey_backup="$journey_applications/.Qiongli.app.qiongli-backup-$transaction_id"

  /bin/mkdir -p -m 700 \
    "$journey_home" \
    "$journey_config" \
    "$journey_state_root" \
    "$journey_state_root/updates" \
    "$journey_state_root/updates/staging" \
    "$journey_transaction_root" \
    "$journey_transaction_root/application" \
    "$journey_applications" \
    "$journey_root/old-extract" \
    "$journey_root/new-extract"
  /usr/bin/ditto -x -k "$archive" "$journey_root/old-extract"
  /usr/bin/ditto -x -k "$archive" "$journey_root/new-extract"
  /bin/mv "$journey_root/old-extract/Qiongli.app" "$journey_destination"
  /bin/mv "$journey_root/new-extract/Qiongli.app" "$journey_staged"
  /bin/rmdir "$journey_root/old-extract" "$journey_root/new-extract"
  journey_old_inode="$(/usr/bin/stat -f '%i' "$journey_destination")"
  journey_staged_inode="$(/usr/bin/stat -f '%i' "$journey_staged")"

  write_state "$journey_state_root/update-state.json" "awaiting-exit"
  /usr/bin/printf '%s' "$health_token" >"$journey_transaction_root/replacement-health-token"
  /bin/chmod 600 "$journey_transaction_root/replacement-health-token"
  write_journal \
    "$journey_transaction_root/replacement-journal.json" \
    "$journey_destination" \
    "$journey_staged" \
    "$journey_backup" \
    "$token_digest"
  journey_helper="$journey_staged/Contents/MacOS/qiongli-update-helper"
  [[ "$(sha256_file "$journey_helper")" == "$helper_sha256" ]] ||
    fail "journey-helper-digest-mismatch"
}

correct_health_token_sha256="$(
  /usr/bin/printf '%s' "$health_token" | /usr/bin/shasum -a 256 | /usr/bin/awk '{print $1}'
)"
valid_lower_hex "$correct_health_token_sha256" 64 || fail "health-token-digest-invalid"

prepare_journey "success" "$correct_health_token_sha256"
HOME="$journey_home" QIONGLI_CONFIG_HOME="$journey_config" PATH="" \
  "$journey_helper" "$transaction_id" \
  >"$journey_root/helper.stdout" 2>"$journey_root/helper.stderr" ||
  fail "successful-replacement-failed"
[[ "$(/usr/bin/stat -f '%i' "$journey_destination")" == "$journey_staged_inode" ]] ||
  fail "successful-replacement-inode-invalid"
[[ ! -e "$journey_backup" && ! -e "$journey_transaction_root" ]] ||
  fail "successful-replacement-cleanup-incomplete"
success_state="$journey_state_root/update-state.json"
[[ "$(plist_raw last_accepted_generation integer "$success_state")" == "2" ]] ||
  fail "successful-replacement-generation-invalid"
[[ "$(plist_raw last_known_good.generation integer "$success_state")" == "2" ]] ||
  fail "successful-replacement-known-good-invalid"
/usr/bin/grep -Eq '"active_transaction"[[:space:]]*:[[:space:]]*null' "$success_state" ||
  fail "successful-replacement-transaction-not-cleared"
/usr/bin/codesign --verify --deep --strict --verbose=2 "$journey_destination" >/dev/null 2>&1 ||
  fail "successful-replacement-signature-invalid"

invalid_health_token_sha256="ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
prepare_journey "rollback" "$invalid_health_token_sha256"
set +e
HOME="$journey_home" QIONGLI_CONFIG_HOME="$journey_config" PATH="" \
  "$journey_helper" "$transaction_id" \
  >"$journey_root/helper.stdout" 2>"$journey_root/helper.stderr"
rollback_status=$?
set -e
[[ "$rollback_status" -ne 0 ]] || fail "failed-health-unexpectedly-succeeded"
/usr/bin/grep -qx 'error: native-update-health-check-failed' "$journey_root/helper.stderr" ||
  fail "failed-health-error-invalid"
[[ "$(/usr/bin/stat -f '%i' "$journey_destination")" == "$journey_old_inode" ]] ||
  fail "failed-health-old-application-not-restored"
[[ ! -e "$journey_backup" && ! -e "$journey_transaction_root" ]] ||
  fail "failed-health-cleanup-incomplete"
rollback_state="$journey_state_root/update-state.json"
[[ "$(plist_raw last_accepted_generation integer "$rollback_state")" == "1" ]] ||
  fail "failed-health-generation-advanced"
[[ "$(plist_raw last_known_good.generation integer "$rollback_state")" == "1" ]] ||
  fail "failed-health-known-good-changed"
/usr/bin/grep -Eq '"active_transaction"[[:space:]]*:[[:space:]]*null' "$rollback_state" ||
  fail "failed-health-transaction-not-cleared"
/usr/bin/codesign --verify --deep --strict --verbose=2 "$journey_destination" >/dev/null 2>&1 ||
  fail "failed-health-restored-signature-invalid"

receipt_xml="$stage/update-journey-receipt.xml"
/usr/bin/plutil -create xml1 "$receipt_xml"
/usr/bin/plutil -insert schema_version -integer 1 -s "$receipt_xml"
insert_string "$receipt_xml" record_type "qiongli-macos-alpha1-update-journey"
insert_string "$receipt_xml" status "passed-test-only"
/usr/bin/plutil -insert publication_allowed -bool false -s "$receipt_xml"
/usr/bin/plutil -insert source -dictionary -s "$receipt_xml"
insert_string "$receipt_xml" source.signing_kind "ad-hoc-test"
insert_string "$receipt_xml" source.archive_sha256 "$archive_sha256"
insert_string "$receipt_xml" source.version "$version"
/usr/bin/plutil -insert checks -dictionary -s "$receipt_xml"
insert_string "$receipt_xml" checks.empty_path "passed"
insert_string "$receipt_xml" checks.successful_atomic_replacement "passed"
insert_string "$receipt_xml" checks.health_commit "passed"
insert_string "$receipt_xml" checks.failed_health_rollback "passed"
insert_string "$receipt_xml" checks.last_known_good_restoration "passed"
insert_string "$receipt_xml" checks.transaction_cleanup "passed"
insert_string "$receipt_xml" checks.shell_or_language_runtime "not-required"
/usr/bin/plutil -insert open_gates -dictionary -s "$receipt_xml"
insert_string "$receipt_xml" open_gates.developer_id "not-run"
insert_string "$receipt_xml" open_gates.notarization "not-run"
insert_string "$receipt_xml" open_gates.gatekeeper "not-run"
insert_string "$receipt_xml" open_gates.network_update_selection "not-run"
insert_string "$receipt_xml" open_gates.clean_machine "not-asserted"
insert_string "$receipt_xml" open_gates.publication "blocked"
insert_string "$receipt_xml" reason \
  "ad-hoc-signed-packaged-helper-proves-replacement-mechanics-only"
/usr/bin/plutil -convert json -r -o "$stage/final-receipt.json" "$receipt_xml"
/bin/chmod 600 "$stage/final-receipt.json"
/bin/mv "$stage/final-receipt.json" "$output"

printf 'macOS Alpha.1 update journey: passed (test-only)\n'
