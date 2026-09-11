---
name: qiongli-{{workflow}}
{{description}}
---

# Qiongli {{workflow}}

Use this entry for the requested `{{workflow}}` task.

If the user selected reply-only (仅回复 / 不处理 / no 处理 / no tools), answer
from the conversation without tools, resource reads, agents or file operations.
Keep that choice until the user explicitly resumes execution, unless limited to
one reply. State missing material; do not load the resources below in this mode.

1. Read `../qiongli-workflow/SKILL.md` for shared guidance, Host tool availability,
   evidence rules and project write permissions.
2. Follow `../qiongli-workflow/workflows/{{workflow}}.md` for the requested work.
   Resolve its resource paths from `../qiongli-workflow/`.
3. Load only the skill cards, references and templates needed for that request.
   Reuse supplied context; a direct task does not start the full research lifecycle.

This is a generated entry. The shared Skill and workflow own the instructions;
do not duplicate their logic or infer additional tools, models or write approval.
