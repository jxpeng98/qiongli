# Development workflow

The active workflow is defined in [AGENTS.md](../AGENTS.md) and
[CONTRIBUTING.md](../CONTRIBUTING.md): inspect the current outcome, make a bounded
change, run affected checks, review the diff, and record the result.

This path remains for older links. Project-local Trellis skills, hooks and agent
roles were retired on September 4, 2026. Task phase transitions, JSONL injection,
mandatory delegation and automatic session bookkeeping are no longer required.

Keep `.trellis/spec/`, `.trellis/tasks/` and `.trellis/workspace/` as project
knowledge and evidence. Existing scripts are optional manual history utilities;
they do not control implementation permission or roadmap priority. Automatic
commits stay disabled. Do not run `trellis init` or `trellis update` to restore
the retired integration unless the user explicitly requests it.
