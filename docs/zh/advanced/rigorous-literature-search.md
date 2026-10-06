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

## 回读已经保存的原文片段 — 2.4.0 候选版

候选版读取接口可以选取来源文件中的一个已知文本字段。先读取项目当前修订号，
通过 `qiongli project document list` 恢复保存记录；只有 `current` 条目中的
`readArguments` 可用于回读。保留其中的路径与整个文件的 SHA-256，再用原始文件
读取接口检查真实结构，确定片段位置。

例如，实际检查确认原文位于 `/segments/2/text` 后，可以执行：

```bash
qiongli project document read \
  --project-id <prj_id> --expected-project-revision <revision> \
  --relative-path sources/<citekey>/<sha256>.json --expected-sha256 <sha256> \
  --json-pointer /segments/2/text --max-bytes 4096 --json
```

Full MCP 的 `qiongli_project_document_read` 提供相同的可选参数 `json_pointer`。
先检查 CLI 帮助或实际工具声明；旧版仍使用原始读取。数组形式的来源文件可能使用
`/0/segments/2/text`，示例路径不能代替检查实际文件。

返回值包含解码后的原文、`jsonPointer` 和 `selectedTextSizeBytes`。字节偏移及
`nextOffsetBytes` 对应这段文本；`sha256` 和 `sourceSizeBytes` 仍对应整个原始
来源文件。分页时保持定位参数、项目修订号和文件摘要不变。定位错误或文件变化时
操作会拒绝，不会自动换用其他片段，也不会搜索其他文件。

将这个位置保留在现有论断或阅读笔记中，同时记录原始文献版本、章节、表格或页码。
判断论断是否得到支持前，还要检查相关上下文和表下注释。字节位置不是论文页码，
原文回读成功也不代表学术核查通过。

## 在保存材料中查找原文 — 2.4.0 候选版

候选版可以在一个明确选定、已绑定版本的保存文件中查找原文。按上述步骤取得
当前有效的文件记录后，使用原文中的短语检索：

```bash
qiongli project document search \
  --project-id <prj_id> --expected-project-revision <revision> \
  --relative-path sources/<citekey>/<sha256>.json --expected-sha256 <sha256> \
  --search-text '原文中的短语' --max-matches 8 --context-bytes 128 --json
```

Full MCP 在 `qiongli_project_document_read` 中加入 `search_text` 即可使用
相同能力，此时不要传入读取模式的 `offset_bytes` 或 `max_bytes`。先检查实际
帮助或工具声明。可选的 `json_pointer` 将范围限定到一个已知文本字段；未指定时，
来源文件会检索解码后的所有字符串值，包括元数据，笔记与检索历史则检索原始文本。
匹配区分大小写、逐字且不重叠，不做翻译、规范化，也不跨字段拼接。

每个命中包含上下文、UTF-8 字节位置，以及能通过读取接口重现上下文的
`readArguments`。`context_bytes` 分别限制命中前后两侧的字节数；需要更多
上下文时，可继续扩大读取范围。不同位置的重复文本分别保留。先检查原始版本、
章节或页码等元数据及实际内容，再判断命中能否支持论断；把经过审阅的位置保留在
现有笔记和证据记录中。字节位置不是论文页码，元数据中的命中也不是正文证据。

分页时将 `nextMatchOffset` 传给 `--match-offset`，将 `searchSha256` 传给
`--expected-search-sha256`，保持检索短语、定位字段、上下文大小、每页数量、
项目修订号及文件摘要不变。绑定变化时操作拒绝。`searchScope`、
`scannedTextFields` 和 `totalMatches` 只描述本次保存材料的检索范围；没有命中
不能说明完整论文中不存在相关内容。其他选定来源可通过各自的有效绑定逐一检索。
跨文献自动遍历、从论断回溯原文并逐项审查，仍属于 2.4 计划的后续工作。
