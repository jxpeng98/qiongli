# 原生桌面 Alpha 安装包

Qiongli 2 桌面包仍是预发布产物。在 Alpha.1 readiness receipt 明确允许发布之前，
CI 生成的包都属于 `assembled-unpublished` 测试证据，不能当作已签名正式版本分发。

## 目标平台

| 平台 | 桌面产物 | CLI 入口 |
|---|---|---|
| macOS | `.app.zip` 中的 `Qiongli.app` | `Qiongli.app/Contents/MacOS/qiongli-cli` |
| Windows | portable application ZIP | `Qiongli/qiongli-cli.exe` |
| Linux | Type 2 `Qiongli-<version>-x86_64.AppImage` | 使用配套 portable CLI 产物 |

架构以产物文件名和 receipt 为准。除非发布记录中存在单独验收的目标 receipt，
Alpha.1 不支持 macOS Intel、Windows Arm64、Linux Arm64、32 位系统、移动端或
浏览器/云端执行。

## 安装和启动

macOS 用户解压后可以把 `Qiongli.app` 移入用户自己的 Applications 目录，再从
Finder 启动。Windows 用户必须完整解压 `Qiongli` 目录并双击 `Qiongli.exe`，
不要把它和 `qiongli-cli.exe` 分开。Linux 用户把 AppImage 设置为可执行文件后
直接运行：

```text
chmod +x Qiongli-<version>-x86_64.AppImage
./Qiongli-<version>-x86_64.AppImage
```

运行这些产物不需要 Rust、Python、Node.js、Cargo、npm 或 pip。Linux 仍依赖
Type 2 AppImage 和原生窗口栈所需的操作系统能力；在最终 readiness receipt
记录干净机器证据之前，不宣称完整兼容性。

桌面启动器只负责打开 UI。终端命令应使用 macOS/Windows 包内明确提供的
`qiongli-cli`，Linux 则使用配套 portable CLI。AppImage 不会转发任意 CLI 参数。

## 卸载和托管状态

删除应用前，先在 Qiongli 中验证并移除不再需要的 receipt-owned Skills 和集成
内容，然后再删除 `.app`、Windows 解压目录或 AppImage。仅删除应用文件不会
暗中删除 Qiongli 托管的 Skills、客户端注册、receipts、配置或无关用户文件。

## 系统信任提示

开发 CI 包没有正式签名，且明确禁止发布。对于声称是公开版本的产物，不要绕过
Gatekeeper、SmartScreen、杀毒软件、企业策略或 Linux 签名检查。可发布的
Alpha.1 必须具备匹配的源码/产物 receipt，并按平台提供维护者控制的 macOS
签名与 notarization、Windows Authenticode 或已签名 Linux 发布元数据。

Alpha.1 不包含安装器、自动更新、托管升级、Marketplace 绕过或 Desktop/Cloud
插件注入。不能因为存在桌面包就推断这些能力已经实现。
