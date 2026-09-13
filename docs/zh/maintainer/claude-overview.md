# 维护工作流程

先读仓库的 `AGENTS.md` 和 `CONTRIBUTING.md`，再从总路线图进入当前计划。
任务状态与已接受证据由进度账本管理。`CLAUDE.md` 补充 Host 使用背景，
不另行定义发布或研究验收结果。

根据[仓库结构](../development/repository-structure.md)，找到负责所需行为的源文件。
原生 CLI、MCP 和安装修改位于 `packages/qiongli-native/`，研究指导位于 `content/`。
生成结果与保留的 1.x 产品源码分别处理。

在本地功能分支修改，运行受影响的检查，审阅差异并作范围明确的提交，再整合到本地 `2.x`。
合入 main 与发布按用户要求及[发布策略](release-branch-policy.md)执行。
检查结果和缺口集中记录在计划与账本；本地合并不能让尚未验证的 Host 流程或发布渠道自动通过验收。

协作时划清独立任务的范围，避免并发修改共享研究记录，并保留用户的 Host 和模型设置。
子代理与跨 Host 交接方式见[协作指南](../advanced/agent-skill-collaboration.md)。
