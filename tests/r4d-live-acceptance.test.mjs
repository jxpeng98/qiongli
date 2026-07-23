import assert from "node:assert/strict";
import test from "node:test";

import {
  AcceptanceError,
  buildReceipt,
  parseArguments,
  validateAgentRun,
  validateBackendStatus,
  validateConnectionTest,
  validateProjectRead,
  validateStableBindings,
} from "../scripts/r4d_live_acceptance.mjs";

const PROJECT_ID = `prj_${"1".repeat(32)}`;

function digest(character) {
  return character.repeat(64);
}

function backendStatus() {
  return {
    schema_version: 1,
    command: "config-backend-status",
    revision: 4,
    backend: {
      schemaVersion: 1,
      backendId: "openai-responses",
      model: "gpt-5.6-sol",
      enabled: true,
      readiness: "ready",
      testAvailable: true,
    },
  };
}

function connectionTest() {
  return {
    schema_version: 1,
    command: "config-backend-test",
    revision: 4,
    test: {
      schemaVersion: 1,
      backendId: "openai-responses",
      model: "gpt-5.6-sol",
      outcome: "passed",
      providerStorage: "disabled",
      hostedTools: "disabled",
    },
  };
}

function agentRun() {
  return {
    schemaVersion: 1,
    runId: `run_${"2".repeat(32)}`,
    backendId: "openai-responses",
    model: "gpt-5.6-sol",
    finishReason: "stop",
    content: "Private model answer canary.",
    providerUsage: {
      inputTokens: 140,
      outputTokens: 20,
      cachedInputTokens: 0,
    },
    executionUsage: {
      elapsedSeconds: 2,
      modelTurns: 2,
      toolCalls: 1,
      processes: 0,
      inputBytes: 200,
      outputBytes: 100,
      networkRequests: 2,
      artifacts: 0,
    },
    toolAudits: [
      {
        schemaVersion: 1,
        runId: `run_${"2".repeat(32)}`,
        callId: `call_${"3".repeat(32)}`,
        toolId: "qiongli_project_read",
        toolClass: "read",
        requestDigest: digest("4"),
        decisionDigest: digest("5"),
        policyRevision: 1,
        startedAtUnixMs: 1,
        finishedAtUnixMs: 2,
        outcome: "completed",
        reasonCode: "tool-host-completed",
        inputBytes: 50,
        outputBytes: 80,
        redactionCount: 0,
        truncated: false,
      },
    ],
  };
}

function expectReason(reasonCode, callback) {
  assert.throws(callback, (error) => {
    assert.ok(error instanceof AcceptanceError);
    assert.equal(error.reasonCode, reasonCode);
    return true;
  });
}

test("argument parser requires one explicit acceptance mode", () => {
  const base = [
    "--project-id",
    PROJECT_ID,
    "--expected-project-revision",
    "7",
  ];
  const preflight = parseArguments([...base, "--preflight"]);
  assert.equal(preflight.preflight, true);
  assert.equal(preflight.confirmThreeNetworkRequests, false);
  assert.equal(preflight.expectedProjectRevision, 7);

  const live = parseArguments([...base, "--confirm-three-network-requests"]);
  assert.equal(live.preflight, false);
  assert.equal(live.confirmThreeNetworkRequests, true);

  expectReason("acceptance-mode-required", () => parseArguments(base));
  expectReason("acceptance-mode-required", () =>
    parseArguments([
      ...base,
      "--preflight",
      "--confirm-three-network-requests",
    ]),
  );
  expectReason("project-id-invalid", () =>
    parseArguments([
      "--project-id",
      "../private",
      "--expected-project-revision",
      "7",
      "--preflight",
    ]),
  );
});

