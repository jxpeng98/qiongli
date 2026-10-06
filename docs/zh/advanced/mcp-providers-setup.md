# 配置文献服务

Plugin 接入 MCP 与配置在线文献服务是两件事。
Host 已能使用 Qiongli 工具后，先调用 `qiongli_config_status` 和
`qiongli_literature_status`，查看哪些服务已经配置、当前可用。无需再装一个 MCP 包。

## 配置一个服务

请 Host 为所需服务调用 `qiongli_configure_provider`，打开返回的本地配置链接，
在页面中填写凭据，避免把密钥贴进会话。`qiongli_open_config_wizard` 是兼容入口。
明确要求直接写入某个配置字段时，也可使用 `qiongli_save_provider_config`；
工具不会在结果中返回密钥原文。

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
