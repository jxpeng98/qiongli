---
title: Research Graph 完整示例
description: 用虚构研究材料走通记录、关系图和来源回查，并直接查看生成的离线 HTML。
---

<script setup>
import { withBase } from 'vitepress'
</script>

# 从研究摘录到 Research Graph

这个示例研究“短休息是否与即时回忆表现有关”。所有材料均为虚构，DemoA、DemoB
只是演示标识，不是真实论文或可引用的研究结论。你可以直接查看生成页面，不用安装
Qiongli、Python 或 Node；也可以在仓库中重新跑一遍真实 CLI 流程。

<p><a :href="withBase('/demos/research-graph.html')" target="_blank" rel="noopener">打开完整的交互示例</a> · <a :href="withBase('/demos/research-graph.html')" download>下载离线 HTML</a> · <a :href="withBase('/demos/research-graph.snapshot.json')" download>下载快照 JSON</a></p>

<iframe :src="withBase('/demos/research-graph.html')" title="Research Graph 虚构研究示例" loading="lazy" sandbox="allow-scripts allow-downloads" style="width:100%;height:960px;border:1px solid var(--vp-c-divider);border-radius:8px"></iframe>

内嵌页面可以独立滚动；想看完整关系图，建议打开上面的完整页面。它使用 CLI 的
同一份 HTML 模板，包含这次实际生成的数据，后续项目变化不会自动更新它。

## 先试这三步

1. 选择 **CLM-1**，查看两条 `supports` 关系。点击关系后，可见每份材料的限制：
   没有随机分配、只测了一项任务、没有延迟测量。`cites` 只表示引用，不等于支持证据。
2. 选择 **DEC-1**，看到已审阅的 `informs` 关系；选择 **DEC-2**，看到提议状态。
   决策影响研究取舍，但不会变成支持论点的证据。
3. 选择 **CLM-2**。它保留“长期改善尚未验证”的缺口，没有 `supports` 关系。
   下方来源检查也会列出这项问题。

实际输出有 **9 个研究记录、8 条非结构关系**，其中 2 条为支持关系。
页面还会列出尚未填写的规范来源；这是一个有意保留缺口的入门项目，不是完成的论文。

## 整个流程做了什么

| 步骤 | 示例中的内容 | 谁负责 |
|---|---|---|
| 原始摘录 | `notes/synthetic-reading-notes.md` 的 DemoA / Table 1 与 DemoB / Table 2 | 示例提供虚构材料 |
| 规范记录 | `evidence/claim-evidence-ledger.csv` 保留论点、来源位置和限制 | 示例已提供经过检查的整理结果 |
| 研究取舍 | `context/decision_log.md` 与 `manuscript/claims_evidence_map.md` | 区分已接受决定、提议与稿件引用 |
| 注册与投影 | CLI 先 preview，再以对应摘要 apply；随后读取 Graph | 现有原生 CLI |
| 查看与回查 | 同一快照生成 HTML，来源命令返回台账片段和定位行 | 现有 Graph 视图与来源读取接口 |

CLI 并没有自动读懂原始散文或 PDF。真实研究中，需要你或模型先核对材料，再通过已有
审批流程保存规范记录。本示例证明这些记录如何进入 Graph、如何追溯，不能证明模型
已经完成可靠的自动提取。

## 在本地复现

以下命令面向仓库开发环境，需要 Rust 构建工具和 Python 3。仅查看上面的 HTML 不需要这些工具。

```bash
cargo build --manifest-path packages/qiongli-native/Cargo.toml --locked -p qiongli
python3 tooling/scripts/build_research_graph_example.py \
  --cli packages/qiongli-native/target/debug/qiongli \
  --destination /tmp/qiongli-graph-demo
```

Windows 使用对应的 `qiongli.exe`，并选择一个父目录已存在的新输出目录。脚本拒绝
覆盖已有目录；重跑时换一个新目录，不会替你删除旧结果。

脚本复制 `tests/fixtures/research_graph_example/`，在输出目录中创建独立的 Qiongli
配置，并完成演示项目注册。它只对这个新副本确认 preview/apply，不注册到你的日常
配置，不修改原始示例文件，也不启动浏览器。输出包括：

| 文件 | 用途 |
|---|---|
| `research-graph.html` | 可直接打开的离线页面 |
| `snapshot.json` | 完整 Graph 与 readiness 数据 |
| `source-excerpt.json` | 真实来源读取的结果，定位到台账第 2 行 |
| `proof.json` | 二进制与输入文件摘要、关系数量及通过的检查 |
| `commands.json` | 本次实际执行的 CLI 参数与退出码 |
| `project/`、`config/` | 保留的演示项目和独立配置 |

网站示例中的来源命令绑定了生成时的项目和快照，不能直接用于你自己的项目。
复现后，请使用新 HTML 中的命令、同一个 CLI，并为该进程指定输出中的
`QIONGLI_CONFIG_HOME` 路径；脚本结束时会打印这个路径。也可以直接读取保存的
`source-excerpt.json`，不必重新执行命令。

## 检查依据

<p><a :href="withBase('/demos/research-graph.proof.json')">查看这次运行的检查记录</a> · <a :href="withBase('/demos/research-graph.source.json')">查看真实来源回查结果</a></p>

生成脚本检查了证据限制、未支持论点、决策关系、来源定位、错误版本拒绝、重复投影
一致性、HTML 与 JSON 一致性，以及原始文件和项目文件未被查询修改。CLI 调用使用空
PATH，未调用外部模型或在线服务。仓库检查还会核对公开示例与当前模板、输入文件是否一致。

这些是合成数据和程序行为检查。真实浏览器画面、系统剪贴板及下载行为仍需实测，
不能仅凭示例生成成功就判定它们或科研质量已经通过验收。
