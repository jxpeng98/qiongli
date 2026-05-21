# 命名策略

对外主名使用 **穷理**。

完整体系名使用 **穷理证澈**。

方法论或核心证据治理模块名使用 **证澈**。

## 命名职责

| 范围 | 名称 | 用法 |
|------|------|------|
| 对外产品、plugin display、文档标题 | Qiongli / `穷理` | 用于 marketplace、介绍文案和一般传播。 |
| 完整体系、长说明 | Qiongli Zhengche / `穷理证澈` | 用于描述完整的学术研究 workflow 系统。 |
| 方法论、evidence ledger、citation risk、claim traceability | Zhengche / `证澈` | 用于描述让 claim、引用、假设和推理可审计的核心方法。 |
| 技术标识 | plugin/PyPI/CLI 使用 `qiongli`，便携 skill 使用 `qiongli-workflow` | 用于 manifest、安装路径、release scripts 和 package metadata。 |
| 旧兼容别名 | `research-skills`、`research_skills`、`rsk`、`rsw` | 在迁移窗口内继续作为兼容入口保留。 |

## 解释

`穷理` 是对外身份：围绕研究问题，追究其理，追到证据、方法和论证结构。

`证澈` 是核心方法：让证据、引用、假设和推理链条清澈、透明、可审。

`穷理证澈` 则表示完整体系：既能深入追问研究问题，也能保持证据链和学术判断可追踪。

## 迁移规则

新的公开和技术表面统一使用 Qiongli 标识：

- Repository: `https://github.com/jxpeng98/qiongli`
- Plugin ID: `qiongli`
- Portable skill ID: `qiongli-workflow`
- CLI: `qiongli`、`ql`
- Python distribution: `qiongli`

旧别名（`research-skills`、`research_skills`、`rsk`、`rsw`）在迁移窗口内继续保留。未来如果要移除旧别名，必须作为单独的 breaking-change release 处理，并写清升级说明。
