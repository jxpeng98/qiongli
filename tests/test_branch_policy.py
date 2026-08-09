from __future__ import annotations

import unittest
from pathlib import Path

from qiongli.source_layout import RepoLayout


REPO_ROOT = Path(__file__).resolve().parents[1]
LAYOUT = RepoLayout(REPO_ROOT)


def read(path: str) -> str:
    if path.startswith("scripts/"):
        return (LAYOUT.scripts / Path(path).relative_to("scripts")).read_text(encoding="utf-8")
    return (REPO_ROOT / path).read_text(encoding="utf-8")


class BranchPolicyTests(unittest.TestCase):
    def test_ci_routes_legacy_and_native_branches_to_separate_workflows(self) -> None:
        legacy_ci = read(".github/workflows/ci.yml")
        native_ci = read(".github/workflows/native-ci.yml")
        install_check = read(".github/workflows/install-check.yml")

        legacy_filter = 'branches: ["main", "master", "dev"]'
        native_filter = 'branches: ["2.x"]'
        old_filter = 'branches: ["main", "master", "dev", "2.x"]'

        self.assertEqual(legacy_ci.count(legacy_filter), 2)
        self.assertEqual(install_check.count(legacy_filter), 2)
        self.assertEqual(native_ci.count(native_filter), 2)
        self.assertNotIn(old_filter, legacy_ci)
        self.assertNotIn(old_filter, install_check)
        self.assertIn("workflow_dispatch:", legacy_ci)
        self.assertIn("workflow_dispatch:", install_check)
        self.assertIn("workflow_dispatch:", native_ci)
        self.assertIn("tooling/release/acceptance/**", legacy_ci)
        self.assertIn("tooling/release/acceptance/**", native_ci)

    def test_ci_workflow_cancels_stale_runs_and_splits_test_tiers(self) -> None:
        content = read(".github/workflows/ci.yml")

        self.assertIn("concurrency:", content)
        self.assertIn("cancel-in-progress: true", content)
        self.assertIn("cache: pip", content)
        self.assertIn("test-tier: full", content)
        self.assertIn("test-tier: windows-smoke", content)
        self.assertIn("test-tier: macos-smoke", content)
        self.assertIn("if: matrix.test-tier == 'full'", content)
        self.assertIn("if: matrix.test-tier == 'windows-smoke'", content)
        self.assertIn("if: matrix.test-tier == 'macos-smoke'", content)
        self.assertIn("      - name: Run Windows smoke tests", content)
        self.assertIn("      - name: Run macOS CTR inventory smoke tests", content)
        windows_modules = (
            "tests.test_install_qiongli",
            "tests.test_bootstrap_qiongli",
            "tests.test_universal_installer",
            "tests.test_command_runtime",
            "tests.test_release_automation",
            "tests.test_branch_policy",
            "tests.test_2x_branch_point",
            "tests.test_arc_201_adrs",
            "tests.test_frozen_2x_architecture_baseline",
            "tests.test_repository_source_validator",
            "tests.test_ctr_201_inventory",
            "tests.test_ctr_201_cli_inventory",
            "tests.test_ctr_201_orchestrator_inventory",
            "tests.test_ctr_201_content_inventory",
            "tests.test_ctr_201_cli_runtime_inventory",
            "tests.test_ctr_201_orchestrator_runtime_inventory",
        )
        for module in windows_modules:
            self.assertIn(module, content)
        windows_start = content.index("      - name: Run Windows smoke tests")
        windows_end = content.index(
            "      - name: Run macOS CTR inventory smoke tests", windows_start
        )
        windows_block = content[windows_start:windows_end]
        self.assertEqual(
            [windows_block.index(module) for module in windows_modules],
            sorted(windows_block.index(module) for module in windows_modules),
        )
        self.assertNotIn(
            "scripts/extract_ctr_201_cli_runtime_inventory.py",
            windows_block,
        )
        self.assertNotIn(
            "scripts/extract_ctr_201_orchestrator_runtime_inventory.py",
            windows_block,
        )
        self.assertIn('./scripts/release_preflight.sh --quick --materialize-out "$RUNNER_TEMP/qiongli-preflight-dist"', content)

    def test_ci_runs_ctr_inventory_smoke_on_macos_after_shared_validation(
        self,
    ) -> None:
        content = read(".github/workflows/ci.yml")
        job_start = content.index("  cross-platform-tests:")
        job_end = content.index("  shell-release-gates:", job_start)
        job = content[job_start:job_end]

        for runner, tier in (
            ("ubuntu-latest", "full"),
            ("windows-latest", "windows-smoke"),
            ("macos-latest", "macos-smoke"),
        ):
            with self.subTest(runner=runner):
                self.assertIn(
                    f"          - os: {runner}\n            test-tier: {tier}",
                    job,
                )

        validate_step = "      - name: Validate CTR-201A/B/C/D/E/F inventories"
        macos_step = "      - name: Run macOS CTR inventory smoke tests"
        self.assertIn(validate_step, job)
        self.assertIn(macos_step, job)
        macos_start = job.index(macos_step)
        macos_block = job[macos_start:]
        self.assertIn("if: matrix.test-tier == 'macos-smoke'", macos_block)
        self.assertIn("python -m unittest", macos_block)
        macos_modules = (
            "tests.test_ctr_201_inventory",
            "tests.test_ctr_201_cli_inventory",
            "tests.test_ctr_201_orchestrator_inventory",
            "tests.test_ctr_201_content_inventory",
            "tests.test_ctr_201_cli_runtime_inventory",
            "tests.test_ctr_201_orchestrator_runtime_inventory",
        )
        for module in macos_modules:
            self.assertIn(module, macos_block)
        self.assertEqual(
            [macos_block.index(module) for module in macos_modules],
            sorted(macos_block.index(module) for module in macos_modules),
        )
        self.assertNotIn(
            "scripts/extract_ctr_201_cli_runtime_inventory.py",
            macos_block,
        )
        self.assertNotIn(
            "scripts/extract_ctr_201_orchestrator_runtime_inventory.py",
            macos_block,
        )
        self.assertLess(job.index(validate_step), macos_start)

    def test_ci_validates_ctr_201a_b_c_d_e_f_inventories_before_distribution_work(
        self,
    ) -> None:
        content = read(".github/workflows/ci.yml")
        self.assertIn("      - name: Setup Node for accepted npm parser oracle", content)
        self.assertIn("        if: matrix.test-tier == 'full'", content)
        self.assertIn('          node-version: "20"', content)
        compile_step = "      - name: Compile CTR-201A/B/C/D/E/F inventory gates"
        validate_step = "      - name: Validate CTR-201A/B/C/D/E/F inventories"
        validate_command = "python scripts/validate_ctr_201_inventory.py"
        materialize_command = (
            "python scripts/materialize_distribution_payloads.py --target all "
            '--out "$RUNNER_TEMP/qiongli-dist" --force'
        )

        self.assertIn(compile_step, content)
        self.assertIn(validate_step, content)
        self.assertIn(validate_command, content)
        self.assertIn(materialize_command, content)
        compile_start = content.index(compile_step)
        compile_end = content.index(
            "      - name: Resolve generated payload comparison base",
            compile_start,
        )
        compile_block = content[compile_start:compile_end]
        compiled_paths = (
            "scripts/validate_ctr_201_inventory.py",
            "tooling/scripts/validate_ctr_201_inventory.py",
            "scripts/extract_ctr_201_cli_inventory.py",
            "tooling/scripts/extract_ctr_201_cli_inventory.py",
            "scripts/extract_ctr_201_orchestrator_inventory.py",
            "tooling/scripts/extract_ctr_201_orchestrator_inventory.py",
            "scripts/extract_ctr_201_content_inventory.py",
            "tooling/scripts/extract_ctr_201_content_inventory.py",
            "scripts/extract_ctr_201_cli_runtime_inventory.py",
            "tooling/scripts/extract_ctr_201_cli_runtime_inventory.py",
            "scripts/extract_ctr_201_orchestrator_runtime_inventory.py",
            "tooling/scripts/extract_ctr_201_orchestrator_runtime_inventory.py",
            "tests/test_ctr_201_inventory.py",
            "tests/test_ctr_201_cli_inventory.py",
            "tests/test_ctr_201_orchestrator_inventory.py",
            "tests/test_ctr_201_content_inventory.py",
            "tests/test_ctr_201_cli_runtime_inventory.py",
            "tests/test_ctr_201_orchestrator_runtime_inventory.py",
        )
        for compiled_path in compiled_paths:
            self.assertIn(compiled_path, compile_block)
        self.assertEqual(
            [compile_block.index(path) for path in compiled_paths],
            sorted(compile_block.index(path) for path in compiled_paths),
        )
        self.assertLess(content.index(compile_step), content.index(validate_step))
        self.assertLess(content.index(validate_step), content.index(materialize_command))

    def test_2x_native_ci_has_independent_three_platform_rust_gate(self) -> None:
        content = read(".github/workflows/native-ci.yml")
        start = content.index("  rust-native-foundation:")
        end = content.index("  desktop-package-assembly:", start)
        job = content[start:end]

        self.assertIn("name: Rust native foundation (${{ matrix.platform }})", job)
        self.assertIn("fail-fast: false", job)
        for platform, runner in (
            ("Linux", "ubuntu-latest"),
            ("macOS", "macos-latest"),
            ("Windows", "windows-latest"),
        ):
            with self.subTest(platform=platform):
                self.assertIn(
                    f"          - platform: {platform}\n            os: {runner}", job
                )
        self.assertIn("uses: dtolnay/rust-toolchain@1.97.0", job)
        self.assertIn("components: rustfmt, clippy", job)
        self.assertIn("Reject injected target-specific Rust flags", job)
        self.assertIn("CARGO_TARGET_*_RUSTFLAGS", job)
        self.assertEqual(job.count("CARGO_HOME:"), 5)
        self.assertIn('CARGO_ENCODED_RUSTFLAGS: ""', job)
        self.assertIn('RUSTC_WRAPPER: ""', job)
        self.assertIn('RUSTFLAGS: ""', job)
        commands = (
            "cargo fmt --manifest-path packages/qiongli-native/Cargo.toml --all -- --check",
            "cargo check --manifest-path packages/qiongli-native/Cargo.toml --workspace --all-targets --all-features --locked",
            "cargo clippy --manifest-path packages/qiongli-native/Cargo.toml --workspace --all-targets --all-features --locked -- -D warnings",
            "cargo test --manifest-path packages/qiongli-native/Cargo.toml --workspace --all-targets --all-features --locked",
        )
        for command in commands:
            self.assertIn(command, job)
        self.assertEqual(
            [job.index(command) for command in commands],
            sorted(job.index(command) for command in commands),
        )
        self.assertNotIn("continue-on-error", job)
        self.assertNotRegex(job, r"(?m)^\s+if:")
        self.assertNotIn("cache:", job)

    def test_native_promotion_requires_successful_ci_for_exact_head(self) -> None:
        native_ci = read(".github/workflows/native-ci.yml")
        start = native_ci.index("  dispatch-community-alpha-promotion:")
        dispatch = native_ci[start:]

        for job in (
            "native-change-boundary",
            "rust-native-foundation",
            "desktop-package-assembly",
            "packaged-product-acceptance",
            "lite-runtime-compatibility",
            "lite-alpha-candidate-acceptance",
        ):
            self.assertIn(f"      - {job}", dispatch)
        self.assertIn("success()", dispatch)
        self.assertIn("github.event_name != 'pull_request'", dispatch)
        self.assertIn("github.ref == 'refs/heads/2.x'", dispatch)
        self.assertIn("actions: write", dispatch)
        self.assertIn("GH_TOKEN: ${{ github.token }}", dispatch)
        self.assertIn("gh workflow run native-community-alpha-promotion.yml", dispatch)
        self.assertIn("--ref 2.x", dispatch)
        self.assertIn('-f "source_commit=$SOURCE_COMMIT"', dispatch)
        self.assertIn('-f "native_ci_run_id=$NATIVE_CI_RUN_ID"', dispatch)

        promotion = read(".github/workflows/native-community-alpha-promotion.yml")
        self.assertNotIn("workflow_run:", promotion)
        self.assertIn("workflow_dispatch:", promotion)
        self.assertNotIn("\n  push:\n", promotion)
        self.assertIn("actions: read", promotion)
        self.assertIn("native_ci_run_id:", promotion)
        self.assertIn("REQUESTED_SOURCE_COMMIT: ${{ inputs.source_commit }}", promotion)
        self.assertIn("NATIVE_CI_RUN_ID: ${{ inputs.native_ci_run_id }}", promotion)
        self.assertIn('"repos/$GITHUB_REPOSITORY/actions/runs/$NATIVE_CI_RUN_ID"', promotion)
        self.assertIn('[[ "$(jq -r \'.name\' <<<"$native_ci")" == "Native CI" ]]', promotion)
        self.assertIn('[[ "$(jq -r \'.head_sha\' <<<"$native_ci")" == "$actual_source_commit" ]]', promotion)
        self.assertIn('[[ "$(jq -r \'.status\' <<<"$native_ci")" == "completed" ]]', promotion)
        self.assertIn('[[ "$(jq -r \'.conclusion\' <<<"$native_ci")" == "success" ]]', promotion)
        self.assertIn('[[ "$actual_source_commit" == "$(git rev-parse origin/2.x)" ]]', promotion)

    def test_2x_native_ci_does_not_start_legacy_language_runtimes(self) -> None:
        content = read(".github/workflows/native-ci.yml")
        forbidden = (
            "actions/setup-python",
            "actions/setup-node",
            "python -m",
            "python3 ",
            "npm ",
            "packages/qiongli-literature-mcpb",
            "cross-platform-tests",
            "shell-release-gates",
            "bootstrap_qiongli",
        )
        for marker in forbidden:
            with self.subTest(marker=marker):
                self.assertNotIn(marker, content)
        self.assertIn("packages/qiongli-lite-mcp/Cargo.toml", content)
        self.assertIn("R2 Lite compatibility (Linux)", content)

    def test_ci_materializes_payloads_to_runner_temp_before_strict_research_validation(self) -> None:
        content = read(".github/workflows/ci.yml")
        materialize_cmd = 'python scripts/materialize_distribution_payloads.py --target all --out "$RUNNER_TEMP/qiongli-dist" --force'
        validate_cmd = 'python scripts/validate_research_standard.py --root "$RUNNER_TEMP/qiongli-dist" --strict'

        self.assertIn(materialize_cmd, content)
        self.assertIn(validate_cmd, content)
        self.assertLess(content.index(materialize_cmd), content.index(validate_cmd))
        self.assertNotIn("python scripts/materialize_distribution_payloads.py --target all --in-place", content)

    def test_ci_rejects_generated_payload_edits_before_sync_steps(self) -> None:
        content = read(".github/workflows/ci.yml")
        resolver_step = "      - name: Resolve generated payload comparison base"
        guard_cmd = "          python scripts/check_generated_payload_edits.py"
        frozen_guard_cmd = (
            "          python scripts/check_frozen_migration_baseline.py"
        )
        architecture_guard_cmd = (
            "          python scripts/check_frozen_2x_architecture_baseline.py"
        )
        materialize_cmd = 'python scripts/materialize_distribution_payloads.py --target all --out "$RUNNER_TEMP/qiongli-dist" --force'

        self.assertIn(resolver_step, content)
        self.assertIn(guard_cmd, content)
        self.assertIn(frozen_guard_cmd, content)
        self.assertIn(architecture_guard_cmd, content)
        self.assertIn(materialize_cmd, content)
        self.assertLess(content.index(resolver_step), content.index(guard_cmd))
        self.assertLess(content.index(guard_cmd), content.index(materialize_cmd))
        self.assertLess(content.index(frozen_guard_cmd), content.index(materialize_cmd))
        self.assertLess(
            content.index(architecture_guard_cmd), content.index(materialize_cmd)
        )

    def test_generated_payload_guard_uses_event_aware_comparison_base(self) -> None:
        content = read(".github/workflows/ci.yml")

        self.assertIn("PULL_REQUEST_BASE: ${{ github.base_ref }}", content)
        self.assertIn("PUSH_BEFORE: ${{ github.event.before }}", content)
        self.assertIn('base_ref="origin/$PULL_REQUEST_BASE"', content)
        self.assertIn('base_ref="$PUSH_BEFORE"', content)
        self.assertIn('base_ref="HEAD^"', content)
        self.assertIn('base_ref="$(git rev-list --max-parents=0 HEAD)"', content)
        self.assertIn('echo "base-ref=$base_ref" >> "$GITHUB_OUTPUT"', content)
        self.assertIn(
            "GENERATED_PAYLOAD_BASE: "
            "${{ steps.generated-payload-base.outputs.base-ref }}",
            content,
        )
        self.assertIn(
            "      - name: Check generated payload edits\n        shell: bash",
            content,
        )
        self.assertIn(
            "      - name: Protect frozen migration baseline\n        shell: bash",
            content,
        )
        self.assertIn(
            "      - name: Protect frozen 2.x architecture baseline\n"
            "        shell: bash",
            content,
        )
        self.assertNotIn("python scripts/validate_repository_source.py", content)
        self.assertIn('--base-ref "$GENERATED_PAYLOAD_BASE"', content)
        self.assertEqual(
            content.count('--base-ref "$GENERATED_PAYLOAD_BASE"'), 3
        )
        self.assertNotIn("--base-ref origin/dev", content)

    def test_ci_audits_staged_payload_after_injected_project_defaults(self) -> None:
        content = read(".github/workflows/ci.yml")
        inject_cmd = "bash scripts/inject_project_toml.sh"
        payload_cmd = 'python scripts/materialize_distribution_payloads.py --target all --out "$RUNNER_TEMP/qiongli-dist" --force'
        audit_cmd = 'python scripts/audit_distribution_payloads.py --root "$RUNNER_TEMP/qiongli-dist"'
        validate_cmd = 'python scripts/validate_research_standard.py --root "$RUNNER_TEMP/qiongli-dist" --strict'
        unit_cmd = "python -m unittest discover -s tests -v"

        self.assertIn(inject_cmd, content)
        self.assertIn(payload_cmd, content)
        self.assertIn(audit_cmd, content)
        self.assertIn(validate_cmd, content)
        self.assertIn(unit_cmd, content)
        self.assertLess(content.index(inject_cmd), content.index(payload_cmd))
        self.assertLess(content.index(payload_cmd), content.index(audit_cmd))
        self.assertLess(content.index(payload_cmd), content.index(validate_cmd))
        self.assertLess(content.index(payload_cmd), content.index(unit_cmd))

    def test_windows_ci_smoke_tier_skips_heavy_payload_materialization(self) -> None:
        content = read(".github/workflows/ci.yml")
        materialize_step = '''      - name: Materialize distribution payloads
        if: matrix.test-tier == 'full'
        run: python scripts/materialize_distribution_payloads.py --target all --out "$RUNNER_TEMP/qiongli-dist" --force'''
        audit_step = '''      - name: Audit staged distribution payloads
        if: matrix.test-tier == 'full'
        run: python scripts/audit_distribution_payloads.py --root "$RUNNER_TEMP/qiongli-dist"'''

        self.assertIn(materialize_step, content)
        self.assertIn(audit_step, content)

    def test_release_workflow_preserves_legacy_branches_and_blocks_native_publish(self) -> None:
        content = read("scripts/release_automation.sh")
        self.assertIn('DEV_PRERELEASE_BRANCH="dev"', content)
        self.assertIn('if is_prerelease_tag "$repo_tag" && [[ "$current_branch" == "$DEV_PRERELEASE_BRANCH" ]]; then', content)
        self.assertIn("Stable releases use primary branch ($primary_branch); prerelease releases may run from $DEV_PRERELEASE_BRANCH", content)
        self.assertIn('push_branch="$current_branch"', content)
        self.assertIn('source_branch="$(normalize_field "$version_input" source_branch)"', content)
        self.assertIn('if [[ "$release_line" == "native-2x" ]]; then', content)
        self.assertIn("RLS-201/PKG gate: native", content)
        self.assertIn("run pre from the ${source_branch} release line", content)

    def test_maintainer_policy_documents_official_plugin_and_branch_roles(self) -> None:
        content = read("docs/maintainer/release-branch-policy.md")
        self.assertIn("official public marketplace", content)
        self.assertIn("jxpeng98/skillsplace", content)
        self.assertIn("`dev`", content)
        self.assertIn("`main`", content)
        self.assertIn("stable release", content)

    def test_bilingual_policy_freezes_1x_and_assigns_native_work_to_2x(self) -> None:
        policies = {
            "docs/maintainer/release-branch-policy.md": (
                "No normal features.",
                "does **not**\ncontain the A8 workflow-filter changes",
                "90 days after Qiongli 2 stable",
                "immutable guard is preventive only",
                "manually dispatchable against a named `2.x` ref",
                "diagnostic and are not required checks",
            ),
            "docs/zh/maintainer/release-branch-policy.md": (
                "不接受常规功能",
                "**不包含**之后在 `dev` 提交的 A8",
                "Qiongli 2 stable 发布后 90 天",
                "immutable guard 才能",
                "指定的 `2.x` ref 手动触发",
                "诊断证据，不是 2.x 原生开发的 required checks",
            ),
        }

        for path, localized_markers in policies.items():
            content = read(path)
            with self.subTest(path=path):
                self.assertIn("`release/1.x-python`", content)
                self.assertIn("`v1.19.0-beta.1`", content)
                self.assertIn("`8d2e99866ce4c4efb8b3b5e0265c0c1f89a36b0f`", content)
                self.assertIn("`2.x`", content)
                self.assertIn("Rust", content)
                self.assertIn("pull request", content)
                self.assertIn("forward-port", content)
                self.assertIn("equivalence evidence", content)
                self.assertIn("frozen-baseline guard", content)
                self.assertIn(
                    "tooling/migration/baselines/v1.19.0-beta.1/manifest.json",
                    content,
                )
                self.assertIn("capture --check", content)
                self.assertIn("https://github.com/jxpeng98/qiongli/rules/18797579", content)
                self.assertIn("ruleset 18797579", content)
                self.assertIn("ruleset `18800504`", content)
                self.assertIn("`Native CI`", content)
                self.assertIn("`Native 2.x change boundary`", content)
                for context in (
                    "`Rust native foundation (Linux)`",
                    "`Rust native foundation (macOS)`",
                    "`Rust native foundation (Windows)`",
                ):
                    self.assertIn(context, content)
                for marker in localized_markers:
                    self.assertIn(marker, content)

    def test_maintainer_policy_documents_codex_dist_refs(self) -> None:
        content = read("docs/maintainer/release-branch-policy.md")
        self.assertIn("Codex dist refs", content)
        self.assertIn("refs/heads/codex/v<version>", content)
        self.assertIn("scripts/publish-codex-dist-ref.mjs", content)
        self.assertIn("Claude dist refs", content)
        self.assertIn("refs/heads/claude/v<version>", content)
        self.assertIn("legacy 1.x\nrelease postflight publishes platform dist refs", content)
        self.assertIn("A native 2.x alpha dry-run never publishes these refs", content)
        self.assertIn("plugins/qiongli/.codex-plugin/plugin.json", content)
        self.assertIn("plugins/qiongli-next/.codex-plugin/plugin.json", content)
        self.assertIn("plugins/qiongli-next/.claude-plugin/plugin.json", content)

    def test_maintainer_policy_documents_naming_decision(self) -> None:
        content = read("docs/maintainer/naming-policy.md")
        self.assertIn("**Qiongli**", content)
        self.assertIn("**Qiongli Zhengche**", content)
        self.assertIn("**Zhengche**", content)
        self.assertIn("qiongli", content)


if __name__ == "__main__":
    unittest.main()
