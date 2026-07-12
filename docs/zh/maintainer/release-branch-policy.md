# 发布分支策略

Python 主导的 1.x 已完成验收并冻结。本仓库使用 `2.x` 承接 Rust 原生开发，
以 `release/1.x-python` 保存已验收的 1.x 兼容性 oracle 和 critical-fix
维护线，以 `dev` 保存发布后的交接基线；`main` 继续作为旧稳定发布分支。

## 分支职责

| 分支 | 职责 | 允许的变更 |
|------|------|------------|
| `2.x` | Rust 原生活跃开发、集成和 2.x 预发布源 | 原生 Rust workspace 与产品功能、contract/resource loader、原生 CLI/UI/MCP/orchestrator、installer、测试、文档、CI 和 2.x 发布工具。Python 与 Node 只能作为冻结 oracle 或构建期测试输入，不能成为生产运行时依赖。 |
| `dev` | 已验收的 1.x 交接和发布后基线集成端点 | A8 baseline 证据、分支治理、文档、测试和交接元数据。不再接收 1.x 产品功能，也不承载 Rust 原生产品实现。 |
| `release/1.x-python` | 已验收的 1.x tag、兼容性 oracle 和 critical-fix-only 维护线 | 仅允许通过 PR 修复获批的严重安全问题或发布损坏，以及这些修复所需的最小测试、发布元数据和文档。不接受常规功能。 |
| `main` | 旧稳定发布源 | 稳定发布证据和明确批准的紧急维护。不再进行常规 1.x 功能开发。 |

原生功能 PR 应合入 `2.x`。最终 1.x beta 之后，`dev` 只用于 A8 交接和
跨版本治理。原生实现不得回合到 `dev`、`main` 或
`release/1.x-python`。

## 1.x 维护治理

`release/1.x-python` 指向已验收的 annotated tag `v1.19.0-beta.1`
（`8d2e99866ce4c4efb8b3b5e0265c0c1f89a36b0f`），不是 `dev` 的动态
副本。该分支固定在已验收 tag，因此它**不包含**之后在 `dev` 提交的 A8
workflow filter 变更；不得声称冻结分支已经具有这些后续 workflow 定义。

