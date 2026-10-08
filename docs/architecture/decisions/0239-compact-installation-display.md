# ADR 0239: Compact Installation Display

- Status: Accepted
- Date: 2026-10-08
- Task: CLI-402
- Authority: the maintainer reports successful 2.5.1 installation but excessive
  scrolling and asks for a clearer display based on mature CLI installers.
- Supersedes only ADR 0238's default append-only five-second reporting. Its
  command, approval, batch, diagnostic-privacy and recovery owners remain intact.

Separate each Host, approval preview, execution and final summary. Preserve all
fields of write/registration previews and their confirmations. Default DSH output
retains each numbered result and exact failing step, but omits duplicate START,
successful launch contexts and captured-output observations. Failures automatically
show the existing safe diagnostics and exact commands. `--verbose` retains full
stage/launch diagnostics as append-only output, including the original reason
codes; it never reveals raw manager output or credential values.

The shared installation writer owns one temporary, short ASCII status line on
recognized Unix terminals with a known width of at least 32 columns. It updates
from the existing five-second callback and clears before any ordinary output or
prompt. It never moves above its line, clears the screen, hides the cursor or
changes terminal input mode. A long-running command shows elapsed/limit time,
not an invented download percentage. Only presentation changes: command argv,
environment, deadlines, output bounds and Pi's private umask remain unchanged.

`--plain`, `--verbose`, `NO_COLOR`, `CI`, absent/unsupported TERM, unavailable
terminal size, narrow windows and unsupported consoles use append-only output.
Compact plain waiting messages are at most once per 30 seconds; verbose retains
five-second messages. Non-terminal installation still refuses before writes;
display flags do not make interactive approvals scriptable. Existing dry-run
and structured query output are unchanged. No new rendering dependency or
network service is introduced; Unix size lookup uses the existing rustix crate.

Verification covers real PTY captures, bounded redraws and prompt cleanup,
plain/verbose fallbacks, exact approval previews, privacy, failure attribution,
batch continuation, EOF/output interruption and parser compatibility. Synthetic
managers prove presentation/control flow, not a live Host install or session.
Rollback restores the prior renderer without changing installed files/profiles.

Design references: [pnpm reporters](https://pnpm.io/cli/install#--reportername)
separate terminal and append-only output; [Clack tasks and spinners](https://github.com/bombshell-dev/clack/blob/main/packages/prompts/README.md)
separate pending tasks and results; [GitHub CLI environment options](https://cli.github.com/manual/gh_help_environment)
offer verbose diagnostics and accessible progress controls. Qiongli retains its
native Rust and explicit approval contracts rather than copying those libraries.
