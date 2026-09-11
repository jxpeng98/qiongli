# Qiongli 2 CLI：安装与直接下载

Qiongli 2 以原生 CLI 为入口，不需要打开或安装 Qiongli App。
同一版本、同一平台的 GitHub 二进制包、npm 和 PyPI 包使用相同的原生可执行文件。

## 常用命令（beta.3）

Beta.3 提供更短的命令入口、按操作显示的帮助，以及易读的终端输出。
`ql` 是 `qiongli` 的短名称，两者用法相同。

| 要做什么 | 简短入口 | 此前 beta.2 的写法 |
|---|---|---|
| 查看常用命令 | `qiongli` | `qiongli --help` |
| 检查本地状态和问题 | `qiongli doctor` | 命令相同，输出为 JSON |
| 审查已安装的 CLI 版本 | `qiongli setup` | `qiongli install migrate --interactive` |
| 查看 CLI 安装和 Host | `qiongli install` | `qiongli install inventory` |
| 列出研究项目 | `qiongli project` | `qiongli project list` |
| 查看一个项目 | `qiongli project show <id>` | `qiongli project show --project-id <id>` |
| 查看配置 | `qiongli config` | `qiongli config show` |
| 查看内置内容配置档 | `qiongli content` | `qiongli content list` |
| 以 stdio 接入 Full MCP | `qiongli mcp serve --profile full` | 还需加上 `--transport stdio` |

不必在一页中查找所有参数。例如，`qiongli help project create` 和
`qiongli project create --help` 都只显示创建项目的用法。
`qiongli help all` 提供完整命令参考，包括高级安装操作。
`qiongli update` 查询受管理安装的更新状态；通过 npm、pip 或 Cargo 安装的包，
仍使用原包管理器升级。

终端中的查询默认显示易读摘要；重定向时保留原有输出格式。脚本可以明确使用
`--json`，需要保存易读报告时则使用 `--text`。格式参数放在命令开头或末尾，只选一个：

```sh
qiongli status --json
qiongli doctor --text > qiongli-doctor.txt
```

`setup` 借用了 1.x 熟悉的入口名称，在 2.x 中专门用于审查 CLI 安装，不会配置 Host、
更换模型或卸载程序。无参数运行改为显示帮助，需要审查版本时再运行 `qiongli setup`。
项目写入仍需预览、批准和修订检查；易读预览完整保留审批所需的参数，程序处理时使用 `--json`。

## 独立二进制下载 {#standalone-binary-download}

**推荐直接下载：解压后就能运行，不用先安装 Python、Node.js、Rust 或包管理器。**
你可以直接在解压目录使用，PATH 配置是可选项。

