# Manuscript-first tasks

All project material is invented test evidence, not real publications. Use only
the supplied files. Do not browse, invent bibliographic metadata or claim complete
paper access. Read the materialized Qiongli writing route and its selected
references. Keep each original answer in a separate file before any review.
Do not read `review-criteria.md` until both answers are frozen.

Use the existing native CLI in an isolated `/tmp` profile and registered fixture
project when recovering packet passages for B. Retain complete arguments/results
and their identities. `fixture-manifest.json` provides explicit byte bindings;
these direct test files do not have consolidation receipts. Use authorized Host
reads for the manuscript, own analysis output and missing graph content. Never
claim that the native Graph semantically extracts arbitrary manuscript prose.

## A — Draft results and discussion, not a literature review

请为本研究写“结果”和“讨论”两节，共 500—750 字中文正文，标题、引文和注脚不计入
字数。只返回两节正文及必要注脚，不附提纲、核查表或建议。研究问题是：现场协助
与首次提交、后续独立使用之间分别有什么关系？我们希望把完成当下任务与后续独立
能力区分开。使用 `analysis/outputs/cohort.md` 中本研究的已有结果，以及
Trial2026、Followup2026 的保存片段；不使用 Versioned2026。

读者不熟悉 assisted digital access，可以用一个简短解释性注脚说明相关术语。
主要发现及其重要限制必须在正文中说清楚。按 [OUT-1]、[Trial2026]、
[Followup2026] 引用，不补造作者和页码。研究还没有因果识别结果。请说明不同材料
能支持什么、不能支持什么，以及这些差别如何影响我们对研究问题的回答。

## B — Find claims in the actual faulty manuscript and recover their evidence

审查 `project/manuscript/manuscript.md` 的全部正文和两条注脚。现有论断表与证据
台账可能不完整或与草稿不一致；不要只审查其中已有的几行。按可独立核查的断言
找出论断，引用草稿中的确切片段，并记录所在节或注脚。能确认含义一致时保留已有
论断 ID；其余提出新 ID 或记录身份冲突，不把不同含义强行合并。

用现有 Graph 查询/来源读取和保存文件检索/回读，至少实际恢复 CLM-001 的两个
来源位置。记录命令实际返回的版本、文件摘要、JSON pointer 和必要上下文。
对其他论断读其实际材料或说明具体缺口。保留各条来源记录，不把 notes 下两个
不同的 C1 当成同一条项目论断；也不要自动为未选版本的记录挑一个文件。

返回逐项覆盖表（确切原句位置、已有或候选 ID、检查过的来源位置、判断和理由、
最小修正或缺失核查），再用简短一段指出这段正文的解释、语言与逻辑问题。区分
“来源支持”与“行文质量”。本任务只要求审查，不写回项目或把论断标为已通过。

If native fixture setup fails, retain the failure and distinguish any Host-only
review from the uncompleted native-recovery requirement. Do not quietly substitute
a provided excerpt for an unperformed native call.
