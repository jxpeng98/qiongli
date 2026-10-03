# 发布 Qiongli 原生渠道包

2.x 的 PyPI 包包含 Python 启动器和原生程序；npm 包包含 Node 启动器和原生程序，
Cargo 发布 Rust 源码包。GitHub Release 压缩包则包含可执行文件、README 和许可证。
各渠道使用同一产品版本与 CLI 描述，同时保留各自的安装说明。
2.x 不会把旧 Python 应用作为运行时发布。

## 版本与来源

版本来源是 `packages/qiongli-native/Cargo.toml`。
例如，`2.0.0-beta.6` 在 Python 中写作 `2.0.0b6`，正式版 `2.0.0` 在各渠道保持同一版本。
预发布来自 `2.x`，使用 npm 的 `next` 和 crates.io 的预发布版本；
正式版来自审阅后的 `main`，使用 npm 的 `latest`。Cargo 没有 `next` 标签。
当前源码版本为 `2.1.1`；发布需要下述绑定标签的工作流，合入 main 本身不会发布。

## Beta 通道策略

Beta 并非每次正式发布的必经步骤。正式版推进 npm `latest`，`next` 可以保留在此前的测试版，
不必只为移动 `next` 再发一个 Beta。创建 tag 前先确定目标渠道。

## 先验证，再发布

按[发布分支策略](../maintainer/release-branch-policy.md)和 `tooling/release/automation.md`
执行。Native CLI distribution 会在 macOS ARM64、Windows x64 和 Linux x64/ARM64 构建并验证安装。
两个 Linux 目标均使用 Ubuntu 22.04，wheel 须审计为 glibc 2.35+ 兼容。
新发布包必须包含四个目标的验证记录；ARM64 修复须在 2.1.1 之后的新版本发布。
组装阶段核对平台产物、版本、摘要和生成的渠道包。Cargo 另有源码包检查。
一次源码构建通过，不能代替完整发布检查。

本地原生 CLI 发布检查沿用现有入口：

```sh
bash scripts/release_ready.sh --version 2.1.1 --cli-github \
  --staging-dir /tmp/qiongli-2.1.1-qualified
```

准备新版本时替换为目标版本，使用仓库之外尚不存在的暂存目录，完成所需检查并获得明确发布授权后，再创建不可改写的标签。
仅推送标签不会发布原生包。需要调度 **Release Automation**（`release-automation.yml`），
设置 `mode=post`、准确的 `v2.*` 标签及 `create_release=true`。
其中的原生发布器验证指定源码，并将核验后的产物交给 Release 和各渠道发布工作流。

## 发布凭据

PyPI 使用为 `publish-pypi.yml` 配置的 Trusted Publisher；npm 使用已配置的发布凭据。
Cargo 目前在 Actions 中读取仓库的 `CARGO_REGISTRY_TOKEN`，不会退回本地上传。
密钥不应进入发布说明、包内容或日志。

旧 Python 构建器和 TestPyPI 路径属于 `release/1.x-python`。
修改 main 的工作流，不会同步改变冻结分支。旧版预检脚本用于兼容检查，不能作为原生发布证据。
只有维护 1.x 时，才查阅[保留的旧版发布指南](https://github.com/jxpeng98/qiongli/blob/5a3ab87fcba67dfbe700f895c0321e455bbbc914/docs/zh/advanced/publish-pypi.md)。

已发布版本和附件不可改写。包有问题时，应通过同样的检查发布新版本，而不是替换原标签下的文件。
