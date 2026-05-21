from __future__ import annotations

import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
RELEASE_AUTOMATION = REPO_ROOT / "scripts" / "release_automation.sh"
RELEASE_READY = REPO_ROOT / "scripts" / "release_ready.sh"
RELEASE_PREFLIGHT = REPO_ROOT / "scripts" / "release_preflight.sh"
RELEASE_POSTFLIGHT = REPO_ROOT / "scripts" / "release_postflight.sh"
PYPI_PREFLIGHT = REPO_ROOT / "scripts" / "pypi_preflight.sh"
RELEASE_WORKFLOW = REPO_ROOT / ".github" / "workflows" / "release-automation.yml"
PUBLISH_PYPI_WORKFLOW = REPO_ROOT / ".github" / "workflows" / "publish-pypi.yml"
VERIFY_RELEASE_TAG = REPO_ROOT / "scripts" / "verify_release_tag_version.sh"
CHANGELOG_SECTION = REPO_ROOT / "scripts" / "changelog_section.py"


class ReleaseAutomationTests(unittest.TestCase):
    def test_release_automation_script_supports_publish_mode(self) -> None:
        content = RELEASE_AUTOMATION.read_text(encoding="utf-8")

        self.assertIn("<pre|post|publish>", content)
        self.assertIn("publish --version 0.1.0", content)
        self.assertIn("publish --tag v0.1.0", content)
        self.assertIn("./scripts/release_ready.sh --version", content)
        self.assertIn("publish mode requires --version or --tag", content)
        self.assertIn('version_from_tag="$2"', content)
        self.assertIn('repo_tag_from_version="$(normalize_field "$version" repo_version)"', content)
        self.assertIn('repo_tag_from_tag="$(normalize_field "$version_from_tag" repo_version)"', content)
        self.assertIn('if [[ -n "$version" && -n "$version_from_tag" && "$repo_tag_from_version" != "$repo_tag_from_tag" ]]; then', content)
        self.assertIn("--maintainer-smoke", content)
        self.assertIn("git add CHANGELOG.md", content)
        self.assertIn('git add "release/${repo_tag}.md"', content)
        self.assertIn('git tag -a "$repo_tag"', content)
        self.assertIn('git push "$push_remote" "$push_branch" "$repo_tag"', content)
        self.assertIn('plugins/qiongli/.codex-plugin/plugin.json', content)
        self.assertIn('.claude-plugin/marketplace.json', content)
        self.assertIn('plugins/qiongli/.claude-plugin/plugin.json', content)
        self.assertIn('plugins/qiongli/gemini-extension.json', content)
        self.assertIn('plugins/qiongli/skills/qiongli-workflow', content)
        self.assertIn('packages/npm-qiongli', content)
        self.assertIn('npm_preflight.sh', content)
        self.assertIn('./scripts/release_postflight.sh --tag "$repo_tag"', content)

    def test_publish_mode_allows_beta_release_from_dev_only(self) -> None:
        content = RELEASE_AUTOMATION.read_text(encoding="utf-8")

        self.assertIn('DEV_PRERELEASE_BRANCH="dev"', content)
        self.assertIn('release_branch="$primary_branch"', content)
        self.assertIn('if is_prerelease_tag "$repo_tag" && [[ "$current_branch" == "$DEV_PRERELEASE_BRANCH" ]]; then', content)
        self.assertIn('release_branch="$DEV_PRERELEASE_BRANCH"', content)
        self.assertIn('Current branch: $current_branch; push branch: $push_branch; expected release branch: $release_branch', content)

    def test_release_postflight_waits_for_required_workflows(self) -> None:
        content = RELEASE_POSTFLIGHT.read_text(encoding="utf-8")

        self.assertIn('REQUIRED_WORKFLOWS=("CI" "Checkout Install Check")', content)
        self.assertNotIn('REQUIRED_WORKFLOWS=("CI" "Install Check")', content)
        self.assertIn("--wait-ci", content)
        self.assertIn("query_ci_status", content)
        self.assertIn('ci_json_file="$(mktemp)"', content)
        self.assertNotIn("CI_JSON_PAYLOAD=", content)
        self.assertIn('observed = sorted({r.get("name") or "unknown" for r in runs if r.get("head_sha") == commit})', content)
        self.assertIn('labels.append("observed=" + ",".join(observed))', content)
        self.assertIn('refs/remotes/origin/$branch', content)
        self.assertIn('refresh_branch_ref "$RELEASE_BRANCH" "$RELEASE_BRANCH_REF"', content)
        self.assertIn('git fetch --force --no-tags origin "$fetch_ref"', content)
        self.assertIn('python3 scripts/changelog_section.py --version "$version" --output "$TEMP_RELEASE_NOTES"', content)
        self.assertIn('RELEASE_NOTES_LABEL="CHANGELOG.md [${version}]"', content)
        self.assertIn('bash ./scripts/verify_release_tag_version.sh --tag "$TAG"', content)
        self.assertIn("gh release view", content)
        self.assertIn("--prerelease", content)
        self.assertIn('scripts/build_marketplace_artifacts.py --tag "$TAG" --dist-dir dist', content)
        self.assertIn('MARKETPLACE_ARTIFACTS=(', content)
        self.assertIn('gh release upload "$TAG" --repo "$REPO_SLUG" --clobber "${MARKETPLACE_ARTIFACTS[@]}"', content)

    def test_release_postflight_accepts_beta_tags_reachable_from_dev(self) -> None:
        content = RELEASE_POSTFLIGHT.read_text(encoding="utf-8")

        self.assertIn('DEV_PRERELEASE_BRANCH="dev"', content)
        self.assertIn('select_release_branch_ref()', content)
        self.assertIn('if is_prerelease_tag "$tag" && branch_ref="$(detect_branch_ref "$DEV_PRERELEASE_BRANCH")"; then', content)
        self.assertIn('RELEASE_BRANCH="${release_branch_record%%$\'\\t\'*}"', content)
        self.assertIn('refresh_branch_ref "$RELEASE_BRANCH" "$RELEASE_BRANCH_REF"', content)
        self.assertIn('git merge-base --is-ancestor "$LOCAL_TAG_COMMIT" "$RELEASE_BRANCH_REF"', content)
        self.assertIn('query_ci_status "$REPO_SLUG" "$RELEASE_BRANCH" "$LOCAL_TAG_COMMIT"', content)

    def test_release_ready_includes_plugin_distribution_versions(self) -> None:
        content = RELEASE_READY.read_text(encoding="utf-8")

        self.assertIn('plugins/qiongli/.codex-plugin/plugin.json', content)
        self.assertIn('.claude-plugin/marketplace.json', content)
        self.assertIn('plugins/qiongli/.claude-plugin/plugin.json', content)
        self.assertIn('plugins/qiongli/gemini-extension.json', content)
        self.assertIn('packages/npm-qiongli|packages/npm-qiongli/*', content)
        self.assertIn(
            'plugins/qiongli/skills/qiongli-workflow|plugins/qiongli/skills/qiongli-workflow/*',
            content,
        )

    def test_release_ready_does_not_print_manual_publish_steps(self) -> None:
        content = RELEASE_READY.read_text(encoding="utf-8")

        self.assertIn("prepare+verify completed; publish mode owns commit/tag/push", content)
        self.assertNotIn('git tag -a ${REPO_TAG}', content)
        self.assertNotIn('git push origin main --tags', content)

    def test_release_preflight_fails_fast_on_logged_stage_errors(self) -> None:
        content = RELEASE_PREFLIGHT.read_text(encoding="utf-8")

        self.assertIn("run_logged_stage()", content)
        self.assertIn('statuses=("${PIPESTATUS[@]}")', content)
        self.assertIn('"[preflight] FAIL: ${label} failed with exit code ${command_status}"', content)
        self.assertIn('run_logged_stage "validator" "$validator_log" "${validate_cmd[@]}"', content)
        self.assertIn('run_logged_stage "unit tests" "$unit_log" python3 -m unittest discover -s tests -v', content)
        self.assertIn('run_logged_stage "smoke (${smoke_tier} tier)" "$smoke_log" ./scripts/run_beta_smoke.sh --tier "$smoke_tier"', content)

    def test_release_preflight_syncs_npm_payload_before_tests(self) -> None:
        content = RELEASE_PREFLIGHT.read_text(encoding="utf-8")

        self.assertIn('echo "[preflight] sync npm payload"', content)
        self.assertIn("python3 scripts/sync_npm_package_payload.py", content)
        self.assertLess(
            content.index("python3 scripts/sync_npm_package_payload.py"),
            content.index('run_logged_stage "validator" "$validator_log" "${validate_cmd[@]}"'),
        )
        self.assertLess(
            content.index("python3 scripts/sync_npm_package_payload.py"),
            content.index('run_logged_stage "unit tests" "$unit_log" python3 -m unittest discover -s tests -v'),
        )

    def test_release_preflight_runs_controller_mode_evals_as_warning_stage(self) -> None:
        content = RELEASE_PREFLIGHT.read_text(encoding="utf-8")

        self.assertIn("run_warning_stage()", content)
        self.assertIn('eval_log="$(mktemp -t qiongli-controller-evals.XXXXXX.log)"', content)
        self.assertIn(
            'run_warning_stage "controller-mode evals" "$eval_log" python3 scripts/run_controller_mode_evals.py evals/controller_modes',
            content,
        )
        self.assertIn('"[preflight] WARN: ${label} failed with exit code ${command_status}"', content)

    def test_release_preflight_preserves_stage_logs_on_failure(self) -> None:
        content = RELEASE_PREFLIGHT.read_text(encoding="utf-8")

        self.assertIn("cleanup_logs()", content)
        self.assertIn('FAILED_STAGE=""', content)
        self.assertIn('FAILED_LOG=""', content)
        self.assertIn('FAILED_STATUS=""', content)
        self.assertIn('FAILED_STAGE="$label"', content)
        self.assertIn('FAILED_LOG="$log_file"', content)
        self.assertIn('FAILED_STATUS="$command_status"', content)
        self.assertIn('"[preflight] failure summary: ${FAILED_STAGE} exited with ${FAILED_STATUS}"', content)
        self.assertIn('tail -n 120 "$FAILED_LOG"', content)
        self.assertIn('local status="$?"', content)
        self.assertIn('if [[ "$status" -eq 0 ]]; then', content)
        self.assertNotIn('trap \'rm -f "$validator_log" "$unit_log" "$smoke_log"\' EXIT', content)

    def test_release_preflight_reports_missing_pyyaml_dependency(self) -> None:
        content = RELEASE_PREFLIGHT.read_text(encoding="utf-8")

        self.assertIn('require_python_module yaml PyYAML', content)
        self.assertIn("[preflight] missing Python dependency: ${package} (module: ${module})", content)
        self.assertIn("python3 -m pip install -e .", content)

    def test_release_preflight_does_not_print_manual_publish_steps(self) -> None:
        content = RELEASE_PREFLIGHT.read_text(encoding="utf-8")

        self.assertIn("preflight completed; publish mode owns tag/push", content)
        self.assertNotIn('git tag -a $TAG', content)
        self.assertNotIn('git push origin $TAG', content)

    def test_pypi_preflight_checks_release_build_dependencies(self) -> None:
        content = PYPI_PREFLIGHT.read_text(encoding="utf-8")

        self.assertIn("require_python_module build build", content)
        self.assertIn("require_python_module twine twine", content)
        self.assertIn("python3 -m pip install -e . build twine", content)

    def test_pypi_preflight_does_not_print_manual_publish_steps(self) -> None:
        content = PYPI_PREFLIGHT.read_text(encoding="utf-8")

        self.assertIn("package preflight completed; publish mode owns tag/release flow", content)
        self.assertNotIn("git tag v<version>", content)
        self.assertNotIn("Publish to TestPyPI", content)

    def test_changelog_section_script_extracts_versioned_sections(self) -> None:
        content = CHANGELOG_SECTION.read_text(encoding="utf-8")

        self.assertIn('HEADING_RE = re.compile(r"^## \\[(?P<version>[^\\]]+)\\](?P<suffix>.*)$")', content)
        self.assertIn('parser.add_argument("--version", required=True', content)
        self.assertIn('print(f"[changelog] missing version section: {args.version}"', content)
        self.assertIn('Path(args.output).write_text(section, encoding="utf-8")', content)

    def test_prerelease_note_generator_points_to_publish_mode(self) -> None:
        content = (REPO_ROOT / "scripts" / "generate_release_notes.sh").read_text(encoding="utf-8")

        self.assertIn('PUBLISH_CMD="./scripts/release_automation.sh publish --tag ${TAG} --skip-bump"', content)
        self.assertNotIn('PUBLISH_CMD="./scripts/release_automation.sh publish --version ${VERSION_HINT} --skip-bump"', content)
        self.assertIn('PUBLISH_CMD="${PUBLISH_CMD} --from-tag ${FROM_TAG}"', content)
        self.assertIn('${PUBLISH_CMD}', content)
        self.assertIn('release_ready.sh --version', content)
        self.assertNotIn('git push origin main --tags', content)

    def test_release_workflow_exposes_publish_mode(self) -> None:
        content = RELEASE_WORKFLOW.read_text(encoding="utf-8")

        self.assertIn("push:", content)
        self.assertIn('tags:\n      - "v*"', content)
        self.assertIn("- publish", content)
        self.assertIn("maintainer_smoke:", content)
        self.assertIn("version:", content)
        self.assertIn('if [[ -n "${{ inputs.version }}" ]]; then', content)
        self.assertIn('elif [[ -n "$tag" ]]; then', content)
        self.assertIn('args+=(--tag "$tag")', content)
        self.assertNotIn("publish mode requires 'version' input", content)
        self.assertIn("fetch-depth: 0", content)
        self.assertIn('git fetch --force --prune origin +refs/heads/*:refs/remotes/origin/* +refs/tags/*:refs/tags/*', content)
        self.assertIn('if [[ "${{ github.event_name }}" == "push" ]]; then', content)
        self.assertIn('mode="post"', content)
        self.assertIn('args+=(--maintainer-smoke)', content)
        self.assertIn('args+=(--create-release)', content)
        self.assertIn('bash scripts/verify_release_tag_version.sh --tag "$tag"', content)
        self.assertIn('git config user.name "github-actions[bot]"', content)
        self.assertIn("python -m pip install -e . build twine", content)

    def test_publish_pypi_workflow_verifies_tag_matches_repo_version(self) -> None:
        content = PUBLISH_PYPI_WORKFLOW.read_text(encoding="utf-8")

        self.assertIn('bash scripts/verify_release_tag_version.sh --tag "${GITHUB_REF_NAME}"', content)

    def test_verify_release_tag_script_checks_expected_files(self) -> None:
        content = VERIFY_RELEASE_TAG.read_text(encoding="utf-8")

        self.assertIn('scripts/sync_versions.py "$TAG" --print-field package_version', content)
        self.assertIn('scripts/sync_versions.py "$TAG" --print-field npm_version', content)
        self.assertIn('pyproject.toml', content)
        self.assertIn('qiongli/__init__.py', content)
        self.assertIn('skills/registry.yaml', content)
        self.assertIn('qiongli-workflow/VERSION', content)
        self.assertIn('qiongli-workflow/skills/registry.yaml', content)
        self.assertIn('packages/npm-qiongli/package.json', content)
        self.assertIn('packages/npm-qiongli/payload/qiongli-workflow/VERSION', content)
        self.assertIn('packages/npm-qiongli/payload/qiongli-workflow/skills/registry.yaml', content)
        self.assertIn('packages/npm-qiongli/python-runtime/qiongli/__init__.py', content)
        self.assertIn('packages/npm-qiongli/python-runtime/skills/registry.yaml', content)
        self.assertIn('plugins/qiongli/.codex-plugin/plugin.json', content)
        self.assertIn('plugins/qiongli/skills/qiongli-workflow/VERSION', content)
        self.assertIn('plugins/qiongli/skills/qiongli-workflow/skills/registry.yaml', content)
        self.assertIn('.claude-plugin/marketplace.json', content)
        self.assertIn('plugins/qiongli/.claude-plugin/plugin.json', content)
        self.assertIn('plugins/qiongli/gemini-extension.json', content)


if __name__ == "__main__":
    unittest.main()
