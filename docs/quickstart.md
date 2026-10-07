# Quickstart

## 1. Install and connect your Host

Download the program or choose a package manager from the [installation guide](guide/install.md). Run in a terminal:

```sh
qiongli install
qiongli mcp check
```

Use `./qiongli` if it is not on PATH, or `.\qiongli.exe` in PowerShell. Choose Plugin and your Host, then confirm the file changes and registration separately.

In the next update after 2.4.0, choose **5 — Install all detected Hosts**, or run
`qiongli install all`, to set up every detected supported Host CLI in one flow.
Missing clients are listed and skipped. Each Plugin includes Skills and MCP;
each selected Host still has its own preview and required confirmations.

## 2. Start a research task

Open a new Host session. Ask it to list Qiongli tools and call `qiongli_config_status`, then try:

> Read this paper. Explain the findings, point to the evidence and flag what remains uncertain.

In Codex, use `$qiongli-paper-read` or describe the task naturally. Start from the material you have; see [research tasks](guide/task-recipes.md).

## 3. Save and continue

Review changes before saving. When a stage is complete, ask for a summary with sources, decisions and open questions. Original files stay intact; you select and delete files yourself.

To follow links between claims and sources, see the [Research Graph example](examples/research-graph.md) and [CLI commands](guide/cli-2x.md). For a conversation without tools, use `$no-qiongli` or say “reply only”.
