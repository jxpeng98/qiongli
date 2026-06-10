import test from "node:test";
import assert from "node:assert/strict";
import { mkdtempSync, rmSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { readConfig, providerStatus } from "../server/config.mjs";

test("provider status redacts configured secrets", () => {
  const config = readConfig({
    QIONGLI_MCPB_OPENALEX_EMAIL: " person@example.com ",
    QIONGLI_MCPB_SEMANTIC_SCHOLAR_API_KEY: " secret-key ",
    QIONGLI_MCPB_DEFAULT_LIMIT: "12"
  });

  const status = providerStatus(config);
  const serialized = JSON.stringify(status);

  assert.equal(config.openalexEmail, "person@example.com");
  assert.equal(config.semanticScholarApiKey, "secret-key");
  assert.equal(config.defaultLimit, 12);
  assert.equal(status.status, "ok");
  assert.equal(status.capability_mode, "provider_connected");
  assert.equal(status.providers.openalex, "configured");
  assert.equal(status.providers.semantic_scholar, "configured");
  assert.equal(status.providers.crossref, "not_implemented");
  assert.equal(status.providers.pubmed, "not_implemented");
  assert.equal(serialized.includes("secret-key"), false);
  assert.equal(serialized.includes("person@example.com"), false);
});

test("readConfig defaults invalid and blank limits and clamps numeric limits", () => {
  assert.equal(readConfig({ QIONGLI_MCPB_DEFAULT_LIMIT: "" }).defaultLimit, 10);
  assert.equal(readConfig({ QIONGLI_MCPB_DEFAULT_LIMIT: "invalid" }).defaultLimit, 10);
  assert.equal(readConfig({ QIONGLI_MCPB_DEFAULT_LIMIT: "0" }).defaultLimit, 1);
  assert.equal(readConfig({ QIONGLI_MCPB_DEFAULT_LIMIT: "51" }).defaultLimit, 50);
  assert.equal(readConfig({ QIONGLI_MCPB_DEFAULT_LIMIT: "12" }).defaultLimit, 12);
});

test("provider status reports OpenAlex usable without email", (t) => {
  const configHome = mkdtempSync(path.join(os.tmpdir(), "qiongli-mcpb-empty-config-"));
  t.after(() => rmSync(configHome, { recursive: true, force: true }));

  const status = providerStatus(readConfig({ QIONGLI_CONFIG_HOME: configHome }));

  assert.equal(status.capability_mode, "provider_connected");
  assert.deepEqual(status.providers, {
    openalex: "configured_without_email",
    semantic_scholar: "missing",
    crossref: "not_implemented",
    pubmed: "not_implemented"
  });
});
