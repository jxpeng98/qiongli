# Upgrade / Auto-Upgrade Guide (No Fork Required)

本指南说明如何在使用 `qiongli` 时：
1) 检测是否有新版本；2) 自动化升级；3) 在不 fork、不 git clone 的情况下完成升级。

## 0) 选择升级入口

### 方案 A：Shell bootstrap（不依赖 Python）

这个路径只需要 `bash` 和 `curl`/`wget`、`tar`：

```bash
curl -fsSL https://raw.githubusercontent.com/jxpeng98/qiongli/main/scripts/bootstrap_qiongli.sh | bash -s -- \
  --repo <owner>/<repo> \
  --project-dir /path/to/project \
  --target all \
  --overwrite
```

说明：
- bootstrap 会下载所选 release 压缩包，并执行其中自带的 `scripts/install_qiongli.sh`。
- 默认也会安装 shell CLI：`qiongli`、`ql`、`research-skills`、`rsk`、`rsw`。
- 默认 shell CLI 目录：`${QIONGLI_BIN_DIR:-${RESEARCH_SKILLS_BIN_DIR:-~/.local/bin}}`。
- 如果只想更新 workflow 资产，可加 `--no-cli`；如果要改安装位置，可加 `--cli-dir <path>`。
- `--doctor` 是可选项，只有系统存在 `python3` 时才会执行。
- 远程 bootstrap 只支持 `--mode copy`；如果你需要 `--mode link`，请保留本地 clone。

### 方案 B：Python CLI（可选）

本仓库也提供 `pyproject.toml` 包，适合需要可复用升级器 CLI 的场景：

```bash
pipx install qiongli
# 提供 3 个等价命令（任选其一）：
# - qiongli
# - rsk
# - rsw
# 你也可以设置 `QIONGLI_REPO=<owner>/<repo>` 后省略 --repo
qiongli check --repo <owner>/<repo>
qiongli upgrade --repo <owner>/<repo> --target all --doctor
qiongli init --project-dir /path/to/project
```

> 注意：pip 安装/升级的是 qiongli CLI package；真正刷新三端全局 skill 目录的动作，仍由 `qiongli upgrade` 来执行。项目内文件现在改为显式更新：需要时使用 `qiongli init` 或 `qiongli upgrade --parts project ...`。

`qiongli upgrade` 是内容/assets 刷新命令，不会升级已安装的 qiongli CLI package。在 npm/npx 下，`update`、`refresh` 和 `upgrade` 都只会从当前已安装 npm package 重新应用 assets；`upgrade` 是覆盖式刷新别名。指定上游 release archive、channel/package self-update 和 `qiongli self-update` 属于完整运行时路径：先 `pipx install qiongli`。

## 1) 你需要升级的到底是什么？

这个项目只有一类“安装目标”：

- **全局 skill 安装目录**（让 Codex / Claude Code / Antigravity / Hermes 原生识别 skill 或 workflow 入口）
  - Codex: `${CODEX_HOME:-~/.codex}/skills/qiongli-workflow`
  - Claude: `${CLAUDE_CODE_HOME:-~/.claude}/skills/qiongli-workflow`
  - Antigravity（全局）: `${ANTIGRAVITY_HOME:-~/.gemini/antigravity}/skills/qiongli-workflow`
  - Hermes: `${HERMES_HOME:-~/.hermes}/skills/qiongli-workflow`

升级的本质就是：**把这些目标路径覆盖为新版本**。

_注：项目内文件（如 `.env`）只有在需要时显式执行 `qiongli init --project-dir .` 或 `qiongli upgrade --parts project` 才会更新。_

---

## 2) 检测是否有新版本（推荐）

```bash
# 如果已设置 QIONGLI_REPO，可省略 --repo
qiongli check --repo <owner>/<repo>
# 或在仓库内运行（等价）：
python3 scripts/qiongli_update.py check --repo <owner>/<repo>
```

