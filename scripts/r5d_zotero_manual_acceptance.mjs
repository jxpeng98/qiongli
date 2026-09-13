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
const DEFAULT_AUTOMATED_RECEIPT = resolve(
  REPOSITORY_ROOT,
  "dist/macos-acceptance/current/qiongli-r5d-zotero-acceptance.receipt.json"
);
const DEFAULT_OUTPUT = resolve(
  REPOSITORY_ROOT,
  "dist/macos-acceptance/current/qiongli-r5d-zotero-manual-acceptance.receipt.json"
);
const SHA256_PATTERN = /^[0-9a-f]{64}$/;
const SOURCE_COMMIT_PATTERN = /^[0-9a-f]{40}$/;
const PRODUCT_VERSION_PATTERN = /^\d+\.\d+\.\d+(?:-(?:alpha|beta)\.\d+)?$/;
const OPERATOR_ID_PATTERN = /^[0-9A-Za-z][0-9A-Za-z._-]{0,63}$/;
const MAX_RECEIPT_BYTES = 256 * 1024;
const REQUIRED_AUTOMATED_CHECKS = Object.freeze([
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
const REQUIRED_PENDING_MANUAL_GATES = Object.freeze([
  "zoteroOwnedInstallConfirmation",
  "restartActivationObservation",
  "displayedAppStateReview",
  "companionDisable",
  "companionRemoval"
]);
const REQUIRED_CONFIRMATIONS = Object.freeze([
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
  return `Record Qiongli R5D Zotero manual acceptance

Usage:
  node scripts/r5d_zotero_manual_acceptance.mjs \\
    [--automated-receipt <absolute-json>] \\
    [--out <absolute-json>] \\
    --operator-id <non-personal-label> \\
    --confirm <gate> [...]

Options:
  --list-gates  Print the required gate identifiers and exit.
  -h, --help    Show this help.

This command does not inspect or modify a Zotero profile. It records a bounded,
non-publishing human attestation only after every gate has been exercised
against the same clean-commit packaged App named by the automated receipt.
`;
}

function parseArguments(argv) {
  const result = {
    automatedReceipt: DEFAULT_AUTOMATED_RECEIPT,
    output: DEFAULT_OUTPUT,
    operatorId: null,
    confirmations: [],
    help: false,
    listGates: false
  };
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
    if (!["--automated-receipt", "--out", "--operator-id", "--confirm"].includes(option)) {
      fail("r5d-zotero-manual-option-invalid");
    }
    const value = argv[index + 1];
    if (!value || value.startsWith("--")) {
      fail("r5d-zotero-manual-option-value-missing");
    }
    if (option === "--automated-receipt" || option === "--out") {
      if (!isAbsolute(value)) {
        fail("r5d-zotero-manual-path-invalid");
      }
      result[option === "--out" ? "output" : "automatedReceipt"] = value;
    } else if (option === "--operator-id") {
      if (result.operatorId !== null) {
        fail("r5d-zotero-manual-operator-duplicate");
      }
      result.operatorId = value;
    } else {
      result.confirmations.push(value);
    }
    index += 1;
  }
  return result;
}

function sha256(value) {
  return createHash("sha256").update(value).digest("hex");
}

function readAutomatedReceipt(path) {
  let metadata;
  try {
    metadata = lstatSync(path);
  } catch {
    fail("r5d-zotero-manual-automated-receipt-missing");
  }
  if (!metadata.isFile()
    || metadata.isSymbolicLink()
    || metadata.size < 1
    || metadata.size > MAX_RECEIPT_BYTES) {
    fail("r5d-zotero-manual-automated-receipt-invalid");
  }
  const bytes = readFileSync(path);
  let receipt;
  try {
    receipt = JSON.parse(bytes.toString("utf8"));
  } catch {
    fail("r5d-zotero-manual-automated-receipt-invalid");
  }
  const manualGates = receipt.manualGates;
  const checks = receipt.checks;
  const companion = receipt.companion;
  if (receipt.schemaVersion !== 1
    || receipt.recordType !== "qiongli-r5d-zotero-automated-acceptance"
    || receipt.status !== "accepted-automated-nonpublishing"
    || receipt.publicationAllowed !== false
    || !PRODUCT_VERSION_PATTERN.test(receipt.productVersion ?? "")
    || !SOURCE_COMMIT_PATTERN.test(receipt.productBuild ?? "")
    || !SHA256_PATTERN.test(receipt.executableSha256 ?? "")
    || !SHA256_PATTERN.test(receipt.packageManifestSha256 ?? "")
    || !checks
    || REQUIRED_AUTOMATED_CHECKS.some((check) => checks[check] !== true)
    || !manualGates
    || Object.keys(manualGates).length !== REQUIRED_PENDING_MANUAL_GATES.length
    || REQUIRED_PENDING_MANUAL_GATES.some(
      (gate) => manualGates[gate] !== "not-run"
    )
    || companion?.version !== "0.3.0"
    || companion?.endpointVersion !== "2"
    || companion?.zoteroMinimumVersion !== "8.0"
    || companion?.zoteroMaximumVersion !== "9.0.*"
    || !SHA256_PATTERN.test(companion?.xpiSha256 ?? "")
    || !SHA256_PATTERN.test(companion?.artifactManifestSha256 ?? "")
    || !SHA256_PATTERN.test(companion?.updateManifestSha256 ?? "")
    || companion?.releaseTag !== `v${receipt.productVersion}`
    || companion?.updateLink
      !== `https://github.com/jxpeng98/qiongli/releases/download/${companion.releaseTag}/qiongli-zotero-companion-0.3.0.xpi`) {
    fail("r5d-zotero-manual-automated-receipt-invalid");
  }
  return { bytes, receipt };
}

function validateConfirmations(values) {
  const provided = new Set(values);
  if (provided.size !== values.length) {
    fail("r5d-zotero-manual-confirmation-duplicate");
  }
  const unknown = values.filter((value) => !REQUIRED_CONFIRMATIONS.includes(value));
  if (unknown.length > 0) {
    fail("r5d-zotero-manual-confirmation-invalid");
  }
  const missing = REQUIRED_CONFIRMATIONS.filter((value) => !provided.has(value));
  if (missing.length > 0) {
    fail(`r5d-zotero-manual-confirmation-missing:${missing.join(",")}`);
  }
}

function validateOutput(path) {
  if (existsSync(path)) {
    fail("r5d-zotero-manual-output-exists");
  }
  const parent = dirname(path);
  let metadata;
  try {
    metadata = lstatSync(parent);
  } catch {
    fail("r5d-zotero-manual-output-parent-invalid");
  }
  if (!metadata.isDirectory()
    || metadata.isSymbolicLink()
    || realpathSync(parent) !== parent) {
    fail("r5d-zotero-manual-output-parent-invalid");
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
    if (descriptor !== undefined) {
      closeSync(descriptor);
    }
    try {
      unlinkSync(temporary);
    } catch (error) {
      if (error?.code !== "ENOENT") throw error;
    }
  }
}

function main(argv = process.argv.slice(2)) {
  const options = parseArguments(argv);
  if (options.help) {
    process.stdout.write(usage());
    return 0;
  }
  if (options.listGates) {
    process.stdout.write(`${REQUIRED_CONFIRMATIONS.join("\n")}\n`);
    return 0;
  }
  if (!OPERATOR_ID_PATTERN.test(options.operatorId ?? "")) {
    fail("r5d-zotero-manual-operator-invalid");
  }
  validateConfirmations(options.confirmations);
  validateOutput(options.output);
  const automated = readAutomatedReceipt(options.automatedReceipt);
  const source = automated.receipt;
  const receipt = {
    schemaVersion: 1,
    recordType: "qiongli-r5d-zotero-manual-acceptance",
    status: "accepted-manual-nonpublishing",
    publicationAllowed: false,
    recordedAtUnix: Math.floor(Date.now() / 1000),
    operatorId: options.operatorId,
    source: {
      productVersion: source.productVersion,
      productBuild: source.productBuild,
      executableSha256: source.executableSha256,
      packageManifestSha256: source.packageManifestSha256,
      automatedReceiptFile: basename(options.automatedReceipt),
      automatedReceiptSha256: sha256(automated.bytes)
    },
    companion: {
      version: source.companion.version,
      endpointVersion: source.companion.endpointVersion,
      xpiSha256: source.companion.xpiSha256,
      updateManifestSha256: source.companion.updateManifestSha256
    },
    confirmations: Object.fromEntries(
      REQUIRED_CONFIRMATIONS.map((confirmation) => [confirmation, "confirmed"])
    ),
    constraints: {
      disposableProfile: true,
      syncDisabled: true,
      qiongliProfileMutationBeforeZoteroConfirmation: false,
      evidencePublishingAllowed: false
    },
    reason: (
      "manual observations are bound to the clean-commit packaged acceptance "
      + "receipt; parent Beta publication gates remain independent"
    )
  };
  writeReceipt(options.output, receipt);
  process.stdout.write(`${JSON.stringify({
    status: receipt.status,
    publicationAllowed: false,
    productBuild: receipt.source.productBuild,
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
      : "r5d-zotero-manual-internal-error";
    process.stderr.write(`R5D Zotero manual acceptance failed: ${reasonCode}\n`);
    process.exitCode = 1;
  }
}
