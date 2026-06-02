# 规范约定

## 术语

- `content/` 是学术内容的 canonical source。
- `packages/` 放所有可安装或可发布的包源。
- `tooling/` 放维护自动化和运营资产。
- 根目录 `scripts/` 是稳定 wrapper 层；真正实现编辑 `tooling/scripts/`。
- 根目录 `qiongli-workflow/`、`plugins/qiongli/`、`.agent/`、`.gemini/` 是生成后的 distribution 形状；源文件在 `content/` 或 `packages/qiongli-plugin/`。

## 编辑顺序

跨多层修改时，优先按这个顺序：

1. `content/standards/`：contract 或 routing 真源。
2. `content/roles/` 与 `content/skills/`：责任归属或执行行为。
3. `content/templates/`：稳定结构化输出。
4. `tooling/pipelines/`、`content/workflow/workflows/`、`packages/qiongli-plugin/platforms/`：编排或入口 UX。
5. `packages/python-qiongli/src/qiongli/`：只有运行时执行逻辑需要变化时才改。
6. generated payloads：只能通过 staged materialization 生成。

## 修改落点

| 变化类型 | 放这里 |
|---|---|
| artifact paths、task outputs、quality gates | `content/standards/research-workflow-contract.yaml` |
| runtime routing、MCP requirements、skill requirements | `content/standards/mcp-agent-capability-map.yaml` |
| functional ownership、thresholds、tone | `content/roles/` |
| 可复用 task 行为 | `content/skills/` |
| 可复用 markdown/table 结构 | `content/templates/` |
| subject catalog 或 overlays | `content/subjects/` |
| domain/venue profile 数据 | `content/skills/domain-profiles/`、`content/venue-profiles/` |
| pipeline 编排 | `tooling/pipelines/` |
| 维护自动化 | `tooling/scripts/` |
| public script 入口兼容 | 根目录 `scripts/` wrapper，仅在兼容层变化时 |
| Python runtime、CLI、installer、bridges | `packages/python-qiongli/src/qiongli/` |
| plugin manifest 或 command source | `packages/qiongli-plugin/` |
| Agent/Gemini platform command source | `packages/qiongli-plugin/platforms/` |
| npm package wrapper | `packages/npm-qiongli/` |
| eval cases、rubrics、runners | `evals/` |

## 新增 Skill 的门槛

只有同时满足以下条件，才新增 internal top-level skill：

1. 有 typed inputs 和 typed outputs。
2. 拥有 `RESEARCH/[topic]/` 下稳定 artifact path。
3. 值得被 pipeline 或 task 直接依赖。
4. 有独立 failure modes、review expectations 或 quality-gate 价值。

否则优先扩展现有 skill、template、provider adapter、role 或 pipeline step。

## Generated Output 规则

不要直接编辑生成 payload。使用：

```bash
python3 scripts/materialize_distribution_payloads.py --target all --out /tmp/qiongli-dist --force
```

受 ignore 和 guard 保护的生成路径包括：

- `qiongli-workflow/`
- `plugins/qiongli/`
- `.agent/`
- `.gemini/`
- `packages/python-qiongli/src/qiongli/payload/`
- `packages/npm-qiongli/payload/`
- `packages/npm-qiongli/python-runtime/`

## `research_skills`

`research_skills` 是位于 `packages/python-qiongli/src/research_skills/` 的 deprecated compatibility shim。迁移窗口内保持兼容，但新 import 和新文档应使用 `qiongli`。
