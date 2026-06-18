import test from "node:test";
import assert from "node:assert/strict";
import { buildQueryPlan, buildSearchIntent } from "../server/query.mjs";

test("buildQueryPlan uses finance/econ profile without substring false positives", () => {
  const intent = buildSearchIntent({ query: "riverbank erosion systematic review" });
  const plan = buildQueryPlan(
    { query: "riverbank erosion systematic review", search_depth: "deep" },
    intent,
    { searchDepth: "deep" }
  );

  assert.equal(plan.domain, "general");
  assert.deepEqual(plan.domain_terms, []);
  assert.deepEqual(
    plan.queries.map((query) => query.query),
    [
      "riverbank erosion systematic review",
      "riverbank erosion systematic review review",
      "riverbank erosion systematic review systematic review"
    ]
  );
});

test("buildQueryPlan exposes domain profile metadata for finance/econ routing", () => {
  const intent = buildSearchIntent({ query: "asset pricing corporate finance" });
  const plan = buildQueryPlan(
    { query: "asset pricing corporate finance", search_depth: "deep" },
    intent,
    { searchDepth: "deep" }
  );

  assert.equal(plan.domain, "finance_economics");
  assert.equal(plan.domain_profile.id, "finance_economics");
  assert.equal(plan.domain_profile.label, "Finance and Economics");
  assert.deepEqual(plan.domain_terms, ["asset pricing", "corporate finance"]);
  assert.deepEqual(
    plan.queries.map((query) => query.query),
    [
      "asset pricing corporate finance",
      "asset pricing corporate finance working paper",
      "asset pricing corporate finance JEL G12",
      "asset pricing corporate finance review"
    ]
  );
});
