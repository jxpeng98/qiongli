#!/bin/bash
set -euo pipefail

umask 077

usage() {
  cat <<'EOF'
Usage: macos_alpha1_sign_notarize.sh \
  --artifact-dir ABSOLUTE_PATH \
  --expected-source-commit HEX \
  --expected-package-sha256 HEX \
  --output-dir ABSOLUTE_NEW_DIRECTORY \
  [--test-only-ad-hoc | --community-alpha | --production] \
  [--preserve-signed-canonical]

The default mode verifies the exact unsigned Alpha.1 package and emits only a
non-publishing source-acceptance receipt. --test-only-ad-hoc exercises the
signing boundary without a production identity or notarization.
--community-alpha creates the separately labelled ad-hoc-signed, not-notarized
free distribution candidate. --production signs with a Developer ID
Application identity already available to codesign, submits with an existing
notarytool Keychain profile, staples the accepted ticket, verifies Gatekeeper
assessment, and emits both the signed application ZIP used by self-update and a
drag-to-Applications DMG used for first install. All artifacts remain
non-publishing until the applicable final release ledger is closed.

Production mode requires these environment variables:
  QIONGLI_MACOS_SIGNING_IDENTITY
  QIONGLI_MACOS_NOTARY_PROFILE
  QIONGLI_MACOS_EXPECTED_TEAM_ID

The command never accepts private-key files or credential passwords, never
creates Keychain credentials, and never tags, publishes, or installs anything.

--preserve-signed-canonical is required for a package carrying
.qiongli-product-control.json. The canonical runtime must already have the
signature appropriate for the selected mode. The command verifies its
signature and product-control digest, never signs it again, and fails if its
bytes change while signing the remaining bundle.
EOF
}

fail() {
  printf 'macOS Alpha.1 signing boundary failed: %s\n' "$1" >&2
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

valid_credential_reference() {
  local value="$1"
  [[ -n "$value" && "${#value}" -le 256 ]] || return 1
  case "$value" in
    -*|*$'\n'*|*$'\r'*) return 1 ;;
  esac
}

valid_team_id() {
  local value="$1"
  [[ "${#value}" -eq 10 ]] || return 1
  case "$value" in
    *[!A-Z0-9]*) return 1 ;;
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

create_disk_image() {
  local source="$1"
  local destination="$2"
  if [[ -x /usr/sbin/diskutil ]] && /usr/sbin/diskutil help image create from >/dev/null 2>&1; then
    /usr/sbin/diskutil image create from \
      --format UDZO \
      --volumeName Qiongli \
      "$source" \
      "$destination"
  else
    /usr/bin/hdiutil create \
      -fs HFS+ \
      -format UDZO \
      -volname Qiongli \
      -srcfolder "$source" \
      "$destination"
  fi
}

artifact_dir=""
expected_source_commit=""
expected_package_sha256=""
output_dir=""
mode="preflight"
preserve_signed_canonical="false"

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
    --output-dir)
      [[ $# -ge 2 ]] || { usage >&2; exit 2; }
      output_dir="$2"
      shift 2
      ;;
    --test-only-ad-hoc)
      [[ "$mode" == "preflight" ]] || fail "mode-conflict"
      mode="ad-hoc-test"
      shift
      ;;
    --community-alpha)
      [[ "$mode" == "preflight" ]] || fail "mode-conflict"
      mode="community-alpha"
      shift
      ;;
    --production)
      [[ "$mode" == "preflight" ]] || fail "mode-conflict"
      mode="production"
      shift
      ;;
    --preserve-signed-canonical)
      [[ "$preserve_signed_canonical" == "false" ]] || fail "preserve-signed-canonical-duplicate"
      preserve_signed_canonical="true"
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
[[ -n "$artifact_dir" && -n "$expected_source_commit" && -n "$expected_package_sha256" && -n "$output_dir" ]] || {
  usage >&2
  exit 2
}
valid_absolute_path "$artifact_dir" || fail "artifact-directory-invalid"
valid_absolute_path "$output_dir" || fail "output-directory-invalid"
[[ -d "$artifact_dir" && ! -L "$artifact_dir" ]] || fail "artifact-directory-invalid"
[[ ! -e "$output_dir" && ! -L "$output_dir" ]] || fail "output-directory-exists"
output_parent="$(/usr/bin/dirname "$output_dir")"
[[ -d "$output_parent" && ! -L "$output_parent" ]] || fail "output-parent-invalid"
case "${#expected_source_commit}" in
  40|64) valid_lower_hex "$expected_source_commit" "${#expected_source_commit}" || fail "source-commit-invalid" ;;
  *) fail "source-commit-invalid" ;;
esac
valid_lower_hex "$expected_package_sha256" 64 || fail "expected-package-digest-invalid"
if [[ "$preserve_signed_canonical" == "true" && "$mode" == "preflight" ]]; then
  fail "preserve-signed-canonical-requires-signing-mode"
