# 🚨 故障排除与统一错误码指南

本文档列出了在使用 Qiongli Orchestrator CLI 时可能遇到的所有标准 `ERR-RS-*` 错误码，以及它们的根本原因和标准解决方案。

> **专业提示：** 如果遇到任何异常行为，你的第一步永远应该是运行交互式诊断工具：
> `python3 -m bridges.orchestrator doctor --cwd .`

---

## 客户端 Skill 发现

### Codex 对 `/qiongli` 没有反应。

- **原因**：Codex 不会把 Qiongli 暴露成自定义 slash command。Qiongli 在 Codex 中是 skill，不是 slash command。
- **修复**：
  - 安装或升级后重启 Codex。
  - 运行 `/skills`，确认列表里有 `qiongli`。
  - 使用 `$qiongli` 调用主 skill，例如 `$qiongli plan a literature review on ai-in-education`。
  - 对 plugin-first 安装，也可以调用生成的 workflow wrapper，例如 `$qiongli-lit-review ai-in-education`。
  - 如果 `/skills` 只看到 `research-paper-workflow`，运行 `qiongli upgrade --target codex --overwrite` 刷新当前包，重启 Codex 后再检查。
- **说明**：安装目录仍然可能叫 `qiongli-workflow`，这是正常的。用户可见的 skill 名称应该是 `qiongli`。

### Codex 看不到 `/lit-review`、`/academic-write` 或其他 workflow 命令。

- **原因**：CLI 管理的 Codex plugin 会把 workflow wrappers 打包到 `commands/`，但当前 Codex plugin 发现面不会把它们显示成独立的 `/lit-review` slash 项。Codex 使用 skill entries，所以 Qiongli 会为 plugin 安装生成 `qiongli-*` wrapper skills。
- **修复**：
  - 如果 `/skills` 里能看到 wrapper，使用 `$qiongli-lit-review <topic>`。
  - 使用主 skill 时，可写 `$qiongli plan a literature review on <topic>` 或 `$qiongli run lit-review on <topic>`。
  - 也可以直接写自然语言请求，例如 “Use Qiongli to run a literature review on <topic>”；skill 入口应该把它路由到 Stage B / `lit-review`。
  - 升级到包含 Codex wrapper skills 的版本后，运行 `qiongli install --target codex --surface plugin --overwrite` 重新安装本地 Codex plugin，然后重启 Codex。
- **说明**：Codex 仍然不会显示 `/lit-review` 这种 slash command。plugin 安装应显示 `qiongli-lit-review` 这类 skill wrapper；skills-only 安装可以继续使用 `$qiongli run lit-review on <topic>`。

### Codex 文献任务在已安装 Qiongli 时仍提示 `strategy_only`。

- **原因**：当前 Codex 会话可能尚未加载 Qiongli MCP tools，或者 workflow 跳过了文献能力 preflight。
- **修复**：
  - 运行 `qiongli check --json`，确认 `installed.codex.plugin_mcp.installed` 为 true。
  - 在 Codex 会话中要求 Qiongli 调用 `qiongli_literature_status`；如果返回 `capability_mode: provider_connected`，就可以使用 provider-backed literature workflow。
  - 要求 workflow 在检索执行前写入 `qiongli_search_plan`。如果 provider MCP 不可用但 Codex 的 `native search` 可用，应使用 `search_execution_mode: native_only`；如果 provider MCP 和 `native search` 都可用，应使用 `search_execution_mode: hybrid_search`。
  - 将 `provider_capability_mode` 和 `search_execution_mode` 分开记录：provider capability 可以是 `strategy_only`，但整次执行仍可能是 `native_only` 或 `hybrid_search`。
  - MCP servers must not call Codex or Claude native search directly. Active agent 执行 `native_search_queries`，并把 native search 记录为 `native:codex_web_search` 或 `native:claude_web_search`。
  - 如果看不到 `qiongli_literature_status`，重启 Codex 后再检查。
  - 如果重启后插件 MCP 仍不可见，再使用显式 standalone fallback：`qiongli install --target codex --parts mcp`。

### Claude Code 看不到 `/paper`、`/lit-review`。

- **原因**：workflow command discovery 文件未安装、客户端尚未重启，或你只安装了 Codex-facing skill package。
- **修复**：
  - 需要跨客户端全局使用时，运行 `qiongli upgrade --target all --overwrite`。
  - 重启 Claude Code。
  - 如果只安装了原生 plugin，确认该 plugin 已在客户端启用。

---

## 环境与认证 (ENV)

当 Orchestrator 无法在环境中找到所需的 CLI 二进制文件（如 `claude` 或 `codex`）或相应的 API 密钥时，会发生这些错误。

