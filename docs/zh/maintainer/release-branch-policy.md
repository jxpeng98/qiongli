# 发布分支策略

Python 主导的 1.x 已完成验收并冻结。本仓库使用 `2.x` 承接 Rust 原生开发，
以 `release/1.x-python` 保存已验收的 1.x 兼容性 oracle 和 critical-fix
维护线，以 `dev` 保存发布后的交接基线。`main` 现承载合并后的原生产品，
作为 2.x 正式版的发布源（ADR 0227）。

## 分支职责

| 分支 | 职责 | 允许的变更 |
|------|------|------------|
| `2.x` | Rust 原生活跃开发、集成和 2.x 预发布源 | 原生 Rust workspace 与产品功能、contract/resource loader、原生 CLI/UI/MCP/orchestrator、installer、测试、文档、CI 和 2.x 发布工具。Python 与 Node 只能作为冻结 oracle 或构建期测试输入，不能成为生产运行时依赖。 |
| `dev` | 已验收的 1.x 交接和发布后基线集成端点 | A8 baseline 证据、分支治理、文档、测试和交接元数据。不再接收 1.x 产品功能，也不承载 Rust 原生产品实现。 |
| `release/1.x-python` | 已验收的 1.x tag、兼容性 oracle 和 critical-fix-only 维护线 | 仅允许通过 PR 修复获批的严重安全问题或发布损坏，以及这些修复所需的最小测试、发布元数据和文档。不接受常规功能。 |
| `main` | 集成后的原生产品和 2.x 正式版发布源 | 审阅后的 2.x 集成、必要修复、原生构建验证和正式版发布元数据。 |

原生功能 PR 应合入 `2.x`。最终 1.x beta 之后，`dev` 只用于 A8 交接和
跨版本治理。收到主线收口或发布指令后，将审阅后的 `2.x` 合入 `main`；
原生实现不得合入 `dev` 或 `release/1.x-python`。

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

当前开发直接在本地完成：从本地 `2.x` 创建分支，修改、运行相关检查、审阅并
提交，然后用 `git merge --ff-only` 合并到本地 `2.x`。不要求 pull request，
也不等待 GitHub CI。维护者的开发指令已覆盖范围内的本地分支、提交和合并。
推送和远端规则调整另属独立操作；下文 GitHub 规则只描述可选远端协作，
不构成本地合并门禁。

`2.x` 只能在 normalized 1.x baseline 冻结后，从精确且干净的 A8 交接
commit 创建。该分支继续承接原生开发和预发布；审阅后的集成进入 `main`，
用于正式交付。

`Native CI` 对以 `main` 或 `2.x` 为目标的 pull request 自动运行；合入后的 push 不会
重复启动。明确创建 candidate 时仍可手动触发。必需检查为：

- `Native 2.x change boundary`；
- `Rust native foundation (Linux)`；
- `Rust native foundation (Windows)`。

ready source PR 在 Actions 的 Linux、Windows 上运行无 GUI workspace 测试；
macOS 改为本机手动运行并保留绑定源码的收据，见[本地验证指南](../../development/local-macos-checks.md)。
format 和 CLI Clippy 只在 Linux 运行一次。共享 native 源码、构建或 Desktop
改动增加 Linux 桌面消费者检查；专用 CLI/MCP 改动跳过前端。只有 Lite 或未知的
工具输入、Lite 的共享 runtime 依赖改动运行独立 Lite compatibility。
draft PR 暂缓 native 测试。

