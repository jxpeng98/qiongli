<div align="center">
  <h1>穷理（Qiongli）</h1>
  <p><strong>用 AI agent 做学术研究，同时保留可复查证据链。</strong></p>
  <p>从阅读、设计到写作，穷理按当前任务提供指导，并把来源、决定和阶段成果保存在可追踪的研究记录中。</p>
  <p>
    <a href="https://www.npmjs.com/package/qiongli"><img alt="npm latest version" src="https://img.shields.io/npm/v/qiongli/latest?style=flat-square&amp;logo=npm&amp;label=npm%20latest"></a>
    <a href="https://www.npmjs.com/package/qiongli?activeTab=versions"><img alt="npm next version" src="https://img.shields.io/npm/v/qiongli/next?style=flat-square&amp;logo=npm&amp;label=npm%20next&amp;color=cb3837"></a>
    <a href="https://pypi.org/project/qiongli/"><img alt="PyPI latest version" src="https://img.shields.io/pypi/v/qiongli?style=flat-square&amp;logo=pypi&amp;label=PyPI%20latest"></a>
  </p>
  <p>
    <a href="README.md">English README</a> ·
    <a href="docs/zh/index.md">中文文档</a> ·
    <a href="docs/index.md">Docs</a> ·
    <a href="docs/zh/quickstart.md">快速开始</a> ·
    <a href="docs/zh/guide/cli-2x.md#standalone-binary-download">安装</a> ·
    <a href="docs/zh/guide/cli-2x.md">CLI</a>
  </p>
</div>

## 穷理 2.x

`main` 已整合原生 2.x，`2.x` 继续用于开发与预发布。当前版本仍标记为
`2.0.0-beta.6`；主分支合并不代表正式版已经发布。

穷理是面向学术研究的 Rust 原生 CLI，随包提供 Skills、模板和 Lite/Full MCP。
你可以继续使用自己的 Codex、Claude Code 和模型设置，穷理负责研究记录、来源、
可审阅的修改以及阶段交接。使用 CLI 不需要安装穷理桌面 App。

安装与更新使用同一套向导：在终端运行 `qiongli install` 打开向导，
或用 `qiongli install plugin` 复用已有 Host 注册。`qiongli mcp check` 检查本地协议。
推荐安装的 Plugin 已包含研究 Skills 和 MCP；独立 Skills 是可选的文件导出。
[2.x 使用指南](docs/zh/guide/cli-2x.md)说明了各入口与首次 Host 会话的检查方法。

可以先查看离线 [Research Graph 示例页面](docs/zh/examples/research-graph.md)，
再按需要使用研究指导、仅回复入口和可选上下文 Hook。

## Qiongli 2.x 独立二进制下载

**下载、解压、运行。不用先安装 Python、Node.js、Rust 或包管理器。**