### `[ERR-RS-ENV-001]` 缺少 API 密钥或无效。
- **原因**：请求了一个活跃的 AI agent，但环境中缺少其认证密钥。
- **修复**：检查你的 `.env` 文件或直接在终端中导出密钥。
  - Claude: `export ANTHROPIC_API_KEY="sk-ant-..."`
  - Codex: `export OPENAI_API_KEY="sk-proj-..."`

### `[ERR-RS-ENV-002]` 未安装所需的 CLI 工具或未将其添加到 PATH 中。
- **原因**：尚未安装包含目标模型的 Node.js 二进制包装器。
- **修复**：全局安装底层的 CLI 工具：
  - `npm install -g @anthropic-ai/claude-code`

### `[ERR-RS-ENV-003]` `curl: (60) SSL certificate problem: certificate is not yet valid`。
- **原因**：安装命令已经连到了 GitHub，但在下载脚本前 TLS 校验失败。常见原因是系统时间不对、CA 证书包过旧，或者公司代理拦截了 HTTPS。
- **修复**：
  - 先检查系统时间：`date -u` 和 `timedatectl status`。
  - 刷新 CA 证书：
    - Debian/Ubuntu：`sudo apt-get update && sudo apt-get install --reinstall ca-certificates curl`
    - RHEL/CentOS/Fedora：`sudo dnf reinstall ca-certificates curl`，然后执行 `sudo update-ca-trust`
  - 如果在公司代理后面，需要把代理的根证书导入系统信任库。
  - 重试时请确认脚本名和变量展开都写对：
    - `curl -fsSL https://raw.githubusercontent.com/jxpeng98/qiongli/main/scripts/bootstrap_qiongli.sh | bash -s -- --project-dir "$PWD" --target all`
  - 除非你明确接受安全风险，否则不要用 `curl -k` 绕过证书校验。

---

## 配置与标准 (CFG)

当 YAML 契约或 JSON 配置映射表包含无效设置，或者你请求了无法映射的任务/配置文件时，会发生这些错误。

### `[ERR-RS-CFG-001]` 指定了未知或无效的 agent profile。
- **原因**：你传递了一个在 `standards/agent-profiles.example.json` 中不存在的 `--profile`（例如 `--profile fast-writer`）。
- **修复**：检查拼写。使用 `task-run --help` 或查看 JSON 文件以确认可用的 profiles（例如 `default`, `academic-strict`, `bilingual-collaborator`）。

### `[ERR-RS-CFG-002]` 无法读取或解析标准的 YAML 契约文件。
- **原因**：`standards/` 目录缺少关键的标准文件（如 `research-workflow-contract.yaml`），或者 YAML 语法被破坏。
- **修复**：从版本控制中恢复 `.yaml` 文件。运行 `python3 scripts/validate_research_standard.py --strict` 来精确定位 YAML 语法错误。

### `[ERR-RS-CFG-003]` 请求了未知的 Task ID 或无效的阶段逻辑。
- **原因**：你尝试运行 `rsk task-run --task-id X99`，但 `X99` 在平台映射表中不存在。
- **修复**：参考 `standards/research-workflow-contract.yaml` 查看所有有效的任务 ID（A1-I8）。

---

## 执行与编排 (EXE)

这些错误发生在实际生成和 agent 运行时阶段。

### `[ERR-RS-EXE-001]` 所有请求的并行 agent 均执行失败。
- **原因**：你请求了 `parallel`（并行）执行，但所有 agent 都立即崩溃或返回了安全约束违规。
- **修复**：检查 `.agent/logs` 中的 agent 日志。这通常发生由于 `<cwd>` 目录完全为空导致 agent 缺乏上下文，或者是因为各种 API 的速率限制同时耗尽。

### `[ERR-RS-EXE-002]` 子进程执行超时。
- **原因**：某个任务超过了硬编码的最大等待时间（通常为几分钟）。
- **修复**：单次 prompt 的工作范围过大。请将你的 `task.md` 拆分为更小的子任务。

---

## 模型上下文协议 (MCP)

这些错误专门与 AI 用来与你的文件系统、搜索引擎或代码库交互的工具 (MCP) 有关。

### `[ERR-RS-MCP-001]` 未配置所需的 MCP provider。
- **原因**：某项任务（如 `scholarly-search`）明确要求使用你尚未在环境中配置的 MCP 工具。
- **修复**：查看 `.env.example`。设置适当的环境变量（例如 `RESEARCH_MCP_METADATA_REGISTRY_CMD="..."`）。或者，去掉 `--mcp-strict` 参数运行。

### `[ERR-RS-MCP-002]` MCP provider 命令执行失败或返回错误。
- **原因**：你配置的 MCP 服务器崩溃、返回了非 JSON 格式的输出，或者超时。
- **修复**：如果使用 Zotero MCP 或其他社区 MCP，请确保对应的 node 服务器正在运行且你本地的 API 密钥有效。使用 `doctor` 命令独立测试该 MCP。
