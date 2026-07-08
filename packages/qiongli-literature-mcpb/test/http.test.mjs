import test from "node:test";
import assert from "node:assert/strict";
import { fetchJsonWithRetry, RETRYABLE_STATUSES } from "../server/providers/http.mjs";

test("fetchJsonWithRetry retries retryable HTTP statuses", async () => {
  const calls = [];
  const fetchImpl = async (url) => {
    calls.push(new URL(url));
    if (calls.length === 1) {
      return {
        ok: false,
        status: 503,
        headers: new Headers(),
        async json() {
          return { error: "temporary" };
        }
      };
    }

    return {
      ok: true,
      status: 200,
      headers: new Headers(),
      async json() {
        return { ok: true };
      }
    };
  };

  const result = await fetchJsonWithRetry({
    provider: "test_provider",
    url: new URL("https://example.test/search"),
    fetchImpl
  });

  assert.deepEqual(RETRYABLE_STATUSES, [429, 500, 502, 503, 504]);
  assert.equal(calls.length, 2);
  assert.equal(result.error, null);
  assert.equal(result.status, 200);
  assert.equal(result.attempts, 2);
  assert.deepEqual(result.body, { ok: true });
});

test("fetchJsonWithRetry does not retry non-retryable HTTP statuses", async () => {
  let calls = 0;
  const fetchImpl = async () => {
    calls += 1;
    return {
      ok: false,
      status: 404,
      headers: new Headers(),
      async json() {
        return { redacted_fixture: "do-not-leak" };
      }
    };
  };

  const result = await fetchJsonWithRetry({
    provider: "test_provider",
    url: new URL("https://example.test/missing"),
    fetchImpl
  });

  assert.equal(calls, 1);
  assert.equal(result.body, null);
  assert.equal(result.status, 404);
  assert.equal(result.attempts, 1);
  assert.equal(result.error, "test_provider HTTP 404");
  assert.equal(JSON.stringify(result).includes("do-not-leak"), false);
});

test("fetchJsonWithRetry retries transient fetch exceptions", async () => {
  let calls = 0;
  const fetchImpl = async () => {
    calls += 1;
    if (calls === 1) {
      throw new TypeError("temporary network failure with secret-token");
    }

    return {
      ok: true,
      status: 200,
      headers: new Headers(),
      async json() {
        return { recovered: true };
      }
    };
  };

  const result = await fetchJsonWithRetry({
    provider: "test_provider",
    url: new URL("https://example.test/network"),
    fetchImpl
  });

  assert.equal(calls, 2);
  assert.equal(result.error, null);
  assert.equal(result.status, 200);
  assert.equal(result.attempts, 2);
  assert.deepEqual(result.body, { recovered: true });
});

test("fetchJsonWithRetry sanitizes exhausted fetch exceptions", async () => {
  let calls = 0;
  const fetchImpl = async () => {
    calls += 1;
    throw new TypeError("temporary network failure with secret-token");
  };

  const result = await fetchJsonWithRetry({
    provider: "test_provider",
    url: new URL("https://example.test/network"),
    fetchImpl,
    maxAttempts: 2
  });

  assert.equal(calls, 2);
  assert.equal(result.body, null);
  assert.equal(result.status, null);
  assert.equal(result.attempts, 2);
  assert.equal(result.error, "test_provider request failed: TypeError");
  assert.equal(JSON.stringify(result).includes("secret-token"), false);
});