从 [GitHub Release v2.0.0-beta.3](https://github.com/jxpeng98/qiongli/releases/tag/v2.0.0-beta.3)
选择与你的操作系统和 CPU 对应的压缩包：

| 平台 | 完整 CLI 二进制包 |
|---|---|
| macOS Apple Silicon / ARM64 | [qiongli-2.0.0-beta.3-aarch64-apple-darwin.tar.gz](https://github.com/jxpeng98/qiongli/releases/download/v2.0.0-beta.3/qiongli-2.0.0-beta.3-aarch64-apple-darwin.tar.gz) |
| Windows x64 | [qiongli-2.0.0-beta.3-x86_64-pc-windows-msvc.zip](https://github.com/jxpeng98/qiongli/releases/download/v2.0.0-beta.3/qiongli-2.0.0-beta.3-x86_64-pc-windows-msvc.zip) |
| Linux x64 / glibc 2.35+ | [qiongli-2.0.0-beta.3-x86_64-unknown-linux-gnu.tar.gz](https://github.com/jxpeng98/qiongli/releases/download/v2.0.0-beta.3/qiongli-2.0.0-beta.3-x86_64-unknown-linux-gnu.tar.gz) |

Windows beta 版本已将 C 运行库编入程序，不需要另装 Visual C++ 运行库。
Linux 使用系统自带库，要求 glibc 2.35+。

压缩包内有 `qiongli`（Windows 为 `qiongli.exe`）、`README.md` 和 `LICENSE`。
研究 Skills、模板及 Lite/Full MCP 资源已经嵌入可执行文件，无需额外下载资源目录或克隆仓库。
模型 Host 和在线文献服务仍需单独配置。

请在 Release 的 **Assets** 中选择上述平台包。页面自动生成的 **Source code** 是需要编译的源码，
`.tgz` 是 npm 包，`.whl` 是 Python 包，`qiongli-next-…-plugin-…` 是 Host 插件包。
当前版本不提供 Intel Mac、Linux ARM 或 Windows ARM 原生构建。

### 1. 校验下载文件

下载同一 Release 的 [SHA256SUMS](https://github.com/jxpeng98/qiongli/releases/download/v2.0.0-beta.3/SHA256SUMS)。
在下载目录中，运行与你的平台对应的命令，将结果与 `SHA256SUMS` 中该文件名对应的摘要比较；一致后再继续。

```sh
# macOS
shasum -a 256 qiongli-2.0.0-beta.3-aarch64-apple-darwin.tar.gz
# Linux
sha256sum qiongli-2.0.0-beta.3-x86_64-unknown-linux-gnu.tar.gz
```

```powershell
# Windows
Get-FileHash .\qiongli-2.0.0-beta.3-x86_64-pc-windows-msvc.zip -Algorithm SHA256
```

### 2. 解压后直接运行

使用一个新目录，保留已有安装和研究文件。在 macOS 终端中：

```sh
mkdir qiongli-2.0.0-beta.3-macos-arm64
tar -xzf qiongli-2.0.0-beta.3-aarch64-apple-darwin.tar.gz -C qiongli-2.0.0-beta.3-macos-arm64
cd qiongli-2.0.0-beta.3-macos-arm64
./qiongli --version
./qiongli --help
./qiongli content list
```

在 Linux 终端中：

```sh
mkdir qiongli-2.0.0-beta.3-linux-x64
tar -xzf qiongli-2.0.0-beta.3-x86_64-unknown-linux-gnu.tar.gz -C qiongli-2.0.0-beta.3-linux-x64
cd qiongli-2.0.0-beta.3-linux-x64
./qiongli --version
./qiongli --help
./qiongli content list
```

Windows 用户在下载目录打开 PowerShell：

```powershell
Expand-Archive -Path .\qiongli-2.0.0-beta.3-x86_64-pc-windows-msvc.zip -DestinationPath .\qiongli-2.0.0-beta.3-windows-x64
Set-Location .\qiongli-2.0.0-beta.3-windows-x64
.\qiongli.exe --version
.\qiongli.exe --help
.\qiongli.exe content list
```

版本应显示 `qiongli 2.0.0-beta.3`。独立包只提供 `qiongli` 可执行文件；`ql` 别名由 npm / PyPI / Cargo 安装提供。

### 3. 可选：加入 PATH

你可以一直使用可执行文件的绝对路径。若希望在任意目录输入 `qiongli`，将解压目录加入用户 PATH：
macOS / Linux 使用自己的 Shell 配置，Windows 使用用户环境变量设置；完成后打开新终端。

macOS / Linux 用 `type -a qiongli`，PowerShell 用 `Get-Command qiongli -All` 检查实际运行的路径，
再运行 `qiongli --version`，避免旧版本或包管理器的入口优先于新版本。

升级时将新版本解压到另一个目录，验证后再更新 PATH 或 Host 配置中的路径。
保留旧二进制及研究数据以便回退；切换二进制不会撤销数据迁移。

## 检查和迁移已有 CLI

Beta.2 可以列出本机可见的穷理安装，并提供交互式迁移建议。如果旧版在 PATH 中
排在前面，请用新安装程序的完整路径运行下面的命令：

```sh
qiongli install inventory --paths exact
qiongli install migrate --interactive
```

清单会合并同一安装的别名，并区分包元数据版本与当前运行版本。检测范围包括
PATH、常见用户安装目录及已配置的 Cargo、Python、npm 位置；其他环境和 Shell
函数需要单独检查。检测不会执行来源不明的程序。`doctor` 默认隐藏实际路径，
需要时再明确查看完整路径。

向导让你选择准备使用的安装，并逐项查看其他版本的归档或卸载说明。直接按回车
会保留现有设置。选择默认版本只生成建议，不会修改 PATH 或 Host 配置，也不会
删除、移动或归档任何文件。卸载前要确认文件归属：旧包可能与新版共用启动入口。
研究文件、配置和 Plugin 缓存不属于 CLI 清理范围。

Beta.3 无参数运行时显示帮助，通过 `qiongli setup` 主动打开向导。
此前的 beta.2 会在终端中无参数运行时直接打开向导。如需在 npm 安装过程中显示向导，
可以为本次安装授权穷理的脚本，并让脚本连接终端：

```sh
npm install -g qiongli@next --allow-scripts=qiongli --foreground-scripts
```

新版 npm 会对尚未明确授权的 `postinstall` 脚本发出警告。警告本身不代表安装失败，
也不代表脚本已被阻止；启用严格脚本策略后则可能报错。
`--allow-scripts=qiongli` 只为本次命令明确授权穷理的脚本，不修改已保存的 npm 配置。
`--foreground-scripts` 让脚本使用当前终端；输入和输出都连接终端时，向导才会出现。
详见 [npm 脚本设置](https://docs.npmjs.com/cli/v11/commands/npm-install/#allow-scripts)。

使用 `--ignore-scripts` 禁用安装脚本，仍可正常使用 CLI。已经安装成功时，无需重装，
直接用新版可执行文件的完整路径运行 `setup` 即可。
pip 和 Cargo 用户也在安装完成后运行向导。脚本、帮助与版本查询、MCP 启动不会弹出交互提示。

包管理器安装应保留在原位置，并记录版本及原环境以便重装；确认是独立发布包后，
才适合另行复制完整备份并校验。归档副本本身不会停用旧命令。


## 安装和升级随包 Plugin、Skills（2.x 开发版）

当前开发版新增以下入口，尚未包含在 beta.3 发布标签中。以 Codex 为例：

```sh
mkdir -p "$HOME/qiongli-plugins/codex"
qiongli install plugin --target codex \
  --destination "$HOME/qiongli-plugins/codex/qiongli-next"
```

命令先显示文件变更，确认后导出当前 CLI 的原生二进制、研究 Skills 和 Full MCP。
随后单独显示 Host 操作，再次确认后，调用 Codex 官方插件命令完成注册和启用。
Plugin 本身无需 Python、Node 或 Cargo 运行时；Codex 仍需事先安装。
Claude Code 使用 `--target claude`，并另建一个导出父目录，例如
`$HOME/qiongli-plugins/claude`。

通过原安装渠道更新 CLI 后，刷新同一个 Plugin 目录：

```sh
qiongli upgrade plugin --target codex \
  --destination "$HOME/qiongli-plugins/codex/qiongli-next"
```

`update plugin` 与 `upgrade plugin` 等效。重复运行 `install plugin` 也会核对并更新
已有的完整导出。更新 Host 缓存前会校验收据，只有完全匹配的旧插件才会通过官方命令
移除并重装，具体命令会列在确认页。其他来源的已启用穷理插件、被修改的文件、来源路径
冲突或意外的安装范围会阻止注册，需要你在 Host 中自行处理。

每次确认直接按回车都表示取消。取消 Host 注册或 Host 命令失败时，已导出的文件会保留；
解决问题后，以同一目标和路径重新运行 `upgrade plugin` 即可。成功提示表示官方安装状态
和缓存文件已核对，请开启新的 Host 会话加载 Skills 和 Full MCP。实际工具调用仍需在
新会话中检查，安装过程不会更换模型。

只需要独立内容文件时：

```sh
qiongli install skills
qiongli upgrade skills --preset current-project --profile full
```

默认安装完整内容到 `$HOME/.qiongli-skills`；`current-project` 使用当前目录中的
`.qiongli-skills`。这一步不会把文件自动注册为 Host Plugin。更新已有内容时需指定
原来的 profile，避免无意切换配置档。

脚本可加 `--dry-run --json` 生成文件计划，审查后用 `qiongli app apply` 和对应摘要、
批准参数执行。该方式只导出文件；Host 注册使用交互入口或手动执行官方命令。
管道输入不能代替交互确认。

`qiongli upgrade cli` 会列出 npm、pip、Cargo 和 GitHub 二进制包的升级方式，
不会自行调用包管理器。Plugin 和 Skills 更新的是当前 CLI 内置内容，不会下载新版 CLI。

## 接入 MCP 与 Plugin

在 Host 的 MCP 配置中，将 command 设为 `qiongli` 可执行文件的**绝对路径**，参数设为：

```text
mcp serve --profile full --transport stdio
```

需要 Lite MCP 时使用 `--profile lite`。Full 提供项目、Graph 和交接等工具，Lite 提供较小的文献工具集。
模型与凭据由 Host 管理；下载 CLI 不会自动注册或激活 Plugin。
接入后应能看到真实工具，并成功执行一次只读调用。

Alpha.8 也支持将当前可执行文件、Full MCP 与研究资源导出为用户批准的本地 Plugin 来源，
再通过 Host 的插件机制注册；详见[本地 Plugin 导出步骤](../../guide/cli-2x.md#export-a-local-plugin-source)。
现有项目写入仍需要对应的预览、批准和修订检查，安装不改变这些要求。

## npm / pip 安装 {#package-managers}

如果更习惯包管理器，可任选一个入口：

```sh
npm install --global qiongli@next
```

或者在 Python 虚拟环境中运行：

```sh
python -m pip install --pre qiongli==2.0.0b2
```

npm 需要 Node 18+，PyPI 需要 Python 3.9+；两者都提供 `qiongli` 和 `ql`。
安装后分别用 `--version` 核对版本。通过原包管理器升级包管理器安装的版本。

Cargo 从源码构建同一个 CLI，需要 Rust 1.97+ 和本机链接器。

```sh
cargo install qiongli --version 2.0.0-beta.3 --locked
```

安装后可使用 `qiongli` 和 `ql`。Cargo 没有 `next` 渠道，预发布版需指定完整版本号。
如果希望解压后立即使用，请选择上方的独立二进制包。

更多命令边界见[英文 CLI 指南](../../guide/cli-2x.md#which-surface-owns-which-command)。
旧版 `qiongli setup`、`check`、`project init` 等命令属于 1.x，不能直接套用到原生 2.x。
