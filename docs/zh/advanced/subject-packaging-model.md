# Subject Packaging Model

Qiongli 使用一套 canonical workflow、稳定的 full local runtime，以及用于 compatibility 和 release artifacts 的可安装 subject packages。本页从用户和开发者两个视角说明项目级 subject guidance、`core`、specialized subject、coverage、composite 和 local customization 的区别。

## 用户视角

普通 CLI / local plugin 使用中，installed package 应该保持为稳定的 core/full runtime：

```bash
qiongli install --target all
```

每个项目的 subject 行为保存在 `.qiongli/guidance_manifest.yaml`，而不是靠重新安装不同 client package。使用：

```bash
qiongli project init --project-dir .
qiongli project set-subject finance --project-dir .
qiongli project set-venue journal-of-finance --project-dir .
qiongli project set-method-lens event-study --project-dir .
qiongli project status --project-dir .
```

如果 manifest 不存在，Qiongli 使用隐式 `active_subject: auto`：先使用 core guidance，再根据当前任务临时推断 subject 和 method lens，并在改变项目本地状态前写出可审计 proposal。持久 subject 变更只来自显式 project 命令或已接受的 guidance proposal。

当用户或客户端确认、锁定某个运行时学科后，Qiongli 会在
`.qiongli/guidance.d/subject-runtime.md` 写入一个受管理的项目级片段。之后的任务运行会通过
local guidance 读取这个片段，因此安装好的 adaptive core 工作流仍然以 core 为基础，同时为当前项目叠加已确认的学科层。
执行 `qiongli subject reset --cwd .` 会禁用这个受管理片段，并让项目回到 adaptive core 推断模式。

安装后的 skill 目录仍然是 `qiongli-workflow`。同一客户端安装一次只有一个 active package。Subject-specific installs 仍然用于 Desktop ZIP、focused packages、compatibility testing 和 release payloads。

| 场景 | 命令 | 含义 |
|---|---|---|
| 普通 CLI / local plugin 使用 | `qiongli install --target all` | 多项目共享的稳定 full runtime |
| 当前项目使用 finance guidance | `qiongli project set-subject finance --project-dir .` | 写入项目本地 subject context |
| 没有配置项目 subject | 无需命令 | 隐式 `active_subject: auto`，使用 core guidance 和临时推断 |
| Focused Desktop/Web ZIP | `qiongli install --subject economics --coverage focused --target all` | 有意控制体积的上传包 |
| Compatibility 或 release payload testing | `qiongli install --subject economics --target all` | 作为 installed package 的全量框架 + economics 专精 |
| 官方交叉学科 ZIP/package | `qiongli install --subject economics-accounting --target all` | 全量框架 + economics/accounting composite layer |
| 本地生成 package 规则 | `python3 scripts/materialize_subject_package.py --custom-dir <path> ...` | 生成一次带本地 overlays、profiles 或 skills 的 package |

### Core

`core` 是默认 subject。它是用户在还不想选择学科时使用的通用 Qiongli workflow。

它提供：

- 共享 workflow contracts 和 task IDs
- 共享 templates、standards、references 和 roles
- canonical generic skills
- `complete` coverage 下的广泛 domain 和 venue profile 可用性

当项目还处于探索阶段、跨领域但没有明确专精方向，或者用户只是想要完整通用框架时，使用 `core`。普通 CLI / local plugin 使用中，它也是配合项目级 guidance 的稳定 installed runtime。

### Specialized Subjects

`economics`、`political-economy`、`geoeconomics` 这类 specialized subject 保留同一套 Qiongli workflow，只是在其上叠加学科层。它不会 fork workflow contract。

subject 可以增加：

- selected domain 和 venue profiles
- generic skills 的 append overlays
- 针对特定 Quality Bar 或 Common Pitfalls 的 section replacement
- 少量 subject-specific skills

CLI/npm 默认是 `coverage=complete`。因此：

```text
qiongli install --subject economics
= full core framework + economics overlays + economics-specific skills
```

这不是删减包。只有显式使用 `--coverage focused` 时，才会生成精简 subject package。

日常项目工作中，优先使用 `qiongli project set-subject <subject>`，不要为了切换项目 subject 重新安装专精 package。只有在准备 Desktop ZIP、专门测试 focused 或 complete package、验证 release payload，或维护需要单个 active subject package 的兼容安装面时，才使用 `qiongli install --subject ...`。

### Coverage Modes

`complete` 和 `focused` 解决的是不同用户需求。

| Coverage | 适合场景 | package 形态 |
|---|---|---|
| `complete` | 想要全量框架，同时需要 subject 专精 | all core generic skills + selected subject overlays + subject-specific skills |
| `focused` | Desktop/Web ZIP、上传文件数限制、明确想要精简安装 | selected subject profiles、selected effective skills 和 subject-specific skills |

不确定怎么选时，用 `complete`。

### Composite Subjects

Composite subject 是官方维护的交叉学科包，例如 `economics-accounting`。

