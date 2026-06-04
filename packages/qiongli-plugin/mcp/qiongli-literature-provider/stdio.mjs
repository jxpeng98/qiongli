import readline from "node:readline";

const DEFAULT_PROTOCOL_VERSION = "2024-11-05";

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
  const rl = readline.createInterface({ input, crlfDelay: Infinity });
  const handlers = { serverInfo, listTools, handleToolCall };

  for await (const line of rl) {
    const trimmed = line.trim();
    if (!trimmed) {
      continue;
    }

    let message;
    try {
      message = JSON.parse(trimmed);
    } catch {
      output.write(`${JSON.stringify(errorResponse(null, -32700, "Parse error"))}\n`);
      continue;
    }

    const response = await handleJsonRpcMessage(message, handlers);
    if (response) {
      output.write(`${JSON.stringify(response)}\n`);
    }
  }
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
