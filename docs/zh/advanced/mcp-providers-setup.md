# 配置文献服务

Plugin 接入 MCP 与配置在线文献服务是两件事。
Host 已能使用 Qiongli 工具后，先调用 `qiongli_config_status` 和
`qiongli_literature_status`，查看哪些服务已经配置、当前可用。无需再装一个 MCP 包。

## 配置一个服务

下列统一入口属于 2.5.3 之后的原生源码更新。先运行 `qiongli provider --help` 确认
安装的程序已经包含它；2.5.3 标签中的原生版本没有新密钥 CLI 录入入口，其 MCP 配置工具
返回不可用。CLI 与 Plugin 各自携带程序，更新 CLI 后也需要刷新 Plugin。

```sh
qiongli provider setup
qiongli provider setup openalex
qiongli provider setup pubmed --no-open
qiongli provider status
qiongli provider test openalex --confirm-network-request
```

`setup` 打开带临时令牌的本地页面。使用 `--no-open` 时，保持终端进程运行，手动打开
显示的链接。选择一个服务，填写字段、核对启用状态，勾选确认后保存。空白字段保留
现有内容，删除须明确选择；删除 key 也会停用该服务。链接十分钟后或保存成功一次后
失效。设置过程不访问外部文献服务；`status` 与 `doctor` 别名只查看本地状态。
`test` 要求显式网络参数，只返回指定服务的一次有范围限制的连接检查，成功请求即使
没有论文结果也算连接通过，不返回论文正文或凭据。

在 Host 中，请它为所需服务调用 `qiongli_configure_provider`，打开返回的本地链接，
在页面输入 key。`qiongli_open_config_wizard` 是同一页面的兼容入口。设置 arXiv 时，
不传 provider 参数，在页面选择 arXiv；现有 MCP 的 provider 参数枚举保持兼容。
原生 MCP 不自动
打开浏览器。不要把 key 贴进会话；原生 `qiongli_save_provider_config` 会给出设置页
指引，不写入通过会话传来的密钥。保存后或其他设置入口增加配置修订号后，同一原生
MCP 会话能重新读取状态。
本入口覆盖 CLI 及其 Lite/Full MCP profile，也包括内含 Full MCP 的 Plugin。
独立 Lite 包仍报告设置不可用，应先检查实际运行时回复，不能保证它会打开设置页。

macOS 密钥使用现有 Keychain。普通 `~/.config/qiongli/v2/settings.json` 只保存凭据
引用、服务开关和公开联系字段；`QIONGLI_CONFIG_HOME` 可以改变共享配置根目录。同一
用户和配置目录下，各 Host 的 MCP 共用 provider 设置。Windows/Linux 的原生凭据
适配器尚未实现：密钥写入会明确失败，Crossref 联系邮箱和 arXiv 开关仍可设置。
没有明文配置或 API-key 环境变量回退。若保存完成但显示清理待处理，应保留该提示，
不能认定旧密钥已经清理。模型凭据继续由 Host 管理。

| 服务 | Qiongli 2 使用的配置字段 |
|---|---|
| OpenAlex（`openalex`） | 启用需填写 `api_key`；`email` 可选 |
| Semantic Scholar（`semantic_scholar`） | 启用需填写 `api_key` |
| Crossref（`crossref`） | 启用需填写 `email` |
| PubMed（`pubmed`） | 启用需填写 `api_key` |
| arXiv（`arxiv`） | 不需要凭据；未停用时可用 |

这是 Qiongli 的配置要求，不代表这些服务的每种 API 请求都必须带凭据。
Lite 与 Full 使用相同的配置契约，源文件是
`content/mcp-contracts/provider-config.schema.json`。
保存后再调用 `qiongli_literature_status`。配置完成不保证每次请求成功；
检索结果会另行报告服务错误或只返回部分结果的情况。

## 规划并执行检索

`qiongli_search_plan` 根据传入的能力和材料，帮助当前代理在文献服务、
Host 原生搜索与用户提供的来源之间安排检索。返回的 `search_execution_mode`
可能是 `provider_connected`、`native_only`、`hybrid_search` 或 `strategy_only`。
另一个字段 `provider_capability_mode` 只表示 `provider_connected` 或 `strategy_only`。
其中的 `native_search_queries` 是给 Host 的查询建议，并不代表已经执行。

`qiongli_literature_search` 负责有范围限制的服务检索。MCP 不会调用 Host 的原生搜索；
这部分由当前代理使用自己实际可用的工具完成。两条路径都不可用时，应保留检索方案并说明缺口。
服务返回空结果，也不能据此断言没有相关论文。

本地文献请使用独立的 [Zotero 工具](mcp-zotero-integration.md)。
综述的方法和覆盖范围，见[严谨文献检索](rigorous-literature-search.md)。
Python 的 `RESEARCH_MCP_<PROVIDER>_CMD` 适配器配置属于 1.x，不是当前原生运行时的接入方式。
