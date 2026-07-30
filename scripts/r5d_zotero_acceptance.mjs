#!/usr/bin/env node

import { createHash, randomBytes } from "node:crypto";
import {
  existsSync,
  lstatSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  renameSync,
  rmSync,
  unlinkSync,
  writeFileSync
} from "node:fs";
import { dirname, isAbsolute, join, resolve } from "node:path";
import { tmpdir } from "node:os";
import { fileURLToPath, pathToFileURL } from "node:url";
import { spawnSync } from "node:child_process";

const SCRIPT_DIRECTORY = dirname(fileURLToPath(import.meta.url));
const REPOSITORY_ROOT = resolve(SCRIPT_DIRECTORY, "..");
const DEFAULT_APP = resolve(REPOSITORY_ROOT, "dist/macos/Qiongli.app");
const DEFAULT_RECEIPT = resolve(
  REPOSITORY_ROOT,
  "dist/macos/qiongli-r5d-zotero-acceptance.receipt.json"
);
const SHA256_PATTERN = /^[0-9a-f]{64}$/;
const PRODUCT_VERSION_PATTERN = /^\d+\.\d+\.\d+(?:-(?:alpha|beta)\.\d+)?$/;
const MAX_OUTPUT_BYTES = 8 * 1024 * 1024;
const RELEASE_REPOSITORY = "jxpeng98/qiongli";
const APP_SNAPSHOT_SCHEMA_VERSION = 14;

class AcceptanceError extends Error {
  constructor(reasonCode) {
    super(reasonCode);
    this.reasonCode = reasonCode;
  }
}

function fail(reasonCode) {
  throw new AcceptanceError(reasonCode);
}

function usage() {
  return `Qiongli R5D Zotero automated acceptance

Usage:
  node scripts/r5d_zotero_acceptance.mjs [--app <absolute-Qiongli.app>] [--receipt <absolute-json>]

This is non-publishing automated evidence. It verifies the packaged Companion
identity, a read-only App snapshot in an isolated HOME, the native state matrix,
and the disposable Companion/Full MCP search and approved-write lifecycle.
Zotero-owned installation confirmation, restart, disable, and removal remain
manual gates.
`;
}

function parseArguments(argv) {
  const result = { app: DEFAULT_APP, receipt: DEFAULT_RECEIPT, help: false };
  for (let index = 0; index < argv.length; index += 1) {
    const option = argv[index];
    if (option === "-h" || option === "--help") {
      result.help = true;
      continue;
    }
    if (option !== "--app" && option !== "--receipt") {
      fail("r5d-zotero-acceptance-option-invalid");
    }
    const value = argv[index + 1];
    if (!value || value.startsWith("--") || !isAbsolute(value)) {
      fail("r5d-zotero-acceptance-path-invalid");
    }
    result[option === "--app" ? "app" : "receipt"] = value;
    index += 1;
  }
  return result;
}

function sha256(value) {
  return createHash("sha256").update(value).digest("hex");
}

function readRegularFile(path, reasonCode, maximumBytes = 2 * 1024 * 1024) {
  let metadata;
  try {
    metadata = lstatSync(path);
  } catch {
    fail(reasonCode);
  }
  if (!metadata.isFile() || metadata.isSymbolicLink() || metadata.size < 1 || metadata.size > maximumBytes) {
    fail(reasonCode);
  }
  return readFileSync(path);
}

function run(command, args, { env = process.env, timeout = 300_000, reasonCode } = {}) {
  const outcome = spawnSync(command, args, {
    cwd: REPOSITORY_ROOT,
    env,
    encoding: "utf8",
    maxBuffer: MAX_OUTPUT_BYTES,
    timeout
  });
  if (outcome.error || outcome.status !== 0) {
    if (process.env.QIONGLI_ACCEPTANCE_DIAGNOSTICS === "1") {
      const detail = [
        `command: ${command} ${args.join(" ")}`,
        `status: ${String(outcome.status)}`,
        `signal: ${String(outcome.signal)}`,
        `error: ${outcome.error?.message ?? ""}`,
        "stdout:",
        outcome.stdout ?? "",
        "stderr:",
        outcome.stderr ?? ""
      ].join("\n");
      process.stderr.write(`${detail.slice(0, 32 * 1024)}\n`);
    }
    fail(reasonCode ?? "r5d-zotero-acceptance-command-failed");
  }
  return outcome.stdout;
}

