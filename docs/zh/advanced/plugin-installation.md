# Plugin 配置

首次安装运行 `qiongli install`，按[快速开始](../quickstart.md)操作。本页说明可选参数与自定义导出。

## 选择 Host

```sh
qiongli install plugin --target codex
qiongli install plugin --target claude
qiongli install plugin --target deepseek
qiongli install plugin --target antigravity
qiongli install plugin --target codex,deepseek
```

不填 `--target` 时显示菜单，`--target all` 逐个处理。取消或失败会停止后续步骤，保留已完成的安装；每个 Host 单独批准。Codex 与 Claude 使用各自的源目录，单个 Host 可用 `--destination` 指定父目录已存在的路径。

文件导出与官方 Host 注册分别确认。取消注册会保留导出文件，不改变 Host 设置；处理失败原因后对同一 Host 重试。保留旧源目录与缓存便于恢复。已知 Codex 冲突可确认迁移，Claude 冲突需手动停用。

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

不安装全局 CLI 也可以接入：在 DeepSeek Desktop 的 **Add plugin → Official npm registry** 中填 `qiongli@2.4.0`，或运行：

```sh
dsh plugin --profile desktop add qiongli@2.4.0
```

把 `desktop` 换成自己的 profile。包内含 22 个 Skill 入口、Full MCP 和对应平台程序。每个 profile 保留一份穷理，重新加载后检查实际工具。

向导优先使用已存在的 Desktop profile，否则使用 `web`。新 CLI profile 由官方 `web` 模板初始化；Desktop 的保留 profile 由 Desktop 自行初始化。DSH 使用自己的包管理器和 profile；`--destination`、上下文 `--hooks` 需要另选 Codex/Claude。

更新时通过向导或管理器指定版本。Desktop 对话框按其说明移除后重新添加，保留 profile 与模型设置。开发者导出和外部任务见[外部 Agent 协作](../../advanced/external-host-coordination.md)。

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
或 `qiongli-antigravity`，更新时复用同一路径。包内含 22 个 Skill 入口、原生程序
和 Full MCP，采用根目录 `plugin.json`、`mcp_config.json`。

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
