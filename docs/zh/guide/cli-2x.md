# Qiongli 2 CLI：安装与直接下载

Qiongli 2 以原生 CLI 为入口，不需要打开或安装 Qiongli App。
同一版本、同一平台的 GitHub 二进制包、npm 和 PyPI 包使用相同的原生可执行文件。


## beta.5 的安装与命令改进

在终端中运行 `qiongli install` 或 `qiongli upgrade` 即可打开向导。
`qiongli install plugin` 同时负责安装和更新，`upgrade plugin`、`update plugin`
使用相同流程。向导选择 Host，并复用它已经登记的源目录。也可以加
`--target codex`、`--target claude` 或 `--target all`。all 对两个 Host 分别确认，
取消或失败就停止后续步骤；新源目录必须各自独立，且父目录已经存在。
单个 Host 仍可指定 `--destination`。脚本保留参数明确的 `--dry-run` 预览，
重定向输出时的裸 `install` 仍返回只读清单。

Plugin 源文件不需要放进 `~/.agents/skills`。完成官方注册后，Host 会从自己的
Plugin 缓存中加载 Skills 和 MCP。如果已启用另一个 Qiongli Plugin，新流程会在
导出前列出它的名称。请先在 Host 中停用，再重试；旧插件文件不会被删除。
当前 Codex CLI 没有单独的 Plugin 停用命令，请使用 Codex 的插件开关；
`remove` 会删除缓存，不能当作停用使用。仅导出成功，不能算安装完成。

用 `qiongli doctor` 和 `qiongli install list` 检查注册，再开新会话验证实际工具。
独立 Skills 仍导出到 `.qiongli-skills`，不会接入 Host；需要自动注册时选 Plugin。

在终端中直接运行 `config backend`、`project graph`、`project capture`、
`project portfolio` 或 `app plan`，会显示该命令组的用法。脚本保留原有错误码；
项目写入仍需明确的预览和批准参数。


## 从安装到首次使用 {#first-use}

在终端运行 `qiongli install`（仍支持 `--interactive`）。先选择 Plugin，再选择 Codex 或 Claude。
默认导出目录是用户主目录下的 `qiongli-next`，可填写其他绝对路径；父目录必须已存在。
升级时自动复用已登记且通过核验的源目录，第二个 Host 使用单独的目录。选择菜单不会写文件，后续仍需
分别确认文件计划和 Host 注册。只需要指导文件时选 Skills；已有接入时可查看 MCP 配置。

| 入口 | 内容与接入 |
|---|---|
| CLI 包 | 原生程序，包含研究资源与 MCP 实现；不会自动配置 Host |
| CLI 安装的 Plugin | Skills + 原生程序 + Full MCP 配置，32 个工具 |
| 原生 Marketplace 平台 Plugin | Skills + 原生程序 + Lite MCP 配置，14 个工具 |
| 独立 Skills | 导出指导和参考资料，不提供运行中的 MCP，也不自动注册 Host |

MCP 已编译进 `qiongli`，无需另装服务包。Host 根据 Plugin 的配置启动 stdio 子进程，
通常不需要另开终端运行 `mcp serve`。`install skills --profile full` 中的 full 是内容范围，
不代表安装或启动了 Full MCP。用 `qiongli install list` 查看只读清单；
重定向输出时，裸命令 `install` 也保留原来的清单输出。

安装完成后分三步检查：

1. 查看文件与注册结果。取消注册会保留导出文件，可以用原目录重试。
2. 运行 `qiongli mcp check`（或 `--profile lite`）：检查当前 CLI 的初始化、工具列表和
   一次只读调用。它不验证 Plugin 缓存、Host 会话或在线服务。
3. 新开 Host 会话，要求列出实际 Qiongli 工具并调用 `qiongli_config_status`；
   文献服务的配置再用 `qiongli_literature_status` 检查。工具缺失时不能声称已就绪。

然后用一份你提供的材料试运行：阅读并保留来源位置、提出规范研究记录、审阅后保存，
再检查 Graph 和阶段总结。保存走原有批准流程，总结不删除原文件。