fi

artifact_real="$(cd "$artifact_dir" && /bin/pwd -P)"
output_parent_real="$(cd "$output_parent" && /bin/pwd -P)"
output_name="$(/usr/bin/basename "$output_dir")"
[[ -n "$output_name" && "$output_name" != "." && "$output_name" != ".." ]] || fail "output-directory-invalid"
output_real="$output_parent_real/$output_name"
case "$output_real" in
  "$artifact_real"|"$artifact_real"/*) fail "output-must-not-modify-source-artifact" ;;
esac

signing_identity="${QIONGLI_MACOS_SIGNING_IDENTITY:-}"
notary_profile="${QIONGLI_MACOS_NOTARY_PROFILE:-}"
expected_team_id="${QIONGLI_MACOS_EXPECTED_TEAM_ID:-}"
if [[ "$mode" == "production" ]]; then
  valid_credential_reference "$signing_identity" || fail "production-signing-identity-missing-or-invalid"
  valid_credential_reference "$notary_profile" || fail "production-notary-profile-missing-or-invalid"
  valid_team_id "$expected_team_id" || fail "production-team-id-missing-or-invalid"
fi

script_dir="$(cd "$(/usr/bin/dirname "$0")" && /bin/pwd -P)"
acceptance_script="$script_dir/macos_alpha1_acceptance.sh"
[[ -f "$acceptance_script" && -x "$acceptance_script" && ! -L "$acceptance_script" ]] || fail "source-acceptance-entry-invalid"

stage="$(/usr/bin/mktemp -d "$output_parent_real/.qiongli-macos-signing.XXXXXX")"
output_reserved="false"
mounted_dmg=""
cleanup() {
  if [[ -n "$mounted_dmg" ]]; then
    /usr/bin/hdiutil detach "$mounted_dmg" -force >/dev/null 2>&1 || true
  fi
  if [[ "$output_reserved" == "true" ]]; then
    /bin/rm -rf -- "$output_real"
  fi
  /bin/rm -rf -- "$stage"
}
trap cleanup EXIT HUP INT TERM
/bin/mkdir -m 700 "$stage/result" "$stage/source"
/bin/mkdir -m 700 "$output_real" || fail "output-directory-reservation-failed"
output_reserved="true"

source_manifest="$artifact_real/qiongli-desktop-package.manifest.json"
source_receipt="$artifact_real/qiongli-desktop-package.receipt.json"
for source_file in "$source_manifest" "$source_receipt"; do
  [[ -f "$source_file" && ! -L "$source_file" ]] || fail "source-artifact-file-invalid"
done
source_package_file="$(plist_raw package_file string "$source_receipt")"
[[ "$source_package_file" == "Qiongli-2.0.0-alpha.1-macOS-arm64.source.zip" ]] || fail "source-package-name-invalid"
source_archive="$artifact_real/$source_package_file"
[[ -f "$source_archive" && ! -L "$source_archive" ]] || fail "source-artifact-file-invalid"
/bin/cp "$source_manifest" "$source_receipt" "$source_archive" "$stage/source/"

unsigned_acceptance="$stage/result/qiongli-macos-alpha1-unsigned-acceptance.receipt.json"
"$acceptance_script" \
  --artifact-dir "$stage/source" \
  --expected-source-commit "$expected_source_commit" \
  --expected-package-sha256 "$expected_package_sha256" \
  --output "$unsigned_acceptance"

manifest="$stage/source/qiongli-desktop-package.manifest.json"
package_receipt="$stage/source/qiongli-desktop-package.receipt.json"
package_file="$(plist_raw package_file string "$package_receipt")"
archive="$stage/source/$package_file"
manifest_sha256="$(sha256_file "$manifest")"
acceptance_sha256="$(sha256_file "$unsigned_acceptance")"

signing_kind="not-run"
signing_status="not-run"
actual_team_id="not-run"
notary_status="not-run"
notary_submission_id="not-run"
stapling_status="not-run"
gatekeeper_status="not-run"
production_signing_gate="not-run"
final_archive_name=""
final_archive_sha256=""
final_archive_size="0"
final_launcher_sha256=""
final_canonical_sha256=""
final_update_helper_sha256=""
installer_artifact_name=""
installer_artifact_sha256=""
installer_artifact_size="0"
installer_signing_kind="not-run"
installer_signing_status="not-run"
installer_team_id="not-run"
installer_notary_status="not-run"
installer_notary_submission_id="not-run"
installer_stapling_status="not-run"
installer_gatekeeper_status="not-run"

if [[ "$mode" != "preflight" ]]; then
  release_asset_base="Qiongli-2.0.0-alpha.1-macOS-arm64"
  final_archive_name="$release_asset_base.zip"
  installer_artifact_name="$release_asset_base.dmg"
  /bin/mkdir -m 700 "$stage/extracted"
  /usr/bin/ditto -x -k "$archive" "$stage/extracted"

  app="$stage/extracted/Qiongli.app"
  launcher="$app/Contents/MacOS/Qiongli"
  canonical="$app/Contents/MacOS/qiongli-cli"
  update_helper="$app/Contents/MacOS/qiongli-update-helper"
  internal_manifest="$app/Contents/Resources/.qiongli-desktop-package.json"
  product_control="$app/Contents/Resources/.qiongli-product-control.json"
  [[ -d "$app" && ! -L "$app" ]] || fail "application-bundle-invalid"
  [[ -f "$launcher" && -x "$launcher" && ! -L "$launcher" ]] || fail "application-launcher-invalid"
  [[ -f "$canonical" && -x "$canonical" && ! -L "$canonical" ]] || fail "application-canonical-binary-invalid"
  [[ -f "$update_helper" && -x "$update_helper" && ! -L "$update_helper" ]] || fail "application-update-helper-invalid"
  [[ -f "$internal_manifest" && ! -L "$internal_manifest" ]] || fail "application-manifest-invalid"
  /usr/bin/cmp -s "$manifest" "$internal_manifest" || fail "application-manifest-mismatch"
  first_symlink="$(/usr/bin/find "$app" -type l -print -quit)"
  [[ -z "$first_symlink" ]] || fail "application-symlink-not-allowed"
  /usr/bin/xattr -cr "$app"

  canonical_before_signing="$(sha256_file "$canonical")"
  manifest_product_control_sha256="$(plist_raw product_control_sha256 string "$manifest" || true)"
  if [[ "$preserve_signed_canonical" == "true" ]]; then
    [[ -f "$product_control" && ! -L "$product_control" ]] || fail "application-product-control-missing"
    valid_lower_hex "$manifest_product_control_sha256" 64 || fail "application-product-control-manifest-invalid"
    [[ "$(sha256_file "$product_control")" == "$manifest_product_control_sha256" ]] || fail "application-product-control-digest-mismatch"
    [[ "$(plist_raw canonical_binary_sha256 string "$product_control")" == "$canonical_before_signing" ]] || fail "application-product-control-canonical-mismatch"
    /usr/bin/codesign --verify --strict --verbose=2 "$canonical" >"$stage/canonical.preverified" 2>&1 || fail "canonical-presigned-verification-failed"
    /usr/bin/codesign -d --verbose=4 "$canonical" >"$stage/canonical-presigned.details" 2>&1 || fail "canonical-presigned-details-unavailable"
    if [[ "$mode" == "ad-hoc-test" || "$mode" == "community-alpha" ]]; then
      /usr/bin/grep -q '^Signature=adhoc$' "$stage/canonical-presigned.details" || fail "canonical-presigned-kind-invalid"
    else
      /usr/bin/grep -q '^Authority=Developer ID Application:' "$stage/canonical-presigned.details" || fail "canonical-presigned-authority-invalid"
      /usr/bin/grep -q '^Timestamp=' "$stage/canonical-presigned.details" || fail "canonical-presigned-timestamp-missing"
      canonical_team_id="$(/usr/bin/awk -F= '/^TeamIdentifier=/{print $2; exit}' "$stage/canonical-presigned.details")"
      [[ "$canonical_team_id" == "$expected_team_id" ]] || fail "canonical-presigned-team-id-mismatch"
    fi
  elif [[ -n "$manifest_product_control_sha256" || -e "$product_control" ]]; then
    fail "product-control-requires-preserved-canonical"
  fi

  if [[ "$mode" == "ad-hoc-test" || "$mode" == "community-alpha" ]]; then
    if [[ "$mode" == "community-alpha" ]]; then
      signing_kind="ad-hoc-community-alpha"
      production_signing_gate="not-required-community-alpha"
    else
      signing_kind="ad-hoc-test"
    fi
    if [[ "$preserve_signed_canonical" == "false" ]]; then
      /usr/bin/codesign --force --options runtime --timestamp=none --sign - "$canonical" >"$stage/canonical.sign" 2>&1 || fail "canonical-ad-hoc-signing-failed"
    fi
    /usr/bin/codesign --force --options runtime --timestamp=none --sign - "$update_helper" >"$stage/update-helper.sign" 2>&1 || fail "update-helper-ad-hoc-signing-failed"
    /usr/bin/codesign --force --options runtime --timestamp=none --sign - "$launcher" >"$stage/launcher.sign" 2>&1 || fail "launcher-ad-hoc-signing-failed"
    /usr/bin/codesign --force --options runtime --timestamp=none --sign - "$app" >"$stage/application.sign" 2>&1 || fail "application-ad-hoc-signing-failed"
  else
    signing_kind="developer-id-application"
    if [[ "$preserve_signed_canonical" == "false" ]]; then
      /usr/bin/codesign --force --options runtime --timestamp --sign "$signing_identity" "$canonical" >"$stage/canonical.sign" 2>&1 || fail "canonical-production-signing-failed"
    fi
    /usr/bin/codesign --force --options runtime --timestamp --sign "$signing_identity" "$update_helper" >"$stage/update-helper.sign" 2>&1 || fail "update-helper-production-signing-failed"
    /usr/bin/codesign --force --options runtime --timestamp --sign "$signing_identity" "$launcher" >"$stage/launcher.sign" 2>&1 || fail "launcher-production-signing-failed"
    /usr/bin/codesign --force --options runtime --timestamp --sign "$signing_identity" "$app" >"$stage/application.sign" 2>&1 || fail "application-production-signing-failed"
  fi

  /usr/bin/codesign --verify --strict --verbose=2 "$canonical" >"$stage/canonical.verify" 2>&1 || fail "canonical-signature-verification-failed"
  /usr/bin/codesign --verify --strict --verbose=2 "$update_helper" >"$stage/update-helper.verify" 2>&1 || fail "update-helper-signature-verification-failed"
  /usr/bin/codesign --verify --strict --verbose=2 "$launcher" >"$stage/launcher.verify" 2>&1 || fail "launcher-signature-verification-failed"
  /usr/bin/codesign --verify --deep --strict --verbose=2 "$app" >"$stage/application.verify" 2>&1 || fail "application-signature-verification-failed"
  if [[ "$preserve_signed_canonical" == "true" ]]; then
    [[ "$(sha256_file "$canonical")" == "$canonical_before_signing" ]] || fail "canonical-changed-after-product-control-finalization"
  fi
  /usr/bin/codesign -d --verbose=4 "$app" >"$stage/codesign.details" 2>&1 || fail "application-signature-details-unavailable"
  /usr/bin/grep -q 'flags=.*runtime' "$stage/codesign.details" || fail "hardened-runtime-not-recorded"
  /usr/bin/grep -q '^Identifier=io.github.jxpeng98.qiongli$' "$stage/codesign.details" || fail "signed-application-identifier-invalid"
  signing_status="passed"

  if [[ "$mode" == "ad-hoc-test" || "$mode" == "community-alpha" ]]; then
    /usr/bin/grep -q '^Signature=adhoc$' "$stage/codesign.details" || fail "ad-hoc-signature-not-recorded"
    actual_team_id="not-set-ad-hoc"
  else
    /usr/bin/grep -q '^Authority=Developer ID Application:' "$stage/codesign.details" || fail "developer-id-authority-not-recorded"
    /usr/bin/grep -q '^Timestamp=' "$stage/codesign.details" || fail "trusted-timestamp-not-recorded"
    actual_team_id="$(/usr/bin/awk -F= '/^TeamIdentifier=/{print $2; exit}' "$stage/codesign.details")"
    [[ "$actual_team_id" == "$expected_team_id" ]] || fail "production-team-id-mismatch"

    pre_notary_archive="$stage/qiongli-macos-alpha1-pre-notary.app.zip"
    /usr/bin/ditto -c -k --keepParent "$app" "$pre_notary_archive"
    /usr/bin/xcrun notarytool submit "$pre_notary_archive" \
      --keychain-profile "$notary_profile" \
      --wait --timeout 45m --output-format json \
      >"$stage/notary-result.json" 2>"$stage/notary.stderr" || fail "notary-submission-failed"
    notary_status="$(plist_raw status string "$stage/notary-result.json")"
    notary_submission_id="$(plist_raw id string "$stage/notary-result.json")"
    [[ "$notary_status" == "Accepted" && -n "$notary_submission_id" && "${#notary_submission_id}" -le 128 ]] || fail "notary-result-not-accepted"

    /usr/bin/xcrun stapler staple "$app" >"$stage/stapler.stdout" 2>"$stage/stapler.stderr" || fail "ticket-stapling-failed"
    /usr/bin/xcrun stapler validate "$app" >"$stage/stapler-validate.stdout" 2>"$stage/stapler-validate.stderr" || fail "stapled-ticket-validation-failed"
    /usr/bin/codesign --verify --deep --strict --verbose=2 "$app" >"$stage/post-staple.verify" 2>&1 || fail "post-staple-signature-verification-failed"
    stapling_status="passed"
  fi

  final_archive="$stage/result/$final_archive_name"
  /usr/bin/ditto -c -k --keepParent "$app" "$final_archive"
  /bin/mkdir -m 700 "$stage/final-verification"
  /usr/bin/ditto -x -k "$final_archive" "$stage/final-verification"
  final_app="$stage/final-verification/Qiongli.app"
  final_launcher="$final_app/Contents/MacOS/Qiongli"
  final_canonical="$final_app/Contents/MacOS/qiongli-cli"
  final_update_helper="$final_app/Contents/MacOS/qiongli-update-helper"
  final_internal_manifest="$final_app/Contents/Resources/.qiongli-desktop-package.json"
  final_product_control="$final_app/Contents/Resources/.qiongli-product-control.json"
  [[ -d "$final_app" && ! -L "$final_app" ]] || fail "final-archive-application-invalid"
  [[ -f "$final_launcher" && -x "$final_launcher" && ! -L "$final_launcher" ]] || fail "final-archive-launcher-invalid"
  [[ -f "$final_canonical" && -x "$final_canonical" && ! -L "$final_canonical" ]] || fail "final-archive-canonical-binary-invalid"
  [[ -f "$final_update_helper" && -x "$final_update_helper" && ! -L "$final_update_helper" ]] || fail "final-archive-update-helper-invalid"
  [[ -f "$final_internal_manifest" && ! -L "$final_internal_manifest" ]] || fail "final-archive-manifest-invalid"
  /usr/bin/cmp -s "$manifest" "$final_internal_manifest" || fail "final-archive-source-manifest-mismatch"
  if [[ "$preserve_signed_canonical" == "true" ]]; then
    [[ -f "$final_product_control" && ! -L "$final_product_control" ]] || fail "final-archive-product-control-missing"
    [[ "$(sha256_file "$final_product_control")" == "$manifest_product_control_sha256" ]] || fail "final-archive-product-control-digest-mismatch"
    [[ "$(sha256_file "$final_canonical")" == "$canonical_before_signing" ]] || fail "final-archive-canonical-drift"
  fi
  final_symlink="$(/usr/bin/find "$final_app" -type l -print -quit)"
  [[ -z "$final_symlink" ]] || fail "final-archive-symlink-not-allowed"
  /usr/bin/codesign --verify --deep --strict --verbose=2 "$final_app" >"$stage/final-archive.verify" 2>&1 || fail "final-archive-signature-verification-failed"
  /usr/bin/codesign -d --verbose=4 "$final_app" >"$stage/final-archive-codesign.details" 2>&1 || fail "final-archive-signature-details-unavailable"
  /usr/bin/grep -q 'flags=.*runtime' "$stage/final-archive-codesign.details" || fail "final-archive-hardened-runtime-not-recorded"
  /usr/bin/grep -q '^Identifier=io.github.jxpeng98.qiongli$' "$stage/final-archive-codesign.details" || fail "final-archive-application-identifier-invalid"
  if [[ "$mode" == "ad-hoc-test" || "$mode" == "community-alpha" ]]; then
    /usr/bin/grep -q '^Signature=adhoc$' "$stage/final-archive-codesign.details" || fail "final-archive-ad-hoc-signature-not-recorded"
  else
    final_team_id="$(/usr/bin/awk -F= '/^TeamIdentifier=/{print $2; exit}' "$stage/final-archive-codesign.details")"
    [[ "$final_team_id" == "$expected_team_id" ]] || fail "final-archive-team-id-mismatch"
    /usr/bin/xcrun stapler validate "$final_app" >"$stage/final-archive-stapler.stdout" 2>"$stage/final-archive-stapler.stderr" || fail "final-archive-stapled-ticket-invalid"
    /usr/sbin/spctl --assess --type execute --verbose=4 "$final_app" >"$stage/final-archive-gatekeeper.stdout" 2>"$stage/final-archive-gatekeeper.stderr" || fail "final-archive-gatekeeper-assessment-failed"
    gatekeeper_status="passed"
    production_signing_gate="passed"
  fi
  final_archive_sha256="$(sha256_file "$final_archive")"
  final_archive_size="$(/usr/bin/stat -f '%z' "$final_archive")"
  final_launcher_sha256="$(sha256_file "$final_launcher")"
  final_canonical_sha256="$(sha256_file "$final_canonical")"
  final_update_helper_sha256="$(sha256_file "$final_update_helper")"

  if [[ "$mode" == "ad-hoc-test" || "$mode" == "community-alpha" ]]; then
    if [[ "$mode" == "community-alpha" ]]; then
      installer_signing_kind="ad-hoc-community-alpha"
    else
      installer_signing_kind="ad-hoc-test"
    fi
    installer_team_id="not-set-ad-hoc"
  else
    installer_signing_kind="developer-id-application"
    installer_team_id="$actual_team_id"
  fi
  installer_artifact="$stage/result/$installer_artifact_name"
  /bin/mkdir -m 700 "$stage/dmg-root"
  /usr/bin/ditto "$final_app" "$stage/dmg-root/Qiongli.app"
  /bin/ln -s /Applications "$stage/dmg-root/Applications"
  create_disk_image "$stage/dmg-root" "$installer_artifact" \
    >"$stage/installer-dmg-create.stdout" 2>"$stage/installer-dmg-create.stderr" || fail "installer-dmg-creation-failed"

  if [[ "$mode" == "ad-hoc-test" || "$mode" == "community-alpha" ]]; then
    /usr/bin/codesign --force --timestamp=none --sign - "$installer_artifact" \
      >"$stage/installer-dmg.sign" 2>&1 || fail "installer-dmg-ad-hoc-signing-failed"
  else
    /usr/bin/codesign --force --timestamp --sign "$signing_identity" "$installer_artifact" \
      >"$stage/installer-dmg.sign" 2>&1 || fail "installer-dmg-production-signing-failed"
  fi
  /usr/bin/codesign --verify --strict --verbose=2 "$installer_artifact" \
    >"$stage/installer-dmg.verify" 2>&1 || fail "installer-dmg-signature-verification-failed"
  /usr/bin/codesign -d --verbose=4 "$installer_artifact" \
    >"$stage/installer-dmg-codesign.details" 2>&1 || fail "installer-dmg-signature-details-unavailable"
  if [[ "$mode" == "ad-hoc-test" || "$mode" == "community-alpha" ]]; then
    /usr/bin/grep -q '^Signature=adhoc$' "$stage/installer-dmg-codesign.details" || fail "installer-dmg-ad-hoc-signature-not-recorded"
  else
    dmg_team_id="$(/usr/bin/awk -F= '/^TeamIdentifier=/{print $2; exit}' "$stage/installer-dmg-codesign.details")"
    [[ "$dmg_team_id" == "$expected_team_id" ]] || fail "installer-dmg-team-id-mismatch"
    /usr/bin/xcrun notarytool submit "$installer_artifact" \
      --keychain-profile "$notary_profile" \
      --wait --timeout 45m --output-format json \
      >"$stage/installer-dmg-notary-result.json" 2>"$stage/installer-dmg-notary.stderr" || fail "installer-dmg-notary-submission-failed"
    installer_notary_status="$(plist_raw status string "$stage/installer-dmg-notary-result.json")"
    installer_notary_submission_id="$(plist_raw id string "$stage/installer-dmg-notary-result.json")"
    [[ "$installer_notary_status" == "Accepted" && -n "$installer_notary_submission_id" && "${#installer_notary_submission_id}" -le 128 ]] || fail "installer-dmg-notary-result-not-accepted"
    /usr/bin/xcrun stapler staple "$installer_artifact" \
      >"$stage/installer-dmg-stapler.stdout" 2>"$stage/installer-dmg-stapler.stderr" || fail "installer-dmg-ticket-stapling-failed"
    /usr/bin/xcrun stapler validate "$installer_artifact" \
      >"$stage/installer-dmg-stapler-validate.stdout" 2>"$stage/installer-dmg-stapler-validate.stderr" || fail "installer-dmg-stapled-ticket-validation-failed"
    installer_stapling_status="passed"
    /usr/sbin/spctl --assess --type open --context context:primary-signature --verbose=4 "$installer_artifact" \
      >"$stage/installer-dmg-gatekeeper.stdout" 2>"$stage/installer-dmg-gatekeeper.stderr" || fail "installer-dmg-gatekeeper-assessment-failed"
    installer_gatekeeper_status="passed"
  fi
  installer_signing_status="passed"
  /usr/bin/hdiutil verify "$installer_artifact" \
    >"$stage/installer-dmg-hdiutil-verify.stdout" 2>"$stage/installer-dmg-hdiutil-verify.stderr" || fail "installer-dmg-container-verification-failed"
  /bin/mkdir -m 700 "$stage/dmg-mount"
  /usr/bin/hdiutil attach \
    -readonly \
    -nobrowse \
    -noautoopen \
    -mountpoint "$stage/dmg-mount" \
    "$installer_artifact" \
    >"$stage/installer-dmg-attach.stdout" 2>"$stage/installer-dmg-attach.stderr" || fail "installer-dmg-attach-failed"
  mounted_dmg="$stage/dmg-mount"
  dmg_entry_count="$(/usr/bin/find "$mounted_dmg" -mindepth 1 -maxdepth 1 -print | /usr/bin/wc -l | /usr/bin/tr -d ' ')"
  [[ "$dmg_entry_count" == "2" ]] || fail "installer-dmg-layout-invalid"
  [[ -d "$mounted_dmg/Qiongli.app" && ! -L "$mounted_dmg/Qiongli.app" ]] || fail "installer-dmg-application-invalid"
  [[ -L "$mounted_dmg/Applications" ]] || fail "installer-dmg-applications-link-invalid"
  [[ "$(/usr/bin/readlink "$mounted_dmg/Applications")" == "/Applications" ]] || fail "installer-dmg-applications-link-invalid"
  mounted_manifest="$mounted_dmg/Qiongli.app/Contents/Resources/.qiongli-desktop-package.json"
  [[ -f "$mounted_manifest" && ! -L "$mounted_manifest" ]] || fail "installer-dmg-manifest-invalid"
  /usr/bin/cmp -s "$manifest" "$mounted_manifest" || fail "installer-dmg-source-manifest-mismatch"
  /usr/bin/codesign --verify --deep --strict --verbose=2 "$mounted_dmg/Qiongli.app" \
    >"$stage/installer-dmg-application.verify" 2>&1 || fail "installer-dmg-application-signature-invalid"
  /usr/bin/hdiutil detach "$mounted_dmg" \
    >"$stage/installer-dmg-detach.stdout" 2>"$stage/installer-dmg-detach.stderr" || fail "installer-dmg-detach-failed"
  mounted_dmg=""
  installer_artifact_sha256="$(sha256_file "$installer_artifact")"
  installer_artifact_size="$(/usr/bin/stat -f '%z' "$installer_artifact")"
fi

receipt_xml="$stage/signing-receipt.xml"
/usr/bin/plutil -create xml1 "$receipt_xml"
/usr/bin/plutil -insert schema_version -integer 1 -s "$receipt_xml"
insert_string "$receipt_xml" record_type "qiongli-macos-alpha1-signing-boundary"
case "$mode" in
  preflight) receipt_status="verified-unsigned-source-nonpublishing" ;;
  ad-hoc-test) receipt_status="ad-hoc-signed-test-only" ;;
  community-alpha) receipt_status="community-alpha-ad-hoc-signed-nonpublishing-candidate" ;;
  production) receipt_status="signed-notarized-nonpublishing-candidate" ;;
esac
insert_string "$receipt_xml" status "$receipt_status"
/usr/bin/plutil -insert publication_allowed -bool false -s "$receipt_xml"
if [[ "$mode" == "community-alpha" ]]; then
  insert_string "$receipt_xml" distribution_class "community-alpha"
  insert_string "$receipt_xml" platform_trust "macos-ad-hoc-not-notarized"
fi
/usr/bin/plutil -insert source -dictionary -s "$receipt_xml"
insert_string "$receipt_xml" source.product_source_commit "$expected_source_commit"
insert_string "$receipt_xml" source.unsigned_package_file "$package_file"
insert_string "$receipt_xml" source.unsigned_package_sha256 "$expected_package_sha256"
insert_string "$receipt_xml" source.unsigned_manifest_sha256 "$manifest_sha256"
insert_string "$receipt_xml" source.unsigned_acceptance_receipt_sha256 "$acceptance_sha256"
/usr/bin/plutil -insert final_artifact -dictionary -s "$receipt_xml"
if [[ "$mode" == "preflight" ]]; then
  final_artifact_status="not-produced"
else
  final_artifact_status="produced-nonpublishing"
fi
insert_string "$receipt_xml" final_artifact.status "$final_artifact_status"
insert_string "$receipt_xml" final_artifact.file "$final_archive_name"
/usr/bin/plutil -insert final_artifact.size_bytes -integer "$final_archive_size" -s "$receipt_xml"
insert_string "$receipt_xml" final_artifact.sha256 "$final_archive_sha256"
insert_string "$receipt_xml" final_artifact.launcher_sha256 "$final_launcher_sha256"
insert_string "$receipt_xml" final_artifact.canonical_binary_sha256 "$final_canonical_sha256"
insert_string "$receipt_xml" final_artifact.update_helper_sha256 "$final_update_helper_sha256"
/usr/bin/plutil -insert installer_artifact -dictionary -s "$receipt_xml"
insert_string "$receipt_xml" installer_artifact.status "$final_artifact_status"
insert_string "$receipt_xml" installer_artifact.kind "macos-disk-image"
insert_string "$receipt_xml" installer_artifact.layout "drag-to-applications"
insert_string "$receipt_xml" installer_artifact.file "$installer_artifact_name"
/usr/bin/plutil -insert installer_artifact.size_bytes -integer "$installer_artifact_size" -s "$receipt_xml"
insert_string "$receipt_xml" installer_artifact.sha256 "$installer_artifact_sha256"
/usr/bin/plutil -insert installer_signing -dictionary -s "$receipt_xml"
insert_string "$receipt_xml" installer_signing.kind "$installer_signing_kind"
insert_string "$receipt_xml" installer_signing.verification "$installer_signing_status"
insert_string "$receipt_xml" installer_signing.team_identifier "$installer_team_id"
/usr/bin/plutil -insert installer_notarization -dictionary -s "$receipt_xml"
insert_string "$receipt_xml" installer_notarization.status "$installer_notary_status"
insert_string "$receipt_xml" installer_notarization.submission_id "$installer_notary_submission_id"
insert_string "$receipt_xml" installer_notarization.stapling "$installer_stapling_status"
insert_string "$receipt_xml" installer_notarization.gatekeeper_assessment "$installer_gatekeeper_status"
/usr/bin/plutil -insert signing -dictionary -s "$receipt_xml"
insert_string "$receipt_xml" signing.kind "$signing_kind"
insert_string "$receipt_xml" signing.verification "$signing_status"
insert_string "$receipt_xml" signing.team_identifier "$actual_team_id"
/usr/bin/plutil -insert signing.canonical_signature_preserved -bool "$preserve_signed_canonical" -s "$receipt_xml"
/usr/bin/plutil -insert notarization -dictionary -s "$receipt_xml"
insert_string "$receipt_xml" notarization.status "$notary_status"
insert_string "$receipt_xml" notarization.submission_id "$notary_submission_id"
insert_string "$receipt_xml" notarization.stapling "$stapling_status"
insert_string "$receipt_xml" notarization.gatekeeper_assessment "$gatekeeper_status"
/usr/bin/plutil -insert open_gates -dictionary -s "$receipt_xml"
insert_string "$receipt_xml" open_gates.clean_machine "not-asserted"
insert_string "$receipt_xml" open_gates.displayed_window "not-observed"
insert_string "$receipt_xml" open_gates.manual_scale "not-run"
insert_string "$receipt_xml" open_gates.voiceover "not-run"
insert_string "$receipt_xml" open_gates.contrast "not-run"
insert_string "$receipt_xml" open_gates.production_signing "$production_signing_gate"
insert_string "$receipt_xml" open_gates.publication "blocked"
case "$mode" in
  preflight) reason="source-verified-signing-and-notarization-not-run" ;;
  ad-hoc-test) reason="ad-hoc-signature-tests-mechanism-only-and-is-not-a-distributable-trust-anchor" ;;
  community-alpha) reason="community-alpha-candidate-requires-three-target-promotion-and-release-trust-binding" ;;
  production) reason="signed-notarized-candidate-still-requires-final-release-ledger-and-maintainer-authorization" ;;
esac
insert_string "$receipt_xml" reason "$reason"
/usr/bin/plutil -convert json -r -o "$stage/result/qiongli-macos-alpha1-signing.receipt.json" "$receipt_xml"

if [[ "$mode" == "production" ]]; then
  update_receipt_xml="$stage/update-signing-receipt.xml"
  /usr/bin/plutil -create xml1 "$update_receipt_xml"
  /usr/bin/plutil -insert schema_version -integer 1 -s "$update_receipt_xml"
  insert_string "$update_receipt_xml" record_type "qiongli-macos-update-signing"
  insert_string "$update_receipt_xml" status "signed-notarized-candidate"
  /usr/bin/plutil -insert publication_allowed -bool false -s "$update_receipt_xml"
  /usr/bin/plutil -insert source -dictionary -s "$update_receipt_xml"
  insert_string "$update_receipt_xml" source.product_source_commit "$expected_source_commit"
  insert_string "$update_receipt_xml" source.unsigned_manifest_sha256 "$manifest_sha256"
  /usr/bin/plutil -insert final_artifact -dictionary -s "$update_receipt_xml"
  insert_string "$update_receipt_xml" final_artifact.status "produced"
  insert_string "$update_receipt_xml" final_artifact.file "$final_archive_name"
  /usr/bin/plutil -insert final_artifact.size_bytes -integer "$final_archive_size" -s "$update_receipt_xml"
  insert_string "$update_receipt_xml" final_artifact.sha256 "$final_archive_sha256"
  insert_string "$update_receipt_xml" final_artifact.launcher_sha256 "$final_launcher_sha256"
  insert_string "$update_receipt_xml" final_artifact.canonical_binary_sha256 "$final_canonical_sha256"
  insert_string "$update_receipt_xml" final_artifact.update_helper_sha256 "$final_update_helper_sha256"
  /usr/bin/plutil -insert signing -dictionary -s "$update_receipt_xml"
  insert_string "$update_receipt_xml" signing.kind "developer-id-application"
  insert_string "$update_receipt_xml" signing.verification "passed"
  insert_string "$update_receipt_xml" signing.team_identifier "$actual_team_id"
  /usr/bin/plutil -insert notarization -dictionary -s "$update_receipt_xml"
  insert_string "$update_receipt_xml" notarization.status "accepted"
  insert_string "$update_receipt_xml" notarization.stapling "passed"
  insert_string "$update_receipt_xml" notarization.gatekeeper_assessment "passed"
  /usr/bin/plutil -convert json -r \
    -o "$stage/result/qiongli-desktop-2.0.0-alpha.1-macos-aarch64.signing.receipt.json" \
    "$update_receipt_xml"
fi
/bin/cp "$manifest" "$stage/result/qiongli-desktop-package.manifest.json"
if [[ "$mode" == "community-alpha" ]]; then
  /bin/cp "$package_receipt" "$stage/result/qiongli-desktop-package.receipt.json"
fi
/bin/chmod 600 "$stage/result"/*.json

for result_file in "$stage/result"/*; do
  /bin/mv "$result_file" "$output_real/"
done
output_reserved="false"
printf 'macOS Alpha.1 signing boundary: %s (publication blocked)\n' "$mode"
