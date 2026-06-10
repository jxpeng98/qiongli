import test from "node:test";
import assert from "node:assert/strict";
import { EventEmitter } from "node:events";
import { PassThrough } from "node:stream";
import { startJsonRpcStdioServer } from "../server/stdio.mjs";

function frame(message) {
  const payload = JSON.stringify(message);
  return `Content-Length: ${Buffer.byteLength(payload, "utf8")}\r\n\r\n${payload}`;
}

function readFramedMessage(buffer) {
  const separator = buffer.indexOf("\r\n\r\n");
  assert.notEqual(separator, -1);
  const header = buffer.slice(0, separator).toString("utf8");
  const match = /^Content-Length:\s*(\d+)$/m.exec(header);
  assert.ok(match);
  const length = Number(match[1]);
  const payload = buffer.slice(separator + 4, separator + 4 + length).toString("utf8");
  return JSON.parse(payload);
}

test("stdio server handles Content-Length framed initialize messages", async () => {
  const input = new PassThrough();
  const output = new PassThrough();
  const chunks = [];

  output.on("data", (chunk) => chunks.push(chunk));

  const server = startJsonRpcStdioServer({
    input,
    output,
    serverInfo: {
      name: "qiongli-literature-provider",
      version: "0.1.3"
    },
    listTools: () => [],
    handleToolCall: async () => ({ content: [] })
  });

  input.end(
    frame({
      jsonrpc: "2.0",
      id: 0,
      method: "initialize",
      params: {
        protocolVersion: "2025-11-25",
        capabilities: {
          extensions: {
            "io.modelcontextprotocol/ui": {
              mimeTypes: ["text/html;profile=mcp-app"]
            }
          }
        },
        clientInfo: {
          name: "claude-ai",
          version: "0.1.0"
        }
      }
    })
  );
  await server;

  const response = readFramedMessage(Buffer.concat(chunks));
  assert.equal(response.jsonrpc, "2.0");
  assert.equal(response.id, 0);
  assert.equal(response.result.protocolVersion, "2025-11-25");
  assert.deepEqual(response.result.capabilities, { tools: {} });
});

test("stdio server handles raw initialize messages without waiting for newline", async () => {
  const input = new PassThrough();
  const output = new PassThrough();
  const chunks = [];

  output.on("data", (chunk) => chunks.push(chunk));

  const server = startJsonRpcStdioServer({
    input,
    output,
    serverInfo: {
      name: "qiongli-literature-provider",
      version: "0.1.3"
    },
    listTools: () => [],
    handleToolCall: async () => ({ content: [] })
  });

  input.end(JSON.stringify({
    jsonrpc: "2.0",
    id: 0,
    method: "initialize",
    params: {
      protocolVersion: "2025-11-25"
    }
  }));

  await new Promise((resolve, reject) => {
    output.once("data", resolve);
    setTimeout(() => reject(new Error("timed out waiting for raw JSON response")), 50);
  });
  await server;

  const response = JSON.parse(Buffer.concat(chunks).toString("utf8"));
  assert.equal(response.jsonrpc, "2.0");
  assert.equal(response.id, 0);
  assert.equal(response.result.protocolVersion, "2025-11-25");
});

test("stdio server handles Claude NodeHost event-only stdin streams", async () => {
  const input = new EventEmitter();
  const output = new PassThrough();
  const chunks = [];

  output.on("data", (chunk) => chunks.push(chunk));

  const server = startJsonRpcStdioServer({
    input,
    output,
    serverInfo: {
      name: "qiongli-literature-provider",
      version: "0.1.3"
    },
    listTools: () => [],
    handleToolCall: async () => ({ content: [] })
  });

  input.emit("data", JSON.stringify({
    jsonrpc: "2.0",
    id: 0,
    method: "initialize",
    params: {
      protocolVersion: "2025-11-25"
    }
  }) + "\n");
  input.emit("end");
  await server;

  const response = JSON.parse(Buffer.concat(chunks).toString("utf8"));
  assert.equal(response.jsonrpc, "2.0");
  assert.equal(response.id, 0);
  assert.equal(response.result.serverInfo.version, "0.1.3");
});