test("preflight validators require ready Keychain and exact project revision", () => {
  assert.equal(validateBackendStatus(backendStatus()).backend.readiness, "ready");
  assert.equal(
    validateProjectRead(
      {
        schemaVersion: 1,
        libraryRevision: 2,
        project: {
          projectId: PROJECT_ID,
          semanticRevision: 7,
          lifecycle: "active",
          health: "healthy",
        },
      },
      PROJECT_ID,
      7,
    ).project.semanticRevision,
    7,
  );

  const missing = backendStatus();
  missing.backend.readiness = "credential-missing";
  expectReason("r4d-backend-not-ready", () => validateBackendStatus(missing));
  expectReason("r4d-project-not-ready", () =>
    validateProjectRead(
      {
        schemaVersion: 1,
        libraryRevision: 2,
        project: {
          projectId: PROJECT_ID,
          semanticRevision: 8,
          lifecycle: "active",
          health: "healthy",
        },
      },
      PROJECT_ID,
      7,
    ),
  );
});

test("live validators require non-stored connection and a real ToolHost loop", () => {
  assert.equal(validateConnectionTest(connectionTest()).test.outcome, "passed");
  assert.equal(validateAgentRun(agentRun()).executionUsage.toolCalls, 1);

  const stored = connectionTest();
  stored.test.providerStorage = "enabled";
  expectReason("r4d-connection-test-invalid", () =>
    validateConnectionTest(stored),
  );

  const noTool = agentRun();
  noTool.executionUsage.modelTurns = 1;
  noTool.executionUsage.networkRequests = 1;
  noTool.executionUsage.toolCalls = 0;
  noTool.toolAudits = [];
  expectReason("r4d-agent-run-invalid", () => validateAgentRun(noTool));

  const processEscape = agentRun();
  processEscape.executionUsage.processes = 1;
  expectReason("r4d-agent-run-invalid", () =>
    validateAgentRun(processEscape),
  );
});

test("live acceptance rejects source, config, or project binding drift", () => {
  const sourceCommit = "6".repeat(40);
  const preflight = {
    embeddedBuild: sourceCommit,
    backendStatus: backendStatus(),
  };
  const postflight = {
    embeddedBuild: sourceCommit,
    backendStatus: backendStatus(),
  };
  validateStableBindings({
    sourceCommit,
    postflightSourceCommit: sourceCommit,
    preflight,
    postflight,
    connectionTest: connectionTest(),
  });

  const changedConfig = connectionTest();
  changedConfig.revision += 1;
  expectReason("r4d-config-revision-drift", () =>
    validateStableBindings({
      sourceCommit,
      postflightSourceCommit: sourceCommit,
      preflight,
      postflight,
      connectionTest: changedConfig,
    }),
  );
  expectReason("r4d-postflight-binding-drift", () =>
    validateStableBindings({
      sourceCommit,
      postflightSourceCommit: "7".repeat(40),
      preflight,
      postflight,
      connectionTest: connectionTest(),
    }),
  );
});

test("receipt contains hashes and counts but no project, prompt, or answer", () => {
  const run = agentRun();
  const receipt = buildReceipt({
    sourceCommit: "6".repeat(40),
    executableBytes: Buffer.from("canonical executable"),
    version: "2.0.0-alpha.1",
    backendStatus: backendStatus(),
    projectId: PROJECT_ID,
    expectedProjectRevision: 7,
    connectionTest: connectionTest(),
    agentRun: run,
    recordedAtUnix: 1_800_000_000,
  });
  const serialized = JSON.stringify(receipt);
  assert.equal(receipt.status, "accepted");
  assert.equal(receipt.fullRun.toolCalls, 1);
  assert.deepEqual(receipt.fullRun.toolIds, ["qiongli_project_read"]);
  assert.match(receipt.executableSha256, /^[0-9a-f]{64}$/);
  assert.match(receipt.projectBindingSha256, /^[0-9a-f]{64}$/);
  assert.match(receipt.fullRun.contentSha256, /^[0-9a-f]{64}$/);
  assert.ok(!serialized.includes(PROJECT_ID));
  assert.ok(!serialized.includes(run.content));
  assert.ok(!serialized.includes("Call one offered"));
});
