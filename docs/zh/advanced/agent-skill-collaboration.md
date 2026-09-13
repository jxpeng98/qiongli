# Agent 协作与可选 Hook

Qiongli 2 让模型在你选择的 Host 中运行。穷理提供研究规范、可核对来源的
交接材料和项目工具。CLI 导出的 Plugin 包含 Skills 与 Full MCP，但安装 Plugin 本身不会
创建其他 Agent，也不会更换你选择的模型。

## 仅回复，不执行操作 {#reply-only}

Qiongli 2.0 包含独立的 **`no-qiongli`** Skill。在 Codex 中可以使用
`$no-qiongli`；其他 Host 可从已安装的 Skills 中选择 `no-qiongli`。也可以自然地说
**“NoQ问理”**、**“仅回复”**、**“不处理”** 或 **“no 处理”**，例如：

> 接下来仅回复：请解释下面这段文字，不调用工具，不读取或保存文件。

这个入口直接回答，不先加载穷理主 Skill 或研究工作流。Codex 和 Claude Plugin 都将
它放在 `skills/no-qiongli/SKILL.md`，与研究入口并列；更新 Plugin 并重新加载 Skills
后才能发现新入口。独立 Skills 导出包含 `workflow/no-qiongli/SKILL.md`，可以把这个
自包含目录作为一个 Skill 交给 Host 安装。导出本身不注册 Host 或连接 MCP，CLI 中
也没有 `qiongli no-qiongli` 子命令。

模型只根据对话中已经可见的内容回答，不读取额外的 Skill 资源，不检索、不调用
MCP 或其他代理，也不检查项目、更新 Graph。材料不足时，说明缺口或请你贴出相关
内容。这个选择持续到你明确要求恢复执行；如果只想限制一次，可以说“本次仅回复”。
使用工作流快捷入口时也遵循这个选择。普通请求沿用原有路由，引文里的“no”不会切换模式。

这是一条 Skills 行为约束，不是 CLI 参数或 Host 的工具权限锁。它不会改变更高优先级
的 Host 规则、取消已经运行的任务，也不会关闭 Host 自动触发的 Hook。如需移除这份
Plugin 的上下文 Hook，请另外运行 `qiongli install plugin --hooks off`，并完成安装确认。

## 按任务选择协作方式

| 任务需要 | 建议方式 | 怎样判断完成 |
|---|---|---|
| 独立的方法或证据判断 | 让新的原生子代理阅读限定来源，先独立给出意见 | 收到真实审查结果、执行身份和阅读范围 |
| 写作后复核 | 先写作，再审查这一份具体候选稿 | 候选稿身份、审查意见和主代理的处理记录一致 |
| 可拆开的工作 | 子代理分别负责不同产物或文件 | 收齐结果，再检查整合后的内容 |
| 请另一个 Host 审查或修改 | 交接任务包，收回审查报告或修改提案 | 任务、来源版本和候选稿对应；分歧有明确记录 |

可以直接说：“请用一个独立 Agent 检查这段结果是否符合研究设计，先返回意见，
不要直接改正文。”普通小修改仍可由当前会话完成，不必固定安排几个 Agent 或几轮讨论。

`model-collaborator` 会使用当前 Host 实际提供的代理工具。主代理说明目标、允许
阅读的材料、文件分工和返回要求，再收集真实结果。角色名称或已排队任务不能证明
独立审查完成。没有可用审查者时，这项要求保持未完成；同一会话切换角色只能算自查。