在 [GitHub Release v2.0.0-beta.6](https://github.com/jxpeng98/qiongli/releases/tag/v2.0.0-beta.6)
中选择对应平台的完整 CLI，解压后运行 `./qiongli --help`（Windows PowerShell 使用 `.\qiongli.exe --help`）。
研究 Skills、模板和 Lite/Full MCP 资源都在程序里，不需要额外安装。
你可以直接在解压目录使用，安装 App 或配置 PATH 都不是前提。

| 平台 | 二进制压缩包 |
|---|---|
| macOS Apple Silicon（ARM64） | [下载 `.tar.gz`](https://github.com/jxpeng98/qiongli/releases/download/v2.0.0-beta.6/qiongli-2.0.0-beta.6-aarch64-apple-darwin.tar.gz) |
| Windows x64 | [下载 `.zip`](https://github.com/jxpeng98/qiongli/releases/download/v2.0.0-beta.6/qiongli-2.0.0-beta.6-x86_64-pc-windows-msvc.zip) |
| Linux x64（glibc 2.35+） | [下载 `.tar.gz`](https://github.com/jxpeng98/qiongli/releases/download/v2.0.0-beta.6/qiongli-2.0.0-beta.6-x86_64-unknown-linux-gnu.tar.gz) |

Windows beta 版本已将 C 运行库编入程序，无需另外安装 Visual C++ 运行库。
Linux 使用系统自带库，要求 glibc 2.35+。

运行前使用同一 Release 的 [SHA256SUMS](https://github.com/jxpeng98/qiongli/releases/download/v2.0.0-beta.6/SHA256SUMS)
核对文件；[完整指南](docs/zh/guide/cli-2x.md)说明解压、PATH 和 MCP 接入步骤。
请在 **Assets** 中选择上述平台包，GitHub 自动生成的 **Source code** 是源码。
模型 Host 和在线文献服务仍需单独配置。

使用包管理器时，有 Node.js 18+ 可运行 `npm install --global qiongli@next`；
有 Python 3.9+ 可按 [PyPI 安装说明](docs/zh/guide/cli-2x.md#package-managers)安装。
这两个渠道都自带原生程序。已有 Rust 1.97+ 和本机链接器时，也可以通过 Cargo 从源码安装。

```sh
cargo install qiongli --version 2.0.0-beta.6 --locked
```

Cargo 提供 `qiongli` 和 `ql`。如果不想编译，直接下载上方二进制包即可。

运行 `qiongli` 查看帮助，`qiongli project` 列出项目，
`qiongli setup` 检查已安装的 CLI，并逐项查看归档或卸载说明。npm 可在前台安装时显示向导，
pip 和 Cargo 在安装后运行。向导不会删除文件或修改设置。
详见[安装迁移说明](docs/zh/guide/cli-2x.md#检查和迁移已有-cli)。

## 安装后怎么用

```sh
qiongli --version
qiongli install
qiongli mcp check
qiongli doctor
qiongli setup
qiongli content
qiongli help install plugin
```

`setup` 检查本机可见的 CLI 副本，供你选择常用版本并查看手动处理方法。
它不会卸载程序、移动文件或改动 PATH。`qiongli` 和 `ql` 使用同一套命令；独立下载包
直接提供 `qiongli`，包管理器安装同时提供 `ql`。脚本请使用 `--json`。

`install plugin` 先预览文件变更，确认后导出，再单独确认官方 Host 注册；
再次运行会发现并更新已登记的源目录，`upgrade plugin` 使用相同流程。`install skills` 只导出
`.qiongli-skills` 内容，加载到 Host 请使用 Plugin 安装入口。
`upgrade cli` 给出原渠道升级方法，不替你执行包管理器。

[安装、升级和注册示例](docs/zh/guide/cli-2x.md#install-and-upgrade-bundled-content)
说明了完整步骤。更新 Plugin 后，需要新开 Host 会话检查工具是否可用。
beta.6 的安装向导可选上下文 Hook，首次默认关闭；也可使用
`qiongli install plugin --hooks context` 加入，或用 `--hooks off` 移除配置。

## Skills、MCP 与研究记录

使用 **`no-qiongli`**（Codex 中为 `$no-qiongli`），或直接说“仅回复”“NoQ问理”，
可要求模型只依据对话回答，不调用工具、代理或文件操作。
[只回复的范围与限制](docs/zh/advanced/agent-skill-collaboration.md#reply-only)说明了具体边界。

| 部分 | 用途 |
|---|---|
| Skills / Plugin | 按请求选择阅读、文献综述、研究设计、写作、润色或阶段总结，复用共同指导 |
| Lite MCP | 14 个工具，提供受限的文献、配置、检索规划和 Zotero 操作 |
| Full MCP | 32 个工具，增加项目、Graph 和 Host 交接；写入仍须预览、授权和版本校验 |
| Research Graph | 从规范研究记录重建，将论点、来源和具体位置连接起来；缺少证据的关系保持未确认 |
| 阶段总结 | 保留主要内容、来源、前序和变化记录；只在用户要求时列出逐文件保留建议，删除由用户亲自完成 |

新构建的 Codex Plugin 提供 20 个工作流快捷入口和 `$qiongli` 主入口，
82 张内部技能卡按需读取。Claude 保留研究主 Skill；两个 Host 均包含独立的
`no-qiongli` 入口。原生 Marketplace 平台包默认
运行 Lite MCP；CLI 导出的本地 Plugin 运行 Full MCP。相同的 Skill 名称不代表
相同的工具可用性。

Graph 的改善以稳定标识、多来源、可定位证据、确定性重建和过期读取拒绝来验证。
它需要 Host 将授权材料整理成规范记录，不能自动理解任意目录或 PDF。
[Graph 的使用与验证范围](docs/zh/guide/cli-2x.md#research-graph)列出了实际检查标准。

## 开发与文档

共享研究内容在 `content/`，原生实现位于 `packages/qiongli-native/`。
npm、PyPI 和 Cargo 使用各自的启动入口与安装说明，共用 CLI 功能描述和版本身份。
修改源文件后通过现有构建流程生成包；不要修改已安装缓存或生成副本。

- [快速开始](docs/zh/quickstart.md)与[2.x 命令指南](docs/zh/guide/cli-2x.md)
- [系统架构](docs/zh/architecture.md)与[开发约定](CONTRIBUTING.md)
- [1.x 命令参考](docs/zh/reference/cli.md)：仅用于旧版本和迁移对照
- [1.x 兼容分发构建](docs/development/distribution-materialization.md)：staged materialization 与 npm package contract tests

现有测试检查授权、回滚、包与源码是否对应，以及内容完整性。
性能和 Host 会话效果仍以具体样例的实测为准，不能仅凭 Rust 迁移或文案调整判断。

## 致谢

Academic Idea Funnel 与 Academic Grill Loop 借鉴了 Matt Pocock 的 `grill-me`
思路，并针对研究证据、竞争解释和可行性作了学术调整。感谢
[linux.do](https://linux.do/) 社区的实践讨论和反馈。
