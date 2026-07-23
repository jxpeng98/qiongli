#!/usr/bin/env node

import { createHash, randomBytes } from "node:crypto";
import {
  lstatSync,
  mkdirSync,
  readFileSync,
  renameSync,
  unlinkSync,
  writeFileSync,
} from "node:fs";
import { dirname, isAbsolute, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { spawnSync } from "node:child_process";

const SCRIPT_DIRECTORY = dirname(fileURLToPath(import.meta.url));
const REPOSITORY_ROOT = resolve(SCRIPT_DIRECTORY, "..");
const DEFAULT_APP = resolve(REPOSITORY_ROOT, "dist/macos/Qiongli.app");
const DEFAULT_RECEIPT = resolve(
  REPOSITORY_ROOT,
  "dist/macos/qiongli-r4d-live-acceptance.receipt.json",
);
const PROJECT_ID_PATTERN = /^prj_[0-9a-f]{32}$/;
const SOURCE_COMMIT_PATTERN = /^[0-9a-f]{40}$/;
const SHA256_PATTERN = /^[0-9a-f]{64}$/;
const ACCEPTANCE_PROMPT =
  "Call one offered qiongli_project_* read tool for this registered project before answering. " +
  "Use the tool result as the only project evidence, report one concise verified project fact, " +
  "and state uncertainty if the tool result is insufficient.";

export class AcceptanceError extends Error {
  constructor(reasonCode) {
    super(reasonCode);
    this.name = "AcceptanceError";
    this.reasonCode = reasonCode;
  }
}

function fail(reasonCode) {
  throw new AcceptanceError(reasonCode);
}

export function usage() {
  return `Qiongli R4D live provider acceptance

Usage:
  pnpm run desktop:macos:r4d-acceptance -- \\
    --project-id <prj_id> \\
    --expected-project-revision <revision> \\
    --preflight

  pnpm run desktop:macos:r4d-acceptance -- \\
    --project-id <prj_id> \\
    --expected-project-revision <revision> \\
    --confirm-three-network-requests

Options:
  --preflight
      Build and validate the source App, Keychain-backed backend readiness, and
      project binding without making a provider request.
  --confirm-three-network-requests
      Explicitly permit one non-stored connection test and one bounded Full
      agent run that can make at most two additional provider requests.
  --app <absolute-Qiongli.app>
      Override the App bundle used for acceptance.
  --receipt <absolute-json-path>
      Override the successful live receipt path.
  -h, --help
      Show this help.

The API key must already be saved through the App's Model Backend page. It is
resolved from macOS Keychain and must not be passed through arguments, shell
history, environment variables, or .env files. The receipt contains hashes,
counts, fixed identifiers, and verdicts only; it excludes the key, prompt,
model answer, project path, and tool result.
`;
}

export function parseArguments(argv) {
  const result = {
    app: DEFAULT_APP,
    receipt: DEFAULT_RECEIPT,
    projectId: null,
    expectedProjectRevision: null,
    preflight: false,
    confirmThreeNetworkRequests: false,
    help: false,
    appProvided: false,
    receiptProvided: false,
  };
  for (let index = 0; index < argv.length; index += 1) {
    const option = argv[index];
    if (option === "-h" || option === "--help") {
      result.help = true;
      continue;
    }
    if (option === "--preflight") {
      if (result.preflight) fail("duplicate-preflight-option");
      result.preflight = true;
      continue;
    }
    if (option === "--confirm-three-network-requests") {
      if (result.confirmThreeNetworkRequests) {
        fail("duplicate-network-confirmation");
      }
      result.confirmThreeNetworkRequests = true;
      continue;
    }
    if (
      ![
        "--app",
        "--receipt",
        "--project-id",
        "--expected-project-revision",
      ].includes(option)
    ) {
      fail("unknown-acceptance-option");
    }
    const value = argv[index + 1];
    if (value === undefined || value.startsWith("--")) {
      fail("acceptance-option-value-required");
    }
    index += 1;
    if (option === "--app") {
      if (result.appProvided) fail("duplicate-app-option");
      result.app = value;
      result.appProvided = true;
    } else if (option === "--receipt") {
      if (result.receiptProvided) fail("duplicate-receipt-option");
      result.receipt = value;
      result.receiptProvided = true;
    } else if (option === "--project-id") {
      if (result.projectId !== null) fail("duplicate-project-id-option");
      result.projectId = value;
    } else {
      if (result.expectedProjectRevision !== null) {
        fail("duplicate-project-revision-option");
      }
      if (!/^[1-9][0-9]*$/.test(value)) {
        fail("project-revision-invalid");
      }
      const revision = Number(value);
      if (!Number.isSafeInteger(revision)) fail("project-revision-invalid");
      result.expectedProjectRevision = revision;
    }
  }

  if (result.help) return result;
  if (!PROJECT_ID_PATTERN.test(result.projectId ?? "")) {
    fail("project-id-invalid");
  }
  if (result.expectedProjectRevision === null) {
    fail("project-revision-required");
  }
  if (result.preflight === result.confirmThreeNetworkRequests) {
    fail("acceptance-mode-required");
  }
  if (!isAbsolute(result.app) || !isAbsolute(result.receipt)) {
    fail("acceptance-path-must-be-absolute");
  }
  result.app = resolve(result.app);
  result.receipt = resolve(result.receipt);
  delete result.appProvided;
  delete result.receiptProvided;
  return result;
}

function sha256(value) {
  return createHash("sha256").update(value).digest("hex");
}

function executableForApp(app) {
  return resolve(app, "Contents/MacOS/Qiongli");
}

function runProcess(executable, commandArguments, options = {}) {
  const outcome = spawnSync(executable, commandArguments, {
    cwd: options.cwd ?? REPOSITORY_ROOT,
    encoding: "utf8",
    env: options.env ?? process.env,
    input: options.input,
    maxBuffer: 2 * 1024 * 1024,
    timeout: options.timeout ?? 30_000,
    stdio: ["pipe", "pipe", "pipe"],
  });
  if (outcome.error?.code === "ETIMEDOUT") fail("acceptance-process-timeout");
  if (outcome.error) fail("acceptance-process-unavailable");
  if (outcome.status !== 0) fail(options.failureCode ?? "acceptance-process-failed");
  if (outcome.stderr !== "") fail("acceptance-process-stderr");
  return outcome.stdout;
}

function parseJson(text, failureCode) {
  try {
    return JSON.parse(text);
  } catch {
    fail(failureCode);
  }
}

function childEnvironment() {
  const environment = { ...process.env, PATH: "" };
  delete environment.OPENAI_API_KEY;
  delete environment.OPENAI_KEY;
  return environment;
}

function toolCall(id, name, argumentsValue) {
  return {
    jsonrpc: "2.0",
    id,
    method: "tools/call",
    params: { name, arguments: argumentsValue },
  };
}

function runFullMcp(executable, name, argumentsValue, timeout) {
  const requests = [
    {
      jsonrpc: "2.0",
      id: 1,
      method: "initialize",
      params: { protocolVersion: "2025-11-25", capabilities: {} },
    },
    toolCall(2, name, argumentsValue),
  ];
  const input = `${requests.map((request) => JSON.stringify(request)).join("\n")}\n`;
  const stdout = runProcess(
    executable,
    ["mcp", "serve", "--profile", "full", "--transport", "stdio"],
    {
      env: childEnvironment(),
      input,
      timeout,
      failureCode: "full-mcp-process-failed",
    },
  );
  const responses = stdout
    .split("\n")
    .filter((line) => line.length > 0)
    .map((line) => parseJson(line, "full-mcp-response-invalid"));
  const response = responses.find((candidate) => candidate.id === 2);
  if (response?.result?.isError === true) fail("full-mcp-tool-failed");
  if (response?.result?.structuredContent === undefined) {
    fail("full-mcp-result-missing");
  }
  return response.result.structuredContent;
}

export function validateBackendStatus(output) {
  const backend = output?.backend;
  if (
    output?.schema_version !== 1 ||
    output?.command !== "config-backend-status" ||
    !Number.isSafeInteger(output?.revision) ||
    output.revision < 1 ||
    backend?.schemaVersion !== 1 ||
    backend?.backendId !== "openai-responses" ||
    backend?.model !== "gpt-5.6-sol" ||
    backend?.enabled !== true ||
    backend?.readiness !== "ready" ||
    backend?.testAvailable !== true
  ) {
    fail("r4d-backend-not-ready");
  }
  return output;
}

export function validateProjectRead(output, projectId, expectedRevision) {
  const project = output?.project;
  if (
    output?.schemaVersion !== 1 ||
    !Number.isSafeInteger(output?.libraryRevision) ||
    output.libraryRevision < 1 ||
    project?.projectId !== projectId ||
    project?.semanticRevision !== expectedRevision ||
    project?.lifecycle !== "active" ||
    project?.health !== "ready"
  ) {
    fail("r4d-project-not-ready");
  }
  return output;
}

export function validateConnectionTest(output) {
  const test = output?.test;
  if (
    output?.schema_version !== 1 ||
    output?.command !== "config-backend-test" ||
    !Number.isSafeInteger(output?.revision) ||
    output.revision < 1 ||
    test?.schemaVersion !== 1 ||
    test?.backendId !== "openai-responses" ||
    test?.model !== "gpt-5.6-sol" ||
    test?.outcome !== "passed" ||
    test?.providerStorage !== "disabled" ||
    test?.hostedTools !== "disabled"
  ) {
    fail("r4d-connection-test-invalid");
  }
  return output;
}

export function validateAgentRun(output) {
  const execution = output?.executionUsage;
  const provider = output?.providerUsage;
  const audits = output?.toolAudits;
  if (
    output?.schemaVersion !== 1 ||
    !/^run_[0-9a-f]{32}$/.test(output?.runId ?? "") ||
    output?.backendId !== "openai-responses" ||
    output?.model !== "gpt-5.6-sol" ||
    output?.finishReason !== "stop" ||
    typeof output?.content !== "string" ||
    output.content.trim().length === 0 ||
    Buffer.byteLength(output.content) > 2 * 1024 * 1024 ||
    !Number.isSafeInteger(provider?.inputTokens) ||
    provider.inputTokens < 1 ||
    !Number.isSafeInteger(provider?.outputTokens) ||
    provider.outputTokens < 1 ||
    execution?.modelTurns !== 2 ||
    execution?.networkRequests !== 2 ||
    !Number.isSafeInteger(execution?.toolCalls) ||
    execution.toolCalls < 1 ||
    execution.toolCalls > 16 ||
    execution?.processes !== 0 ||
    execution?.artifacts !== 0 ||
    !Array.isArray(audits) ||
    audits.length !== execution.toolCalls ||
    audits.some(
      (audit) =>
        audit?.schemaVersion !== 1 ||
        audit?.toolClass !== "read" ||
        audit?.outcome !== "completed" ||
        !/^qiongli_project_[a-z0-9_]+$/.test(audit?.toolId ?? "") ||
        !SHA256_PATTERN.test(audit?.requestDigest ?? "") ||
        !SHA256_PATTERN.test(audit?.decisionDigest ?? ""),
    )
  ) {
    fail("r4d-agent-run-invalid");
  }
  return output;
}

export function validateStableBindings({
  sourceCommit,
  postflightSourceCommit,
  preflight,
  postflight,
  connectionTest,
}) {
  if (preflight.embeddedBuild !== sourceCommit) {
    fail("source-app-commit-mismatch");
  }
  if (connectionTest.revision !== preflight.backendStatus.revision) {
    fail("r4d-config-revision-drift");
  }
  if (
    postflight.embeddedBuild !== sourceCommit ||
    postflight.backendStatus.revision !== preflight.backendStatus.revision ||
    postflightSourceCommit !== sourceCommit
  ) {
    fail("r4d-postflight-binding-drift");
  }
}

function currentSourceCommit() {
  const sourceCommit = runProcess(
    "/usr/bin/git",
    ["rev-parse", "--verify", "HEAD"],
    {
    failureCode: "source-commit-unavailable",
    },
  ).trim();
  if (!SOURCE_COMMIT_PATTERN.test(sourceCommit)) {
    fail("source-commit-invalid");
  }
  const status = runProcess("/usr/bin/git", ["status", "--short"], {
    failureCode: "source-status-unavailable",
  });
  if (status !== "") fail("source-worktree-dirty");
  return sourceCommit;
}

function validateAppBundle(app) {
  let appStat;
  let executableStat;
  const executable = executableForApp(app);
  try {
    appStat = lstatSync(app);
    executableStat = lstatSync(executable);
  } catch {
    fail("source-app-missing");
  }
  if (
    !appStat.isDirectory() ||
    appStat.isSymbolicLink() ||
    !executableStat.isFile() ||
    executableStat.isSymbolicLink()
  ) {
    fail("source-app-invalid");
  }
  runProcess("/usr/bin/codesign", ["--verify", "--deep", "--strict", app], {
    failureCode: "source-app-signature-invalid",
  });
  runProcess(executable, ["ui", "--startup-check"], {
    env: childEnvironment(),
    failureCode: "source-app-startup-failed",
  });
  return executable;
}

export function buildReceipt({
  sourceCommit,
  executableBytes,
  version,
  backendStatus,
  projectId,
  expectedProjectRevision,
  connectionTest,
  agentRun,
  recordedAtUnix,
}) {
  const toolIds = [
    ...new Set(agentRun.toolAudits.map((audit) => audit.toolId)),
  ].sort();
  const receipt = {
    schemaVersion: 1,
    kind: "qiongli-r4d-live-provider-acceptance",
    status: "accepted",
    recordedAtUnix,
    sourceCommit,
    executableSha256: sha256(executableBytes),
    productVersion: version,
    target: {
      operatingSystem: process.platform,
      architecture: process.arch,
    },
    projectBindingSha256: sha256(
      `${projectId}\0${expectedProjectRevision}`,
    ),
    backend: {
      backendId: backendStatus.backend.backendId,
      model: backendStatus.backend.model,
      configRevision: backendStatus.revision,
      credentialSource: "macos-keychain",
      freshProcessResolution: true,
      connectionTest: connectionTest.test.outcome,
      providerStorage: connectionTest.test.providerStorage,
      hostedTools: connectionTest.test.hostedTools,
    },
    fullRun: {
      runId: agentRun.runId,
      finishReason: agentRun.finishReason,
      contentSha256: sha256(agentRun.content),
      contentBytes: Buffer.byteLength(agentRun.content),
      providerInputTokens: agentRun.providerUsage.inputTokens,
      providerOutputTokens: agentRun.providerUsage.outputTokens,
      modelTurns: agentRun.executionUsage.modelTurns,
      networkRequests: agentRun.executionUsage.networkRequests,
      toolCalls: agentRun.executionUsage.toolCalls,
      completedToolAudits: agentRun.toolAudits.length,
      toolIds,
      externalProcesses: agentRun.executionUsage.processes,
      artifactWrites: agentRun.executionUsage.artifacts,
    },
    boundaries: {
      explicitNetworkConfirmation: true,
      maximumProviderRequests: 3,
      externalAgentCliPath: "disabled",
      childPathEnvironment: "empty",
      promptPersisted: false,
      modelContentPersisted: false,
      credentialInArgumentsOrEnvironment: false,
      postflightBindingsVerified: true,
    },
  };
  const serialized = `${JSON.stringify(receipt, null, 2)}\n`;
  for (const privateValue of [projectId, agentRun.content, ACCEPTANCE_PROMPT]) {
    if (serialized.includes(privateValue)) fail("r4d-receipt-private-data");
  }
  return receipt;
}

function writeReceiptAtomically(path, receipt) {
  mkdirSync(dirname(path), { recursive: true, mode: 0o700 });
  try {
    const existing = lstatSync(path);
    if (existing.isSymbolicLink() || !existing.isFile()) {
      fail("r4d-receipt-target-unsafe");
    }
  } catch (error) {
    if (error?.code !== "ENOENT") throw error;
  }
  const temporary = `${path}.tmp.${process.pid}.${randomBytes(8).toString("hex")}`;
  try {
    writeFileSync(temporary, `${JSON.stringify(receipt, null, 2)}\n`, {
      encoding: "utf8",
      mode: 0o600,
      flag: "wx",
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

function runOfflinePreflight(options) {
  if (process.platform !== "darwin") fail("r4d-acceptance-requires-macos");
  const executable = validateAppBundle(options.app);
  const versionText = runProcess(executable, ["--version"], {
    env: childEnvironment(),
    failureCode: "source-app-version-failed",
  }).trim();
  if (!/^qiongli [0-9]+\.[0-9]+\.[0-9]+(?:-[0-9A-Za-z.-]+)?$/.test(versionText)) {
    fail("source-app-version-invalid");
  }
  const appSnapshot = parseJson(
    runProcess(executable, ["app", "snapshot"], {
      env: childEnvironment(),
      failureCode: "source-app-snapshot-failed",
    }),
    "source-app-snapshot-invalid",
  );
  if (
    appSnapshot?.schemaVersion !== 1 ||
    appSnapshot?.product?.version !== versionText.slice("qiongli ".length) ||
    typeof appSnapshot?.product?.build !== "string"
  ) {
    fail("source-app-snapshot-invalid");
  }
  const backendStatus = validateBackendStatus(
    parseJson(
      runProcess(executable, ["config", "backend", "status"], {
        env: childEnvironment(),
        failureCode: "backend-status-failed",
      }),
      "backend-status-json-invalid",
    ),
  );
  const projectRead = validateProjectRead(
    runFullMcp(
      executable,
      "qiongli_project_read",
      { project_id: options.projectId },
      30_000,
    ),
    options.projectId,
    options.expectedProjectRevision,
  );
  return {
    executable,
    version: versionText.slice("qiongli ".length),
    embeddedBuild: appSnapshot.product.build,
    backendStatus,
    projectRead,
  };
}

function preflightOutput(preflight, options) {
  return {
    schemaVersion: 1,
    kind: "qiongli-r4d-live-provider-acceptance-preflight",
    status: "ready",
    productVersion: preflight.version,
    backendId: preflight.backendStatus.backend.backendId,
    model: preflight.backendStatus.backend.model,
    readiness: preflight.backendStatus.backend.readiness,
    sourceBinding: SOURCE_COMMIT_PATTERN.test(preflight.embeddedBuild)
      ? "embedded-clean-commit"
      : "unbound-source-build",
    projectBindingSha256: sha256(
      `${options.projectId}\0${options.expectedProjectRevision}`,
    ),
    plannedMaximumProviderRequests: 3,
    receiptPath: "<dist-macos-r4d-live-acceptance-receipt>",
  };
}

export function main(argv = process.argv.slice(2)) {
  const options = parseArguments(argv);
  if (options.help) {
    process.stdout.write(usage());
    return 0;
  }
  const preflight = runOfflinePreflight(options);
  if (options.preflight) {
    process.stdout.write(`${JSON.stringify(preflightOutput(preflight, options))}\n`);
    return 0;
  }

  const sourceCommit = currentSourceCommit();
  if (preflight.embeddedBuild !== sourceCommit) fail("source-app-commit-mismatch");
  const connectionTest = validateConnectionTest(
    parseJson(
      runProcess(
        preflight.executable,
        ["config", "backend", "test", "--confirm-network-request"],
        {
          env: childEnvironment(),
          timeout: 90_000,
          failureCode: "r4d-live-connection-test-failed",
        },
      ),
      "r4d-live-connection-json-invalid",
    ),
  );
  const agentRun = validateAgentRun(
    runFullMcp(
      preflight.executable,
      "qiongli_agent_run",
      {
        projectId: options.projectId,
        expectedProjectRevision: options.expectedProjectRevision,
        prompt: ACCEPTANCE_PROMPT,
        confirmNetworkRequest: true,
      },
      200_000,
    ),
  );
  const postflight = runOfflinePreflight(options);
  validateStableBindings({
    sourceCommit,
    postflightSourceCommit: currentSourceCommit(),
    preflight,
    postflight,
    connectionTest,
  });
  const receipt = buildReceipt({
    sourceCommit,
    executableBytes: readFileSync(preflight.executable),
    version: preflight.version,
    backendStatus: preflight.backendStatus,
    projectId: options.projectId,
    expectedProjectRevision: options.expectedProjectRevision,
    connectionTest,
    agentRun,
    recordedAtUnix: Math.floor(Date.now() / 1000),
  });
  writeReceiptAtomically(options.receipt, receipt);
  process.stdout.write(
    `${JSON.stringify({
      schemaVersion: 1,
      status: "accepted",
      sourceCommit,
      receipt: options.receipt,
      providerRequests: 1 + receipt.fullRun.networkRequests,
      toolCalls: receipt.fullRun.toolCalls,
    })}\n`,
  );
  return 0;
}

const invokedPath = process.argv[1]
  ? pathToFileURL(resolve(process.argv[1])).href
  : "";
if (import.meta.url === invokedPath) {
  try {
    process.exitCode = main();
  } catch (error) {
    const reasonCode =
      error instanceof AcceptanceError
        ? error.reasonCode
        : "r4d-acceptance-internal-error";
    process.stderr.write(`error: ${reasonCode}\n`);
    process.exitCode = 1;
  }
}
