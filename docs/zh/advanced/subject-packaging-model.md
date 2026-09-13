# 学科指导与共享研究流程

Qiongli 使用一套共享研究流程，学科指导在此基础上补充方法、术语和报告要求。
不同学科不需要分别安装一套 CLI。

## 告诉 Host 哪些条件重要

[安装 Plugin](../guide/cli-2x.md#first-use) 后，说明学科、研究问题、可用材料和已经约定的协议，例如：

> 这是一项使用交错双重差分的经济学研究。请检查识别假设，以及这一设计需要哪些诊断。

模型可以读取相关的随包学科资料，再结合任务判断如何使用。
学科档案提供指导，不能当作证据或现行期刊政策；外部要求影响决定时仍要核查。
保存项目决定继续遵守原有的批准和修订检查。

1.x 的 `project set-subject`、`project set-venue` 和 `--domain` 示例不是原生 2.x 命令。
研究背景可以直接告诉 Host，项目操作则以 `qiongli project --help` 为准。

## 内容包术语

`core` 提供共享指导，学科层补充相关档案或具体说明。
`complete` 与 `focused` 表示内容覆盖范围，组合包则合并指定的学科层。
这些是内容概念，不代表额外运行时，也不代表每种学科组合都已在所有渠道发布。
使用 `qiongli content --json` 查看当前构建包含的配置档。

## 维护学科指导

源文件位于 `content/subjects/` 及其 `catalog.yaml`，相关档案在
`content/skills/domain-profiles/` 和 `content/venue-profiles/`。
如果差异只是方法或报告规则，优先补充已有 Skill；确实存在独立、可复用的任务时，再新增 Skill。

现有内容生成工具可用于兼容包检查；原生发布包遵循原生构建契约。
不要直接修改 Plugin 缓存，也不要把生成内容包当作发布原生版本。
详见[扩展指南](extend-qiongli.md)和[仓库结构](../development/repository-structure.md)。
