# Plugin 配置

首次安装运行 `qiongli install`，按[快速开始](../quickstart.md)操作。本页说明可选参数与自定义导出。

## 选择 Host

```sh
qiongli install all
qiongli install plugin --target codex
qiongli install plugin --target claude
qiongli install plugin --target deepseek
qiongli install plugin --target antigravity
qiongli install plugin --target pi
qiongli install plugin --target codex,deepseek
```

**Qiongli 2.5.0：** 使用 `qiongli install all`，或在主安装菜单选
**5（全部安装）**、Plugin 的 Host 菜单选 **6 / all**，即可依次为所有检测到的
受支持客户端 CLI（`codex`、`claude`、`dsh`、`agy`、`pi`）安装 Plugin。每个 Plugin
已包含 Skills 和 Full MCP，无需分别安装。向导会列出选中的客户端和因缺少 CLI
而跳过的客户端；一个也没找到时停止，不执行安装。此命令不安装客户端程序本身。
`upgrade all`、`update all` 更新同一组检测到的客户端；可用
`--language auto|zh|en` 统一选择本轮描述语言。

**开发分支（2.5.0 之后的更新）：** 不填 `--target` 的 Plugin 命令默认自动检测，
主安装菜单直接回车也选择全部检测到的客户端。显式指定目录或 Hook 而未指定
Host 时仍显示 Host 菜单；`--target all` 使用同样的自动选择。
检测覆盖 PATH 和已有的常见 CLI 安装目录；只有桌面应用或配置目录不算可用 CLI。
显式指定 `--target codex,claude` 等列表时，仍要求这些客户端均可用。
每个 Host 保留各自的文件变更与注册批准流程；单个客户端失败或明确拒绝后继续处理
其余客户端，输入关闭／不完整或输出失败才停止整批。已完成的安装保留。源目录自动选择，无需逐个输入：

- Codex / Claude 优先复用已登记目录，或该 Host 已验证的旧 `~/qiongli` 导出。
  首次安装分别使用 `~/qiongli-codex`、`~/qiongli-claude`；预发布使用 `qiongli-next-*`。
- AGY 从已验证的插件缓存恢复原源目录；首次安装用 `~/qiongli-antigravity`。
- Pi 复用已登记的本地包路径；首次安装用 `~/qiongli-pi`。
- DeepSeek 保留原有 profile 选择与官方包管理流程。

全部检测模式保留已有 Hook 设置，新安装默认关闭；语言和文件／注册批准仍需确认。
默认目录含未知文件、内容被修改或路径不安全时拒绝该项，不覆盖或另建编号目录。

单个 Host 可用 `--destination` 指定父目录已存在的路径。Codex / Claude 支持原来的
`qiongli`、`qiongli-next` 末级目录，也支持对应 Host 的新默认目录名。全部安装不接受
`--destination`、`--hooks` 或 `--dry-run`；这些选项需显式选择兼容的 Host。

文件导出与官方 Host 注册分别确认。取消注册会保留导出文件，不改变 Host 设置；处理失败原因后对同一 Host 重试。保留旧源目录与缓存便于恢复。已知 Codex 冲突可确认迁移，Claude 冲突需手动停用。

## 安装进度与失败处理

开发分支显示 `[1/5]` 客户端进度、官方管理器的命令步骤，以及命令运行期间每五秒
更新一次的等待时间。最后分别列出安装成功、失败、跳过和未执行的客户端，并给出
处理提示和定向重试命令。只要有失败，整批就返回非零状态，不会误报全部成功。
回答 `n` 只跳过当前客户端，后续客户端仍分别确认；关闭输入或中断命令可以停止
整批。文件已导出但注册失败／被拒绝时，导出文件保留。

按最终汇总中的命令只重试未完成的客户端，例如：

```sh
qiongli install plugin --target deepseek
```

`host-command-nonzero-exit` 只表示官方管理器失败，不能单凭它判断是网络、版本
还是权限问题。新提示会显示退出码、可识别的错误类别和实际执行命令；DSH 命令
包含当时选择的 profile、确切 npm 版本、registry 和参数。先按提示处理问题并检查
保留文件，重试时选择同一 profile。错误无法识别时，可直接执行显示的 DSH 命令
查看完整诊断。Qiongli 不回显或保存可能包含凭据、配置的原始输出；直接运行后
也不要把其中的凭据贴到公开日志中。

超时不会撤销已有修改，包管理器的子进程也可能仍在运行，重试前应检查其状态。
未知文件、收据变化或 profile 冲突仍会拒绝该客户端安装，继续下一项不会绕过检查。

## 描述语言 {#language}

```sh
qiongli install plugin --target codex --language zh
qiongli install plugin --target claude --language en
```