function validateApp(app) {
  let metadata;
  try {
    metadata = lstatSync(app);
  } catch {
    fail("r5d-zotero-app-missing");
  }
  if (!metadata.isDirectory() || metadata.isSymbolicLink()) {
    fail("r5d-zotero-app-invalid");
  }
  const executable = join(app, "Contents/MacOS/qiongli-cli");
  const resources = join(app, "Contents/Resources");
  const xpiPath = join(resources, "Zotero/qiongli-zotero-companion.xpi");
  const manifestPath = join(resources, "Zotero/qiongli-zotero-companion.manifest.json");
  const packageManifestPath = join(resources, ".qiongli-desktop-package.json");
  const executableBytes = readRegularFile(executable, "r5d-zotero-app-executable-invalid", 128 * 1024 * 1024);
  const xpiBytes = readRegularFile(xpiPath, "r5d-zotero-xpi-invalid");
  const manifestBytes = readRegularFile(manifestPath, "r5d-zotero-artifact-manifest-invalid", 64 * 1024);
  const manifest = JSON.parse(manifestBytes.toString("utf8"));
  const packageManifestBytes = existsSync(packageManifestPath)
    ? readRegularFile(packageManifestPath, "r5d-zotero-package-manifest-invalid")
    : null;
  const packageManifest = packageManifestBytes
    ? JSON.parse(packageManifestBytes.toString("utf8"))
    : null;

  if (manifest.schema_version !== 1
    || manifest.record_type !== "qiongli-zotero-companion-artifact"
    || manifest.status !== "assembled-unpublished"
    || manifest.companion_version !== "0.3.0"
    || manifest.endpoint_version !== "2"
    || manifest.zotero_min_version !== "8.0"
    || manifest.zotero_max_version !== "9.0.*"
    || manifest.artifact_size_bytes !== xpiBytes.length
    || manifest.artifact_sha256 !== sha256(xpiBytes)
    || !SHA256_PATTERN.test(manifest.entry_content_root_sha256 ?? "")) {
    fail("r5d-zotero-artifact-identity-invalid");
  }
  if (packageManifest
    && (packageManifest.zotero_companion?.companion_version !== manifest.companion_version
      || packageManifest.zotero_companion?.endpoint_version !== manifest.endpoint_version
      || packageManifest.zotero_companion?.xpi_sha256 !== manifest.artifact_sha256)) {
    fail("r5d-zotero-package-binding-invalid");
  }
  return {
    executable,
    xpiBytes,
    executableSha256: sha256(executableBytes),
    packageManifestSha256: packageManifestBytes ? sha256(packageManifestBytes) : null,
    productVersion: packageManifest?.artifact?.version ?? null,
    productBuild: packageManifest?.product_source_commit ?? null,
    desktopPackageManifestBound: packageManifest !== null,
    companion: {
      version: manifest.companion_version,
      endpointVersion: manifest.endpoint_version,
      zoteroMinimumVersion: manifest.zotero_min_version,
      zoteroMaximumVersion: manifest.zotero_max_version,
      xpiBytes: xpiBytes.length,
      xpiSha256: manifest.artifact_sha256,
      artifactManifestSha256: sha256(manifestBytes)
    }
  };
}

