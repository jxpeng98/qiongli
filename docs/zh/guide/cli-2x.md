# CLI 命令

面向穷理 **2.5.1**。初次使用见[安装](install.md)或[快速开始](../quickstart.md)。

## 常用命令

| 你想做什么 | 命令 |
|---|---|
| 查看版本 | `qiongli --version` |
| 查看常用命令 | `qiongli` |
| 检查本机状态 | `qiongli doctor` |
| 接入或刷新 Plugin | `qiongli install plugin` |
| 查看 Host 注册 | `qiongli install list` |
| 检查重复 CLI | `qiongli setup` |
| 列出研究项目 | `qiongli project` |
| 查看一个项目 | `qiongli project show <id>` |
| 查看配置 | `qiongli config` |
| 查看内嵌研究内容 | `qiongli content` |
| 检查本地 MCP | `qiongli mcp check` |

包管理器安装同时提供 `ql`，命令相同；直接下载提供 `qiongli`。

用 `qiongli help project create` 或 `qiongli project create --help` 查看单项操作，`qiongli help all` 查看完整参考。加 `--json` 输出结构化数据，`--text` 输出可读报告：

```sh
qiongli status --json
qiongli content --json
qiongli doctor --text
```

## 安装与升级

### 直接下载 {#standalone-binary-download}

