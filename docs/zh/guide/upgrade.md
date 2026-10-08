# 升级与回退

先[备份项目和设置](data-lifecycle.md#备份与恢复)，再更新 CLI 和 Plugin。

## 更新 CLI

通过原安装渠道升级，任选对应的一条：

```sh
npm install --global qiongli@latest
python -m pip install --upgrade "qiongli==2.5.1"
cargo install qiongli --version 2.5.1 --locked
```

直接下载用户：从[安装页](install.md)下载新版本，核对校验值后解压到新目录，再切换 PATH 或程序路径。保留旧程序与备份。

用 `qiongli --version` 确认当前运行的是 2.5.1。`qiongli upgrade cli` 只显示升级方法，不执行包管理器。

## 刷新 Plugin

```sh
qiongli install plugin
qiongli mcp check
```

选择 Host，复用已登记的目录，审阅并确认文件与注册变更。`qiongli upgrade plugin` 是同一流程。随后新开 Host 会话，检查工具并调用 `qiongli_config_status`。

DeepSeek 使用自己的插件管理器；向导会安装与 CLI 匹配的 npm 版本。直接通过 DSH 安装时，见 [DeepSeek 配置](../advanced/plugin-installation.md#deepseek)。

2.5.1 自动选择平台与路径：执行 `qiongli update all`；也可直接用
`qiongli update plugin`。按[自动安装规则](../advanced/plugin-installation.md)复用旧路径
或选择独立默认目录，再逐个确认。2.5.0 的 `update all` 已能检测客户端，
但首次导出仍可能询问路径。新默认目录名需要支持它们的 CLI；回退时保留并复用
旧版已验证的源目录，不要让旧 CLI 接管新目录。

## 从 1.x 迁移

1. 备份项目、配置和旧版本，先用项目副本试用。
2. 按[安装页](install.md)安装 2.5.1，运行 `qiongli setup` 检查重复 CLI。它只提供手动处理建议，不移动或删除文件。
3. 运行 `qiongli install plugin` 接入 Host，保留自己的模型设置。Codex 可确认迁移已知旧插件；Claude 的冲突插件需手动停用。
4. 新开会话检查工具，按 [2.x 命令](cli-2x.md)继续工作。`project init`、`provider setup`、`check` 等旧命令见 [1.x 参考](../reference/cli.md)。

旧项目的格式迁移仍受各自迁移流程约束，安装新 CLI 不会自动完成项目迁移。版本差异见 [2.x 变化](whats-new-2.md)。

## 回退

重新安装原版本，并导出、启用与它匹配的 Plugin。CLI 与 Plugin 各自携带程序，需要一起核对。

**用过新版保存功能的项目，可能无法被旧 CLI 读取。** 2.5.1 保留旧收据读取，但旧程序会拒绝带有新保存类型的收据。此时继续用新版 CLI，或在另一个目录恢复升级前备份；不要删除、改写收据来绕过检查。
