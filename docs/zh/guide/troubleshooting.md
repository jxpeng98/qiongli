# 故障排除

先运行 `qiongli --version`、`qiongli doctor` 和 `qiongli install list`，
确认问题出在 CLI、Plugin 注册，还是 Host 会话。需要结构化诊断时加上 `--json`。

## 终端仍在运行旧版本

macOS/Linux 使用 `type -a qiongli ql`，PowerShell 使用 `Get-Command qiongli,ql -All`
查看命令位置。不同包管理器可能各装一份，由 PATH 决定先运行哪一份。
用新版程序的绝对路径启动，再运行 `qiongli setup` 审查其他安装。
它只给出手动清理建议，不会删除、归档或移动程序及研究文件。

2.x 的 npm 和 PyPI 包都已包含原生 CLI，遇到问题不需要另装 Python Full 运行时。
PATH 与 npm 安装脚本提示见[包管理器安装说明](cli-2x.md#package-managers)。

## Plugin 已导出，但注册失败

保留导出目录，对同一个 Host 重新运行 `qiongli install plugin`，安装器会先核对收据。
如果启用了另一份 Qiongli Plugin，按提示选择迁移或手动停用。
已知 Codex 插件可确认迁移；其他冲突按提示手动停用。
文件被修改或注册指向异常时，应先核对，不要直接清空缓存。
具体流程见[首次使用](cli-2x.md#first-use)。

## 找不到 Skills 或 MCP 工具

确认目标 Plugin 已启用，再新开 Host 会话。Host 从 Plugin 缓存读取 Skills，
无需复制到 `~/.agents/skills`。Codex 中可选 `$qiongli`，或列表中可见的
`$qiongli-paper-read` 等快捷入口，也可以直接用自然语言描述研究任务。
如果只导出了独立 Skills，它不会自动注册 Host 或连接 MCP。

运行 `qiongli mcp check --profile full` 检查本地协议，再请 Host 列出实际工具并调用
`qiongli_config_status`。本地检查成功，不代表会话已经连接。

## 文献检索提示 `strategy_only`

先调用 `qiongli_literature_status`，再判断可用的检索路径。
文献服务配置与 Host 原生搜索（native search）是两种能力。
`qiongli_search_plan` 分别报告 `provider_capability_mode` 和 `search_execution_mode`；
后者可能为 `native_only`、`hybrid_search`、`provider_connected` 或 `strategy_only`。
MCP servers 不会执行 Host 的原生搜索，这部分由当前代理使用实际可用的工具完成。
参照[文献服务配置](../advanced/mcp-providers-setup.md)排查，并如实说明服务错误或访问缺口。

## Graph 页面为空，或内容没有更新

Graph 展示已经保存的研究记录，不会把任意 PDF 自动变成经过验证的论点。
参照 [Graph 示例](../examples/research-graph.md)，审阅并保存结构化记录，刷新项目后重新导出。
旧 HTML 页面不会随项目变化；`--open` 没有弹出窗口时，可以手动打开命令返回的文件路径。
基于旧修订号的来源命令也需要重新导出后再使用。

## Hook 或 Zotero 没有响应

Hook 需要分别检查 Plugin 配置、Host 信任和实际事件是否送达，安装检查不能替代执行验证。
详见 [Hook 验证](../advanced/agent-skill-collaboration.md#optional-context-hooks)。
Zotero 则需要打开应用并启用兼容的 Companion，再调用 `qiongli_zotero_status`。
详见 [Zotero 配置](../advanced/mcp-zotero-integration.md)。

反馈问题时，请附 CLI 版本、平台、安装渠道、命令和相关报错。
分享诊断前，移除私人路径、凭据和研究正文。
