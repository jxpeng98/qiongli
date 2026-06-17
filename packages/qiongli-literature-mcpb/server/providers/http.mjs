export const RETRYABLE_STATUSES = [429, 500, 502, 503, 504];

const DEFAULT_MAX_ATTEMPTS = 3;
const BASE_BACKOFF_MS = 250;

function fetchOptions(fetchImpl, options = {}) {
  if (fetchImpl) {
    return options;
  }

  return { signal: AbortSignal.timeout(15000), ...options };
}

function retryAfterMs(response) {
  if (!response) {
    return null;
  }

  const value = response.headers?.get?.("retry-after");
  if (!value) {
    return null;
  }

  const seconds = Number(value);
  if (Number.isFinite(seconds)) {
    return Math.max(0, seconds * 1000);
  }

  const date = Date.parse(value);
  if (Number.isFinite(date)) {
    return Math.max(0, date - Date.now());
  }

  return null;
}

function backoffMs(attempt, response) {
  return retryAfterMs(response) ?? BASE_BACKOFF_MS * 2 ** Math.max(0, attempt - 1);
}

function wait(ms) {
  return new Promise((resolve) => {
    setTimeout(resolve, ms);
  });
}

function shouldRetry(response, attempt, maxAttempts) {
  return (
    attempt < maxAttempts &&
    RETRYABLE_STATUSES.includes(response.status)
  );
}

export async function fetchJsonWithRetry({ provider, url, fetchImpl, options: requestOptions = {}, maxAttempts = DEFAULT_MAX_ATTEMPTS } = {}) {
  const fetcher = fetchImpl ?? fetch;
  const resolvedUrl = url instanceof URL ? url : new URL(String(url));
  let lastErrorName = "Error";

  for (let attempt = 1; attempt <= maxAttempts; attempt += 1) {
    let response;
    try {
      response = await fetcher(resolvedUrl, fetchOptions(fetchImpl, requestOptions));
    } catch (error) {
      lastErrorName = error?.name ?? "Error";
      if (attempt >= maxAttempts) {
        return {
          body: null,
          error: `${provider} request failed: ${lastErrorName}`,
          status: null,
          attempts: attempt
        };
      }

      if (!fetchImpl) {
        await wait(backoffMs(attempt));
      }
      continue;
    }

    if (response.ok) {
      return {
        body: await response.json(),
        error: null,
        status: response.status,
        attempts: attempt
      };
    }

    if (!shouldRetry(response, attempt, maxAttempts)) {
      return {
        body: null,
        error: `${provider} HTTP ${response.status}`,
        status: response.status,
        attempts: attempt
      };
    }

    if (!fetchImpl) {
      await wait(backoffMs(attempt, response));
    }
  }

  return {
    body: null,
    error: `${provider} request failed: ${lastErrorName}`,
    status: null,
    attempts: maxAttempts
  };
}