升级 CLI 后，运行 `qiongli doctor` 检查 Plugin 是否需要刷新，再执行
`qiongli install plugin` 选择 Host，或用 `--target` 指定。Plugin 保存自己的程序副本；版本号相同也需要核对收据和摘要。
CLI 的多版本清理仍由 `qiongli setup` 提供建议，由用户自行操作。

## 常用命令

常用操作有简短入口，帮助按操作显示，终端查询提供易读输出。
`ql` 是 `qiongli` 的短名称，两者用法相同。

| 要做什么 | 简短入口 | 此前 beta.2 的写法 |
|---|---|---|
| 查看常用命令 | `qiongli` | `qiongli --help` |
| 检查本地状态和问题 | `qiongli doctor` | 命令相同，输出为 JSON |
| 审查已安装的 CLI 版本 | `qiongli setup` | `qiongli install migrate --interactive` |
| 查看 CLI 安装和 Host | `qiongli install list` | `qiongli install inventory` |
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

从 [GitHub Release v2.0.0-beta.5](https://github.com/jxpeng98/qiongli/releases/tag/v2.0.0-beta.5)
选择与你的操作系统和 CPU 对应的压缩包：

| 平台 | 完整 CLI 二进制包 |
|---|---|
| macOS Apple Silicon / ARM64 | [qiongli-2.0.0-beta.5-aarch64-apple-darwin.tar.gz](https://github.com/jxpeng98/qiongli/releases/download/v2.0.0-beta.5/qiongli-2.0.0-beta.5-aarch64-apple-darwin.tar.gz) |
| Windows x64 | [qiongli-2.0.0-beta.5-x86_64-pc-windows-msvc.zip](https://github.com/jxpeng98/qiongli/releases/download/v2.0.0-beta.5/qiongli-2.0.0-beta.5-x86_64-pc-windows-msvc.zip) |
| Linux x64 / glibc 2.35+ | [qiongli-2.0.0-beta.5-x86_64-unknown-linux-gnu.tar.gz](https://github.com/jxpeng98/qiongli/releases/download/v2.0.0-beta.5/qiongli-2.0.0-beta.5-x86_64-unknown-linux-gnu.tar.gz) |

Windows beta 版本已将 C 运行库编入程序，不需要另装 Visual C++ 运行库。
Linux 使用系统自带库，要求 glibc 2.35+。

压缩包内有 `qiongli`（Windows 为 `qiongli.exe`）、`README.md` 和 `LICENSE`。
研究 Skills、模板及 Lite/Full MCP 资源已经嵌入可执行文件，无需额外下载资源目录或克隆仓库。
模型 Host 和在线文献服务仍需单独配置。

请在 Release 的 **Assets** 中选择上述平台包。页面自动生成的 **Source code** 是需要编译的源码，
`.tgz` 是 npm 包，`.whl` 是 Python 包，`qiongli-next-…-plugin-…` 是 Host 插件包。
当前版本不提供 Intel Mac、Linux ARM 或 Windows ARM 原生构建。

### 1. 校验下载文件

下载同一 Release 的 [SHA256SUMS](https://github.com/jxpeng98/qiongli/releases/download/v2.0.0-beta.5/SHA256SUMS)。
在下载目录中，运行与你的平台对应的命令，将结果与 `SHA256SUMS` 中该文件名对应的摘要比较；一致后再继续。

```sh
# macOS
shasum -a 256 qiongli-2.0.0-beta.5-aarch64-apple-darwin.tar.gz
# Linux
sha256sum qiongli-2.0.0-beta.5-x86_64-unknown-linux-gnu.tar.gz
```

```powershell
# Windows
Get-FileHash .\qiongli-2.0.0-beta.5-x86_64-pc-windows-msvc.zip -Algorithm SHA256
```

### 2. 解压后直接运行

使用一个新目录，保留已有安装和研究文件。在 macOS 终端中：

```sh
mkdir qiongli-2.0.0-beta.5-macos-arm64
tar -xzf qiongli-2.0.0-beta.5-aarch64-apple-darwin.tar.gz -C qiongli-2.0.0-beta.5-macos-arm64
cd qiongli-2.0.0-beta.5-macos-arm64
./qiongli --version
./qiongli --help
./qiongli content list
```

在 Linux 终端中：

```sh
mkdir qiongli-2.0.0-beta.5-linux-x64
tar -xzf qiongli-2.0.0-beta.5-x86_64-unknown-linux-gnu.tar.gz -C qiongli-2.0.0-beta.5-linux-x64
cd qiongli-2.0.0-beta.5-linux-x64
./qiongli --version
./qiongli --help
./qiongli content list
```

Windows 用户在下载目录打开 PowerShell：

```powershell
Expand-Archive -Path .\qiongli-2.0.0-beta.5-x86_64-pc-windows-msvc.zip -DestinationPath .\qiongli-2.0.0-beta.5-windows-x64
Set-Location .\qiongli-2.0.0-beta.5-windows-x64
.\qiongli.exe --version
.\qiongli.exe --help
.\qiongli.exe content list
```

版本应显示 `qiongli 2.0.0-beta.5`。独立包只提供 `qiongli` 可执行文件；`ql` 别名由 npm / PyPI / Cargo 安装提供。

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


## 安装和升级随包 Plugin、Skills（beta.5） {#install-and-upgrade-bundled-content}

可以直接安装并注册随包 Plugin；目标和目录也可交给向导选择。以 Codex 为例：

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

beta.5 之后的开发构建会在安装 Plugin 时提供可选的上下文提醒。首次默认关闭，
更新时保留已有选择。运行 `qiongli install plugin --hooks context` 可加入提醒，
`--hooks off` 可移除 Plugin 内的提醒配置。确认页会显示具体命令，Host 信任和实际触发
仍需分别核对，详见 [Hook 安装与验证](/zh/advanced/agent-skill-collaboration#可选的上下文-hook)。
独立 Skills 导出不安装 Hook。

通过原安装渠道更新 CLI 后，刷新已登记的 Plugin：

```sh
qiongli install plugin --target codex
```

命令会从 Host 中发现源目录，无需重新填写。`--target all` 分别处理两个 Host，
每个新 Host 使用独立目录；此时不要指定 `--destination`。

`update plugin` 与 `upgrade plugin` 等效。重复运行 `install plugin` 也会核对并更新
已有的完整导出。更新 Host 缓存前会校验收据，只有完全匹配的旧插件才会通过官方命令
移除并重装，具体命令会列在确认页。其他来源的已启用穷理插件、被修改的文件、来源路径
冲突或意外的安装范围会阻止注册，需要你在 Host 中自行处理。

每次确认直接按回车都表示取消。取消 Host 注册或 Host 命令失败时，已导出的文件会保留；
解决问题后，对同一 Host 重新运行 `install plugin` 即可。成功提示表示官方安装状态
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

Codex Plugin 同时提供 `$qiongli` 总入口和
`$qiongli-paper-read`、`$qiongli-lit-review`、`$qiongli-academic-write` 等 workflow 入口。
这些入口共享同一套研究流程，底层技能卡与模板按需加载，无需手动创建 wrapper。
本地 CLI 导出和 Marketplace 打包都会自动生成它们；已经安装的旧 Plugin
需要更新来源或插件包，并通过 Codex 刷新后才能显示新增入口。

两个 Host 的 Plugin 也都包含独立的 `no-qiongli` Skill。可以在 Codex 中使用
`$no-qiongli`，或自然地说“NoQ问理，仅回复”。入口不调用工具，使用范围和安装说明见
[仅回复入口](../advanced/agent-skill-collaboration.md#reply-only)。

## npm / pip 安装 {#package-managers}

如果更习惯包管理器，可任选一个入口：

```sh
npm install --global qiongli@next
```

或者在 Python 虚拟环境中运行：

```sh
python -m pip install --upgrade "qiongli==2.0.0b5"
```

npm 需要 Node 18+，PyPI 需要 Python 3.9+；两者都提供 `qiongli` 和 `ql`。
安装后分别用 `--version` 核对版本。通过原包管理器升级包管理器安装的版本。

Cargo 从源码构建同一个 CLI，需要 Rust 1.97+ 和本机链接器。

```sh
cargo install qiongli --version 2.0.0-beta.5 --locked
```

安装后可使用 `qiongli` 和 `ql`。Cargo 没有 `next` 渠道，预发布版需指定完整版本号。
如果希望解压后立即使用，请选择上方的独立二进制包。

更多命令边界见[英文 CLI 指南](../../guide/cli-2x.md#which-surface-owns-which-command)。
旧版 `qiongli setup`、`check`、`project init` 等命令属于 1.x，不能直接套用到原生 2.x。

## 安装状态与版本一致性（beta.5） {#installation-state}

`qiongli install list` 和 `qiongli doctor` 读取官方 Host 库存，并复用安装时的本地
Plugin 收据校验。来源、缓存、版本或启用状态不能确认时，诊断会说明需要刷新或无法确认，
不再一律要求重新安装。只读库存命令有时间和输出限制，不会自动重试写入。

“已注册且缓存一致”只说明安装状态；新会话是否加载了 Skills 和 MCP 工具仍需实际检查。
CLI 与 Plugin 使用各自随包的程序，升级 CLI 后要刷新原 Plugin 目录。检查版本时同时看
`qiongli --version`、`qiongli content --json` 和 Host 的 Plugin 库存。

包构建绑定 CLI、内容版本和平台。npm / PyPI 的入口分别只携带各自的启动包装，
Cargo 携带可构建的 Rust 源码；共同的 CLI 描述来自同一份包元数据。
原生文件与资源摘要、发布收据用于追踪对应构建。版本号相同并不足以证明缓存字节一致。

## Research Graph：如何判断比以前更好 {#research-graph}

平时先看终端摘要，需要检查关系时再打开离线图：

```bash
qiongli project graph snapshot --project-id <prj_id> --text
qiongli project graph view --project-id <prj_id> > research-graph-new.html
```

请使用新的文件名，避免 shell 重定向覆盖已有文件。用浏览器打开 HTML 即可；
原生 CLI 不需要额外运行时，也不用启动本地服务。页面可以搜索研究记录、查看所选节点的
相邻关系、证据限制和来源检查。选择记录或关系后，页面会给出绑定当前项目版本和
projection ID 的 `qiongli project graph source` 命令，用来读取对应记录的片段。
再沿记录中的文件、页码等位置检查原始材料。来源改变后，需要刷新项目并重新导出。

这个页面是快照。点击 **Save snapshot JSON** 可以保存同一份投影，作为阶段记录。
保留此前的快照和原始材料；导出不会替你归档或删除它们。HTML 和 JSON 都包含研究内容，
只应分享给有权查看这些记录的人。

审稿和选刊决定可以通过决策日志中的可选 `Related Claims` 列关联已有论点 ID，
关系为 `informs`。只有 `locked` 决定生成已审阅关系，暂定、受阻或待重新考虑的决定
仍为提议状态。这些关系不算支持证据。报告位置、理由、适用的期刊要求和稿件影响
继续保留在原来的决策记录中。

1.x 的 citation graph 用于从文献种子扩展引用与参考文献，并去重候选文献。
2.x 的本地 Research Graph 增加了研究记录之间的联系：论点使用稳定标识，
证据关联来源和具体位置，写作记录与文献记录可以沿同一来源继续追踪。
这两种图的用途不同；不能用增加的节点数量声称文献检索效果或运行速度已经提高。

| 检查目标 | 通过标准 |
|---|---|
| 一个论点使用多项证据 | 保留一个 claim ID 和多条独立支持记录，不丢失来源或位置 |
| 追踪来源 | 每条支持边可回到当前记录；CSV 行顺序改变后仍能定位 |
| 重复导入与重建 | 去重后节点和边一致，不产生重复论点 |
| 避免虚假支持 | 缺少位置、待补证据、身份冲突和越界路径不能生成已确认支持 |
| 跨阶段继续 | 沿用论点 ID、citekey、前序总结和来源记录；总结不替代原始证据 |
| 防止读取旧状态 | CLI/MCP 共用项目 revision，过期读取和未授权修改被拒绝 |

这些标准由现有 Graph、项目服务和 Full MCP 测试检查。原始笔记或 PDF 仍需要 Host
在授权范围内读取、提出规范记录、供你审阅并保存，再重建 Graph。可用
`qiongli project graph --help` 查看当前操作；工具缺失时，不能通过直接改文件绕过审批。

结构测试证明记录和来源关系正确，不能代替研究语义判断。下一步比较同一份材料在新 Host
会话中的提取保真度、未解决证据和操作步骤；没有相同任务的前后测量，就不宣称速度提升。
