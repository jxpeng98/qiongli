# 系统架构

Qiongli 2 通过 Rust CLI、Plugin/Skills 和 Lite/Full MCP 提供研究指导与工具。
模型、认证和会话由你使用的 Host 管理；穷理负责研究记录、来源关联、项目操作和安装收据。
默认 CLI 构建不需要保留的桌面应用。

## 多个入口，共用一套来源

`content/` 保存共享 Skills、工作流、模板和公开 MCP 契约，`qiongli-content`
将它们生成内嵌资源包。`packages/qiongli-native/` 下的原生服务读取这些资源，
统一管理项目状态、修订号、预览、批准、Graph 和 MCP 调用。CLI 与 Host 适配层使用同一套服务。

同一版本、同一平台的 GitHub、npm 和 PyPI 包携带相同的可执行文件，Cargo 则从源码构建。
原生包构建器共用产品描述，各渠道提供自己的启动包装和安装说明。
旧 Python/npm 产品源码保留用于兼容参考。

CLI 导出的 Plugin 包含共享 Skills 与 Full MCP，原生 Marketplace 平台 Plugin 包含 Lite MCP。
Host 读取 Plugin 缓存并启动 stdio 进程，通常无需另装 MCP 包或保持终端运行。
详见 [Plugin 内容](advanced/plugin-first-architecture.md)。

## 研究修改与证据

CLI、Full MCP 和保留的 App 使用相同的项目服务与修订规则。
Graph 根据已保存的研究记录展示关系，保留论点 ID、citekey 和来源位置，
不会把总结或审稿意见变成新的原始证据。可以查看 [Graph 示例](examples/research-graph.md)。

Lite 提供有范围限制的文献与 Zotero 工具，Full MCP 在此基础上增加项目操作。
写入工具 `qiongli_project_capture_apply` 会重新检查预览，要求计划摘要匹配，
且 `approve_filesystem_write=true`。进程内 ToolHost 仍只读，并拒绝这项写入。
一次操作获准不代表可以任意修改。直接访问 Zotero 使用本地 Companion；
无法连接时仍可生成导入文件内容。

子代理通过 Host 的实际工具运行。跨 Host 交接包将限定来源和候选稿交回一个协调代理，
不会转交正式项目的写入权限。现有交接方式和暂不提供的自动化能力，见
[协作指南](advanced/agent-skill-collaboration.md)。

## 决策与维护

架构由 `docs/architecture/decisions/` 下已接受的 ADR 管理。
ADR 0218 明确 CLI 优先、由 Host 执行模型的方向；ADRs 0219–0223 定义原生分发，
ADR 0224 定义确认后的 Host 注册，ADR 0227 将整合后的 `main` 作为正式版来源，
`2.x` 继续用于预发布。合并本身不代表发布或 Host 实际使用已经通过验收。

Tauri/Svelte、App API 和此前的 ACP/All Chat 工作按已接受决策保留维护或推迟处理，
不是使用 CLI 的前提。入口行为不一致时，应修复共享实现，再重新生成受影响的输出。
详见[仓库结构](development/repository-structure.md)与[编辑约定](conventions.md)，
不要改写已接受的决策历史。
