<div align="center">
  <h1>穷理（Qiongli）</h1>
  <p><strong>用 AI agent 做研究，保留可复查的来源。</strong></p>
  <p>在 Codex、Claude Code 或 DeepSeek Harness 中阅读论文、设计研究和写作，继续使用自己的模型设置，保留研究记录。</p>
  <p>
    <a href="https://www.npmjs.com/package/qiongli"><img alt="npm 版本" src="https://img.shields.io/npm/v/qiongli/latest?style=flat-square&amp;logo=npm"></a>
    <a href="https://pypi.org/project/qiongli/"><img alt="PyPI 版本" src="https://img.shields.io/pypi/v/qiongli?style=flat-square&amp;logo=pypi"></a>
  </p>
  <p><a href="README.md">English</a> · <a href="docs/zh/index.md">文档</a> · <a href="tooling/release/v2.5.0.md">2.5.0 发布说明</a></p>
</div>

## 安装

从 [Release v2.5.0](https://github.com/jxpeng98/qiongli/releases/tag/v2.5.0) 下载对应平台的 CLI，核对 `SHA256SUMS` 后解压。运行 `./qiongli --version`（PowerShell：`.\qiongli.exe --version`），无需另装 Python、Node.js 或 Rust。

支持 macOS ARM64、Windows x64、Linux x64/ARM64（glibc 2.35+）。详见[下载与包管理器安装](docs/zh/guide/install.md)。

已有 Node.js 18+，也可以用 npm 安装：

```sh
npm install --global qiongli@latest
```

## 开始使用

在终端运行；未加入 PATH 时，用 `./qiongli` 或 `.\qiongli.exe`：

```sh
qiongli install
qiongli mcp check
```

选择 Host，分别确认文件变更和 Plugin 注册。Plugin 已包含 Skills 与 Full MCP。新开 Host 会话，确认穷理工具可用，再直接提出请求：

> 阅读这篇论文，说明主要发现，标出证据位置，并指出尚不确定的地方。

Codex 中也可以用 `$qiongli` 或 `$qiongli-paper-read`。只想讨论、不调用工具或处理文件时，用 `$no-qiongli` 或说“仅回复”。详见 [Skills 与可选 Hook](docs/zh/advanced/agent-skill-collaboration.md)。

保存项目修改前须预览、授权并核对版本。文献服务和 Zotero 按需配置。Marketplace Plugin 使用 Lite MCP，CLI 安装的 Plugin 使用 Full MCP。

## 升级

通过原安装渠道更新 CLI，再刷新 Plugin：

```sh
qiongli install plugin
```

随后新开 Host 会话。从 1.x 升级时，先备份项目和设置，再安装 2.5.0 并接入 Host。新版保存记录可能需要新版 CLI，换回旧程序不会自动降级项目数据。详见[升级与回退](docs/zh/guide/upgrade.md)、[2.x 版本变化](docs/zh/guide/whats-new-2.md)。

## 文档

- [快速开始](docs/zh/quickstart.md) · [CLI 命令](docs/zh/guide/cli-2x.md)
- [研究任务](docs/zh/guide/task-recipes.md) · [Research Graph 示例](docs/zh/examples/research-graph.md)
- [文献服务与 Zotero 配置](docs/zh/advanced/index.md) · [故障排除](docs/zh/guide/troubleshooting.md)
- [参与开发](CONTRIBUTING.md) · [1.x 与历史资料](docs/zh/legacy/index.md)

## 致谢

Academic Idea Funnel 与 Academic Grill Loop 将 Matt Pocock 的 `grill-me` 思路用于学术选题和研究审议。感谢 [linux.do](https://linux.do/) 社区的讨论与反馈。
