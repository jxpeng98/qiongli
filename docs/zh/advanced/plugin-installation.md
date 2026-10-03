# Plugin 配置

首次安装运行 `qiongli install`，按[快速开始](../quickstart.md)操作。本页说明可选参数与自定义导出。

## 选择 Host

```sh
qiongli install plugin --target codex
qiongli install plugin --target claude
qiongli install plugin --target deepseek
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

不安装全局 CLI 也可以接入：在 DeepSeek Desktop 的 **Add plugin → Official npm registry** 中填 `qiongli@2.2.0`，或运行：

```sh
dsh plugin --profile desktop add qiongli@2.2.0
```

把 `desktop` 换成自己的 profile。包内含 22 个 Skill 入口、Full MCP 和对应平台程序。每个 profile 保留一份穷理，重新加载后检查实际工具。

向导优先使用已存在的 Desktop profile，否则使用 `web`。新 CLI profile 由官方 `web` 模板初始化；Desktop 的保留 profile 由 Desktop 自行初始化。DSH 使用自己的包管理器和 profile；`--destination`、上下文 `--hooks` 需要另选 Codex/Claude。

更新时通过向导或管理器指定版本。Desktop 对话框按其说明移除后重新添加，保留 profile 与模型设置。开发者导出和外部任务见[外部 Agent 协作](../../advanced/external-host-coordination.md)。

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

审阅后通过底层命令应用。目标末级目录须为 `qiongli` 或 `qiongli-next`，父目录须已存在且安全，选用 Host 缓存与 `.qiongli` 状态目录以外的位置：

```sh
qiongli app plan plugin-source-install --target codex \
  --destination /absolute/qiongli > plugin-plan.json
qiongli app apply --plan plugin-plan.json \
  --expected-plan-digest <plan_digest_sha256> --approve-filesystem-write
qiongli app plugin-source-status --target codex --destination /absolute/qiongli
```

使用新的计划文件名，避免覆盖旧计划。应用前核对路径、程序哈希、收据与摘要。导出后用官方 Host 管理器注册；稳定版选择器为 `qiongli@qiongli-cli-local`，预发布为 `qiongli-next@qiongli-cli-local`。

更新用 `plugin-source-update`。移除前先在 Host 注销，再用 `plugin-source-remove`。文件被修改、出现未知文件或符号链接、收据过期时会拒绝替换或移除；管道输入不能授权。注册或刷新后，新开 Host 会话检查实际工具。
