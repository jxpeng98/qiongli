# 严谨文献检索

当你需要说明文献是怎样找到、筛选并用于研究的，可以采用这套流程。
先明确研究问题和已经约定的综述协议，再据此选择来源、时间范围和检查方法。

## Qiongli 提供什么

原生 MCP 可通过已配置的 OpenAlex、Semantic Scholar、Crossref、PubMed 和 arXiv
进行有范围限制的检索。`qiongli_search_plan` 提出路径和查询建议，
`qiongli_literature_search` 执行文献服务请求，返回规范化结果和诊断。
配置方法见[文献服务接入](mcp-providers-setup.md)。

Host 可以结合自身可用的原生搜索和获准读取的本地材料。引用追踪、全文获取和筛选
仍需要实际来源及相应工具。检索方案、网址或元数据条目都不等于已经拿到论文，
旧 Python 版的适配器接口也不能当作已安装的原生工具。

## 保留检索记录

正式综述按任务需要保留 `search_strategy.md`、`search_log.md`、`search_results.csv`、
`dedup_log.csv`、`snowball_log.md`、`bibliography.bib`、`screening/full_text.md`
和 `retrieval_manifest.csv`。记录服务、查询式、日期、筛选条件、排除理由与失败情况，
并在提取和综合时沿用 citekey 与来源位置。保存仍使用项目原有的预览和批准流程。

## 检查覆盖范围，避免夸大结果

任何服务都不能证明检索绝对完整。应按协议选择合适的检查：

| 检查 | 能说明什么 |
|---|---|
| 已知文献召回 | 检索策略是否找到了预期应纳入的论文 |
| 概念与来源覆盖 | 要求的查询组合和数据库是否查过，零结果或不可用路径是否有说明 |
| 重复结果趋于饱和 | 新增查询是否主要返回已有记录；这一点本身不能证明完整性 |
| 引用追踪 | 是否完成并记录了协议要求的前向、后向追踪 |
| 全文可得性 | 哪些待获取的报告已经实际拿到并能够阅读 |

`native_fulltext_queries` 中的建议仍需 Host 执行，并保留候选网址和访问日期。
核验 Zotero 附件时，也要检查实际文件：文献库中存在匹配条目，不代表有可读 PDF；
拿到 PDF，也不代表它支持某个具体论点。

根据实际读到的材料，为提取的论点记录 `evidence_limit`：`full_text`、
`abstract_only`、`metadata_only` 或 `unavailable`。
预印本与正式发表版本不同时，还要保留所用版本的信息。

## 将结果接入 Graph

审阅提取结果后，再提出结构化论点和来源关联，保存时保留不确定性与未解决的冲突。
[Research Graph](../guide/cli-2x.md#research-graph) 用来检查已经保存的关系，
不能代替筛选决定，也不能证明检索召回率。
