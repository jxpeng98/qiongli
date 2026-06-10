import test from "node:test";
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { cp, mkdtemp, rm } from "node:fs/promises";
import os from "node:os";
import path from "node:path";

function frame(message) {
  const payload = JSON.stringify(message);
  return `Content-Length: ${Buffer.byteLength(payload, "utf8")}\r\n\r\n${payload}`;
}

test("server entrypoint starts when installed under a path with spaces", async () => {
  const tmpRoot = await mkdtemp(path.join(os.tmpdir(), "qiongli mcpb "));
  const serverRoot = path.join(tmpRoot, "server");
  await cp(new URL("../server", import.meta.url), serverRoot, { recursive: true });

  try {
    const child = spawn(process.execPath, [path.join(serverRoot, "index.mjs")], {
      stdio: ["pipe", "pipe", "pipe"]
    });
    const stdout = [];
    const stderr = [];
    child.stdout.on("data", (chunk) => stdout.push(chunk));
    child.stderr.on("data", (chunk) => stderr.push(chunk));

    child.stdin.end(
      frame({
        jsonrpc: "2.0",
        id: 0,
        method: "initialize",
        params: {
          protocolVersion: "2025-11-25",
          capabilities: {},
          clientInfo: {
            name: "claude-ai",
            version: "0.1.0"
          }
        }
      })
    );

    await new Promise((resolve) => child.on("close", resolve));
    const output = Buffer.concat(stdout).toString("utf8");

    assert.equal(Buffer.concat(stderr).toString("utf8"), "");
    assert.match(output, /^Content-Length: \d+\r\n\r\n/);
    assert.match(output, /"protocolVersion":"2025-11-25"/);
  } finally {
    await rm(tmpRoot, { recursive: true, force: true });
  }
});
