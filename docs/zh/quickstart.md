# 2.x 快速开始

先下载一个适合当前系统的[原生二进制包](guide/cli-2x.md#standalone-binary-download)，
解压后在终端运行 `./qiongli --version`；Windows 使用 `.\qiongli.exe --version`。
程序自带研究资源，不需要另装 Python、Node.js 或 Rust。希望使用包管理器时，
[安装指南](guide/cli-2x.md)分别说明了各渠道的前提和命令。

## 检查当前安装

加入 PATH 后，可以直接运行：

```sh
qiongli doctor
qiongli setup
qiongli content
qiongli project
```

`setup` 供你检查其他可见的 CLI 副本，不会删除或移动它们。
`project` 显示已经登记的研究项目；用 `qiongli help project create` 查看创建方法。
更新包前保留项目文件与备份，不要把包卸载当作研究资料清理。

## 在 Host 中使用

当前开发版可用 `qiongli install plugin` 导出随包 Plugin，再调用 Codex 或 Claude
的官方命令注册。先按[完整示例](guide/cli-2x.md#install-and-upgrade-bundled-content)
选择导出目录，再分别确认文件和 Host 操作。这些快捷命令尚未包含在 beta.3 发布标签中。

更新后新开 Host 会话，确认实际工具可用。然后直接提出请求，例如“阅读这篇论文，
说明主要结果和证据局限”；新构建的 Codex Plugin 也可以用 `$qiongli-paper-read`。
它会先读取共享指导，再按需读取工作流，不需要重复选择已经明确的研究阶段。

## 保存和继续研究

需要保存结果时先审阅具体修改。Graph 从规范记录重建；普通笔记需要先由 Host
整理为带有来源的记录。阶段完成后，可要求生成保留主要内容、来源与变化的总结。
所有原文件默认保留；如需清理，由你选择具体文件并亲自删除。

[Graph 范围与检查](guide/cli-2x.md#research-graph) · [1.x 历史命令](reference/cli.md)