function validateReleaseArtifact(app, root, productVersion) {
  if (!PRODUCT_VERSION_PATTERN.test(productVersion ?? "")) {
    fail("r5d-zotero-release-version-invalid");
  }
  const releaseTag = `v${productVersion}`;
  const dist = join(root, "release-artifact");
  mkdirSync(dist, { mode: 0o700 });
  run("python3", [
    "scripts/build_zotero_companion.py",
    "--dist-dir",
    dist,
    "--release-tag",
    releaseTag,
    "--repo",
    RELEASE_REPOSITORY
  ], { reasonCode: "r5d-zotero-release-artifact-build-failed" });

  const releaseXpi = readRegularFile(
    join(dist, `qiongli-zotero-companion-${app.companion.version}.xpi`),
    "r5d-zotero-release-xpi-invalid"
  );
  if (!releaseXpi.equals(app.xpiBytes)) {
    fail("r5d-zotero-release-app-xpi-drift");
  }
  const updateManifestBytes = readRegularFile(
    join(dist, "qiongli-zotero-companion-updates.json"),
    "r5d-zotero-update-manifest-invalid",
    64 * 1024
  );
  let updateManifest;
  try {
    updateManifest = JSON.parse(updateManifestBytes.toString("utf8"));
  } catch {
    fail("r5d-zotero-update-manifest-invalid");
  }
  const addons = updateManifest.addons;
  const companionId = "qiongli-zotero-companion@qiongli.local";
  const updates = addons?.[companionId]?.updates;
  const update = Array.isArray(updates) && updates.length === 1
    ? updates[0]
    : null;
  const expectedLink = (
    `https://github.com/${RELEASE_REPOSITORY}/releases/download/`
    + `${releaseTag}/qiongli-zotero-companion-${app.companion.version}.xpi`
  );
  if (!addons
    || Object.keys(addons).length !== 1
    || update?.version !== app.companion.version
    || update?.update_link !== expectedLink
    || update?.update_hash !== `sha256:${app.companion.xpiSha256}`
    || update?.applications?.zotero?.strict_min_version
      !== app.companion.zoteroMinimumVersion
    || update?.applications?.zotero?.strict_max_version
      !== app.companion.zoteroMaximumVersion) {
    fail("r5d-zotero-update-manifest-invalid");
  }
  return {
    releaseTag,
    updateLink: expectedLink,
    updateManifestSha256: sha256(updateManifestBytes)
  };
}

function assertNoZoteroProfile(home) {
  for (const candidate of [
    join(home, ".zotero"),
    join(home, "Zotero"),
    join(home, "Library/Application Support/Zotero")
  ]) {
    if (existsSync(candidate)) {
      fail("r5d-zotero-profile-mutated");
    }
  }
}

function validateIsolatedSnapshot(executable, home) {
  assertNoZoteroProfile(home);
  const stdout = run(executable, ["app", "snapshot"], {
    env: {
      ...process.env,
      HOME: home,
      CODEX_HOME: join(home, ".codex"),
      CLAUDE_CONFIG_DIR: join(home, ".claude"),
      PATH: ""
    },
    timeout: 30_000,
    reasonCode: "r5d-zotero-isolated-snapshot-failed"
  });
  let snapshot;
  try {
    snapshot = JSON.parse(stdout);
  } catch {
    fail("r5d-zotero-isolated-snapshot-invalid");
  }
  if (snapshot.schemaVersion !== APP_SNAPSHOT_SCHEMA_VERSION
    || snapshot.zotero?.state !== "not-observed"
    || snapshot.zotero?.observation !== "not-observed"
    || snapshot.zotero?.connectorAvailable !== false
    || snapshot.zotero?.companionAvailable !== false
    || snapshot.zotero?.availableCompanionVersion !== "0.3.0"
    || snapshot.zotero?.supportedEndpointVersion !== "2"
    || snapshot.zotero?.fallbackImportAvailable !== true) {
    fail("r5d-zotero-isolated-snapshot-invalid");
  }
  assertNoZoteroProfile(home);
  return {
    productVersion: snapshot.product?.version,
    productBuild: snapshot.product?.build
  };
}