`composes` 是 metadata，不是自动 union。它只记录官方 composite subject 是基于哪些 component subject 的期望设计出来的；composite 仍然必须显式声明自己的 ordered groups、profile selection、overlays 和 subject-specific skills。

它不是多个 subject 的自动 union。每个 composite 必须显式定义 ordered groups、profile selection、overlays 和 subject-specific skills。这样可以保持 package 连贯，并避免不同学科规则之间发生隐性冲突。

当研究问题天然跨学科，且单一 subject 无法覆盖重要规范时，使用 composite。

当你希望 active package 明确服务 political economy 或 geoeconomics 时，分别安装对应 subject。它们是两个独立 subject，不是一个 composite pair，也不应该合并进同一个 Desktop/Web ZIP。

### Local Customization

Local customization 用于用户、课题组或项目自己的规则。这类内容不应该马上进入 canonical source。它通过 Python materializer 一次性应用：

```bash
python3 scripts/materialize_subject_package.py \
  --subject economics \
  --coverage complete \
  --source . \
  --custom-dir /path/to/custom-qiongli \
  --out /tmp/qiongli-workflow
```

custom directory 可以包含：

- `subject.yaml`
- `overlays/skills/*.md`
- `skills/registry.yaml`
- `skills/*.md`
- `domain-profiles/*.yaml`
- `venue-profiles/*.yaml`

customization 只影响本次生成输出，不会写回 canonical repository。

普通项目本地 guidance 优先使用 `.qiongli/guidance_manifest.yaml`、`.qiongli/local_guidance.md` 和 `.qiongli/guidance.d/*.md`。materializer 路径用于 generated packages 和 release-style payloads。

## 开发者视角

源码是分层的。generic workflow 行为保持 canonical；学科深度通过 overlays 和 selected subject assets 注入。

| 层级 | 源码位置 | 作用 | 是否进入 `core` |
|---|---|---|---|
| Workflow contract | `qiongli-workflow/`, `standards/`, `references/` | 共享 task IDs、outputs、quality gates | 是 |
| Generic skills | `skills/*/*.md` | canonical cross-disciplinary capability cards | 是 |
| Generic profiles | `skills/domain-profiles/*.yaml`, `venue-profiles/*.yaml` | 可复用的 domain 和 venue metadata | materialize 后进入 core package |
| Subject catalog | `subjects/catalog.yaml` | ordered subject groups 和 selected assets | 只有 materialize 后才进入 package |
| Subject overlays | `subjects/<subject>/overlays/skills/*.md` | append 或替换声明 section | 否 |
| Subject skills | `subjects/<subject>/skills/*.md` | 少量学科专属 skills | 否 |
| Composite subject | `subjects/<composite>/...` | 官方交叉学科 package | 否 |
| Local custom layer | `--custom-dir` | 用户/课题组本地 generated package layer | 否 |

### 什么应该进入 Core

当内容跨多个学科都成立时，放进 `core`：

- 共享 research workflow contracts
- 通用 task definitions
- cross-disciplinary quality gates
- 可复用的 writing、literature、design、code、compliance skills
- 多个 subject 都合理复用的 profiles 或 templates

Core 应该稳定、通用，并且在用户没有选择 subject 时仍然完整可用。

### 什么应该进入 Subject

当内容会显著改变某个学科里的行为时，放进 subject：

- method-specific diagnostics
- discipline-specific failure modes
- venue norms 和 evidence standards
- field-specific data、construct 或 identification requirements
- 针对 selected skills 的更严格 quality bars
- 只有该学科内才成立的 expert capability

subject-specific 内容优先用 overlay 或 profile 更新表达。只有当 workflow 真的需要一个独立专家能力时，才创建新的 subject-specific skill。

### 不应该做什么

避免这些模式：

- 把 generic skill 复制到 `subjects/<subject>/skills/`
- 只是为了改例子就创建 `econ-*` 版 generic skill
- 使用 `replace_sections` 但不声明精确 section name
- 为单一学科修改 workflow contract，除非该 contract 变化确实具有通用价值
- 把 composite subject 当成两个 subject registry 的盲目 union
- 把 local customizations 意外写回 canonical source

### 晋升规则

subject 内容是否进入 core，按这个规则判断：

```text
如果规则跨学科有用，晋升到 core。
如果规则只在某个学科成立，留在 subject。
如果规则只属于某个课题组、项目或用户，留在 custom-dir。
```

### 新增或深化 Subject 的最低要求

每个 official subject 应该提供：

- `subjects/catalog.yaml` 中的 ordered catalog groups
- selected domain profiles 和 venue profiles
- 针对需要学科约束的 generic skills 的 overlays
- overlays 不足以表达时，才增加最小数量 subject-specific skills
- `complete` 和 `focused` coverage 的 materializer tests
- 如果发布 Desktop/Web ZIP，则要有文件数预算 release validation
- 安装、切换和 customization 的文档示例

这个结构保证 subject package 足够专精，同时不会分裂 canonical workflow。