是否能调用子代理，取决于运行中的 Host、配置和模型接入能力。沿用用户的模型设置。
平台能力可参考 [Codex 子代理文档](https://learn.chatgpt.com/docs/agent-configuration/subagents)
和 [Claude Code 子代理文档](https://code.claude.com/docs/en/sub-agents)。

## 与另一个 Host 协作

使用随包的 `templates/agent-handoff.md`，交接目标、允许共享的材料、实际观察到的
来源哈希或修订号、候选稿身份和允许操作。对方用 `templates/agent-review-packet.md`
返回意见，或提交候选修改。已有并获授权的通信工具可以传递任务包；也可以手动转交。
仅准备好材料时，应记录为“等待外部审查”。

收到结果后，主代理先核对它是否针对当前来源和候选稿，再根据证据处理分歧、预览修改。
过期审查需要重新核对，必要时重审。正式研究文件由一个协调者整合；其他代理使用
各自的候选文件或工作树，避免覆盖彼此的工作。交接要保留 claim/decision ID、citekey、
来源位置、方法限制及前一份阶段总结。审查意见不能直接成为 Graph 的原始支持证据。

不要转交私人聊天记录、凭据或审批令牌。当前 Full MCP 运行绑定启动它的 Host 和
已认证读取，任务包不能接管这个检查点或继承审批权限。自动领取跨 Host 任务、并发
修改正式文件及崩溃恢复仍按 ADR 0218 后续实现；目前可以先进行审查和修改提案的交接。

## 可选的上下文 Hook {#optional-context-hooks}

Hook 适合在压缩上下文、恢复会话或启动子代理时，提醒模型重新核对研究状态。
`qiongli hooks context`：从标准输入接收事件 JSON，返回简短提示。
它是原生命令，不依赖 Python、Node、MCP 连接或额外模型调用，不读取项目文件和聊天
记录，也不保存总结、批准写入或强迫已经结束的任务继续运行。不启用 Hook 也可正常使用 Skills。

运行 `qiongli install` 并选择 Plugin，
就可以选择是否加入上下文提醒。首次安装默认关闭，重新安装和升级时保留已有选择。
也可以直接指定：

```sh
qiongli install plugin --hooks context
qiongli install plugin --hooks off
```

确认文件修改前，CLI 会显示 Hook 的事件和命令。配置保存在 Plugin 清单中，调用同包
的原生程序，不需要另外安装脚本运行环境或启动 MCP。每个 Host 分别选择、分别安装。
`--hooks off` 只移除这份 Plugin 中的提醒配置，保留手动添加的 Host 配置。
如果之前已手动配置同一提醒，请在 Host 中检查重复条目，避免收到两次提醒。

注册完成后，Codex 仍需在 `/hooks` 或 Hook 设置中审阅并信任命令；定义变化后可能
需要重新信任。Claude Code 需要 2.1.139 或更新版本以支持 command Hook 的 `args`
字段，安装向导会在写入前检查这一点。可以在 `/hooks` 中核对 Plugin 条目。重新加载 Plugin 或开启新会话后，再通过恢复会话、
压缩上下文或启动子代理确认提醒确实送达。文件导出和注册检查不代表 Hook 已经触发。
查看导出状态可运行 `qiongli app plugin-source-status --target codex --destination /absolute/path/qiongli-next`：
`source.context_hooks` 表示经过收据校验的配置选择，`host_state` 仍为 `not-verified`。
Claude Code 改用 `--target claude`。

如果希望手动配置 Host，可以使用下面的示例。把路径替换成实际原生二进制的绝对路径，保留命令引号。Windows 使用 `qiongli.exe` 的绝对路径，并按 Host 的命令 shell
和 JSON 规则处理引号及转义。

```json
{
  "hooks": {
    "SessionStart": [
      {
        "matcher": "resume|compact",
        "hooks": [
          {
            "type": "command",
            "command": "\"/absolute/path/qiongli\" hooks context",
            "timeout": 5
          }
        ]
      }
    ],
    "SubagentStart": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "\"/absolute/path/qiongli\" hooks context",
            "timeout": 5
          }
        ]
      }
    ]
  }
}
```

Codex 将这些条目合入相应的 `.codex/hooks.json`，然后在 `/hooks` 中审阅并信任；
项目配置还需要项目受信任。Claude Code 将 `hooks` 条目合入相应的
`.claude/settings.json`。保留已有条目，避免在用户级和项目级重复添加同一提示。
具体配置和事件支持以
[Codex Hook 文档](https://learn.chatgpt.com/docs/hooks)和
[Claude Code Hook 文档](https://code.claude.com/docs/en/hooks)为准。

先运行 `qiongli hooks context --help`，旧的已发布 CLI 可能还没有这个命令。
本地检查时，向命令传入 `source: "resume"` 的 SessionStart JSON 并关闭标准输入，
应收到 `hookSpecificOutput.additionalContext`。不支持的事件返回 `{}`；无效或超过
64 KiB 的输入以退出码 1 结束，不回显输入内容。返回提示不代表审查通过或研究已保存。

这里只使用 command Hook，不能假定各 Host 的 prompt、agent、MCP Hook 行为一致。
还需在实际 Host 中确认提示是否送达；协议测试不能证明真实安装已通过验证。
[历史实测矩阵](/zh/guide/agent-host-capability-matrix)继续保留原有证据范围。
