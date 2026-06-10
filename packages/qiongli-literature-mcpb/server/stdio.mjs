const DEFAULT_PROTOCOL_VERSION = "2024-11-05";
const HEADER_SEPARATOR = Buffer.from("\r\n\r\n");

export async function handleJsonRpcMessage(message, handlers) {
  if (!message || typeof message !== "object" || Array.isArray(message)) {
    return errorResponse(null, -32600, "Invalid Request");
  }

  const id = Object.hasOwn(message, "id") ? message.id : null;
  const method = String(message.method ?? "");

  if (!method || method.startsWith("notifications/")) {
    return null;
  }

  try {
    if (method === "initialize") {
      return resultResponse(id, {
        protocolVersion: message.params?.protocolVersion ?? DEFAULT_PROTOCOL_VERSION,
        capabilities: {
          tools: {}
        },
        serverInfo: handlers.serverInfo
      });
    }

    if (method === "tools/list") {
      return resultResponse(id, {
        tools: handlers.listTools()
      });
    }

    if (method === "tools/call") {
      const params = message.params ?? {};
      const result = await handlers.handleToolCall(params.name, params.arguments ?? {});
      return resultResponse(id, result);
    }

    return errorResponse(id, -32601, `Method not found: ${method}`);
  } catch (error) {
    return errorResponse(id, -32603, sanitizeError(error));
  }
}

export async function startJsonRpcStdioServer({
  input = process.stdin,
  output = process.stdout,
  serverInfo,
  listTools,
  handleToolCall
}) {
  const handlers = { serverInfo, listTools, handleToolCall };
  let buffer = Buffer.alloc(0);
  let chain = Promise.resolve();

  return new Promise((resolve, reject) => {
    input.on("data", (chunk) => {
      chain = chain.then(async () => {
        buffer = Buffer.concat([buffer, Buffer.from(chunk)]);
        buffer = await drainBuffer(buffer, handlers, output);
      });
      chain.catch(reject);
    });

    input.on("end", () => {
      chain
        .then(async () => {
          const remaining = buffer.toString("utf8").trim();
          if (remaining) {
            await handleRawPayload(remaining, handlers, output, "line");
          }
        })
        .then(resolve, reject);
    });

    input.on("error", reject);
  });
}

async function drainBuffer(buffer, handlers, output) {
  let remaining = buffer;
  while (remaining.length > 0) {
    if (startsWithContentLength(remaining)) {
      const parsed = readFramedPayload(remaining);
      if (!parsed) {
        break;
      }
      await handleRawPayload(parsed.payload, handlers, output, "framed");
      remaining = parsed.remaining;
      continue;
    }

    const newline = remaining.indexOf(0x0a);
    if (newline === -1) {
      const rawPayload = readRawJsonPayload(remaining);
      if (rawPayload) {
        await handleRawPayload(rawPayload.payload, handlers, output, "line");
        remaining = rawPayload.remaining;
        continue;
      }
      break;
    }
    const line = remaining.slice(0, newline).toString("utf8").trim();
    remaining = remaining.slice(newline + 1);
    if (line) {
      await handleRawPayload(line, handlers, output, "line");
    }
  }
  return remaining;
}

async function handleRawPayload(payload, handlers, output, mode) {
  let message;
  try {
    message = JSON.parse(payload);
  } catch {
    output.write(serializeMessage(errorResponse(null, -32700, "Parse error"), mode));
    return;
  }

  const response = await handleJsonRpcMessage(message, handlers);
  if (response) {
    output.write(serializeMessage(response, mode));
  }
}

function startsWithContentLength(buffer) {
  return buffer.toString("utf8", 0, Math.min(buffer.length, 32)).startsWith("Content-Length:");
}

function readFramedPayload(buffer) {
  const separator = buffer.indexOf(HEADER_SEPARATOR);
  if (separator === -1) {
    return null;
  }

  const header = buffer.slice(0, separator).toString("utf8");
  const match = /^Content-Length:\s*(\d+)$/im.exec(header);
  if (!match) {
    return {
      payload: "",
      remaining: Buffer.alloc(0)
    };
  }

  const length = Number(match[1]);
  const payloadStart = separator + HEADER_SEPARATOR.length;
  const payloadEnd = payloadStart + length;
  if (buffer.length < payloadEnd) {
    return null;
  }

  return {
    payload: buffer.slice(payloadStart, payloadEnd).toString("utf8"),
    remaining: buffer.slice(payloadEnd)
  };
}

function readRawJsonPayload(buffer) {
  const payload = buffer.toString("utf8").trim();
  if (!payload.startsWith("{")) {
    return null;
  }

  try {
    JSON.parse(payload);
  } catch {
    return null;
  }

  return {
    payload,
    remaining: Buffer.alloc(0)
  };
}

function serializeMessage(message, mode) {
  const payload = JSON.stringify(message);
  if (mode === "framed") {
    return `Content-Length: ${Buffer.byteLength(payload, "utf8")}\r\n\r\n${payload}`;
  }
  return `${payload}\n`;
}

function resultResponse(id, result) {
  return {
    jsonrpc: "2.0",
    id,
    result
  };
}

function errorResponse(id, code, message) {
  return {
    jsonrpc: "2.0",
    id,
    error: {
      code,
      message
    }
  };
}

function sanitizeError(error) {
  const message = String(error?.message ?? error ?? "tool call failed");
  return message.replace(/[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}/g, "[redacted-email]");
}
