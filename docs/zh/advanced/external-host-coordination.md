# 外部 Agent 协作

DeepSeek 安装见 [Plugin 配置](plugin-installation.md#deepseek)。已接入的 Host 可以准备交接包，交给 Codex、Claude Code、DeepSeek Harness 或 Antigravity CLI 执行，再收集结果。

## 交接时保留什么

- 明确任务、授权来源和候选版本。
- 记录实际运行的代理与返回结果，核对来源是否变化。
- 由原协调者整合；外部结果不会自动获得项目写入权限。

实际子进程由 Host 启动和监督，Qiongli 核对任务与来源绑定。没有通信工具时，使用手动转交的数据包。

开发者导出、`agent prepare` / `collect` 参数与各 Host 的运行限制，见[英文详细指南](../../advanced/external-host-coordination.md)。多代理用法见[协作指南](../guide/multi-agent.md)。
