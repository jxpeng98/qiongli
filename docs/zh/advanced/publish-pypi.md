# 发布指南（PyPI Package Publishing）

本指南说明如何将 `qiongli` 发布到 PyPI，以及日常版本发布的完整流程。

## 0) 前置条件（一次性配置）

### 0.1 PyPI Trusted Publisher

本项目使用 [Trusted Publisher](https://docs.pypi.org/trusted-publishers/) 机制发布（无需管理 API Token）。

1. 登录 [pypi.org](https://pypi.org)，进入你的账号
2. 如果是**首次发布**（PyPI 上还没有这个包），进入 [Publishing](https://pypi.org/manage/account/publishing/) 页面，在 "Add a new pending publisher" 中填写：
   - **PyPI Project Name**: `qiongli`
   - **Owner**: `jxpeng98`
   - **Repository name**: `qiongli`
   - **Workflow name**: `publish-pypi.yml`
   - **Environment name**: `pypi`
3. 如果**已有该包**，进入包的 Settings → Publishing → "Add a new publisher"，填写同上

### 0.2 GitHub Environment

1. 进入 GitHub 仓库 Settings → Environments
2. 点击 "New environment"，名称填 `pypi`
3. （可选）添加 protection rules（如仅允许 `main` 分支部署、需要审批等）

### 0.3 TestPyPI Trusted Publisher

本仓库已提供 TestPyPI 专用 workflow：`.github/workflows/publish-testpypi.yml`。

1. 登录 [test.pypi.org](https://test.pypi.org)
2. 进入 Account settings → Publishing
3. 添加 pending publisher（或在已有项目下添加 publisher），填写：
   - **PyPI Project Name**: `qiongli`
   - **Owner**: `jxpeng98`
   - **Repository name**: `qiongli`
   - **Workflow name**: `publish-testpypi.yml`
   - **Environment name**: `testpypi`
4. 回到 GitHub 仓库 Settings → Environments，创建环境 `testpypi`

---

## 1) 日常发布流程

### 1.1 一条命令完成发布闭环

推荐维护者直接执行：

```bash
./scripts/release_automation.sh publish --version 0.2.0 --from-tag v0.1.0
./scripts/release_automation.sh publish --version 0.2.0b1 --from-tag v0.2.0
```

`publish` 是标准发布入口。它会自动走完整本地和远端发布闭环：

1. 规范版本号并运行 `release_ready.sh`
2. 自动提交 release-prep 文件
3. 创建并 push release tag
4. 让 tag 触发 GitHub Actions，发布到 PyPI 和 npm
5. 等待 release commit 上必要的 branch workflows：
   - `CI`
   - `Checkout Install Check`
6. 等待必要的 tag publish workflows：
   - `Publish to PyPI`
   - `Publish to npm`
7. 执行 postflight：创建或更新 GitHub Release、上传 plugin artifacts，并写入 acceptance receipt

日常生产发布必须走 `./scripts/release_automation.sh publish`。生产 publish workflows 只是 tag 触发后的执行面，不再作为手动发布入口。

你可以传入稳定版 `0.2.0`，也可以传入 beta 版 `0.2.0b1`。automation 会自动规范成三种表示：

| 层 | 稳定版 | Beta |
|------|------|------|
| PyPI package | `0.2.0` | `0.2.0b1` |
| Skill metadata / registry | `0.2.0` | `0.2.0-beta.1` |
| Portable skill `VERSION` / git tag | `v0.2.0` | `v0.2.0-beta.1` |

其中 package 版本遵循 [PEP 440](https://peps.python.org/pep-0440/)，skill metadata 使用 SemVer 兼容的 prerelease 语法。当前 release tooling 只支持 `stable` 和 `beta`。

默认的 release smoke tier 是保守配置：内置 literature smoke + `doctor`。如果你还想在发版前补跑更重的 `parallel` / `task-run` profile 路径检查，再显式加 `--maintainer-smoke`。

当前 release 文档策略：

- stable 正式版统一维护在 `CHANGELOG.md`
- beta / prerelease 继续使用 `release/<tag>.md`

Beta 通道策略：

- beta 是可选的预发布验证版本，不是每次 stable 前的必经步骤
- 当 release 改动发布自动化、package payload、installer、package metadata、CI 或 publish workflows 这类高风险面时，应先发 beta 验证
- 文档、小修复和低风险维护可以直接发 stable
- 如果 stable 没有对应的新 beta，npm `latest` 会前进，npm `next` 会继续停在上一个 beta
- `next` 表示“最新预发布验证版”，不保证比 `latest` 新；不要为了移动 `next` 而机械发 beta

### 1.2 干跑 / 拆分阶段

如果你只想本地准备和验证，不想创建 tag，可以先跑 `release_ready.sh`：

```bash
./scripts/release_ready.sh --version 0.2.0
./scripts/release_ready.sh --version 0.2.0b1 --from-tag v0.2.0
```

`release_ready.sh` 会执行版本同步、strict validator、仓库单元测试、release-tier smoke、release note evidence 更新、包构建检查、`twine check` 和 wheel 安装 smoke。它不会创建 tag，也不会 push。commit、tag、push、等待 branch CI、等待 tag publish、创建 GitHub Release、上传 plugin artifacts、生成 acceptance receipt 都由 `publish` 模式负责。

如果 beta release 的 GitHub Actions 可能超过本地等待窗口，可以使用 bounded soft wait：

```bash
./scripts/release_automation.sh publish --version 0.15.0b2 --from-tag v0.15.0-beta.1 --ci-timeout-seconds 900 --ci-timeout-mode soft
```

soft 模式仍然会在 workflow 已完成且失败时退出；它只会在 CI 仍 pending 或暂时无法查询时继续，并把状态写入 acceptance receipt。stable release 应继续使用默认 hard 模式。

如果你确实需要拆开执行，入口仍然保留：

```bash
./scripts/release_automation.sh pre --tag v0.2.0 --from-tag v0.1.0
./scripts/release_automation.sh post --tag v0.2.0 --create-release
```

---

## 2) Tag push 之后会发生什么

`publish` 会创建并 push 以 `v*` 开头的 release tag，例如 `v0.2.0` 或 `v0.2.0-beta.1`。这个 tag 会触发 `publish-pypi.yml`：

1. Checkout 代码
2. 运行 `inject_project_toml.sh`（把当前仓库 slug 写入 `qiongli/project.toml`）
3. `python -m build` 构建 sdist + wheel
4. `twine check` 验证包元数据
5. 使用 Trusted Publisher 发布到 PyPI

同一个 tag 也会触发 `publish-npm.yml`，它会校验 bundled npm package，并把 stable 版本发布到 `latest`，把 beta 版本发布到 `next`。

之后 postflight 会等待 release commit 上的 `CI` 和 `Checkout Install Check`。如果必需 workflow 没有匹配到，诊断会同时打印该 commit 上实际观察到的 workflow 名称。

---

## 3) 本地验证（手动 / 可选）

如果你想绕开 `release_ready.sh` 单独跑包预检，可以执行：

```bash
bash scripts/pypi_preflight.sh
bash scripts/pypi_preflight.sh --no-build
```

等价的手动步骤如下：

```bash
# 安装构建工具
pip install build twine

# 注入上游 repo 信息
bash scripts/inject_project_toml.sh

# 构建
python -m build

# 验证
twine check dist/*
```

本地试装：

```bash
pip install dist/qiongli_installer-*.whl
qiongli --help
qiongli check --repo jxpeng98/qiongli
```

---

## 4) 发布到 TestPyPI（建议先做）

使用 GitHub Actions workflow（手动触发，无需打 tag）：

1. 打开 GitHub Actions
2. 选择 **Publish to TestPyPI**
3. 在目标分支点击 **Run workflow**

该 workflow 会自动构建、校验并通过 Trusted Publishing 发布到 TestPyPI。

发布后从 TestPyPI 安装验证：

```bash
pip install --index-url https://test.pypi.org/simple/ qiongli
```

推荐顺序：

- 先运行 **Publish to TestPyPI**，验证安装与 CLI 功能
- 验证通过后，再 push `v*` tag 触发正式 **Publish to PyPI**

---

## 5) 完整发布 Checklist

发版时按以下步骤执行：

- [ ] 确认所有功能已合入 `main`
- [ ] CI 通过（`ci.yml` 绿色）
- [ ] 可选：先运行 `./scripts/release_ready.sh --version <version>` 做本地干跑
- [ ] 运行 `./scripts/release_automation.sh publish --version <version> --from-tag <previous-tag>`
- [ ] 在 GitHub Actions 确认 `Publish to PyPI`、`CI` 和 `Checkout Install Check` 成功
- [ ] 确认 postflight 创建或更新 GitHub Release，并生成 `release/acceptance/<tag>-receipt.md`
- [ ] 验证安装：`pipx install qiongli && rsk --help`

---

## 6) 常见问题

### Q: tag 推送后 Actions 没有触发？

确认 tag 格式是 `v` 开头（如 `v0.1.0-beta.7`），且 `.github/workflows/publish-pypi.yml` 文件已在 `main` 分支上。

### Q: PyPI 发布失败 "403 Forbidden"？

通常是 Trusted Publisher 配置问题：

- 确认 PyPI 上的 workflow name 完全匹配 `publish-pypi.yml`
- 确认 GitHub environment name 完全匹配 `pypi`
- 确认 owner 和 repository name 正确

### Q: 版本号已存在导致上传失败？

PyPI 不允许覆盖已发布的版本。如果需要修复，必须递增版本号（如 `0.1.0b7` → `0.1.0b8`）。

### Q: TestPyPI 是自动触发吗？

不是。`publish-testpypi.yml` 仅支持 `workflow_dispatch`（手动触发），不会增加仓库 tag。
正式 PyPI 仍由 `publish-pypi.yml` 在 `v*` tag 上自动触发。

### Q: 如何撤回一个已发布的版本？

在 PyPI 项目页面可以 "yank" 一个版本（不会从已安装用户处删除，但 `pip install` 不会默认选择被 yank 的版本）。
