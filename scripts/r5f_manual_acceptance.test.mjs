import { createHash } from "node:crypto";
import {
  mkdtempSync,
  readFileSync,
  realpathSync,
  rmSync,
  writeFileSync
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";
import test from "node:test";
import assert from "node:assert/strict";

import { REQUIRED_CONFIRMATIONS } from "./r5f_manual_acceptance.mjs";

const SCRIPT = fileURLToPath(new URL("./r5f_manual_acceptance.mjs", import.meta.url));
const A64 = "a".repeat(64);
const B64 = "b".repeat(64);
const C64 = "c".repeat(64);
const D64 = "d".repeat(64);
const BUILD = "1".repeat(40);
const PRODUCT_CHECKS = [
  "embedded_authority",
  "canonical_signature_preserved",
  "product_control_verified",
  "zotero_companion_artifact_bound",
  "inventory_discovered",
  "skills_materialize_verify_refresh",
  "lite_mcp_self_test",
  "project_three_project_restart",
  "project_app_cli_library_full_mcp_parity",
  "project_artifact_internal_projection",
  "continuity_delivery_restart_replay",
  "continuity_assignment_resolution",
  "continuity_archive_restore_rebuild",
  "continuity_catalog_query_timeline",
  "continuity_path_redacted",
  "provider_keychain_save_replace_restart_remove",
  "cli_schema2_app_authority",
  "managed_operation_plan_apply",
  "standalone_skills_all_targets",
  "cli_plugin_reconcile_remove",
  "codex_install_verify_remove",
  "claude_install_verify_remove",
  "registration_repair",
  "packaged_restart_verification",
  "legacy_migration_fixture_isolated",
  "empty_path_startup"
];
const ZOTERO_CHECKS = [
  "appResourceArtifactBound",
  "desktopPackageManifestBound",
  "releaseArtifactByteIdentity",
  "automaticUpdateManifestBound",
  "startupSnapshotObservationNeutral",
  "isolatedHomeProfileUnchanged",
  "nativeStateMatrix",
  "legacyEndpointRequiresUpdate",
  "disposableSearchCollectionsTagsNotesAttachments",
  "approvedWriteReceiptLifecycle",
  "duplicateAndCuratedMetadataPreservation",
  "endpointShutdownRemoval",
  "importFileFallback"
];
const ZOTERO_CONFIRMATIONS = [
  "isolated-profile-no-sync",
  "missing-companion-fallback",
  "preview-cancel-no-profile-write",
  "legacy-update-handoff",
  "zotero-owned-install",
  "restart-ready-live-contract",
  "search-write-replay-duplicate-redaction",
  "disable-reenable-remove-fallback",
  "disposable-profile-removed"
];

function sha256(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

function writeJson(path, value) {
  const bytes = Buffer.from(`${JSON.stringify(value, null, 2)}\n`);
  writeFileSync(path, bytes, { mode: 0o600 });
  return bytes;
}

function createFixture() {
  const root = realpathSync(mkdtempSync(join(tmpdir(), "qiongli-r5f-manual-")));
  const productPath = join(root, "product.json");
  const zoteroAutomatedPath = join(root, "zotero-automated.json");
  const zoteroManualPath = join(root, "zotero-manual.json");
  const outputPath = join(root, "r5f-manual.json");
  writeJson(productPath, {
    schema_version: 3,
    record_type: "qiongli-packaged-product-acceptance",
    status: "accepted-ad-hoc-nonpublishing",
    publication_allowed: false,
    product_source_commit: BUILD,
    canonical_sha256: A64,
    product_control_sha256: B64,
    signed_archive_sha256: C64,
    zotero_companion: {
      companion_version: "0.3.0",
      endpoint_version: "2",
      xpi_sha256: D64,
      artifact_manifest_sha256: A64
    },
    checks: Object.fromEntries(PRODUCT_CHECKS.map((check) => [check, true]))
  });
  const automatedBytes = writeJson(zoteroAutomatedPath, {
    schemaVersion: 1,
    recordType: "qiongli-r5d-zotero-automated-acceptance",
    status: "accepted-automated-nonpublishing",
    publicationAllowed: false,
    productBuild: BUILD,
    executableSha256: A64,
    companion: {
      version: "0.3.0",
      endpointVersion: "2",
      xpiSha256: D64
    },
    checks: Object.fromEntries(ZOTERO_CHECKS.map((check) => [check, true]))
  });
  writeJson(zoteroManualPath, {
    schemaVersion: 1,
    recordType: "qiongli-r5d-zotero-manual-acceptance",
    status: "accepted-manual-nonpublishing",
    publicationAllowed: false,
    source: {
      productBuild: BUILD,
      executableSha256: A64,
      automatedReceiptSha256: sha256(automatedBytes)
    },
    companion: {
      version: "0.3.0",
      endpointVersion: "2",
      xpiSha256: D64
    },
    confirmations: Object.fromEntries(
      ZOTERO_CONFIRMATIONS.map((confirmation) => [confirmation, "confirmed"])
    ),
    constraints: {
      disposableProfile: true,
      syncDisabled: true,
      qiongliProfileMutationBeforeZoteroConfirmation: false,
      evidencePublishingAllowed: false
    }
  });
  return {
    root,
    productPath,
    zoteroAutomatedPath,
    zoteroManualPath,
    outputPath
  };
}

function run(fixture, confirmations = REQUIRED_CONFIRMATIONS) {
  const argumentsList = [
    SCRIPT,
    "--product-receipt", fixture.productPath,
    "--zotero-automated-receipt", fixture.zoteroAutomatedPath,
    "--zotero-manual-receipt", fixture.zoteroManualPath,
    "--out", fixture.outputPath,
    "--operator-id", "macos-manual-01",
    ...confirmations.flatMap((confirmation) => ["--confirm", confirmation])
  ];
  return spawnSync(process.execPath, argumentsList, { encoding: "utf8" });
}

test("lists every manual gate with an observation contract", () => {
  const result = spawnSync(process.execPath, [SCRIPT, "--", "--list-gates"], {
    encoding: "utf8"
  });
  assert.equal(result.status, 0);
  for (const confirmation of REQUIRED_CONFIRMATIONS) {
    assert.match(result.stdout, new RegExp(`^${confirmation}\\t`, "m"));
  }
});

test("binds complete R5F observations and the R5D chain to one product", () => {
  const fixture = createFixture();
  try {
    const result = run(fixture);
    assert.equal(result.status, 0, result.stderr);
    const receipt = JSON.parse(readFileSync(fixture.outputPath, "utf8"));
    assert.equal(receipt.status, "accepted-manual-nonpublishing");
    assert.equal(receipt.publicationAllowed, false);
    assert.equal(receipt.source.productBuild, BUILD);
    assert.equal(receipt.source.canonicalSha256, A64);
    assert.match(receipt.manualGateContractSha256, /^[0-9a-f]{64}$/);
    assert.deepEqual(receipt.constraints.exactWidths, [375, 768, 1024, 1440]);
    assert.equal(Object.keys(receipt.confirmations).length, REQUIRED_CONFIRMATIONS.length);
  } finally {
    rmSync(fixture.root, { recursive: true, force: true });
  }
});

test("fails closed when any observation is missing", () => {
  const fixture = createFixture();
  try {
    const result = run(fixture, REQUIRED_CONFIRMATIONS.slice(1));
    assert.equal(result.status, 1);
    assert.match(result.stderr, /r5f-manual-confirmation-missing:/);
    assert.equal(result.stdout, "");
  } finally {
    rmSync(fixture.root, { recursive: true, force: true });
  }
});

test("rejects a Zotero manual receipt from another product build", () => {
  const fixture = createFixture();
  try {
    const receipt = JSON.parse(readFileSync(fixture.zoteroManualPath, "utf8"));
    receipt.source.productBuild = "2".repeat(40);
    writeJson(fixture.zoteroManualPath, receipt);
    const result = run(fixture);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /r5f-manual-zotero-manual-receipt-invalid/);
  } finally {
    rmSync(fixture.root, { recursive: true, force: true });
  }
});
