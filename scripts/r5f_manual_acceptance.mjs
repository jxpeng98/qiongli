#!/usr/bin/env node

import { createHash } from "node:crypto";
import {
  closeSync,
  existsSync,
  fsyncSync,
  lstatSync,
  openSync,
  readFileSync,
  realpathSync,
  renameSync,
  unlinkSync,
  writeFileSync
} from "node:fs";
import { basename, dirname, isAbsolute, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const SCRIPT_DIRECTORY = dirname(fileURLToPath(import.meta.url));
const REPOSITORY_ROOT = resolve(SCRIPT_DIRECTORY, "..");
const ACCEPTANCE_ROOT = resolve(REPOSITORY_ROOT, "dist/macos-acceptance/current");
const DEFAULT_PRODUCT_RECEIPT = resolve(
  ACCEPTANCE_ROOT,
  "qiongli-packaged-product-acceptance.receipt.json"
);
const DEFAULT_ZOTERO_AUTOMATED_RECEIPT = resolve(
  ACCEPTANCE_ROOT,
  "qiongli-r5d-zotero-acceptance.receipt.json"
);
const DEFAULT_ZOTERO_MANUAL_RECEIPT = resolve(
  ACCEPTANCE_ROOT,
  "qiongli-r5d-zotero-manual-acceptance.receipt.json"
);
const DEFAULT_OUTPUT = resolve(
  ACCEPTANCE_ROOT,
  "qiongli-r5f-manual-acceptance.receipt.json"
);
const SHA256_PATTERN = /^[0-9a-f]{64}$/;
const SOURCE_COMMIT_PATTERN = /^[0-9a-f]{40}$/;
const OPERATOR_ID_PATTERN = /^[0-9A-Za-z][0-9A-Za-z._-]{0,63}$/;
const MAX_RECEIPT_BYTES = 256 * 1024;

const REQUIRED_PRODUCT_CHECKS = Object.freeze([
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
  "cli_schema3_app_authority",
  "managed_operation_plan_apply",
  "standalone_skills_all_targets",
  "cli_plugin_reconcile_remove",
  "codex_install_verify_remove",
  "claude_install_verify_remove",
  "registration_repair",
  "packaged_restart_verification",
  "legacy_migration_fixture_isolated",
  "empty_path_startup"
]);

const REQUIRED_ZOTERO_AUTOMATED_CHECKS = Object.freeze([
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
]);

const REQUIRED_ZOTERO_CONFIRMATIONS = Object.freeze([
  "isolated-profile-no-sync",
  "missing-companion-fallback",
  "preview-cancel-no-profile-write",
  "legacy-update-handoff",
  "zotero-owned-install",
  "restart-ready-live-contract",
  "search-write-replay-duplicate-redaction",
  "disable-reenable-remove-fallback",
  "disposable-profile-removed"
]);

const MANUAL_GATES = Object.freeze([
  {
    id: "layout-375",
    description: "375px: all named routes, capsules, actions, navigation, and compact toolbar remain usable"
  },
  {
    id: "layout-768",
    description: "768px: all named routes remain free of page overflow, wrapped capsules, and clipped controls"
  },
  {
    id: "layout-1024",
    description: "1024px: content hierarchy, actions, and graph workspace remain stable and reachable"
  },
  {
    id: "layout-1440",
    description: "1440px: wide layouts remain bounded, aligned, and do not create excessive empty or detached regions"
  },
  {
    id: "notification-banner-lifecycle",
    description: "success, warning, and failure banners do not shift layout, dismiss, expire without hover persistence, and replace older notices"
  },
  {
    id: "confirmation-dialog-boundary",
    description: "destination and phase are visible; focus, Escape, backdrop, busy locking, and return focus are correct"
  },
  {
    id: "cli-path-current",
    description: "a new shell resolves the App-managed Qiongli 2 CLI and the displayed version matches the App"
  },
  {
    id: "cli-path-missing",
    description: "the bounded shell test reports a missing command without editing a shell profile"
  },
  {
    id: "cli-path-shadowed-legacy",
    description: "a legacy 1.x or mise/pip/npm command ahead of the managed target is reported as shadowed"
  },
  {
    id: "cli-path-version-mismatch",
    description: "a managed-path binary with nonmatching content is reported as a version mismatch"
  },
  {
    id: "codex-integration-lifecycle",
    description: "Codex install, verify, update/repair, host guidance, restart, and receipt-owned removal are causal"
  },
  {
    id: "claude-integration-lifecycle",
    description: "Claude Code install, verify, update/repair, host guidance, restart, and receipt-owned removal are causal"
  },
  {
    id: "integration-authority-and-restart",
    description: "source builds stay read-only, unmanaged canaries survive, and packaged state is rediscovered after restart"
  },
  {
    id: "skills-qiongli-managed-lifecycle",
    description: "Qiongli-managed standalone Skills install, verify, update, and remove through one receipt-owned target"
  },
  {
    id: "skills-registered-project-lifecycle",
    description: "an explicit active and ready registered project installs, verifies, updates, and removes Skills; archived, drifted, or stale previews fail closed; CLI and GUI retain one target ID and <project> preview label after restart"
  },
  {
    id: "skills-custom-target-lifecycle",
    description: "a native-picked custom target remains path-opaque and has individual verify, update, and remove actions"
  },
  {
    id: "skills-drift-restart-redaction",
    description: "drift is verify-only until recovery; restart rediscovers receipts without a path or stale preview"
  },
  {
    id: "graph-empty-sparse-connected",
    description: "empty, sparse, and connected fixtures distinguish extraction readiness from visualization state"
  },
  {
    id: "graph-risk-revision-path-portfolio",
    description: "risk, revision, path, neighbourhood, community, and portfolio views preserve authoritative evidence"
  },
  {
    id: "graph-bounded-large",
    description: "bounded-large fixtures remain responsive and visibly report truncation rather than completeness"
  },
  {
    id: "graph-keyboard-minimap-accessibility",
    description: "keyboard inspection, minimap, focus order, contrast, semantic zoom, and reduced motion are usable"
  },
  {
    id: "graph-restart-source-state",
    description: "restart rebuilds identical graph identities from authoritative project state and preserves failed-query context"
  }
]);

export const REQUIRED_CONFIRMATIONS = Object.freeze(
  MANUAL_GATES.map((gate) => gate.id)
);

class ManualAcceptanceError extends Error {
  constructor(reasonCode) {
    super(reasonCode);
    this.reasonCode = reasonCode;
  }
}

function fail(reasonCode) {
  throw new ManualAcceptanceError(reasonCode);
}

function usage() {
  return `Record Qiongli R5F control-plane and visualization manual acceptance

Usage:
  pnpm acceptance:r5f:manual-record -- \\
    [--product-receipt <absolute-json>] \\
    [--zotero-automated-receipt <absolute-json>] \\
    [--zotero-manual-receipt <absolute-json>] \\
    [--out <absolute-json>] \\
    --operator-id <non-personal-label> \\
    --confirm <gate> [...]

Options:
  --list-gates  Print every required gate and its observation contract.
  -h, --help    Show this help.

This command performs no installation and does not inspect client profiles. It
records a bounded, non-publishing human attestation only after every gate was
exercised against the exact clean-commit packaged App named by the automated
receipt. The completed R5D Zotero manual receipt must name the same product.
`;
}

function parseArguments(argv) {
  const result = {
    productReceipt: DEFAULT_PRODUCT_RECEIPT,
    zoteroAutomatedReceipt: DEFAULT_ZOTERO_AUTOMATED_RECEIPT,
    zoteroManualReceipt: DEFAULT_ZOTERO_MANUAL_RECEIPT,
    output: DEFAULT_OUTPUT,
    operatorId: null,
    confirmations: [],
    help: false,
    listGates: false
  };
  const valueOptions = new Set([
    "--product-receipt",
    "--zotero-automated-receipt",
    "--zotero-manual-receipt",
    "--out",
    "--operator-id",
    "--confirm"
  ]);
  for (let index = 0; index < argv.length; index += 1) {
    const option = argv[index];
    if (option === "--") {
      continue;
    }
    if (option === "-h" || option === "--help") {
      result.help = true;
      continue;
    }
    if (option === "--list-gates") {
      result.listGates = true;
      continue;
    }
    if (!valueOptions.has(option)) {
      fail("r5f-manual-option-invalid");
    }
    const value = argv[index + 1];
    if (!value || value.startsWith("--")) {
      fail("r5f-manual-option-value-missing");
    }
    if (option.endsWith("-receipt") || option === "--out") {
      if (!isAbsolute(value)) fail("r5f-manual-path-invalid");
      if (option === "--product-receipt") result.productReceipt = value;
      if (option === "--zotero-automated-receipt") result.zoteroAutomatedReceipt = value;
      if (option === "--zotero-manual-receipt") result.zoteroManualReceipt = value;
      if (option === "--out") result.output = value;
    } else if (option === "--operator-id") {
      if (result.operatorId !== null) fail("r5f-manual-operator-duplicate");
      result.operatorId = value;
    } else {
      result.confirmations.push(value);
    }
    index += 1;
  }
  return result;
}

function sha256(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

function readReceipt(path, missingCode, invalidCode) {
  let metadata;
  try {
    metadata = lstatSync(path);
  } catch {
    fail(missingCode);
  }
  if (!metadata.isFile()
    || metadata.isSymbolicLink()
    || metadata.size < 1
    || metadata.size > MAX_RECEIPT_BYTES) {
    fail(invalidCode);
  }
  const bytes = readFileSync(path);
  let receipt;
  try {
    receipt = JSON.parse(bytes.toString("utf8"));
  } catch {
    fail(invalidCode);
  }
  return { bytes, receipt };
}

function readProductReceipt(path) {
  const result = readReceipt(
    path,
    "r5f-manual-product-receipt-missing",
    "r5f-manual-product-receipt-invalid"
  );
  const receipt = result.receipt;
  if (receipt.schema_version !== 3
    || receipt.record_type !== "qiongli-packaged-product-acceptance"
    || receipt.status !== "accepted-ad-hoc-nonpublishing"
    || receipt.publication_allowed !== false
    || !SOURCE_COMMIT_PATTERN.test(receipt.product_source_commit ?? "")
    || !SHA256_PATTERN.test(receipt.canonical_sha256 ?? "")
    || !SHA256_PATTERN.test(receipt.product_control_sha256 ?? "")
    || !SHA256_PATTERN.test(receipt.signed_archive_sha256 ?? "")
    || receipt.zotero_companion?.companion_version !== "0.3.0"
    || receipt.zotero_companion?.endpoint_version !== "2"
    || !SHA256_PATTERN.test(receipt.zotero_companion?.xpi_sha256 ?? "")
    || !SHA256_PATTERN.test(receipt.zotero_companion?.artifact_manifest_sha256 ?? "")
    || !receipt.checks
    || REQUIRED_PRODUCT_CHECKS.some((check) => receipt.checks[check] !== true)) {
    fail("r5f-manual-product-receipt-invalid");
  }
  return result;
}

function readZoteroAutomatedReceipt(path, product) {
  const result = readReceipt(
    path,
    "r5f-manual-zotero-automated-receipt-missing",
    "r5f-manual-zotero-automated-receipt-invalid"
  );
  const receipt = result.receipt;
  if (receipt.schemaVersion !== 1
    || receipt.recordType !== "qiongli-r5d-zotero-automated-acceptance"
    || receipt.status !== "accepted-automated-nonpublishing"
    || receipt.publicationAllowed !== false
    || receipt.productBuild !== product.product_source_commit
    || receipt.executableSha256 !== product.canonical_sha256
    || receipt.companion?.version !== product.zotero_companion.companion_version
    || receipt.companion?.endpointVersion !== product.zotero_companion.endpoint_version
    || receipt.companion?.xpiSha256 !== product.zotero_companion.xpi_sha256
    || !receipt.checks
    || REQUIRED_ZOTERO_AUTOMATED_CHECKS.some((check) => receipt.checks[check] !== true)) {
    fail("r5f-manual-zotero-automated-receipt-invalid");
  }
  return result;
}

function readZoteroManualReceipt(path, product, automated) {
  const result = readReceipt(
    path,
    "r5f-manual-zotero-manual-receipt-missing",
    "r5f-manual-zotero-manual-receipt-invalid"
  );
  const receipt = result.receipt;
  if (receipt.schemaVersion !== 1
    || receipt.recordType !== "qiongli-r5d-zotero-manual-acceptance"
    || receipt.status !== "accepted-manual-nonpublishing"
    || receipt.publicationAllowed !== false
    || receipt.source?.productBuild !== product.product_source_commit
    || receipt.source?.executableSha256 !== product.canonical_sha256
    || receipt.source?.automatedReceiptSha256 !== sha256(automated.bytes)
    || receipt.companion?.version !== product.zotero_companion.companion_version
    || receipt.companion?.endpointVersion !== product.zotero_companion.endpoint_version
    || receipt.companion?.xpiSha256 !== product.zotero_companion.xpi_sha256
    || !receipt.confirmations
    || REQUIRED_ZOTERO_CONFIRMATIONS.some(
      (confirmation) => receipt.confirmations[confirmation] !== "confirmed"
    )
    || receipt.constraints?.disposableProfile !== true
    || receipt.constraints?.syncDisabled !== true
    || receipt.constraints?.qiongliProfileMutationBeforeZoteroConfirmation !== false
    || receipt.constraints?.evidencePublishingAllowed !== false) {
    fail("r5f-manual-zotero-manual-receipt-invalid");
  }
  return result;
}

function validateConfirmations(values) {
  const provided = new Set(values);
  if (provided.size !== values.length) {
    fail("r5f-manual-confirmation-duplicate");
  }
  if (values.some((value) => !REQUIRED_CONFIRMATIONS.includes(value))) {
    fail("r5f-manual-confirmation-invalid");
  }
  const missing = REQUIRED_CONFIRMATIONS.filter((value) => !provided.has(value));
  if (missing.length > 0) {
    fail(`r5f-manual-confirmation-missing:${missing.join(",")}`);
  }
}

function validateOutput(path) {
  if (existsSync(path)) fail("r5f-manual-output-exists");
  const parent = dirname(path);
  let metadata;
  try {
    metadata = lstatSync(parent);
  } catch {
    fail("r5f-manual-output-parent-invalid");
  }
  if (!metadata.isDirectory()
    || metadata.isSymbolicLink()
    || realpathSync(parent) !== parent) {
    fail("r5f-manual-output-parent-invalid");
  }
}

function writeReceipt(path, receipt) {
  const temporary = `${path}.tmp.${process.pid}`;
  let descriptor;
  try {
    descriptor = openSync(temporary, "wx", 0o600);
    writeFileSync(descriptor, `${JSON.stringify(receipt, null, 2)}\n`, "utf8");
    fsyncSync(descriptor);
    closeSync(descriptor);
    descriptor = undefined;
    renameSync(temporary, path);
  } finally {
    if (descriptor !== undefined) closeSync(descriptor);
    try {
      unlinkSync(temporary);
    } catch (error) {
      if (error?.code !== "ENOENT") throw error;
    }
  }
}

export function main(argv = process.argv.slice(2), output = process.stdout) {
  const options = parseArguments(argv);
  if (options.help) {
    output.write(usage());
    return 0;
  }
  if (options.listGates) {
    output.write(MANUAL_GATES.map(
      (gate) => `${gate.id}\t${gate.description}`
    ).join("\n") + "\n");
    return 0;
  }
  if (!OPERATOR_ID_PATTERN.test(options.operatorId ?? "")) {
    fail("r5f-manual-operator-invalid");
  }
  validateConfirmations(options.confirmations);
  validateOutput(options.output);
  const product = readProductReceipt(options.productReceipt);
  const zoteroAutomated = readZoteroAutomatedReceipt(
    options.zoteroAutomatedReceipt,
    product.receipt
  );
  const zoteroManual = readZoteroManualReceipt(
    options.zoteroManualReceipt,
    product.receipt,
    zoteroAutomated
  );
  const receipt = {
    schemaVersion: 1,
    recordType: "qiongli-r5f-manual-acceptance",
    status: "accepted-manual-nonpublishing",
    publicationAllowed: false,
    recordedAtUnix: Math.floor(Date.now() / 1000),
    operatorId: options.operatorId,
    source: {
      productBuild: product.receipt.product_source_commit,
      canonicalSha256: product.receipt.canonical_sha256,
      productControlSha256: product.receipt.product_control_sha256,
      signedArchiveSha256: product.receipt.signed_archive_sha256,
      productReceiptFile: basename(options.productReceipt),
      productReceiptSha256: sha256(product.bytes),
      zoteroAutomatedReceiptFile: basename(options.zoteroAutomatedReceipt),
      zoteroAutomatedReceiptSha256: sha256(zoteroAutomated.bytes),
      zoteroManualReceiptFile: basename(options.zoteroManualReceipt),
      zoteroManualReceiptSha256: sha256(zoteroManual.bytes)
    },
    manualGateContractSha256: sha256(
      Buffer.from(JSON.stringify(MANUAL_GATES), "utf8")
    ),
    confirmations: Object.fromEntries(
      REQUIRED_CONFIRMATIONS.map((confirmation) => [confirmation, "confirmed"])
    ),
    constraints: {
      exactWidths: [375, 768, 1024, 1440],
      isolatedManualHome: true,
      sourceBuildEvidenceAccepted: false,
      authoritativeNativeEvidenceRequired: true,
      evidencePublishingAllowed: false
    },
    reason: (
      "manual control-plane, visualization, and Zotero observations are bound "
      + "to one clean-commit non-publishing packaged acceptance product"
    )
  };
  writeReceipt(options.output, receipt);
  output.write(`${JSON.stringify({
    status: receipt.status,
    publicationAllowed: false,
    productBuild: receipt.source.productBuild,
    confirmations: REQUIRED_CONFIRMATIONS.length,
    output: options.output
  })}\n`);
  return 0;
}

const invokedPath = process.argv[1]
  ? pathToFileURL(resolve(process.argv[1])).href
  : "";
if (import.meta.url === invokedPath) {
  try {
    process.exitCode = main();
  } catch (error) {
    const reasonCode = error instanceof ManualAcceptanceError
      ? error.reasonCode
      : "r5f-manual-internal-error";
    process.stderr.write(`R5F manual acceptance failed: ${reasonCode}\n`);
    process.exitCode = 1;
  }
}