Auto 跟随环境和系统语言，其他语言回退到英文。更新默认保留已保存的选择。只改变描述和支持的显示信息，入口名与工作流正文不变；切换后刷新 Host 或新开会话。

## 可选上下文 Hook

首次安装默认关闭，更新保留已有选择。

```sh
qiongli install plugin --target codex --hooks context
qiongli install plugin --target codex --hooks off
```

预览会列出命令，Host 信任与实际事件送达仍需单独检查，见 [Hook 配置与验证](agent-skill-collaboration.md#optional-context-hooks)。

## DeepSeek Harness {#deepseek}

`qiongli install plugin --target deepseek` 选择 profile，预览官方 DSH 命令，确认信任后安装与 CLI 匹配的 npm 版本，并核对注册与内容收据。模型设置保留。

不安装全局 CLI 也可以接入：在 DeepSeek Desktop 的 **Add plugin → Official npm registry** 中填 `qiongli@2.5.0`，或运行：

```sh
dsh plugin --profile desktop add qiongli@2.5.0
```

把 `desktop` 换成自己的 profile。包内含 22 个 Skill 入口、Full MCP 和对应平台程序。每个 profile 保留一份穷理，重新加载后检查实际工具。

向导优先使用已存在的 Desktop profile，否则使用 `web`。新 CLI profile 由官方 `web` 模板初始化；Desktop 的保留 profile 由 Desktop 自行初始化。DSH 使用自己的包管理器和 profile；`--destination`、上下文 `--hooks` 需要另选 Codex/Claude。

更新时通过向导或管理器指定版本。Desktop 对话框按其说明移除后重新添加，保留 profile 与模型设置。开发者导出和外部任务见[外部 Agent 协作](../../advanced/external-host-coordination.md)。

## Pi coding agent {#pi}

**Qiongli 2.5.0：** 新增实验性 Pi 适配，要求官方 `pi` CLI
0.99.0 或更新版本，并启用内置 MCP。PATH 需能找到官方 Pi 的实际可执行文件；
此适配暂不支持 Pi 的 mise shim。在 Host 菜单选择 **7**，或执行：

```sh
qiongli install plugin --target pi --language zh
qiongli update plugin --target pi
```

`qiongli install all` 也会检测 Pi。包中只展示 `/skill:qiongli` 和
`/skill:no-qiongli` 两个入口，科研子流程保留在内部资源库。随包扩展使用 Pi
内置接口注册原生 Full MCP，按需发现工具，无需另装第三方 MCP 桥接器。

默认源目录为 `~/qiongli-pi`。单独选择 Pi 时，可用 `--destination` 指定以
`qiongli`、`qiongli-next` 或 `qiongli-pi` 结尾的安全绝对路径。更新会复用 Pi
用户设置中已校验的源目录，也能识别相对路径声明。安装器遵循
`PI_CODING_AGENT_DIR`（默认 `~/.pi/agent`），先确认文件变更；需要注册时，再
确认官方 `pi install <source>` 命令。已有模型设置与其他包保留。Unix 上新建的
配置文件采用私有权限；已有链接或组/其他用户可写的配置会被拒绝，安装器不会
修改其权限。若包资源被
筛选、内置 MCP 被禁用，或存在重复 Qiongli 包、独立 Qiongli MCP 配置，先报告
冲突，由用户检查，不会自动覆盖这些选择。

Pi 直接加载源目录，请保留该目录。开启新会话或运行 `/reload`，通过 `/mcp`
查看连接，再让 Pi 发现并调用 `qiongli_config_status`。**终端的 `pi mcp list`
不会加载包扩展。** 项目设置、资源禁用或替代 MCP 扩展仍可能影响会话中的可用性；
注册成功不等于模型会话已验证。本适配暂不提供 Pi 上下文 Hook、签名 Host 集成
或 App `--dry-run`。卸载先执行 `pi remove <source>`，再处理源目录；移除注册
会保留源文件，便于恢复。

接口依据 Pi 官方 [Packages 文档](https://pi.dev/docs/latest/packages)与
[MCP 文档](https://pi.dev/docs/latest/mcp)。

## Antigravity CLI {#antigravity}

2.4.0 新增实验性 Antigravity CLI 适配；2.3.0 程序尚未包含此入口。Codex 仍是
主要科研 Host。AGY 1.3.0 的限定观察覆盖安装、正常结束的状态调用，以及提供
公开工具描述后的正文回读。自动发现、完整科研会话恢复和原始 MCP 响应捕获仍
未通过验证。实验性标记保留这些失败供后续改进，不代表科研流程或 IDE 支持通过。

```sh
qiongli install plugin --target antigravity --language zh
# agy 是同一个目标的别名
qiongli update plugin --target agy
```

先安装 `agy` 1.2.17 或更新版本。向导分别确认源文件导出与官方 `agy plugin
install`、`enable` 命令。默认源目录为 `~/qiongli-antigravity`；单独选择此 Host
时可用 `--destination` 指定安全绝对路径，末级目录为 `qiongli`、`qiongli-next`
或 `qiongli-antigravity`。开发分支更新时可省略目录，从已验证的缓存复用原路径。
包内含原生程序和 Full MCP，采用
根目录 `plugin.json`、`mcp_config.json`。

**Qiongli 2.5.0：** AGY 只公开 `qiongli` 这一个科研入口，并保留独立的
`no-qiongli` 仅回复入口。阅读、文献综述、写作等工作流作为内部资源保留，由主入口
根据请求选用；仍可在提示中指明工作流名称。2.4.0 导出仍有 22 个公开 Skill。
新版 CLI 可用后，先升级 CLI，再执行上面的 Plugin 更新命令，并重新开启 AGY 会话。
更新会经收据核验和官方管理器替换旧副入口。不要手动删除缓存中的 Skill；有修改
或未知文件时，收据核验会停止并要求单独处理。

**安装后保留源目录。** MCP 通过其中程序的绝对路径启动，不依赖未经验证的路径
变量展开。实测 AGY 1.2.17 由官方管理器复制到 `~/.gemini/config/plugins/<name>`；
安装器逐文件核对缓存收据，并单独检查注册和启用状态。缓存有未知或被修改的文件、
另一个穷理 Plugin 已启用，或存在独立配置的穷理 MCP 时会停止并提示处理。
安装器不选择模型，也不迁移登录凭据。

完成 AGY 的工作区信任审阅后，新开会话，用 `/mcp` 确认穷理服务及工具已加载。
调用 `qiongli` Skill，再要求调用 `qiongli_config_status` 并列出
穷理工具。安装成功与会话实际使用分别验证；`agy plugin list` 可查看 Plugin，
但导入的 MCP 不一定出现在独立的 `agy mcp list` 中。请在交互会话中审阅并批准
所需的 MCP 调用；实测无交互 `--print` 会拒绝尚未授权的调用，即使退出码为 0，
仍应检查实际工具结果和 `denied_actions`。移除时先运行
`agy plugin uninstall qiongli`（预发布用 `qiongli-next`），再处理源目录。
当前不提供此 Host 的上下文 Hook、签名集成、`app plan` 或 `--dry-run`。

格式依据官方 [Plugin 文档](https://www.antigravity.google/docs/plugins?tab=cli)和
[MCP 文档](https://antigravity.google/docs/mcp)。IDE 与其他操作系统需另行验证。

## 独立 Skills

```sh
qiongli install skills
qiongli upgrade skills --preset current-project --profile full
```

默认导出 full 内容到 `$HOME/.qiongli-skills`，`current-project` 使用当前目录的 `.qiongli-skills`。已有 profile 保留。这些是指导文件，不自动注册 Host、连接 MCP 或安装 Hook。

## 脚本导出 Plugin {#scripted-export}

Codex/Claude 可先查看只读文件计划：

```sh
qiongli install plugin --target codex --destination /absolute/qiongli --dry-run --json
```

审阅后通过底层命令应用。目标末级目录须为 `qiongli` 或 `qiongli-next`，父目录须已存在且安全，选用 Host 缓存与 `.qiongli` 状态目录以外的位置。在 Unix 上，所有上层目录都不能允许同组用户或其他用户写入。因此，即使直接父目录是私有目录，`/tmp` 或共享工作区的上层目录仍可能触发 `insecure-materialization-parent`。请选择上层目录也满足要求的私有导出位置，再重新预览：

```sh
qiongli app plan plugin-source-install --target codex \
  --destination /absolute/qiongli > plugin-plan.json
qiongli app apply --plan plugin-plan.json \
  --expected-plan-digest <plan_digest_sha256> --approve-filesystem-write
qiongli app plugin-source-status --target codex --destination /absolute/qiongli
```

使用新的计划文件名，避免覆盖旧计划。应用前核对路径、程序哈希、收据与摘要。导出后用官方 Host 管理器注册；稳定版选择器为 `qiongli@qiongli-cli-local`，预发布为 `qiongli-next@qiongli-cli-local`。

更新用 `plugin-source-update`。移除前先在 Host 注销，再用 `plugin-source-remove`。文件被修改、出现未知文件或符号链接、收据过期时会拒绝替换或移除；管道输入不能授权。注册或刷新后，新开 Host 会话检查实际工具。