[安装页](install.md#standalone-binary-download)列出平台下载、校验和解压步骤。独立程序不需要额外语言运行时。

### 包管理器 {#package-managers}

任选原安装渠道，前提见[安装页](install.md#package-managers)：

```sh
npm install --global qiongli@latest
python -m pip install --upgrade "qiongli==2.5.1"
cargo install qiongli --version 2.5.1 --locked
```

### 检查和迁移已有 CLI {#review-existing-cli-installations}

`qiongli setup` 列出可见安装，提供手动归档或卸载建议，不移动文件、删除数据或修改 PATH。旧版排在 PATH 前面时，用新版程序的绝对路径运行。`qiongli install inventory --paths exact` 可显示完整路径。

### 安装和更新随包内容 {#install-and-upgrade-bundled-content}

运行 `qiongli install plugin`，分别确认文件和 Host 计划。更新复用已核验的源目录，`upgrade plugin`、`update plugin` 是同一流程。Plugin 包含 Skills、自带程序和 Full MCP。

`qiongli install skills` 只导出指导文件。`qiongli upgrade cli` 显示升级方法，不替你执行。详见 [Plugin 选项](../advanced/plugin-installation.md)、[升级与回退](upgrade.md)。

### 接入 Host {#connect-a-host}

注册后新开 Host 会话，列出实际工具并调用 `qiongli_config_status`。`qiongli mcp check` 成功只代表当前 CLI 的本地协议通过，详见 [MCP 接入](../advanced/cross-platform-mcp.md)。

### 首次使用 {#first-use}

参照[快速开始](../quickstart.md)，直接描述任务，或在 Codex 中用 `$qiongli-paper-read` 等可见入口。工作流名称是 Host 指令，不是终端子命令。

### Skill 描述语言 {#skill-description-language}

安装器提供 Auto、中文和 English。更新默认保留已保存的语言，详见[语言选项](../advanced/plugin-installation.md#language)。

### 稳定版与 Next 身份 {#stable-and-next-plugin-identities}

从 2.1 起，稳定版 Plugin 使用 `qiongli`，Alpha/Beta 使用 `qiongli-next`，由 CLI 版本决定。已核验的旧源目录可以保留原名称；npm 包名始终是 `qiongli`。

### DeepSeek Harness {#deepseek-harness-installation-21}

用 `qiongli install plugin --target deepseek` 或 DeepSeek 自己的管理器安装，详见 [DeepSeek 配置](../advanced/plugin-installation.md#deepseek)。

### 导出本地 Plugin {#export-a-local-plugin-source}

脚本或自定义注册使用[预览与应用命令](../advanced/plugin-installation.md#scripted-export)。导出后仍需通过官方 Host 注册。

## 安装状态与版本 {#installation-state}

`qiongli install list` 与 `qiongli doctor` 核对官方 Host 清单、源目录和缓存收据。不可用或已变化的安装会如实报告。注册匹配仍需新会话检查实际工具。

CLI 与每个 Plugin 各自携带程序。更新 CLI 后刷新 Plugin，再核对 CLI 版本、`qiongli content --json` 和 Host 清单；版本文字相同不能代替缓存文件核验。

## 项目修改与进阶命令

项目修改需要预览（preview）、授权（approval）和当前修订号（revision）。用 `qiongli project --help` 查看创建、收集、导入、导出等操作，迁移前先读[数据与备份](data-lifecycle.md)。

2.5.1 支持用 `--retrieval-manifest-file` 保存检索历史。旧版安装先通过
`qiongli project capture consolidate --help` 确认是否提供该选项。
预览绝对路径的 JSON 草稿，核对 `retrievalManifestContent`，再用同一草稿、返回的
时间与摘要，以及学术审阅和文件写入两项批准提交。草稿包含 `schemaVersion: 1`、
`previousSha256`（新建为 null，追加为旧文件哈希）和 `attempts`（Stage B 的十一列，
字段名使用 camelCase）。旧行保持完整，失败尝试与后续 Host 获取分别记录，未知元数据
明确保留。绑定的来源包必须已经保存，其哈希不代表完整 PDF。新会话可读取这些历史；
来源变化需要重新审阅。已发布的 2.2.1 CLI 尚无此选项。

先用 `project show` 取得已注册项目的当前修订号，再用 `project document list`
从保存收据恢复文件绑定信息：

```sh
qiongli project document list --project-id <prj_id> \
  --expected-project-revision <revision> --limit 32 --json
```

Full MCP 的对应工具为 `qiongli_project_document_list`。只有 `current` 条目提供
可交给正文读取工具的 `readArguments`。缺失、已改动或不可读取的条目保留保存时的
摘要，需要重新检查，不能直接换成磁盘上的新摘要。续页时将 `nextOffset` 传给
`--offset`，将 `bindingsSha256` 传给 `--expected-bindings-sha256`，保持项目修订号。
该摘要绑定收据历史，各页分别校验其文件；没有合并保存收据的文件不会出现在列表中。
列表不会刷新或写入项目状态，旧版本和 Lite 不会因此获得此能力。

用 `project document read` 可直接读回已保存的笔记、来源包
和 `retrieval_manifest.csv`。使用当前项目修订号，以及从列表中有效条目或已授权的
保存预览、收据、文件读取中取得的整文件哈希：

```sh
qiongli project document read --project-id <prj_id> \
  --expected-project-revision <revision> --relative-path notes/<citekey>.md \
  --expected-sha256 <sha256> --max-bytes 16384 --json
```

Full MCP 的对应工具为 `qiongli_project_document_read`。续读时将 `nextOffsetBytes`
传给 `--offset-bytes`，保持相同修订号和哈希。响应明确标注截断情况，哈希始终对应
整份文件。哈希不符、来源变化、路径不安全或项目待恢复时会拒绝读取；读取不授予写入
批准，也不验证远程论文。已发布的 2.2.1 和 Lite 尚无此能力；此接口不发现未知文件
或替调用者获取未知哈希。

其他 MCP 客户端可用程序绝对路径启动：

```sh
qiongli mcp serve --profile full --transport stdio
```

2.5.1 中，Lite 提供 15 个工具，Full 提供 35 个，包括项目、已保存文档的发现与读取、Graph 和交接。模型与执行仍由 Host 管理。

`app` 命名空间保留底层安装计划，`app apply` 需要计划摘要与明确的文件写入批准。托管产品的安装、更新命令有单独权限要求；包管理器安装的 CLI 通过原渠道升级。`qiongli update` 查看托管更新状态。

`project init`、`check`、`provider setup` 等旧命令见 [1.x 参考](../reference/cli.md)。

## Research Graph {#research-graph}

Graph 从已保存的规范记录连接论点、来源、决定和具体位置。先由 Host 阅读材料，提出记录，再审阅并保存。

```sh
qiongli project graph snapshot --project-id <prj_id> --text
qiongli project graph view --project-id <prj_id> --open
```

`--open` 保存新的私有离线 HTML 快照并请求系统打开，`--save` 只保存。没有窗口时手动打开返回路径。旧快照和项目文件保留。

页面支持搜索、筛选、查看待确认或被拒绝的关系，以及复制绑定当前修订号的来源命令。来源变化后刷新项目并重新导出。筛选只改变显示，下载的 JSON 仍包含完整快照；私有研究导出后也须按原访问范围保管。

Graph 检查记录、来源位置与版本的一致性，缺失证据保持待解决。[Graph 示例](../examples/research-graph.md)提供输入和可复现步骤；结论是否成立仍需研究核查，Graph 不会自动理解任意 PDF。
