# 将 Zotero 用作本地文献库

Qiongli 从在线文献服务查找候选文献，Zotero 保存你的本地资料库。
Qiongli Zotero Companion 通过本机连接，让原生 MCP 工具访问正在运行的 Zotero Desktop。
这条本地路径不需要 Zotero Web API 密钥，也不要求云同步。

## 在 Zotero 中安装 Companion

从对应的 [Qiongli Release](https://github.com/jxpeng98/qiongli/releases)
下载 `qiongli-zotero-companion-*.xpi`，通过 Zotero 的扩展管理器安装，再重启 Zotero。
Companion 0.3.1 声明支持 Zotero 8 至 10.0.x，包括 10.0.2。
安装 Qiongli 的 Host Plugin，不会自动在 Zotero 内安装扩展。

Companion 源码在 `packages/qiongli-zotero-companion/`。维护者可运行
`python3 scripts/build_zotero_companion.py --dist-dir dist` 构建 XPI；
普通用户直接使用 Release 附件，无需下载源码。

## 检查连接并检索

原生 Lite 和 Full 都提供 `qiongli_zotero_status`、`qiongli_zotero_search`、
`qiongli_zotero_upsert_references` 和 `qiongli_zotero_export_import_files`。
直接检索和写入需要 Companion；生成导入文件内容不需要。

先请 Host 调用 `qiongli_zotero_status`。如果 Companion 不可用，检查 Zotero 是否打开、
扩展是否启用。Connector 正在运行，不代表 Companion 已就绪。
确认后，可以检索你允许访问的文献范围，例如：

```json
{"tool":"qiongli_zotero_search","arguments":{"title":"platform governance","limit":10}}
```

原生检索接受 `doi`、`title`、`year`、`citekey`、`creator`、`tag` 或
`collection_path` 等条件，至少要提供一个。查找本地条目时使用这个工具；
原生文献检索接口不接受旧版的 `include_zotero` 开关。

## 保存前先预览

将选定的文献对象通过 `items` 传给 `qiongli_zotero_upsert_references`。
默认的 `dry_run: true` 只预览变更；`update_policy: "fill_blank"` 保留已有的非空字段。
确认具体条目、集合和修改内容后，再授权写入。

应用时，保留预览中的计划内容，设置 `dry_run: false`、`write_intent: "apply"`，
并传回工具返回的 `dry_run_receipt`。计划变动或过期后要重新预览，
不能编造收据，也不能把检索成功当成保存许可。

无法直接访问时，可用 `qiongli_zotero_export_import_files` 的 `records` 参数生成
`references.json`、`references.ris`、`bibliography.bib` 和 `zotero-import-report.md`
的内容。工具返回内容不等于已经保存文件或写入 Zotero；需要确认后保存文件，再通过 Zotero 导入。

原生 CLI 接受 `QIONGLI_ZOTERO_CONNECTOR_URL`
（通常为 `http://127.0.0.1:23119`），远程 Connector 地址会被拒绝。
当前指南不承诺提供原生 Zotero Web API 写入路径。
