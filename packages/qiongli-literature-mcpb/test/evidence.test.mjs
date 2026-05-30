import test from "node:test";
import assert from "node:assert/strict";
import { buildEvidence } from "../server/evidence.mjs";

test("single successful provider produces warnings", () => {
  const evidence = buildEvidence({
    attemptedProviders: ["openalex", "semantic_scholar"],
    successfulProviders: ["openalex"],
    failedProviders: ["semantic_scholar"],
    resultCount: 3
  });

  assert.equal(evidence.capability_mode, "provider_connected");
  assert.equal(evidence.result_count, 3);
  assert.deepEqual(evidence.providers.attempted, ["openalex", "semantic_scholar"]);
  assert.deepEqual(evidence.providers.successful, ["openalex"]);
  assert.deepEqual(evidence.providers.failed, ["semantic_scholar"]);
  assert.deepEqual(evidence.warnings, ["single_successful_provider", "partial_provider_failure"]);
});

test("all failed providers produce strategy only evidence", () => {
  const evidence = buildEvidence({
    attemptedProviders: ["openalex", "openalex", "semantic_scholar"],
    successfulProviders: [],
    failedProviders: ["semantic_scholar", "openalex"],
    resultCount: 0
  });

  assert.equal(evidence.capability_mode, "strategy_only");
  assert.deepEqual(evidence.providers.attempted, ["openalex", "semantic_scholar"]);
  assert.deepEqual(evidence.providers.successful, []);
  assert.deepEqual(evidence.providers.failed, ["semantic_scholar", "openalex"]);
  assert.deepEqual(evidence.warnings, ["all_providers_failed"]);
});

test("evidence redacts secrets from arbitrary input", () => {
  const evidence = buildEvidence({
    attemptedProviders: ["openalex"],
    successfulProviders: ["openalex"],
    failedProviders: [],
    resultCount: 1,
    openalexEmail: "person@example.com",
    semanticScholarApiKey: "secret-key"
  });

  const serialized = JSON.stringify(evidence);
  assert.equal(serialized.includes("person@example.com"), false);
  assert.equal(serialized.includes("secret-key"), false);
});
