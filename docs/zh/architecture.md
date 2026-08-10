# 系统架构

Qiongli 2 是一个自包含的 Rust 原生产品，桌面表现层采用 Tauri 2 / Svelte 5。
打包后的 App 同时携带原生 CLI、内嵌 Skills、Lite/Full MCP、受管理的
Codex/Claude 集成 payload 和 Zotero Companion；运行时不要求用户另装 Python
或 Node。

## 决策边界

`docs/architecture/decisions/` 下已接受的 ADR 控制 2.x。ADR 0210 已用
Tauri/Svelte 取代早期的 AccessKit/egui 表现层选择；ADR 0211 规定模型认证、
对话和执行由受支持的 Host 持有，Qiongli 负责确定性内容、项目状态、工具、
handoff、安装收据和发布身份。

与已接受 ADR 冲突的改变必须先提交替代 ADR。生成 payload 和历史迁移计划不能
覆盖当前决策。

## 可编辑源边界

| 边界 | 可编辑源 | 职责 |
|---|---|---|
| 学术内容与合同 | `content/` | workflow、Skills、templates、roles、standards、Plugin metadata、MCP profiles 与 schemas |
| 原生产品 | `packages/qiongli-native/` | App service、CLI、Lite/Full MCP、项目状态、内嵌资源、集成与发布 runtime |
| App wire contract | `packages/qiongli-app-api/` | 原生 snapshot、intent 和 event 的版本化 TypeScript 解码 |
| 桌面表现层 | `packages/qiongli-desktop/` | Svelte UI 与 typed transport adapter |
| 分发组件 | `packages/qiongli-lite-mcp/`、`packages/qiongli-*-mcpb/`、`packages/qiongli-zotero-companion/` | 独立打包的 MCP 与 Zotero 交付面 |
| 旧版 1.x | `packages/python-qiongli/`、`packages/npm-qiongli/` | 维护中的 1.x 兼容与迁移证据，不是 2.x runtime fallback |
| 维护工具 | `tooling/`；稳定 wrapper 位于 `scripts/` | materialization、validation、packaging、acceptance 与 release automation |
| 证据 | `tests/`、`evals/`、`docs/superpowers/acceptance/` | 聚焦回归、评测资产和已接受收据 |

根目录 `scripts/` 保持稳定入口，具体实现改 `tooling/scripts/`。Plugin 与 Skill
应编辑 `content/` 中的 canonical 输入，再生成 payload；不要把 `dist/`、已安装
客户端目录或生成 plugin tree 当成源文件编辑。

## 产品主链

1. `content/` 定义学术行为、公开 MCP 合同和分发 metadata；
2. `qiongli-content` 生成由原生 executable 消费的确定性 resource pack；
3. 原生 service 统一负责配置、项目状态、preview、approval、mutation、CLI、
   MCP dispatch 和 Host integration；
4. App API 校验原生 wire shape，Svelte 通过 Tauri 展示并发送 typed intent；
5. Plugin/Skills 与 MCP package 向 Codex、Claude Code 暴露同一份内嵌合同；
6. Zotero Companion 只能通过受限 loopback client 访问，import-file export 是
   安全 fallback。

App、CLI、Full MCP 和 Host handoff 必须共用相同的项目 service 与 revision
语义。前端不能自行构造原生 plan、路径、provider model 或 readiness claim。

## MCP 与写入边界

Lite MCP 负责有边界的 provider、literature、planning 和 Zotero 工具；Full MCP
增加已注册项目与 Academic Graph 操作。公开 Full MCP 含一个明确的项目写入工具
`qiongli_project_capture_apply`：它会重新 preview，并要求匹配的 plan digest 和
`approve_filesystem_write=true`。

进程内 ToolHost 仍然只读并拒绝这个写入。因此发布说明必须区分“一个受审批约束的
capture 写入”和“不受限的 Full MCP/ToolHost mutation”。

## 依赖方向

默认采用单向依赖：

1. canonical standards、Skills、MCP schemas 与 Plugin metadata；
2. 原生 domain/project/runtime services；
3. App API 与 CLI/MCP adapters；
4. Svelte 与 Host presentation；
5. materialized packages 与 release evidence。

入口之间出现漂移时，应修复最高层的共同 owner，再重新生成或适配下游；不要新增
第二套项目格式、provider registry、release ledger 或 product backend。

精确目录职责见 [仓库结构](/zh/development/repository-structure)。
