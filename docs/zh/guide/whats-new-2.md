# 2.x 有哪些变化

穷理 2.x 用一个 Rust 原生程序提供 CLI、研究指导和 Lite/Full MCP。继续使用自己的 Host 与模型设置。

## 与 1.x 相比

| 方面 | 1.x 后期 | 2.x |
|---|---|---|
| 安装 | Python 完整运行时与 npm 资产入口职责不同；已有原生 Lite 包 | 二进制解压即用，npm、PyPI、Cargo 提供同一 CLI 能力 |
| Plugin | 原生 Lite 与 Python Full 分开交付 | Lite 和 Full 都由原生程序提供 |
| 使用 | 更多安装选项和预设流程 | 安装向导；按当前任务加载研究指导 |
| 研究记录 | 工作流文件与文献引文发现 | 增加带版本校验的结构化记录、Research Graph 与阶段交接 |

## 2.2.1 的更新

本次修订整理了中英文指南，并将原生发行验证迁移到 Ubuntu 24.04 runner。
科研命令和已保存项目的格式保持不变。软件包范围见
[2.2.1 发布说明](https://github.com/jxpeng98/qiongli/releases/tag/v2.2.1)，
当前下载方式见[安装](install.md)。

## 2.2.0 的新增内容

- 读取公开 HTTPS 全文，保留来源与页码、章节定位。
- 审阅后保存阶段总结、论文笔记与来源数据包；追加笔记保留原文。
- 改进阅读、写作、综合和证据核查指导。
- 增加 Linux ARM64 原生包；Lite 提供 15 个工具，Full 提供 33 个。

检索清单 `retrieval_manifest.csv` 的保存留待后续版本。公开全文读取不包含付费墙绕过或私人库访问。Graph 从规范记录建立关系，研究结论仍需核查来源。

完整范围见 [2.2.0 发布说明](https://github.com/jxpeng98/qiongli/releases/tag/v2.2.0)。安装与升级见[安装](install.md)、[升级与回退](upgrade.md)；旧命令见 [1.x 参考](../reference/cli.md)。
