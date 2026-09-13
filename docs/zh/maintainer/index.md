# 维护者

这一部分面向“要改系统本身”的人，而不是只想使用系统的人。

## 维护者阅读路径

- [维护工作流程](/zh/maintainer/claude-overview)
- [系统架构](/zh/architecture)
- [规范约定](/zh/conventions)
- [本地桌面开发与打包](/zh/development/local-desktop-build)
- [仓库结构](/zh/development/repository-structure)
- [命名策略](/zh/maintainer/naming-policy)
- [发布分支策略](/zh/maintainer/release-branch-policy)
- [扩展 Qiongli](/zh/advanced/extend-qiongli)
- [发布原生渠道包](/zh/advanced/publish-pypi)

## 日常维护

先定位负责该行为的源文件，再运行受影响的检查，更新中英文文档。
当前原生开发在 `2.x` 整合，审阅后的版本按授权合入 `main`。
桌面端资料保留用于维护；旧 Python 分支只承担约定范围内的兼容与关键修复。
