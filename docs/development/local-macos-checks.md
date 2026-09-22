# Optional local macOS checks

[ADR 0231](../architecture/decisions/0231-hosted-three-platform-verification.md)
restores hosted macOS, Linux and Windows verification as the default. The normal
three-platform build, assembly, combined-install and protected publication owners
run in GitHub Actions. A maintainer-uploaded draft or a mandatory local macOS
receipt is no longer a release prerequisite.

Local checks remain optional through existing owners: `scripts/release_ready.sh`
for a named CLI candidate and `pnpm desktop:macos:acceptance` for retained Desktop
acceptance. Keep their actual receipts and exact-source limits; a local pass does
not replace missing target evidence or authorize publication. Existing signing,
upgrade, rollback, Plugin v1/Next compatibility and user-data protections remain.

[ADR 0230](../architecture/decisions/0230-local-macos-verification.md) records the
superseded local-only policy. No remote rule, tag or published artifact changes
follow merely from restoring this configuration.