该维护线由
[ruleset 18797579](https://github.com/jxpeng98/qiongli/rules/18797579)
保护。对 `release/1.x-python` 的修改必须通过 pull request，直接维护推送
不是操作流程。该 ruleset 同时禁止删除和 non-fast-forward 更新，且没有
bypass actor。服务端 ruleset 是实际强制来源，本文记录评审政策。

1.x 维护 PR 只有同时满足以下条件才可接受：

1. 修复严重安全问题或发布损坏，并明确记录例外类型；
2. 在已验收 tag 上复现问题，通过 PR 提交最小安全修复；
3. PR 包含 focused regression tests、适用的完整 release gates，以及与
   变更风险相称的 artifact 或 rollback 证据；
4. 同一行为已经 forward-port 到 Rust 2.x，或者 PR 提供 equivalence
   evidence，说明 Rust 线不受影响并记录后续负责人；
5. 不增加 1.x 功能，也不静默移动冻结 oracle。

计划中的 1.x 支持窗口在 **Qiongli 2 stable 发布后 90 天**结束，除非后续
有明确的新支持决策。安全和发布损坏例外仍需 release owner 决策；该窗口
不代表可以恢复功能开发。

## 2.x 原生分支治理

`2.x` 只能在 normalized 1.x baseline 冻结后，从精确且干净的 A8 交接
commit 创建；此后的原生实现和 2.x 发布工作全部归属该分支。A8 交接
commit 中的 `CI` 与 `Checkout Install Check` workflow 支持针对 `2.x`
的 push 和 pull request。generated-payload guard 在 PR 中解析
`github.base_ref`；push 事件使用事件的前一 commit，并安全回退到可用的
parent/root，而不是把所有分支硬编码为与 `origin/dev` 比较。

frozen-baseline guard 使用同一个 comparison base 作为锚点。如果 base 已经
包含 `tooling/migration/baselines/v1.19.0-beta.1/manifest.json`，CI 会拒绝
修改该版本化 baseline 下的任何文件，也会拒绝修改对应的 1.x baseline
plan 和 JSON Schema。即使同时重写 oracle、manifest 和内部 hash，仍会被
阻止。只有一次性的 A8 落盘 commit，以及从该精确 commit 首次 push
`2.x` 时，comparison base 才允许不存在 anchor；一旦 anchor 进入分支历史，
新的 conformance evidence 必须写入新的版本化路径，不能覆盖冻结的 1.x
证据。因此已验收的 A8 commit 同时是 CI 与 branch point 的信任锚点；初始
runtime outcome 的真实性则由带发布资产的 `capture --check` 重放确认。

首次 push `2.x` 并确认第一轮 `CI` 与 `Checkout Install Check` 通过后，必须
配置 active-development 服务端 ruleset：要求 pull request 和这些精确的
required checks，禁止删除与 non-fast-forward 更新，并且没有 bypass actor。
同一交接点还必须审计现有 `dev` 保护，并记录两条 ruleset identity 或修正
动作。只有当对应 workflow 是 required 时，immutable guard 才能在合入前
阻止变更；没有 branch protection 时，direct push 可能先移动分支，workflow
只能事后报错。

`2.x` 的生产代码必须为 Rust 原生，并保证最终用户零语言运行时依赖。
冻结的 Python Full、Rust Lite 和 Node MCPB 结果只作为兼容性 oracle 与
测试证据，不得变成隐藏的生产依赖。

## 官方 Plugin 接入

公开的官方 marketplace 条目现在由 `jxpeng98/skillsplace` 统一维护，并指向稳定的、生成后的 Qiongli plugin payload：

- Marketplace repository: `https://github.com/jxpeng98/skillsplace`
- Qiongli repository: `https://github.com/jxpeng98/qiongli`
- Stable Codex artifact: `qiongli-core-codex-plugin-<tag>.tar.gz`
- Stable Claude Code artifact: `qiongli-core-claude-plugin-<tag>.tar.gz` 或 `.zip`
- Stable generated payload root: `plugins/qiongli/`

Skillsplace catalog 应跟踪 `main` 和 release tag，而不是 `dev`。A8 交接后，
原生 plugin packaging 测试与预发布验证在 `2.x` 进行；`dev` 只保存 A8
baseline 与治理证据。本仓库不再携带 Codex 或 Claude marketplace catalog
文件，只负责 plugin manifest，并从 canonical source materialize release
payload。

旧版 1.x beta tag 会发布 `qiongli-next` 测试通道，而不是完整的 stable
marketplace matrix。原生 2.x alpha dry-run 不发布任何 dist ref；原生
postflight 会保持阻断，直到 target/package identity 真实且通过验收。旧版
beta 生成的 next artifacts 是：

- `qiongli-next-codex-plugin-<tag>.tar.gz`
- `qiongli-next-claude-plugin-<tag>.tar.gz`
- `qiongli-next-claude-plugin-<tag>.zip`
- `qiongli-next-claude-desktop-skill-core-<tag>.zip`

`qiongli-next` Codex 和 Claude Code plugin artifacts 只安装 `core/complete`
skill package，并保留 bundled Rust Lite literature MCP runtime；不发布
subject-specific plugin variants。Claude plugin ZIP 与 Claude tarball 使用
同一份 plugin payload，用于不接受 `.tar.gz` 的 Claude 上传路径。

本仓库不再跟踪 stable 或 beta plugin payload 目录。`plugins/qiongli/`、`plugins/qiongli-next/`、`packages/qiongli-plugin/`、`packages/qiongli-next-plugin/` 都是生成形状。修改 `content/workflow/`、`content/distribution/plugins.yaml` 或 `tooling/scripts/build_plugin_artifacts.py`，然后 materialize 到 staging 目录做验证。

## 开发流程

1. A8 记录 branch point 后，所有原生功能与 packaging 工作都从 `2.x`
   开始，并通过 PR 合回 `2.x`。
2. 在 PR 的精确 commit 上运行 `CI` 与 `Checkout Install Check`。已经有
   冻结 oracle 的迁移面，还必须提供相对于 A8 baseline 的 equivalence
   evidence。
3. 只有为比较或 artifact 验证时，才把旧 portable payload materialize 到
   staging 目录：

```bash
python3 scripts/materialize_distribution_payloads.py --target all --out /tmp/qiongli-dist --force
```

4. 普通 2.x 工作不得修改冻结的 1.x source 和 baseline。CI 的 immutable
   surface 包括版本化 baseline 目录、`qiongli-1x-baseline-plan.json`、
   `baseline-plan.schema.json`、`baseline-manifest.schema.json` 和
   `oracle-fixture.schema.json`。适用的旧 validator 只作为兼容性证据运行，
   不能成为生产依赖：

```bash
python3 scripts/validate_research_standard.py --strict
python3 -m unittest discover -s tests -v
```

5. 所有 1.x 安全或发布损坏例外必须按照上面的 PR-only 政策进入
   `release/1.x-python`；不得继续把 `dev` 当成功能型 1.x 发布源。
6. B1 原生 preflight 只能作为写入外部 staging 目录的 dry-run。它现在会
   校验 alpha syntax、Cargo version/channel source、独立 channel metadata、
   planned target identity 以及 rollback/promotion 语义。在后续原生产物、
   签名、target acceptance、updater 与公开发布 gates 移除明确的
   `publication_allowed=false` 阻断之前，不得创建或发布 2.x tag。

## 稳定发布规则

已验收的 `v1.19.0-beta.1` 是最后一个计划内、包含功能变更、由 Python
主导的 1.x beta。`main` 继续作为旧稳定源，但不再接收常规 1.x 功能或
release-candidate。任何例外 1.x 发布都必须满足上面的维护决策、PR 证据、
forward-port/equivalence evidence 和 release gates；不得绕过现有 release
automation 的 branch checks。

现有 release automation 仍要求 stable publish mode 从 primary branch 运行，
并在创建 tag 前等待必需的 branch checks；beta publish 则先等待 `dev` 上的
CI/checks，再创建 beta tag 并等待 tag publish workflows。

beta 不是每个 stable release 的必经步骤。只有当 release 改动发布自动化、
package payload、installer、package metadata、CI 或 publish workflows 这类
高风险面时，才需要先用 beta 验证。低风险文档、小修复和维护改动可以直接从
`main` 发 stable。若 stable 没有对应的新 beta，npm `latest` 会前进，npm
`next` 会有意停在上一个 beta；`next` 表示最新预发布验证版，不是必须始终
比 stable 更新的通道。不要为了移动 `next` 而机械发 beta。

2.x stable 与 prerelease 规则随原生发布工具在 `2.x` 上建立。统一
Skillsplace 条目只有在对应原生 release gates 和 artifact acceptance 通过后
才能推进。
