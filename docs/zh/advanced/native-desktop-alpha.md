# 原生桌面 Alpha 安装包

Qiongli 2 桌面包仍是预发布产物。原始 CI 包始终属于
`assembled-unpublished` 测试证据，不能直接分发。首个公开版本计划使用明确标记的
零费用 `community-alpha` 分发类型，但仍须完成精确源码候选提升、穷理自身发布签名、
三平台原生验收和明确的发布授权。

## 分发类型

Community Alpha 不宣称付费操作系统发布者信任：macOS 使用 ad-hoc 签名但没有
Developer ID 和公证，Windows 使用未做 Authenticode 签名的 portable ZIP，Linux
使用 AppImage、穷理签名的发布元数据，并可选内嵌 GPG 签名。此类型只用于预发布
测试，不能进入 Stable。

后续 production 类型仍保留 macOS Developer ID/公证，以及 Windows 可信
Authenticode 和时间戳。两种类型都必须提供穷理的 Ed25519 发布/更新签名、checksum、
SBOM、provenance 和真实的平台 receipt。完整决策见
`docs/superpowers/specs/2026-07-17-qiongli-community-alpha-distribution-note.md`。

## 目标平台

| 平台 | 桌面产物 | CLI 入口 |
|---|---|---|
| macOS | 首次安装 `.dmg` 与自动更新 `.app.zip` | `Qiongli.app/Contents/MacOS/qiongli-cli` |
| Windows | portable application ZIP | `Qiongli/qiongli-cli.exe` |
| Linux | Type 2 `Qiongli-<version>-x86_64.AppImage` | 使用配套 portable CLI 产物 |

架构以产物文件名和 receipt 为准。除非发布记录中存在单独验收的目标 receipt，
Alpha.1 不支持 macOS Intel、Windows Arm64、Linux Arm64、32 位系统、移动端或
浏览器/云端执行。

## 安装和启动

macOS 用户打开 DMG，把 `Qiongli.app` 拖入 Applications，再从 Finder 启动。
Community Alpha 首次尝试启动后，需要在“系统设置 > 隐私与安全性”中对这个应用
选择“仍要打开”，但不能全局关闭 Gatekeeper。
配套 `.app.zip` 保留给穷理的原子自动更新与失败回滚流程，不作为普通首次安装入口。
Windows 用户必须完整解压 `Qiongli` 目录并双击 `Qiongli.exe`，
不要把它和 `qiongli-cli.exe` 分开。SmartScreen 可能提供“更多信息 > 仍要运行”，
但 Smart App Control、杀毒软件或企业策略可能直接阻止未签名版本。不要关闭这些
防护，也不要安装自签名根证书；应改用允许测试的设备。Linux 用户把 AppImage 设置为可执行文件后
直接运行：

```text
chmod +x Qiongli-<version>-x86_64.AppImage
./Qiongli-<version>-x86_64.AppImage
```

运行这些产物不需要 Rust、Python、Node.js、Cargo、npm 或 pip。Linux 仍依赖
Type 2 AppImage 和原生窗口栈所需的操作系统能力；在最终 readiness receipt
记录干净机器证据之前，不宣称完整兼容性。

## R3Q 统一控制面

当前 R3Q 源码把桌面 App 按用户结果重新组织：

- **Overview** 只读显示产品状态并给出下一项建议操作；
- **Skills** 默认使用 Qiongli Managed，也提供已发现的 Codex、Claude Code、
  当前项目和显式 Custom Folder 目标；安装、验证、修复、更新和删除只处理
  receipt-owned 内容；
- **Lite MCP** 分别检查 initialize、精确 tools 列表、代表性离线调用、provider
  readiness、取消/超时，以及独立的客户端 attachment/registration；
- **Literature Providers** 独占 provider 开关、公开联系字段和经过遮罩的 OpenAlex、
  Semantic Scholar 密钥；macOS 原始密钥进入 Keychain，原生配置只保存 opaque ref；
- **Integrations** 分别显示 Client、Source、Skills、Registration、Activation、
  MCP attachment 和 Overall，并为受支持的 Codex/Claude Code 状态提供恢复操作；
- **Global Settings** 只管理产品级默认值，**About** 管理产品身份和 Stable/Beta
  Software Update。

R3Q 仍是 Lite 控制面。Full orchestration、原生 agent 执行和外部 worker 协调属于
R4；Lite MCP 或客户端注册显示 Ready，不代表这些能力已经完成。

任何 R3Q 测试版发布前，都必须在精确 App 上人工完成：保存、替换、重启后测试并
删除一个 provider 密钥；测试 Codex/Claude 冲突、验证、修复和 receipt-owned 删除；
检查路径标签、键盘遍历、100%/200% 缩放、明暗对比度和 VoiceOver 控件名称。
自动 packaged receipt 只检查 secure-store 可用性和 opaque ref 的重启/删除，不会在
无人值守 CI 中触发可能弹窗的 Keychain 授权。

桌面启动器只负责打开 UI。终端命令应使用 macOS/Windows 包内明确提供的
`qiongli-cli`，Linux 则使用配套 portable CLI。AppImage 不会转发任意 CLI 参数。

## 卸载和托管状态

删除应用前，先在 Qiongli 中验证并移除不再需要的 receipt-owned Skills 和集成
内容，然后再删除 `.app`、Windows 解压目录或 AppImage。仅删除应用文件不会
暗中删除 Qiongli 托管的 Skills、客户端注册、receipts、配置或无关用户文件。

## 系统信任提示

开发 CI 包没有正式签名，且明确禁止直接发布。公开 Community Alpha 必须是单独
提升的最终候选，具备匹配的源码/产物 receipt、穷理 Ed25519 元数据、checksum、
SBOM、provenance、三平台原生证据、平台信任警告和明确授权；它不宣称 macOS 公证
或 Windows Authenticode。

只能在系统提供时使用针对单个应用的正常放行入口。不能关闭 Gatekeeper、Smart
App Control、杀毒软件、企业策略或 Linux 完整性检查。若 Windows 主机直接阻止
未签名程序，该设备不属于 Community Alpha 支持范围。production 类型仍要求
macOS Developer ID/公证和 Windows Authenticode。

macOS 签名边界会从同一个已签名 App 同时生成更新 ZIP 和首次安装 DMG。测试模式
使用 ad-hoc 签名并验证 DMG 能挂载、只包含 `Qiongli.app` 与 Applications 链接；
生产模式还会单独签名、公证、staple 并通过 Gatekeeper 检查 DMG。两种产物在最终
发布 ledger 完成前都保持 `publication_allowed: false`。

R3P-B 另外增加了 `--community-alpha` 模式。它与 test-only 模式使用不同的
`.community-alpha.app.zip`、`.community-alpha.dmg` 文件名，并在非发布 receipt
中明确记录 `macos-ad-hoc-not-notarized`。它仍然不能直接发布。

Alpha.1 当前代码已经具备独立 R3P-B workflow，可在合并后针对远端 `2.x` 当前
HEAD 重新构建 macOS DMG/ZIP、Windows portable ZIP 和 Linux AppImage/AppDir
ZIP，并聚合为一个仍不可发布的候选；第一次真实 workflow 尚未执行。R3P-C/R3P-D
已经实现公开 authority、校验清单、SBOM、provenance、中英发布说明、离线
Ed25519 签名和受保护 Environment 精确授权。授权 job 只有只读权限且不持有
私钥；维护者在本机签名和发布，因此 GitHub 不会获得私钥。Marketplace 绕过、
Desktop/Cloud 插件注入仍未开放，不能因为存在桌面包就推断这些后续能力已经实现。
