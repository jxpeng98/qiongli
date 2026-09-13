# 穷理 2.0：与 1.x 相比，变了什么

Qiongli 2.0.0 是 Rust 原生 CLI 的第一个正式版。程序、研究内容和 MCP 实现随同一个
可执行文件交付，你仍可使用自己选择的 Host 和模型。阅读、文献综述、研究设计和写作，
依然是熟悉的研究入口。

变化主要在这些能力如何配合：安装时需要准备的运行环境更少，CLI 和 MCP 共用原生实现，
研究记录也能在对话之外继续查阅。本次正式发布面向 CLI 及其随包接入能力；它不代表
整个桌面替代计划、所有 1.x 项目迁移和实际协作场景都已完成验收。

## 先看主要区别

下表以保留的 1.x 后期实现为参照，包括 1.19 兼容基线。1.x 不同版本的功能和打包方式
有变化，因此不能把每一项差异都套用到所有旧安装上。

| 方面 | 1.x 后期 | 2.0 |
|---|---|---|
| 主运行时 | Python 应用，npm 另有资产管理入口 | Rust 原生 CLI，内嵌研究内容与 Lite/Full MCP |
| 直接安装 | 各入口要求不同；Marketplace 已有独立的原生 Lite 包 | 三个平台的完整 CLI 压缩包，无需另装 Python、Node.js 或 Rust |
| 分发渠道 | Python 与 npm 入口承担不同职责 | GitHub、npm、PyPI 和 Cargo 提供同一原生 CLI，各自保留安装前提 |
| Plugin 与 MCP | 原生 Lite 和 Python Full 使用不同运行路径 | 两种 MCP 都是原生实现；Marketplace 默认 Lite，本地 Plugin 默认 Full |
| 安装体验 | 较多 surface、parts 和 provider 选项 | `install` 提供向导；`install plugin` 同时处理安装和已核验内容的更新 |
| 研究指导 | 更多预设阶段、角色和通用数量要求 | 按目标、已有证据和适用协议提供指导，小任务可以只做所需部分 |
| Graph | 引文发现与扩展服务于文献工作 | Research Graph 进一步关联项目记录中的论点、证据位置、决定和缺口 |
| 项目连续性 | 工作流产物和交接记录 | 结构化采集、版本检查、Graph 快照和阶段总结共用来源标识 |
| 代理协作 | 工作流角色与编排约定 | 明确 Host 子代理和交接材料的任务、来源及返回要求；Host 负责实际执行 |
| 维护方式 | Python 运行时与多条分发路径 | 共享 Rust 服务和规范内容源，构建时统一核对版本、平台及嵌入资源 |

## 先装程序，再接入常用 Host

