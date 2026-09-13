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
    mcp-contracts/          runtime capability registry、schemas 与 fixtures
    roles/                  functional-agent role configs
    subjects/               subject catalog 与 overlays
    schemas/                JSON/YAML schemas
    venue-profiles/         venue profile data

  packages/
    qiongli-native/         canonical Rust-native 2.x workspace 与产品 App
    qiongli-desktop/        编译进原生 App 的 Svelte 5 桌面 UI
    qiongli-app-api/        有类型的 frontend/native IPC contract 与校验
    qiongli-lite-mcp/       冻结的 Rust Lite compatibility package
    python-qiongli/         Python package source 与兼容 shim
    npm-qiongli/            npm wrapper package source
    qiongli-literature-mcpb/ MCPB package source
    qiongli-zotero-companion/ Zotero companion package source

  tooling/
    architecture/           machine-readable native decision inventory
    quality/                repository-only source policy 与 debt baseline
    scripts/                真实维护脚本实现
    pipelines/              paper-type DAG descriptors
    install/                installer manifests 与支持资产
    release/                release docs、receipts、rollback assets

  evals/                    eval cases、rubrics、runner assets
  tests/                    跨包回归测试
  docs/                     VitePress 文档
    architecture/decisions/ 已接受或取代的 native ADR
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
- `content/mcp-contracts/` 是 canonical MCP runtime-contract boundary，不是学术标准；repository-only RC1 工程策略必须保留在 `tooling/quality/`，不能被 materialize。
- `packages/qiongli-native/` 是唯一的 Qiongli 2 native workspace，拥有单一的 `apps/qiongli` 产品 executable；native service crates 必须保留在该 workspace 下，不能复制进生成 plugin。
- `packages/qiongli-desktop/` 负责 Svelte UI，并生成供 Tauri App 使用的静态 `build/`；`packages/qiongli-app-api/` 负责有类型的 IPC contract，UI component 不得复制原生 service logic。
- `research_skills` 作为 deprecated Python compatibility shim 保留在 `packages/python-qiongli/src/research_skills/`。
- 根目录 `.agent/` 由 `content/workflow/` 与 `content/distribution/plugins.yaml` 生成。
- 根目录 `qiongli-workflow/` 由 `content/workflow/` 和同步后的 content mirrors 生成。
- `docs/architecture/decisions/` 是经过 review 的 Qiongli 2 架构决策源；`tooling/architecture/` 保存 validation records，而不是 runtime payload 或 marketplace metadata。
- `plugins/qiongli/`、`plugins/qiongli-next/`、`packages/qiongli-plugin/`、`packages/qiongli-next-plugin/` 都是生成后的 plugin payload 形状，不是 source directory。
