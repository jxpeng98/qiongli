import test from "node:test";
import assert from "node:assert/strict";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { handleSearch, handleSearchPlan } from "../server/index.mjs";

test("disabled arXiv is excluded from planning and search execution", async (t) => {
  const configHome = mkdtempSync(path.join(os.tmpdir(), "qiongli-mcpb-disabled-routing-"));
  t.after(() => rmSync(configHome, { recursive: true, force: true }));
  writeFileSync(
    path.join(configHome, "providers.json"),
    '{"version":1,"providers":{"openalex":{"enabled":false,"api_key":"disabled-secret-canary"},"arxiv":{"enabled":false}}}\n'
  );
  let fetchCalls = 0;
  const context = {
    env: { QIONGLI_CONFIG_HOME: configHome },
    fetchImpl: async () => {
      fetchCalls += 1;
      throw new Error("disabled provider must not be called");
    }
  };

  const plan = handleSearchPlan({ query: "governance" }, context);
  const search = await handleSearch({ query: "governance" }, context);

  assert.equal(plan.provider_capability_mode, "strategy_only");
  assert.equal(plan.search_execution_mode, "strategy_only");
  assert.deepEqual(plan.provider_queries, []);
  assert.equal(fetchCalls, 0);
  assert.equal(search.capability_mode, "strategy_only");
  assert.deepEqual(search.results, []);
  assert.equal(JSON.stringify(search).includes("disabled-secret-canary"), false);
});
