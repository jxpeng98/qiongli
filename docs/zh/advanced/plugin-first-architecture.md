# Plugin 如何接入 Qiongli

在 Codex 或 Claude Code 中，Plugin 可以一次接入研究 Skills 和 MCP。
运行 `qiongli install plugin`，按向导完成文件导出和 Host 官方注册即可。

## 安装内容

| 入口 | 研究指导 | 程序与工具 |
|---|---|---|
| CLI 导出的 Plugin | 共享 Skills、工作流和参考资料 | 随包原生程序；Full MCP，32 个工具 |
| 原生 Marketplace 平台 Plugin | 共享 Skills、工作流和参考资料 | 随包原生程序；Lite MCP，14 个工具 |
| 独立 Skills | 导出的指导和参考资料 | 不连接 MCP，也不自动注册 Host |

两种 MCP 都不依赖 Python 或 Node。包管理器本身仍有要求：npm 需要 Node，
PyPI 需要 Python，Cargo 需要 Rust 来编译。
[独立二进制包](../guide/cli-2x.md#standalone-binary-download)
在所支持的平台上无需安装这些语言运行时。

Host 从已注册的 Plugin 缓存中读取 Skills，并按需启动 MCP。
不用把 Plugin 复制到 `~/.agents/skills`，也不必另开终端让 MCP 一直运行。
如果只导出独立 Skills，还需要通过 Host 自己的方式安装这些文件。

## 更新与确认

通过原渠道升级 CLI 后，再运行 `qiongli install plugin`，向导会复用已登记的源目录。
文件变更和 Host 注册需要分别确认。2.0.0 支持已知 Codex 插件的确认迁移；
仍在使用 beta.6 时，请先升级 CLI，或手动停用冲突的旧插件。
具体步骤见[安装指南](../guide/cli-2x.md#first-use)。

运行 `qiongli doctor` 和 `qiongli install list` 检查注册，然后新开 Host 会话，
核对实际可用的 Qiongli 工具。注册成功并不代表工具已加载、文献服务已连通或 Hook 已执行。
可选 Hook 还需要在 Host 中单独信任。

## 维护时改哪里

共享指导的源文件在 `content/`，原生实现在 `packages/qiongli-native/`。
Plugin 和各渠道包由这些源文件生成，发布时放入临时构建目录，不把生成文件写回源码目录。
修改打包逻辑前，先查看[仓库结构](../development/repository-structure.md)和
[发布策略](../maintainer/release-branch-policy.md)。