function runQualificationTests() {
  run(process.execPath, [
    "--test",
    "packages/qiongli-zotero-companion/test/bridge.test.mjs"
  ], { reasonCode: "r5d-zotero-companion-lifecycle-failed" });
  run(process.execPath, [
    "--test",
    "packages/qiongli-literature-mcpb/test/zotero.test.mjs",
    "packages/qiongli-literature-mcpb/test/tools.test.mjs"
  ], { reasonCode: "r5d-zotero-full-mcp-qualification-failed" });
  run("cargo", [
    "test",
    "--manifest-path",
    "packages/qiongli-native/Cargo.toml",
    "-p",
    "qiongli",
    "zotero_acceptance_state_matrix_never_treats_staged_or_stale_evidence_as_ready",
    "--lib"
  ], { reasonCode: "r5d-zotero-native-state-matrix-failed" });
  run("cargo", [
    "test",
    "--manifest-path",
    "packages/qiongli-native/Cargo.toml",
    "-p",
    "qiongli-platform",
    "zotero_companion"
  ], { reasonCode: "r5d-zotero-platform-artifact-failed" });
  run("python3", [
    "-m",
    "unittest",
    "tests.test_zotero_companion_artifact"
  ], { reasonCode: "r5d-zotero-cross-language-artifact-failed" });
}

function writeReceipt(path, receipt) {
  mkdirSync(dirname(path), { recursive: true, mode: 0o700 });
  const temporary = `${path}.tmp.${process.pid}.${randomBytes(8).toString("hex")}`;
  try {
    writeFileSync(temporary, `${JSON.stringify(receipt, null, 2)}\n`, {
      encoding: "utf8",
      mode: 0o600,
      flag: "wx"
    });
    renameSync(temporary, path);
  } finally {
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
  if (process.platform !== "darwin") {
    fail("r5d-zotero-acceptance-requires-macos");
  }

  const app = validateApp(options.app);
  const isolatedRoot = mkdtempSync(join(tmpdir(), "qiongli-r5d-zotero-"));
  let snapshotIdentity;
  let releaseArtifact;
  try {
    const home = join(isolatedRoot, "home");
    mkdirSync(home, { mode: 0o700 });
    snapshotIdentity = validateIsolatedSnapshot(app.executable, home);
    releaseArtifact = validateReleaseArtifact(
      app,
      isolatedRoot,
      app.productVersion ?? snapshotIdentity.productVersion
    );
    runQualificationTests();
  } finally {
    rmSync(isolatedRoot, { recursive: true, force: true });
  }

  const receipt = {
    schemaVersion: 1,
    recordType: "qiongli-r5d-zotero-automated-acceptance",
    status: "accepted-automated-nonpublishing",
    publicationAllowed: false,
    recordedAtUnix: Math.floor(Date.now() / 1000),
    productVersion: app.productVersion ?? snapshotIdentity.productVersion,
    productBuild: app.productBuild ?? snapshotIdentity.productBuild,
    executableSha256: app.executableSha256,
    packageManifestSha256: app.packageManifestSha256,
    companion: {
      ...app.companion,
      releaseTag: releaseArtifact.releaseTag,
      updateLink: releaseArtifact.updateLink,
      updateManifestSha256: releaseArtifact.updateManifestSha256
    },
    checks: {
      appResourceArtifactBound: true,
      desktopPackageManifestBound: app.desktopPackageManifestBound,
      releaseArtifactByteIdentity: true,
      automaticUpdateManifestBound: true,
      startupSnapshotObservationNeutral: true,
      isolatedHomeProfileUnchanged: true,
      nativeStateMatrix: true,
      legacyEndpointRequiresUpdate: true,
      disposableSearchCollectionsTagsNotesAttachments: true,
      approvedWriteReceiptLifecycle: true,
      duplicateAndCuratedMetadataPreservation: true,
      endpointShutdownRemoval: true,
      importFileFallback: true
    },
    manualGates: {
      zoteroOwnedInstallConfirmation: "not-run",
      restartActivationObservation: "not-run",
      displayedAppStateReview: "not-run",
      companionDisable: "not-run",
      companionRemoval: "not-run"
    }
  };
  writeReceipt(options.receipt, receipt);
  process.stdout.write(`${JSON.stringify({
    status: receipt.status,
    publicationAllowed: false,
    receipt: options.receipt,
    companionVersion: receipt.companion.version,
    endpointVersion: receipt.companion.endpointVersion
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
    const reasonCode = error instanceof AcceptanceError
      ? error.reasonCode
      : "r5d-zotero-acceptance-internal-error";
    process.stderr.write(`R5D Zotero acceptance failed: ${reasonCode}\n`);
    process.exitCode = 1;
  }
}