非运行时文档或仅证据 PR 保留轻量 native contexts；未知路径、workflow、fixture
和空 diff 保守运行全部 PR 检查，删除源码仍按源码处理。`workflow_dispatch`
才运行 Linux/Windows 的桌面、Lite、package 和 candidate 检查。macOS
验收、三平台 Community Alpha 汇总和后续授权改由本地现有工具执行。`Evaluation Truth V1`
也只在 PR head 上运行一次；合入后的 push 不会重复启动这两个 workflow。
日常步骤以 [CONTRIBUTING](https://github.com/jxpeng98/qiongli/blob/2.x/CONTRIBUTING.md)
为准，不需要逐阶段人工确认。

`Legacy Compatibility CI` 与
`Legacy Checkout Install Check` 只对 `dev` 和 `release/1.x-python` 自动运行。
需要核查某个明确的兼容性问题时，维护者仍可对指定的 `2.x` ref 手动触发
它们；其结果是诊断证据，不是 2.x 原生开发的 required checks。

不依赖语言运行时的 native change boundary 在 PR 中解析
`github.base_ref`；手动触发时安全回退到可用的 parent/root。它会拒绝修改已
验收的 Python/Node 产品路径、版本化 1.x
baseline 及其 schema，包括
`tooling/migration/baselines/v1.19.0-beta.1/manifest.json`、2.x branch-point
记录和 ADR 0201-0207。更深层的 frozen-baseline guard 与带发布资产的
`capture --check` 仍可在明确的兼容性调查中手动运行；新的 conformance
evidence 必须写入新的版本化路径。

`2.x` 的远端强制来源为 ruleset `18800504`。9 月 15 日读取时，它要求 pull request 和原来的四个
native required contexts（包括已退役的 macOS）以及 `Evaluation Truth V1`。
获得远端同步授权后只移除 macOS 一项；禁止删除与 non-fast-forward 更新，并且没有 bypass
actor。只有当对应 workflow 是 required 时，immutable guard 才能在合入前
阻止变更；没有服务端保护时，direct push 将不会被验证，因为合入后的 push
不会启动 `Native CI`。

`main` 的 push 运行 Native CLI distribution 和 Cargo 源码安装验证，上传仍需
明确的发布动作。旧 TestPyPI builder 仅允许在 `release/1.x-python` 运行；
修改 main 上的 workflow 不会把它自动带到旧维护分支。2026 年 9 月 13 日检查时，
main 没有分支保护；下次集成应重新核查远端规则。

`main` 和 `2.x` 的生产代码必须为 Rust 原生，并保证最终用户零语言运行时依赖。
冻结的 Python Full、Rust Lite 和 Node MCPB 结果只作为兼容性 oracle 与
测试证据，不得变成隐藏的生产依赖。

## 测试层级

只运行与交付边界匹配的最小层级：

1. **Focused**：业务开发过程中，只运行能否定当前改动的最小检查。变更涉及
   security、authorization、schema、path、ownership 或 data-loss 边界时，
   立即运行对应负向检查。
   在 Apple Silicon macOS 上进行原生开发时，可以增加完整 macOS workspace
   测试，并使用下面的第三方 `cargo-xwin` 命令提前获得 Windows x64 编译反馈。
   使用 `cargo-xwin` 即接受 Microsoft SDK 许可，因此首次使用前必须得到维护者
   明确授权。
2. **Slice**：只用于明确请求的可选远端协作。日常走上述本地分支、相关检查、
   提交和合并流程，不需要 PR 或等待 CI。
3. **Acceptance**：仅在明确的 2.x cutover 或 release candidate 上运行三目标
   package、packaged-product 和 Lite candidate acceptance、当前 live Hosts、
   migration/rollback、trust/supply-chain 与所声明的 manual journeys。

自动 `2.x` PR 不组装三目标产品包，不运行 packaged-product 或 Lite candidate
acceptance，也不触发 Community Alpha promotion；合入后的 push 不启动
`Native CI`。Linux/Windows job 只在明确的 `workflow_dispatch` candidate action 中
运行；macOS 和三平台汇总改为本地执行。Slice 通过只代表集成证据，不代表发布授权。

macOS-first 原生开发从 `packages/qiongli-native/` 运行以下命令，以使用仓库
固定的 Rust toolchain：

```bash
cargo test --workspace --all-targets --all-features --locked
cargo xwin build --workspace --release --target x86_64-pc-windows-msvc --locked
cargo xwin test --workspace --no-run --all-features --target x86_64-pc-windows-msvc --locked
```

第二条命令生成 Windows x64 PE/COFF 产物，第三条只编译 Windows 测试
executable；两者都不等于 Windows runtime pass。受影响的启动、持久化和失败
路径仍需在 Windows guest 或 runner 中运行，ready PR 的原生 Windows context
仍是 Slice 权威。Windows 11 Arm 的 x64 模拟适合作为日常证据，但不代表原生
Windows x64 硬件认证、签名、installer 或 release acceptance。

9 月 15 日读取远端 ruleset `18800504` 时，它仍要求
`Rust native foundation (macOS)`。获得远端同步授权后，只移除这一项旧检查，
其余检查及 PR/ref 保护保留。不能用虚假的绿色任务代替本地收据（ADR 0230）。

## 官方 Plugin 接入

原生发布包含六个按平台区分的 Codex/Claude archive 和
`marketplace-plugins.json`，每个包都携带同版本原生程序。正式版保留
`qiongli-next-<target>` 安装标识，便于现有用户升级；正式版显示 Qiongli，
预发布显示 Qiongli Next。这里的 `next` 是既有 Plugin 标识，不决定 npm 通道。
`jxpeng98/skillsplace` 的目录更新需单独审阅，使用不可变的
`<host>/<target>/v<version>` 分发引用。参见 ADR 0223、0227。
下面的通用包名和分发流程仅描述保留的 **1.x 历史实现**。

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
发布由单独的 Actions owner 使用已验证的平台产物完成；旧 postflight 仍拒绝
原生发布。旧版
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

1. 原生功能和 packaging 工作从本地 `2.x` 开始，通过本地提交和 fast-forward
   合并完成集成。仅在需要远端协作时开 PR；合入 `main` 需明确的收口指令。
2. 开发过程中运行 Focused 检查；切片仍在变化时保持 draft，draft 事件不展开
   原生矩阵。ready 后在 PR 的精确 commit 上运行 `Native CI`。影响 source 的
   改动必须通过 format、check、Clippy、workspace tests、Linux 可移植前端检查、
   Lite compatibility 和冻结边界检查；allowlist 内的非运行时文档或仅证据收尾
   只运行边界与轻量 required contexts。合入后的 push 不重复运行。只有在核查
   明确的兼容性问题时才手动触发旧 workflow；已经有冻结 oracle 的迁移面还应记录
   equivalence evidence。
   Apple Silicon 维护者可在 Slice 前使用 macOS workspace 与上面的
   `cargo-xwin` build/test-compilation 循环，但它不能替代 Windows runtime 或
   required-CI 证据。
3. 只有为比较或 artifact 验证时，才把旧 portable payload materialize 到
   staging 目录：

```bash
python3 scripts/materialize_distribution_payloads.py --target all --out /tmp/qiongli-dist --force
```

4. 普通 2.x 工作不得修改冻结的 1.x source 和 baseline。CI 的 immutable
   surface 包括版本化 baseline 目录、`qiongli-1x-baseline-plan.json`、
   `baseline-plan.schema.json`、`baseline-manifest.schema.json` 和
   `oracle-fixture.schema.json`。适用的旧 validator 只作为手动兼容性证据
   运行，不能成为 2.x required check 或生产依赖：

```bash
python3 scripts/validate_research_standard.py --strict
python3 -m unittest discover -s tests -v
```

5. 所有 1.x 安全或发布损坏例外必须按照上面的 PR-only 政策进入
   `release/1.x-python`；不得继续把 `dev` 当成功能型 1.x 发布源。
6. 明确创建 2.x candidate 时，针对冻结的 `2.x` source 手动 dispatch
   `Native CI`，统一运行三目标 package assembly、packaged acceptance、Lite
   candidate acceptance 和现有 exact promotion dispatch。自动 PR Slice 不得
   作为 candidate 或发布授权。
7. 保留的 B1 preflight 只生成只读计划，校验原生版本、通道、源码身份和回退语义，
   始终保留 `publication_allowed=false`。CLI 验证和发布走 CONTRIBUTING 中的
   原生流程；诊断计划不会授予发布权限。

## 稳定发布规则

原生正式版从冻结的 `main` head 发布，Beta/Alpha 开发继续在 `2.x`。
本地安装包验证允许干净的这两个分支。发布必须在 Actions 中，使用版本与源码
匹配的不可变 tag、已审阅的说明、通过三平台验证的产物。正式版还要求 tag
等于触发发布时的远端 main；发布过程中保持主线冻结。GitHub 将正式版标为
latest，npm 正式版用 `latest`，预发布用 `next`；PyPI wheel 和 Cargo crate
使用同一个原生版本。合并或构建通过本身不代表发布完成，也不提升项目验收状态。

`v1.19.0-beta.1` 仍是已验收的 Python oracle。1.x 例外维护进入
`release/1.x-python`，遵循其 PR 规则并保留 forward-port/equivalence evidence。
旧发布工具需要修复时，另做范围明确的维护变更，不恢复从原生 main 发布 Python
旧包。已有 Release、分发引用和已验收基线保持不可变。
