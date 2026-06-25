# Zotero 集成：本地 Reference Database

Qiongli 会把 Zotero 当成本地 reference database 使用，而不是把 Zotero
替换成 OpenAlex、Semantic Scholar、Crossref、PubMed 或 arXiv 这类发现型 provider。
推荐流程是：先用 Qiongli 的文献 provider 检索和补全 metadata，再把选中的
reference 通过 Qiongli Zotero companion 写入本地 Zotero Desktop。

这个 local-first 路径不需要 Zotero Web API key，也不要求 Zotero 云同步。
如果本地 Zotero 不可用，Qiongli 仍会生成可导入文件：
`references.json`、`references.ris` 和 `bibliography.bib`。

## 组件

| 组件 | 作用 |
| --- | --- |
| Qiongli literature MCPB | 规范化 reference、映射 metadata、去重、暴露 Zotero tools，并生成导入文件。 |
| Qiongli Zotero companion | 一个很薄的 Zotero Desktop plugin，用来注册 `/qiongli/*` 本地 connector endpoints。 |
| Zotero Desktop | 保存本地 reference library、collections、tags 和用户手动维护的 metadata。 |

companion 位于 `packages/qiongli-zotero-companion/`。它不是独立 MCP server，
而是 Qiongli MCPB 的本地 Zotero 桥。

直接写入本地 Zotero 需要在 Zotero Desktop 里安装这个 Qiongli companion
plugin。不需要额外安装第三方 Zotero 插件。没有 companion 时，Qiongli 仍可以
生成可导入文件。

在仓库根目录构建可安装扩展：

```bash
python3 scripts/build_zotero_companion.py --dist-dir dist
```

然后在 Zotero Desktop 的 add-on manager 中安装生成的
`qiongli-zotero-companion-*.xpi`，并重启 Zotero。

## 检查本地状态

运行：

```json
{ "tool": "qiongli_zotero_status", "arguments": {} }
```

这个工具会检查：

1. Zotero Desktop connector server：`http://127.0.0.1:23119/connector/ping`。
2. Qiongli companion endpoint：`http://127.0.0.1:23119/qiongli/ping`。
3. 可导入文件 fallback 是否可用。

可能的状态：

- `ok`：Zotero Desktop 和 Qiongli Zotero companion 都可用。
- `companion_missing`：Zotero Desktop 正在运行，但 companion plugin 未安装或未加载。
- `fallback_only`：无法连接 Zotero Desktop；改用可导入文件。
- `disabled`：本地 Zotero 模式被配置关闭。

## 显式启用本地 Zotero 来源检索

`qiongli_literature_search` 默认不会搜索 Zotero。只有当你明确传入
`include_zotero: true` 时，Zotero 才会作为额外的本地 reference source：

```json
{
  "tool": "qiongli_literature_search",
  "arguments": {
    "query": "platform governance",
    "include_zotero": true,
    "zotero_tag": "project:platform-governance"
  }
}
```

只存在于 Zotero 的本地条目会返回 `provider: "zotero"` 和
`source_type: "local_reference_database"`。外部 provider 的结果如果 DOI 或
title/year 已经存在于 Zotero，会带上 `local_zotero_match`，方便判断是否已经保存。

## 保存检索结果

先检索：

```json
{
  "tool": "qiongli_literature_search",
  "arguments": {
    "query": "platform governance systematic review",
    "search_mode": "review",
    "per_provider_limit": 50
  }
}
```

再 dry-run 写入 Zotero：

```json
{
  "tool": "qiongli_zotero_upsert_references",
  "arguments": {
    "records": [
      {
        "title": "Platform Governance in Practice",
        "authors": ["Smith, Alex"],
        "year": 2024,
        "doi": "10.1000/platform-governance",
        "venue": "Organization Science",
        "provider": "openalex",
        "source_id": "W123"
      }
    ],
    "collection_path": "Qiongli/platform-governance/To Screen",
    "tags": ["project:platform-governance", "status:to-screen"]
  }
}
```

默认是 dry run。真正写入时需要显式设置 `dry_run: false`。桥接层会优先用 DOI
匹配 Zotero 里已有条目，再用 title/year fallback。默认策略只补空字段、添加
identifier、tags 和 collection membership，不覆盖用户已经在 Zotero 中手动维护的
title、authors、date、publication title 或 abstract。

带 DOI 的写入默认会先使用 Crossref registry metadata 做补全。Crossref
verification 只填补空字段，不等于人工核查。新建或更新的候选条目仍会加上
`qiongli:imported` 和 `qiongli:needs-review`。通过 Crossref 匹配的条目会加
`qiongli:crossref-verified`；如果 incoming metadata 和 Crossref registry
metadata 在 title 或 year 上存在实质冲突，会加 `qiongli:metadata-conflict`，
并在 `verification.crossref.conflicts` 中返回冲突详情。

## 可导入文件 Fallback

companion 不可用时，可以生成导入文件：

```json
{
  "tool": "qiongli_zotero_export_import_files",
  "arguments": {
    "records": [
      {
        "title": "Fallback Paper",
        "authors": ["Smith, Alex"],
        "year": 2024,
        "doi": "10.1000/fallback"
      }
    ]
  }
}
```

输出包括：

- `references.json`：Zotero CSL-JSON 导入。
- `references.ris`：Zotero、EndNote、Mendeley 通用。
- `bibliography.bib`：BibTeX 工作流。
- `zotero-import-report.md`：记录数量、Crossref verification 汇总和 fallback 操作说明。

## 配置

本地模式只允许 loopback connector URL。

```bash
QIONGLI_ZOTERO_LOCAL_ENABLED=true
QIONGLI_ZOTERO_CONNECTOR_URL=http://127.0.0.1:23119
QIONGLI_ZOTERO_WRITE_POLICY=explicit
QIONGLI_ZOTERO_UPDATE_POLICY=fill_blank
QIONGLI_ZOTERO_DEFAULT_COLLECTION_PATH="Qiongli/[topic]/To Screen"
QIONGLI_ZOTERO_DEFAULT_REVIEW_TAGS="qiongli:imported,qiongli:needs-review"
QIONGLI_ZOTERO_CROSSREF_VERIFICATION_ENABLED=true
```

`QIONGLI_ZOTERO_CONNECTOR_URL` 必须指向 `127.0.0.1`、`localhost` 或 `::1`。
非 loopback URL 会被拒绝。

## Web API 模式

Zotero Web API 支持通过具备写权限的 API key 写入。这个模式未来可以用于
cloud-sync workflow，但它不是 Qiongli 的默认 Zotero 集成路径。默认路径是：
本地 Zotero Desktop + Qiongli Zotero companion；本地写入不可用时，生成可导入文件。
