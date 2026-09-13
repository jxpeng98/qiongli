# Maintainer workflow

Start with the repository's `AGENTS.md` and `CONTRIBUTING.md`, then read the current
bounded plan linked from the master roadmap. The program ledger owns task status
and accepted evidence. `CLAUDE.md` provides Host-specific context, not a separate
source of release or research acceptance.

Trace the requested behaviour to its existing source using
[repository structure](../development/repository-structure.md). For native CLI,
MCP or installation work, use `packages/qiongli-native/`; for research guidance,
use `content/`. Keep generated outputs and retained 1.x product sources separate.

Use a local feature branch, run affected checks, review the diff and make scoped
commits before local integration into `2.x`. Main integration and publication
follow the requested scope and [release policy](release-branch-policy.md).
Record checks and gaps once in the plan and ledger. A local merge does not make
an unverified Host journey or release channel accepted.

For collaboration, keep independent tasks bounded and avoid concurrent edits to
shared research records. Preserve the user's Host and model settings. Instructions
for subagents and handoffs are in the [collaboration guide](../advanced/agent-skill-collaboration.md).
