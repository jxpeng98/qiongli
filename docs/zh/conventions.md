# 源文件与编辑约定

Qiongli 2 的 CLI、Plugin/Skills 和 MCP 共享研究指导。
修改时沿用现有职责划分，才能让各入口保持一致。

| 位置 | 负责内容 |
|---|---|
| `content/` | 研究 Skills、工作流、模板、学科指导及公开契约 |
| `packages/qiongli-native/` | Rust CLI、项目服务、Graph、MCP 和原生安装 |
| `tooling/scripts/` | 维护自动化、内容生成与打包 |
| 根目录 `scripts/` | 调用维护实现的稳定入口 |
| `docs/` 和 `docs/zh/` | 英文与中文文档 |

跨层修改时，先确定共享契约，再改负责该行为的实现，最后重新生成受影响的输出。
不要把 `qiongli-workflow/`、`.agent/`、生成的 Plugin、包内资源或已安装的 Host 缓存当作源文件。
保留的 Python 与 npm 产品目录属于 1.x 兼容线，原生包使用单独的构建入口。

只有独立且可复用的任务才需要新 Skill；其他情况优先补充已有 Skill、阶段参考、模板或学科档案。
修改指导时保留任务 ID、产物路径、citekey 和来源位置。
允许模型自主选择方法，不代表可以取消批准或修订检查。

用户文档先说明要做什么，再解释操作结果。区分 CLI 安装、Plugin 注册、会话工具和在线服务，
使用当前原生命令，中英文同步更新，并为历史指南标明版本。
生成文档应修改其来源后重新生成。Humanizer 润色改善表达和衔接，但不改变要求与限制。

更多说明见[仓库结构](/zh/development/repository-structure)、
[扩展指南](/zh/advanced/extend-qiongli)和[发布策略](/zh/maintainer/release-branch-policy)。
