export async function probeConnector(config, context = {}) {
  return probeText(`${config.connector_url}/connector/ping`, context);
}

export async function probeCompanion(config, context = {}) {
  return probeJson(`${config.connector_url}/qiongli/ping`, context);
}

export async function postCompanionJson(config, endpoint, payload, context = {}) {
  const response = await fetchJson(`${config.connector_url}${endpoint}`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json"
    },
    body: JSON.stringify(payload)
  }, context);
  return response;
}

async function probeText(url, context = {}) {
  try {
    const response = await fetchWithOptions(url, {}, context);
    return {
      available: response.ok,
      status: response.status,
      body: await response.text()
    };
  } catch (error) {
    return {
      available: false,
      status: null,
      error: sanitizedError(error)
    };
  }
}

async function probeJson(url, context = {}) {
  try {
    const response = await fetchWithOptions(url, {}, context);
    if (!response.ok) {
      return {
        available: false,
        status: response.status
      };
    }
    return {
      available: true,
      status: response.status,
      body: await response.json()
    };
  } catch (error) {
    return {
      available: false,
      status: null,
      error: sanitizedError(error)
    };
  }
}

async function fetchJson(url, options, context = {}) {
  const response = await fetchWithOptions(url, options, context);
  if (!response.ok) {
    return {
      status: "error",
      http_status: response.status,
      error_code: "companion_http_error",
      message: `Qiongli Zotero companion HTTP ${response.status}`
    };
  }
  return response.json();
}

function fetchWithOptions(url, options = {}, context = {}) {
  const fetcher = context.fetchImpl ?? fetch;
  if (context.fetchImpl) {
    return fetcher(url, options);
  }

  return fetcher(url, {
    signal: AbortSignal.timeout(5000),
    ...options
  });
}

function sanitizedError(error) {
  return String(error?.message ?? error ?? "request failed").slice(0, 160);
}