希望安装步骤最少，可以从 [Release v2.0.0](https://github.com/jxpeng98/qiongli/releases/tag/v2.0.0)
下载对应平台的**完整 CLI 压缩包**，核对 `SHA256SUMS`，解压后运行 `./qiongli --help`。
Windows PowerShell 使用 `.\qiongli.exe --help`。支持 macOS ARM64、Windows x64 和
Linux x64；Linux 需要 glibc 2.35+，Windows 的 C 运行库已编入程序。
GitHub 自动生成的 Source code 压缩包用于源码构建，不是这个安装入口。

通过包管理器安装时，npm 需要 Node.js 18+，PyPI 需要 Python 3.9+，Cargo 需要
Rust 1.97+ 和本机链接器。这些是渠道本身的要求，独立二进制和导出的原生 Plugin
不需要这些语言运行时。完整命令见[下载与安装指南](cli-2x.md#standalone-binary-download)。

运行 `qiongli install`，可以选择 Plugin、独立 Skills 或 MCP 配置。推荐的本地 Plugin
包含 Skills、程序副本和 Full MCP 配置。Host 从自己的 Plugin 缓存加载它们，不必复制到
`~/.agents/skills`，也没有需要另装的 Qiongli MCP 服务包或长期运行的后台终端。

再次运行 `qiongli install plugin`，会发现已登记的源目录并提供更新；`upgrade plugin`
和 `update plugin` 使用相同流程。文件变更与官方 Host 注册分别预览、分别确认。
第二个 Host 使用独立目录。完成摘要集中显示版本、源目录、缓存、已核验的注册状态，
以及还需要检查的会话工具或 Hook。

对于已知的 Codex 穷理插件冲突，2.0 会先列出需要停用的条目，确认 Host 操作后再迁移。
旧源目录、缓存、其他 Plugin 和模型设置都会保留。不支持的插件身份或发生变化的配置
会阻止操作；Claude 仍采用手动处理冲突的方式。如果停用旧插件后注册失败，安装器会
说明当前状态，你可以重试，或在 Codex 中重新启用旧插件。

## 分清 Plugin 实际提供哪些工具

| 安装方式 | MCP 配置 | 提供的能力 |
|---|---|---|
| CLI 安装的本地 Plugin | Full，32 个工具 | 文献与 Zotero 操作，以及项目、Graph 和 Host 交接 |
| 原生 Marketplace 平台 Plugin | Lite，14 个工具 | 受限的文献、配置、检索规划与 Zotero 操作 |
| 独立 Skills 导出 | 不自动接入 MCP | 研究指导和参考资料 |

两种 MCP 都已编译进程序。原生 Marketplace 包启动时不需要 Node/npm 桥接、Python
解释器或临时下载可执行文件。1.x 后期已经有独立的原生 Lite 包；2.0 将原生交付扩展到
完整 CLI 和 Full MCP。Skill 名称相同，不代表当前 Host 一定拥有全部工具。

`qiongli doctor` 和 `qiongli install list` 检查本地安装状态，`qiongli mcp check`
检查当前 CLI 的协议。实际接入还需新开 Host 会话，列出工具并调用 `qiongli_config_status`；
在线文献配置再通过 `qiongli_literature_status` 核对。文件导出成功不等于会话已就绪。
Host 应用、服务凭据和 Zotero 配置仍按实际需要单独准备。

## 给模型明确方向，保留判断空间

Skills 保留来源归属、方法限制和可审阅的修改，减少了小任务中的固定轮次和冗长流程。
正式综述仍遵循选定的协议；只询问一个段落时，无需因此生成整套项目文档。

期刊工作保留两个方向：已有目标期刊时，核对要求并提出适配建议；已有稿件时，先判断
稿件内容，再寻找合适期刊。建议需要结合实际文章类型、证据和当前期刊要求。
Humanizer 改善中英文的表达和衔接，同时保留论点、数字、引用及不确定性。

在 Codex 中使用 `$no-qiongli`，或说“仅回复”“NoQ问理”“reply only”，可以要求模型
只在对话中回答，不调用工具、代理或处理文件。这是模型指导，不是运行时权限开关。
可选上下文 Hook 用于提醒连续性和批准边界，首次安装本地 Plugin 时默认关闭。
启用后仍需 Host 信任并核对一次真实事件。见[协作与 Hook 指南](../advanced/agent-skill-collaboration.md)。

## 在 Research Graph 中沿着论点查看来源

Host 先读取已授权材料，提出结构化研究记录。你审阅并批准保存后，穷理才能据此重建
Graph，保留论点 ID、引用键和来源位置。已审阅、待确认和已拒绝的关系分别显示；
缺少证据的地方保留为缺口，不会自动补成支持关系。

```sh
qiongli project graph view --project-id <prj_id> --save
qiongli project graph view --project-id <prj_id> --open
```

第一条命令保存新的私有 HTML 快照，第二条还会请求操作系统打开它。页面可以搜索、筛选
记录，沿有向关系查看证据、决定和缺口，并复制绑定导出版本的来源查询命令。
它无需服务器，可离线打开；内容变化后需要重新导出，因为页面是快照，不是实时编辑器。
导出的研究可能包含私有内容，页面筛选不会从快照文件中删掉其他记录。

这项能力补充了文献引文发现，但不会自动理解任意 PDF，也不能证明论点正确或代替模型
阅读。可以先看[可复现的 Graph 示例](../examples/research-graph.md)，其中故意保留了
一个证据不足的论点，展示它应该如何呈现。

阶段总结保存主要成果、来源、前序阶段和本轮变化，让后续会话能够继续工作，但不会把
总结本身当成新的证据。需要清理时，只提供逐文件的保留建议，由用户选择并亲自删除；
总结过程不会悄悄移除原始文件。

## 通过已有 Host 使用协作能力

穷理约定任务范围、来源与候选版本，以及如何把发现交回协调者；真正的子代理由 Host
提供。同一对话换一个角色仍是自查，不能算独立审查。跨 Host 协作使用已有且获授权的
通信工具，或准备由你转交的材料，不会另起通用后台调度器或自动跨设备服务。

协调者在合入前检查返回证据和版本身份。规范研究文件的写入继续使用现有预览、批准和
并发版本校验。把任务委派出去，不等于允许另一个代理直接覆盖研究文件。

## 从 1.x 迁移时，保留回退路径

1. 备份研究文件、配置并记下旧安装版本。更换程序不会让数据迁移自动变得可逆。
2. 选择一个常用 CLI 渠道。在 Unix 上用 `type -a qiongli ql`，或在 PowerShell 中用
   `Get-Command qiongli,ql -All`，检查实际执行哪个程序。npm、pip 和 Cargo 不会相互
   覆盖；`qiongli setup` 只提供清理建议，由你决定移除哪些文件或包。
3. 安装 2.0.0，用 `qiongli --version` 核对，再运行 `qiongli install plugin`。
   更新 Plugin 时复用已登记的源目录。
4. 新开 Host 会话检查工具，先用项目副本试运行，再迁移重要工作。旧 shell 命令并非
   全部兼容，请对照 [2.x 命令指南](cli-2x.md)和保留的 [1.x 参考](../reference/cli.md)。
5. 回退时使用保留的压缩包，或重装准确的旧包版本，再通过受支持的 Host 流程恢复对应
   Plugin。研究文件留在原处；如果迁移改变了数据，需要单独恢复备份。

npm 正式渠道使用 `qiongli@latest`，`next` 可能仍指向之前的 Beta；PyPI 和 Cargo 可固定
`2.0.0`。`upgrade cli` 说明原渠道的升级命令，不会代你运行包管理器或转换安装方式。

## 发布检查能说明什么

发布流水线在三个受支持平台构建和安装同一源码，核对版本与摘要，在空 PATH 下运行
解压后的程序，并检查 Lite/Full MCP 和渠道包安装。Cargo 源码发布另有检查。
公开附件的下载核验由发布工作流完成；提交工作流不等于发布已经结束。

Rust 让原生执行路径不再依赖 Python，共享实现也减少了维护中的重复。这些是架构变化，
不能直接换算成经过测量的性能提升或维护成本下降。全量 1.x 迁移、实际 Hook 触发、
浏览器交互、所有 Host/模型组合和跨 Host 研究流程仍需各自的验证证据。
本次 CLI 正式版不会直接退役保留的桌面实现或 1.x 源码。
