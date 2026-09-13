# 数据所有权与生命周期

本政策适用于当前原生 Qiongli 2 产品。Qiongli 的项目与产品状态保存在本地；Agent
Host 与远程 Provider 各自保存并管理其记录。

## 所有权边界

| 数据 | 所有者与位置 |
| --- | --- |
| 项目文件与 Qiongli 私有状态 | 用户拥有完整项目目录，包括 `<project>/.qiongli/v2`。 |
| Qiongli 2 全局状态 | 用户拥有实际生效的 v2 根目录：默认是 `<user-home>/.config/qiongli/v2`，配置后是 `$QIONGLI_CONFIG_HOME/v2`。 |
| Provider 凭据 | 用户拥有凭据。受支持的系统中，Qiongli 把密钥存入操作系统凭据存储，并只在原生配置中保存不透明引用；没有受支持的存储时，Qiongli 会安全失败，不把密钥写入普通配置。 |
| Plugins、Skills、CLI 文件和客户端入口 | 相应 Agent Host 或 Qiongli 安装 receipt 拥有已安装的集成状态。 |
| Agent Host 聊天与 transcript | 这些记录由 Agent Host 管理，不属于 Qiongli 项目数据。 |
| 原生 App All Chat 历史（开发预览） | 用户拥有 `<project>/.qiongli/all-chat/run_*.json`；私有日志与研究产物、已验收证据分别管理。 |
| 远程 Provider 记录 | Provider 决定其服务中的保留与删除行为。 |

## 备份与恢复

要生成完整、可恢复的检查点：

1. 停止可能写入 Qiongli 状态的 App、CLI、MCP server 和 agent。
2. 备份每个完整项目目录，包括隐藏文件及整个 `<project>/.qiongli`，其中包含 `<project>/.qiongli/v2` 和 All Chat 历史。
3. 备份完整的 Qiongli 2 全局根目录：`<user-home>/.config/qiongli/v2` 或
   `$QIONGLI_CONFIG_HOME/v2`。
4. 使用操作系统的安全凭据导出或恢复流程单独备份 Provider 凭据。
5. 让项目与全局副本来自同一个停止写入的检查点，并验证备份可读。

在 Qiongli 停止时恢复项目目录和全局根目录。启动后，通过正常项目流程重新注册
Research Library 中缺失的项目，并允许可重建的派生索引重新生成。Qiongli 当前没有
一条命令完成全产品备份、恢复或清除。

### App 私有会话历史

原生开发版 App 在 `<project>/.qiongli/all-chat/run_*.json` 的 version 1 日志中保存
明确提交的消息、上下文和来源标签，以及有界的公开 Agent 活动、权限结果和生命周期
事件。用户输入本身可能含有私密信息，应按研究数据保护。日志不主动采集凭据、隐藏
推理或原始工具载荷。Agent Host 的历史仍由 Host 管理；浏览器 fixture 仅在内存中
预览，不提供重启恢复。

重启后显示最近的会话，未结束的工作标记为中断，不会自动重放消息、权限选择或项目
写入；当前不支持 load/resume。新会话保留旧文件，App 暂未提供历史归档浏览。
保留规则为不自动过期、不自动淘汰；每个项目最多 **32 个会话**，每份日志最多
**64 轮**、**2,048 条公开更新**、**2,304 条日志记录**及 **8 MiB**。
达到上限后停止写入，不删除旧记录。损坏、截断或不支持的版本会阻止恢复和新建会话，
并保留原始字节；检查前请停止 App 并验证完整备份。

存储复用私有目录和文件权限、原子替换及摘要 CAS。
`.qiongli/.all-chat-session.lock` 持有操作系统写入租约，`.qiongli/.all-chat.lock`
串行化文件操作。残留锁文件不等于活动会话，不要删除锁来绕过仍运行的写入者。
备份与恢复时应停止所有写入者。主动删除时，仅在停止 App 且备份完成后删除明确选择的
`run_*.json`；重启后会显示剩余日志中最近的有效会话。卸载不会清除这些文件。
会话内容不进入 portable 项目导出或产品诊断，私有快照和 Debug 格式也不输出会话内容。

## Portable 项目导出

需要在机器之间移动经过隐私过滤的项目快照时使用 portable export：

```bash
qiongli project export preview --project-id PROJECT_ID --destination DESTINATION
qiongli project export apply --project-id PROJECT_ID --destination DESTINATION \
  --expected-plan-digest DIGEST --approve-filesystem-write
```

结果包含 `qiongli-portable-project.json` 和 `project/` 目录。它**不是完整备份**：会排除
Qiongli 私有状态、绝对路径、客户端配置、凭据、session、聊天、conversation、
transcript、Git 元数据、依赖/构建/缓存目录、`.env` 文件，以及可识别的密钥或私钥
文件。需要恢复能力而不是交换快照时，应使用上面的完整检查点。

## 卸载与删除

- App 的 **Remove selected** 只删除选中的 Qiongli 所有客户端集成状态。
- App 的 **Remove CLI** 只删除 receipt 拥有的 CLI 文件，或恢复 receipt 记录的准确前任。
- Agent Host marketplace manager 删除其拥有的 plugin 或 Skill 状态。
- 旧版 `qiongli remove` 只删除选中的 CLI 管理资产。
- 注销项目只移除 Research Library 注册，不删除项目目录。

这些操作不会删除项目目录、Qiongli 2 全局数据根目录、Agent Host 聊天、操作系统凭据
或远程 Provider 记录。保留数据和卸载软件是两个独立决定。

完成并验证备份后，主动删除应只针对用户明确选择清除的项目目录、实际全局 v2 根目录、
安全凭据、Host 记录和 Provider 记录，并使用相应 Host、操作系统与 Provider 的删除控制。
不要使用宽泛的递归清理命令。迁移或回滚仍需要时，应保留必要的 1.x 源码与状态。

## 1.x 支持终止

`v1.19.0-beta.1` 是已验收的最后一个包含功能变更的 1.x 版本。计划中的 1.x 支持窗口
在 **Qiongli 2 Stable 实际发布后 90 天**结束。Alpha、Beta、本政策和普通源码合并都不
启动该计时，因此目前还没有固定的日历终止日期。

窗口期间，1.x 只接受获批的关键安全或发布阻断修复，不恢复常规功能开发。独立的
REL-906 迁移与回滚手册将说明 1.x 与 2.x 之间的操作迁移。支持终止不会自动删除用户
数据。维护权限以[发布分支政策](/zh/maintainer/release-branch-policy)为准。

### 研究片段比较预览

开发版片段比较把原生资料清单（所选正文、行范围、来源与方法摘要）记录在既有私有
消息上下文中；固定演示候选作为公开 Agent 活动显示。这些内容属于研究资料，遵循
上述会话保留与删除规则。离线 fixture 不向模型发送来源。App 重启后可以阅读历史，
但不会恢复未保存候选的提交权限或资料授权；再次提交前需新建比较并读取当前来源。
Capture 接收与后续学术归并复用既有项目存储。删除聊天日志不会删除已保存的项目记录。
