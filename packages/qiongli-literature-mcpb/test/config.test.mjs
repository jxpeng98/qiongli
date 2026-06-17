import test from "node:test";
import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { readConfig, providerStatus } from "../server/config.mjs";

test("provider status redacts configured secrets", () => {
  const config = readConfig({
    QIONGLI_MCPB_OPENALEX_API_KEY: " openalex-secret-key ",
    QIONGLI_MCPB_OPENALEX_EMAIL: " person@example.com ",
    QIONGLI_MCPB_SEMANTIC_SCHOLAR_API_KEY: " secret-key ",
    QIONGLI_MCPB_CROSSREF_EMAIL: " crossref@example.com ",
    QIONGLI_MCPB_PUBMED_API_KEY: " pubmed-secret-key ",
    QIONGLI_MCPB_DEFAULT_LIMIT: "12"
  });

  const status = providerStatus(config);
  const serialized = JSON.stringify(status);

  assert.equal(config.openalexApiKey, "openalex-secret-key");
  assert.equal(config.openalexEmail, "person@example.com");
  assert.equal(config.semanticScholarApiKey, "secret-key");
  assert.equal(config.crossrefEmail, "crossref@example.com");
  assert.equal(config.pubmedApiKey, "pubmed-secret-key");
  assert.equal(config.defaultLimit, 12);
  assert.equal(status.status, "ok");
  assert.equal(status.capability_mode, "provider_connected");
  assert.equal(status.providers.openalex, "configured");
  assert.equal(status.providers.semantic_scholar, "configured");
  assert.equal(status.providers.crossref, "configured");
  assert.equal(status.providers.pubmed, "configured");
  assert.equal(serialized.includes("secret-key"), false);
  assert.equal(serialized.includes("pubmed-secret-key"), false);
  assert.equal(serialized.includes("openalex-secret-key"), false);
  assert.equal(serialized.includes("person@example.com"), false);
  assert.equal(serialized.includes("crossref@example.com"), false);
});

test("readConfig defaults invalid and blank limits and clamps numeric limits", () => {
  assert.equal(readConfig({ QIONGLI_MCPB_DEFAULT_LIMIT: "" }).defaultLimit, 25);
  assert.equal(readConfig({ QIONGLI_MCPB_DEFAULT_LIMIT: "invalid" }).defaultLimit, 25);
  assert.equal(readConfig({ QIONGLI_MCPB_DEFAULT_LIMIT: "0" }).defaultLimit, 1);
  assert.equal(readConfig({ QIONGLI_MCPB_DEFAULT_LIMIT: "51" }).defaultLimit, 50);
  assert.equal(readConfig({ QIONGLI_MCPB_DEFAULT_LIMIT: "12" }).defaultLimit, 12);
});

test("provider status requires OpenAlex API key and reports optional email separately", (t) => {
  const configHome = mkdtempSync(path.join(os.tmpdir(), "qiongli-mcpb-empty-config-"));
  t.after(() => rmSync(configHome, { recursive: true, force: true }));

  const status = providerStatus(readConfig({ QIONGLI_CONFIG_HOME: configHome }));

  assert.equal(status.capability_mode, "strategy_only");
  assert.deepEqual(status.providers, {
    openalex: "missing",
    semantic_scholar: "missing",
    crossref: "missing",
    pubmed: "missing"
  });
});

test("readConfig reads Crossref and PubMed values from shared config", (t) => {
  const configHome = mkdtempSync(path.join(os.tmpdir(), "qiongli-mcpb-provider-config-"));
  t.after(() => rmSync(configHome, { recursive: true, force: true }));

  const configPath = path.join(configHome, "providers.json");
  mkdirSync(path.dirname(configPath), { recursive: true });
  writeFileSync(
    configPath,
    `${JSON.stringify({
      providers: {
        crossref: {
          enabled: true,
          email: "stored-crossref@example.com"
        },
        pubmed: {
          enabled: true,
          api_key: "stored-pubmed-key"
        }
      }
    })}\n`
  );

  const config = readConfig({ QIONGLI_CONFIG_HOME: configHome });

  assert.equal(config.crossrefEmail, "stored-crossref@example.com");
  assert.equal(config.pubmedApiKey, "stored-pubmed-key");
});
