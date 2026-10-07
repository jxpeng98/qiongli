# 快速开始

## 1. 安装并接入 Host

按[安装指南](guide/install.md)下载程序或用包管理器安装。在终端运行：

```sh
qiongli install
qiongli mcp check
```

未加入 PATH 时，用 `./qiongli`；PowerShell 用 `.\qiongli.exe`。选择 Plugin 和 Host，分别确认文件变更与注册。

2.4.0 之后的下一次更新可选择 **5 — 全部安装**，或运行 `qiongli install all`，
在一次流程中为所有检测到的受支持客户端 CLI 安装 Plugin，包括
[Pi coding agent](advanced/plugin-installation.md#pi)。缺少的客户端会列出并
跳过；Plugin 已包含 Skills 和 MCP，每个选中的 Host 仍保留各自的预览与必要确认。

## 2. 开始一项研究任务

新开 Host 会话，请它列出穷理工具并调用 `qiongli_config_status`，再提出请求：

> 阅读这篇论文，说明主要发现，标出证据位置，并指出尚不确定的地方。

Codex 中可用 `$qiongli-paper-read`，也可直接描述任务。你可以从当前材料开始；见[研究任务](guide/task-recipes.md)。

## 3. 保存并继续

保存前审阅具体修改。阶段完成后，可以要求“总结这一阶段，保留来源、决定和待解决问题”。原文件会保留；删除由你自己选择并执行。

需要查看论点与来源的关系，见 [Research Graph 示例](examples/research-graph.md)和 [CLI 命令](guide/cli-2x.md)。只想讨论时，用 `$no-qiongli` 或说“仅回复”。