说明：
- `--repo` 用于查询 GitHub 最新 release tag。
- 若检测到“本地/已安装版本 < 最新版本”，该命令会返回 exit code `1`（方便写自动化）。
- 你可以设置默认上游来省略 `--repo`：
  - 环境变量：`export QIONGLI_REPO=<owner>/<repo>`
  - 若你在 `qiongli` 仓库 clone 里运行，且已配置 git remote（优先 `upstream`，其次 `origin`），也可省略 `--repo`
  - 或在你的项目根目录添加 `qiongli.toml`（便于提交到项目仓库，适合 CI）

示例（项目根目录）：

```toml
# qiongli.toml
[upstream]
repo = "<owner>/<repo>" # 或 Git URL
```

此后可直接运行：

```bash
qiongli check
qiongli upgrade --target all --doctor
qiongli init --project-dir .
```

---

## 3) 自动升级（不需要 fork，不需要 git clone）

直接下载 GitHub release 压缩包并执行其中的安装脚本：

```bash
curl -fsSL https://raw.githubusercontent.com/jxpeng98/qiongli/main/scripts/bootstrap_qiongli.sh | bash -s -- \
  --repo <owner>/<repo> \
  --project-dir /path/to/your/project \
  --target all \
  --overwrite
```

如果机器上有 Python，也可以继续使用 CLI：

```bash
# 如果已设置 QIONGLI_REPO，可省略 --repo
qiongli upgrade \
  --repo <owner>/<repo> \
  --target all \
  --mode copy \
  --doctor

# 或在仓库内运行（等价）：
python3 scripts/qiongli_update.py upgrade \
  --repo <owner>/<repo> \
  --target all \
  --mode copy \
  --doctor
```

要点：
- 这个方式**不依赖 git**，也不要求你把仓库 clone 到本地。
- shell bootstrap 路径**不依赖 Python**。
- shell CLI 本身也可以在无 Python 环境下执行 `check`、`upgrade`、`align`。
- 从 v1.9.0 开始，完整运行时默认 upgrade 是 plugin-first：先安装完整本地 plugin surface，再在新安装成功后清理旧 global skills 和 Codex/Claude 独立 MCP config。
- npm/npx upgrade 仍属于免 Python asset 路径，默认刷新 skills surface；只有 bundled/supported 的 plugin-lite 输出需要用 `--surface plugin` 或 `--surface both` 显式启用。
- 如果在完整运行时里明确要保留旧版 skills-only 升级路径，使用 `--surface skills --profile partial`。
- 只有显式加 `--parts project` 时，才会刷新项目内 workflow 资产。
- 私有仓库或遇到 API 限流时，建议设置：`GITHUB_TOKEN` 或 `GH_TOKEN`。
- 默认使用“最新 release tag”；shell bootstrap 和 `qiongli upgrade` 也都支持显式指定版本：
  - `--ref v0.1.0-beta.6 --ref-type tag`
  - `--ref main --ref-type branch`

升级后建议重启客户端（Codex / Claude Code / Antigravity / Hermes）。

---

## 4) 另一种“自动升级”：link 安装 + git pull（适合长期维护）

如果你愿意保留一份本地仓库（不需要 fork，只需 clone 一次），推荐：

1) 安装时用 `--mode link`（用软链接指向仓库，后续更新无需重复 install）：

```bash
./scripts/install_qiongli.sh --target all --mode link --overwrite
python3 -m qiongli.cli init --project-dir /path/to/project --target all --overwrite
```

2) 更新时只需：

```bash
git pull
```

因为安装目标是软链接，仓库内容更新后，三端 skill 与 workflows 会自动同步到最新版本。

---

## 5) 自动化建议（可选）

你可以用 cron/CI 做“每周检查 + 有更新则升级”：

1) 定期 check：
```bash
qiongli check --repo <owner>/<repo>
```
2) 返回码为 1 时执行 upgrade：
```bash
qiongli upgrade --repo <owner>/<repo> --target all
qiongli init --project-dir /path/to/project
```

如果你希望我把这套升级检测做成 Codex Automation（定期跑并生成 inbox 结果），告诉我运行频率和要覆盖的 project 路径即可。
