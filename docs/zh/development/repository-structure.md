# 仓库结构

这一页是当前维护者应遵守的 source layout contract。

```text
/
  content/                  学术内容 canonical source
    workflow/               生成 qiongli-workflow package 的源
    distribution/           生成 plugin payload 的 metadata 源
    skills/                 internal skill specs
    templates/              可复用 artifact templates
    standards/              contracts、capability maps、policies
    roles/                  functional-agent role configs
    subjects/               subject catalog 与 overlays
    schemas/                JSON/YAML schemas
    venue-profiles/         venue profile data

  packages/
    python-qiongli/         Python package source 与兼容 shim
    npm-qiongli/            npm wrapper package source
    qiongli-literature-mcpb/ MCPB package source

  tooling/
    scripts/                真实维护脚本实现
    pipelines/              paper-type DAG descriptors
    install/                installer manifests 与支持资产
    release/                release docs、receipts、rollback assets

  evals/                    eval cases、rubrics、runner assets
  tests/                    跨包回归测试
  docs/                     VitePress 文档
  scripts/                  稳定 wrapper entrypoints
```

## 生成 Artifact 形状

这些路径可能在 staging 或本地维护时出现，但不是 canonical source：

- `qiongli-workflow/`
- `plugins/qiongli/`
- `plugins/qiongli-next/`
- `.agent/`
- `packages/python-qiongli/src/qiongli/payload/`
- `packages/npm-qiongli/payload/`
- `packages/npm-qiongli/python-runtime/`
- `packages/qiongli-plugin/`
- `packages/qiongli-next-plugin/`

使用 staged materialization 生成：

```bash
python3 scripts/materialize_distribution_payloads.py --target all --out /tmp/qiongli-dist --force
```

## 兼容边界

- 根目录 `scripts/` 为 CI、文档和用户习惯保持稳定。除非 wrapper contract 本身变化，否则编辑 `tooling/scripts/`。
- `research_skills` 作为 deprecated Python compatibility shim 保留在 `packages/python-qiongli/src/research_skills/`。
- 根目录 `.agent/` 由 `content/workflow/` 与 `content/distribution/plugins.yaml` 生成。
- 根目录 `qiongli-workflow/` 由 `content/workflow/` 和同步后的 content mirrors 生成。
- `plugins/qiongli/`、`plugins/qiongli-next/`、`packages/qiongli-plugin/`、`packages/qiongli-next-plugin/` 都是生成后的 plugin payload 形状，不是 source directory。
