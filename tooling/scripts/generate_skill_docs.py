#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path
import sys

REPO_ROOT = Path(__file__).resolve().parents[2]
PYTHON_SOURCE_ROOT = REPO_ROOT / "packages" / "python-qiongli" / "src"
for import_root in (PYTHON_SOURCE_ROOT, REPO_ROOT):
    if str(import_root) not in sys.path:
        sys.path.insert(0, str(import_root))

# Reuse the frozen registry reader/table renderer as build tooling only.
from qiongli import skill_docs


def generate_skill_reference_docs(root: Path) -> dict[str, str]:
    entries = skill_docs.load_skill_doc_entries(root)
    profiles = "\n".join(f"- `{name}`" for name in skill_docs.load_domain_profile_ids(root))
    en = """# Skills guide

> Auto-generated from `content/skills/registry.yaml` by `python3 scripts/generate_skill_docs.py`.
> Edit the registry or the generator, then regenerate this page.

Describe your research task in the Host, or select a visible Qiongli Skill.
In Codex, use `$qiongli` or a shortcut such as `$qiongli-paper-read`.
The shared guidance helps the model choose the relevant work; it does not require
every stage to run. Task IDs identify research activities, not native CLI commands.

The tables below list the internal research Skills. They are not all separate
Host entries. Supplemental cards and subject profiles support these Skills;
they do not create additional agents or MCP tools.

The descriptions and use cases below come from the same registry in both languages.
"""
    zh = """# Skills 指南

> 本页由 `python3 scripts/generate_skill_docs.py` 根据 `content/skills/registry.yaml` 生成。
> 需要修改时，请更新注册信息或生成器，再重新生成本页。

在 Host 中描述研究任务，或选择列表中可见的 Qiongli Skill 即可开始。
Codex 可以使用 `$qiongli` 或 `$qiongli-paper-read` 等快捷入口。
共享指导帮助模型选择相关工作，不要求执行每个阶段。
任务 ID 用来标识研究活动，不是原生 CLI 命令。

下表列出内部研究 Skills，并不意味着每项都有独立的 Host 入口。
补充卡片和学科档案为这些 Skills 提供参考，也不会创建额外代理或 MCP 工具。
显示名称、使用场景和中文说明来自注册信息，中英文表格共享同一份技能清单。
"""
    result = {}
    for lang, intro, overview, table in (
        ("en", en, skill_docs._build_stage_overview_en, skill_docs._build_skills_by_stage_en),
        ("zh", zh, skill_docs._build_stage_overview_zh, skill_docs._build_skills_by_stage_zh),
    ):
        if lang == "en":
            footer = """## Subject guidance

Describe the field, method and protocol in your request. The Host can read relevant
bundled profiles; the 1.x `--domain` switch is not a native 2.x CLI option.
Profiles guide the work, but do not establish evidence or current journal policy.

The bundled `references/discipline-guidance.md` selects focused guides for
economics/finance/accounting, business/society/policy, education/psychology, health/biomedicine,
computing/engineering, environment/spatial research and humanities/language/law.
They organize common questions, methods, evidence and delivery checks across A–M,
and load only when relevant. These guides do not add runtime subject IDs or Host entries.

Available profiles:

{profiles}

## Continue

Use [task examples](/guide/task-recipes), [the CLI guide](/guide/cli-2x) or
[collaboration guidance](/advanced/agent-skill-collaboration).
For changes to the framework, see [Extend Qiongli](/advanced/extend-qiongli).
"""
            path = "docs/reference/skills.md"
        else:
            footer = """## 学科指导

在请求中说明学科、方法和协议，Host 可以读取相关的随包档案。
1.x 的 `--domain` 开关不是原生 2.x CLI 选项。
档案提供研究指导，不能代替证据或现行期刊政策。

随包的 `references/discipline-guidance.md` 按需选择七组指南：经金会、管理与社会政策、
教育心理、医药健康、计算机工程、环境地理、人文语言法学。各组整理常见研究问题、
方法、证据和交付检查，并与 A–M 阶段衔接；只有相关内容会被加载。
这些指南扩充参考内容，不增加运行时学科 ID 或 Host 技能入口。

当前包含的档案：

{profiles}

## 继续阅读

查看[任务示例](/zh/guide/task-recipes)、[CLI 指南](/zh/guide/cli-2x)，
或[协作指南](/zh/advanced/agent-skill-collaboration)。
修改框架时，请先读[扩展 Qiongli](/zh/advanced/extend-qiongli)。
"""
            path = "docs/zh/reference/skills.md"
        result[path] = "\n\n".join((intro.strip(), overview(entries), table(entries), footer.format(profiles=profiles).strip())) + "\n"
    return result


def main() -> int:
    for relative_path, content in generate_skill_reference_docs(REPO_ROOT).items():
        (REPO_ROOT / relative_path).write_text(content, encoding="utf-8")
        print(f"[WRITE] {relative_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
