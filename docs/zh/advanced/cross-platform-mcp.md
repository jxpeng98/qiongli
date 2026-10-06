# 将 MCP 接入 Host

Qiongli 2 的 Lite 和 Full MCP 都编译在原生程序内，不依赖 Python 或 Node。
CLI 导出的 Plugin 配置 Full（33 个工具），原生 Marketplace 平台 Plugin 配置 Lite
（15 个工具）。两者都有文献配置、检索和本地 Zotero 工具；Full 还提供受管理的研究项目工作流。

## Codex 与 Claude Code

运行 `qiongli install plugin`，选择 Host。注册后的 Plugin 包含程序和 MCP 配置，
安装后新开一个会话即可检查工具。具体步骤见[首次使用](../guide/cli-2x.md#first-use)。

## 其他本地 MCP 客户端

如果客户端支持 stdio 命令，将解压后的可执行文件绝对路径填入配置，并使用这些参数：

```text
mcp serve --profile full --transport stdio
```

只需要较小的工具集时，改用 `--profile lite`。应由客户端启动进程；
在另一个终端运行同一命令，并不会把它连接到客户端。
当前原生 CLI 在这里支持 stdio，旧 Python 版的 HTTP 服务用法不适用。
无法启动本地进程的客户端需要另行支持的接入方式，不能自行填写一个不存在的 HTTP 地址。

## 分层检查连接

运行 `qiongli mcp check --profile full`，检查当前程序的初始化、工具列表和一次只读调用。
然后在 Host 会话中查看实际工具，并调用 `qiongli_config_status`。
文献服务用 `qiongli_literature_status` 检查配置，再通过获准的检索确认服务能够响应。
本地 MCP 检查不能代替 Host 连接或在线服务验证。

`qiongli_search_plan` 返回检索方案，包括 `search_execution_mode` 和原生查询建议。
模式包括 `provider_connected`、`native_only`、`hybrid_search` 和 `strategy_only`。
`provider_capability_mode` 则单独报告 `provider_connected` 或 `strategy_only`。
MCP 不会代替 Host 调用 Codex 或 Claude 的原生搜索；Host 使用自己的工具，并分别记录
这些结果、文献服务结果与用户提供的材料（`user_corpus`）。

更多说明见[文献服务配置](mcp-providers-setup.md)和[Plugin 内容](plugin-first-architecture.md)。
模型选择与权限仍由 Host 管理，接入 MCP 不会更改它们。
