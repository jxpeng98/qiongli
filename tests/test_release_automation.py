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
PUBLISH_NPM_WORKFLOW = REPO_ROOT / ".github" / "workflows" / "publish-npm.yml"
VERIFY_RELEASE_TAG = REPO_ROOT / "scripts" / "verify_release_tag_version.sh"
CHANGELOG_SECTION = REPO_ROOT / "scripts" / "changelog_section.py"
RELEASE_AUTOMATION_DOC = REPO_ROOT / "release" / "automation.md"
PUBLISH_PYPI_DOC = REPO_ROOT / "docs" / "advanced" / "publish-pypi.md"
PUBLISH_PYPI_ZH_DOC = REPO_ROOT / "docs" / "zh" / "advanced" / "publish-pypi.md"
RELEASE_BRANCH_POLICY_DOC = REPO_ROOT / "docs" / "maintainer" / "release-branch-policy.md"
RELEASE_BRANCH_POLICY_ZH_DOC = REPO_ROOT / "docs" / "zh" / "maintainer" / "release-branch-policy.md"


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
        self.assertIn('acceptance_out="release/acceptance/${repo_tag}-receipt.md"', content)
        self.assertIn('./scripts/release_postflight.sh --tag "$repo_tag" --acceptance-out "$acceptance_out"', content)
        self.assertIn('git add "$acceptance_out"', content)
        self.assertIn('chore: record release ${repo_tag} acceptance', content)
        self.assertIn('git push "$push_remote" "$push_branch"', content)
        self.assertIn('plugins/qiongli/.codex-plugin/plugin.json', content)
        self.assertIn('plugins/qiongli/.claude-plugin/plugin.json', content)
        self.assertIn('plugins/qiongli/gemini-extension.json', content)
        self.assertIn('plugins/qiongli/skills/qiongli-workflow', content)
        self.assertIn('docs/reference/skills.md', content)
        self.assertIn('docs/zh/reference/skills.md', content)
        self.assertNotIn('qiongli-workflow/skills/registry.yaml', content)
        self.assertIn('qiongli/payload', content)
        self.assertIn('packages/npm-qiongli', content)
        self.assertIn('package-lock.json', content)
        self.assertIn('npm_preflight.sh', content)
        self.assertIn('./scripts/release_postflight.sh --tag "$repo_tag"', content)

    def test_docs_define_optional_beta_channel_policy(self) -> None:
        docs = "\n".join(
            path.read_text(encoding="utf-8")
            for path in (
                RELEASE_AUTOMATION_DOC,
                PUBLISH_PYPI_DOC,
                PUBLISH_PYPI_ZH_DOC,
                RELEASE_BRANCH_POLICY_DOC,
                RELEASE_BRANCH_POLICY_ZH_DOC,
            )
        )

        self.assertIn("Beta releases are optional validation releases", docs)
        self.assertIn("Beta channel policy", docs)
        self.assertIn("Beta 通道策略", docs)
        self.assertIn("beta 不是每个 stable release 的必经步骤", docs)
        self.assertIn("npm `latest` advances", docs)
        self.assertIn("npm `next` remains on the previous beta", docs)
        self.assertIn("不要为了移动 `next` 而机械发 beta", docs)

    def test_publish_mode_allows_beta_release_from_dev_only(self) -> None:
        content = RELEASE_AUTOMATION.read_text(encoding="utf-8")

        self.assertIn('DEV_PRERELEASE_BRANCH="dev"', content)
        self.assertIn('release_branch="$primary_branch"', content)
        self.assertIn('if is_prerelease_tag "$repo_tag" && [[ "$current_branch" == "$DEV_PRERELEASE_BRANCH" ]]; then', content)
        self.assertIn('release_branch="$DEV_PRERELEASE_BRANCH"', content)
        self.assertIn('Current branch: $current_branch; push branch: $push_branch; expected release branch: $release_branch', content)

    def test_publish_mode_syncs_generated_payloads_before_commit_and_tag(self) -> None:
        content = RELEASE_AUTOMATION.read_text(encoding="utf-8")

        sync_skill = 'bash scripts/sync_skill_package.sh --target all'
        sync_npm = "python3 scripts/sync_npm_package_payload.py"
        audit = "python3 scripts/audit_distribution_payloads.py"
        verify = 'bash scripts/verify_release_tag_version.sh --tag "$repo_tag"'
        git_add = "git add \\"
        tag = 'git tag -a "$repo_tag"'

        for expected in (sync_skill, sync_npm, audit, verify):
            self.assertIn(expected, content)
            self.assertLess(content.index(expected), content.index(git_add))
            self.assertLess(content.index(expected), content.index(tag))

    def test_release_postflight_waits_for_branch_and_tag_workflows(self) -> None:
        content = RELEASE_POSTFLIGHT.read_text(encoding="utf-8")

        self.assertIn('BRANCH_REQUIRED_WORKFLOWS=("CI" "Checkout Install Check")', content)
        self.assertIn('TAG_REQUIRED_WORKFLOWS=("Publish to PyPI" "Publish to npm")', content)
        self.assertNotIn('REQUIRED_WORKFLOWS=("CI" "Install Check")', content)
        self.assertIn("--wait-ci", content)
        self.assertIn("query_actions_status", content)
        self.assertIn('ci_json_file="$(mktemp)"', content)
        self.assertNotIn("CI_JSON_PAYLOAD=", content)
        self.assertIn('observed = sorted({r.get("name") or "unknown" for r in runs if r.get("head_sha") == commit})', content)
        self.assertIn('labels.append("observed=" + ",".join(observed))', content)
        self.assertIn('query_actions_status "$REPO_SLUG" "$RELEASE_BRANCH" "$LOCAL_TAG_COMMIT" "${BRANCH_REQUIRED_WORKFLOWS[@]}"', content)
        self.assertIn('query_actions_status "$REPO_SLUG" "$TAG" "$LOCAL_TAG_COMMIT" "${TAG_REQUIRED_WORKFLOWS[@]}"', content)
        self.assertIn('CI_STATUS="success:branch-and-tag"', content)
        self.assertIn('refs/remotes/origin/$branch', content)
        self.assertIn('refresh_branch_ref "$RELEASE_BRANCH" "$RELEASE_BRANCH_REF"', content)
        self.assertIn('git fetch --force --no-tags origin "$fetch_ref"', content)
        self.assertIn('python3 scripts/changelog_section.py --version "$version" --output "$TEMP_RELEASE_NOTES"', content)
        self.assertIn('RELEASE_NOTES_LABEL="CHANGELOG.md [${version}]"', content)
        self.assertIn('bash ./scripts/verify_release_tag_version.sh --tag "$TAG"', content)
        self.assertIn("gh release view", content)
        self.assertIn("--prerelease", content)
        self.assertIn('scripts/build_plugin_artifacts.py --tag "$TAG" --dist-dir dist', content)
        self.assertIn('PLUGIN_ARTIFACTS=(', content)
        self.assertIn('"dist/qiongli-core-codex-plugin-${TAG}.tar.gz"', content)
        self.assertIn('"dist/qiongli-economics-codex-plugin-${TAG}.tar.gz"', content)
        self.assertIn('"dist/qiongli-accounting-codex-plugin-${TAG}.tar.gz"', content)
        self.assertIn('"dist/qiongli-business-codex-plugin-${TAG}.tar.gz"', content)
        self.assertIn('"dist/qiongli-finance-codex-plugin-${TAG}.tar.gz"', content)
        self.assertIn('"dist/qiongli-political-economy-codex-plugin-${TAG}.tar.gz"', content)
        self.assertIn('"dist/qiongli-geoeconomics-codex-plugin-${TAG}.tar.gz"', content)
        self.assertIn('"dist/qiongli-economics-accounting-codex-plugin-${TAG}.tar.gz"', content)
        self.assertIn('"dist/qiongli-core-claude-plugin-${TAG}.tar.gz"', content)
        self.assertIn('"dist/qiongli-economics-claude-plugin-${TAG}.tar.gz"', content)
        self.assertIn('"dist/qiongli-accounting-claude-plugin-${TAG}.tar.gz"', content)
        self.assertIn('"dist/qiongli-business-claude-plugin-${TAG}.tar.gz"', content)
        self.assertIn('"dist/qiongli-finance-claude-plugin-${TAG}.tar.gz"', content)
        self.assertIn('"dist/qiongli-political-economy-claude-plugin-${TAG}.tar.gz"', content)
        self.assertIn('"dist/qiongli-geoeconomics-claude-plugin-${TAG}.tar.gz"', content)
        self.assertIn('"dist/qiongli-economics-accounting-claude-plugin-${TAG}.tar.gz"', content)
        self.assertIn('"dist/qiongli-claude-desktop-skill-core-${TAG}.zip"', content)
        self.assertIn('"dist/qiongli-claude-desktop-skill-economics-${TAG}.zip"', content)
        self.assertIn('"dist/qiongli-claude-desktop-skill-business-${TAG}.zip"', content)
        self.assertIn('"dist/qiongli-claude-desktop-skill-finance-${TAG}.zip"', content)
        self.assertIn('"dist/qiongli-claude-desktop-skill-political-economy-${TAG}.zip"', content)
        self.assertIn('"dist/qiongli-claude-desktop-skill-geoeconomics-${TAG}.zip"', content)
        self.assertIn('"dist/qiongli-claude-desktop-skill-${TAG}.zip"', content)
        self.assertIn('gh release upload "$TAG" --repo "$REPO_SLUG" --clobber "${PLUGIN_ARTIFACTS[@]}"', content)
        self.assertIn('release_args+=("${PLUGIN_ARTIFACTS[@]}")', content)

    def test_release_postflight_accepts_beta_tags_reachable_from_dev(self) -> None:
        content = RELEASE_POSTFLIGHT.read_text(encoding="utf-8")

        self.assertIn('DEV_PRERELEASE_BRANCH="dev"', content)
        self.assertIn('select_release_branch_ref()', content)
        self.assertIn('if is_prerelease_tag "$tag" && branch_ref="$(detect_branch_ref "$DEV_PRERELEASE_BRANCH")"; then', content)
        self.assertIn('RELEASE_BRANCH="${release_branch_record%%$\'\\t\'*}"', content)
        self.assertIn('refresh_branch_ref "$RELEASE_BRANCH" "$RELEASE_BRANCH_REF"', content)
        self.assertIn('git merge-base --is-ancestor "$LOCAL_TAG_COMMIT" "$RELEASE_BRANCH_REF"', content)
        self.assertIn('query_actions_status "$REPO_SLUG" "$RELEASE_BRANCH" "$LOCAL_TAG_COMMIT" "${BRANCH_REQUIRED_WORKFLOWS[@]}"', content)

    def test_release_ready_includes_plugin_distribution_versions(self) -> None:
        content = RELEASE_READY.read_text(encoding="utf-8")

        self.assertIn('plugins/qiongli/.codex-plugin/plugin.json', content)
        self.assertIn('plugins/qiongli/.claude-plugin/plugin.json', content)
        self.assertIn('plugins/qiongli/gemini-extension.json', content)
        self.assertIn('packages/npm-qiongli|packages/npm-qiongli/*', content)
        self.assertIn('qiongli/payload|qiongli/payload/*', content)
        self.assertIn('package-lock.json', content)
        self.assertIn('docs/reference/skills.md', content)
        self.assertIn('docs/zh/reference/skills.md', content)
        self.assertIn(
            'plugins/qiongli/skills/qiongli-workflow|plugins/qiongli/skills/qiongli-workflow/*',
            content,
        )
        self.assertNotIn('qiongli-workflow/skills/registry.yaml', content)

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

        sync_skill = 'bash "$ROOT_DIR/scripts/sync_skill_package.sh" --target all'
        self.assertIn('echo "[preflight] sync npm payload"', content)
        self.assertIn(sync_skill, content)
        self.assertIn("python3 scripts/sync_npm_package_payload.py", content)
        self.assertIn('echo "[preflight] sync skill reference docs"', content)
        self.assertIn("python3 scripts/generate_skill_docs.py", content)
        self.assertLess(
            content.index(sync_skill),
            content.index("python3 scripts/audit_distribution_payloads.py"),
        )
        self.assertLess(
            content.index("python3 scripts/sync_npm_package_payload.py"),
            content.index("python3 scripts/generate_skill_docs.py"),
        )
        self.assertLess(
            content.index("python3 scripts/generate_skill_docs.py"),
            content.index('run_logged_stage "validator" "$validator_log" "${validate_cmd[@]}"'),
        )
        self.assertLess(
            content.index("python3 scripts/generate_skill_docs.py"),
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

    def test_release_workflow_is_diagnostic_wrapper_not_publish_entrypoint(self) -> None:
        content = RELEASE_WORKFLOW.read_text(encoding="utf-8")

        self.assertIn("workflow_dispatch:", content)
        self.assertNotIn("push:", content)
        self.assertNotIn('tags:\n      - "v*"', content)
        self.assertNotIn("- publish", content)
        self.assertIn("maintainer_smoke:", content)
        self.assertNotIn("      version:\n", content)
        self.assertNotIn('if [[ -n "${{ inputs.version }}" ]]; then', content)
        self.assertNotIn('elif [[ -n "$tag" ]]; then', content)
        self.assertIn('args+=(--tag "$tag")', content)
        self.assertNotIn("publish mode requires 'version' input", content)
        self.assertIn("fetch-depth: 0", content)
        self.assertIn('git fetch --force --prune origin +refs/heads/*:refs/remotes/origin/* +refs/tags/*:refs/tags/*', content)
        self.assertNotIn('if [[ "${{ github.event_name }}" == "push" ]]; then', content)
        self.assertNotIn('mode="post"', content)
        self.assertIn('args+=(--maintainer-smoke)', content)
        self.assertNotIn("if: ${{ github.event_name == 'push' || inputs.mode != 'publish' }}", content)
        self.assertNotIn('if [[ "${{ github.event_name }}" == "push" || "${{ inputs.mode }}" != "publish" ]]; then', content)
        self.assertNotIn('if [[ "$mode" == "publish"', content)
        self.assertIn('args+=(--create-release)', content)
        self.assertIn('bash scripts/verify_release_tag_version.sh --tag "$tag"', content)
        self.assertIn('git config user.name "github-actions[bot]"', content)
        self.assertIn("python -m pip install -e . build twine", content)
        self.assertIn("./scripts/release_automation.sh \"$mode\" \"${args[@]}\"", content)

    def test_publish_pypi_workflow_verifies_tag_matches_repo_version(self) -> None:
        content = PUBLISH_PYPI_WORKFLOW.read_text(encoding="utf-8")

        self.assertIn('bash scripts/verify_release_tag_version.sh --tag "${RELEASE_TAG}"', content)
        self.assertNotIn('bash scripts/verify_release_tag_version.sh --tag "${GITHUB_REF_NAME}"', content)

    def test_tag_publish_workflows_do_not_expose_manual_publish_dispatch(self) -> None:
        for workflow in (PUBLISH_PYPI_WORKFLOW, PUBLISH_NPM_WORKFLOW):
            with self.subTest(workflow=workflow.name):
                content = workflow.read_text(encoding="utf-8")

                self.assertNotIn("workflow_dispatch:", content)
                self.assertNotIn("inputs.tag", content)
                self.assertIn("push:", content)
                self.assertIn('tags:\n      - "v*"', content)
                self.assertIn("ref: ${{ github.ref }}", content)
                self.assertIn("RELEASE_TAG: ${{ github.ref_name }}", content)
                self.assertIn('bash scripts/verify_release_tag_version.sh --tag "${RELEASE_TAG}"', content)

    def test_tag_publish_workflows_sync_generated_payloads_before_version_verify(self) -> None:
        for workflow in (PUBLISH_PYPI_WORKFLOW, PUBLISH_NPM_WORKFLOW):
            with self.subTest(workflow=workflow.name):
                content = workflow.read_text(encoding="utf-8")

                verify = 'bash scripts/verify_release_tag_version.sh --tag "${RELEASE_TAG}"'
                install = "python -m pip install -e ."
                self.assertIn("bash scripts/sync_skill_package.sh --target all", content)
                self.assertIn("python3 scripts/sync_npm_package_payload.py", content)
                self.assertIn(install, content)
                self.assertLess(content.index(install), content.index("bash scripts/sync_skill_package.sh --target all"))
                self.assertLess(content.index(install), content.index("python3 scripts/sync_npm_package_payload.py"))
                self.assertLess(content.index("bash scripts/sync_skill_package.sh --target all"), content.index(verify))
                self.assertLess(content.index("python3 scripts/sync_npm_package_payload.py"), content.index(verify))

    def test_release_workflow_syncs_generated_payloads_before_version_verify(self) -> None:
        content = RELEASE_WORKFLOW.read_text(encoding="utf-8")

        verify = 'bash scripts/verify_release_tag_version.sh --tag "$tag"'
        install = "python -m pip install -e . build twine"
        self.assertIn("bash scripts/sync_skill_package.sh --target all", content)
        self.assertIn("python3 scripts/sync_npm_package_payload.py", content)
        self.assertIn(install, content)
        self.assertLess(content.index(install), content.index("bash scripts/sync_skill_package.sh --target all"))
        self.assertLess(content.index(install), content.index("python3 scripts/sync_npm_package_payload.py"))
        self.assertLess(content.index("bash scripts/sync_skill_package.sh --target all"), content.index(verify))
        self.assertLess(content.index("python3 scripts/sync_npm_package_payload.py"), content.index(verify))

    def test_verify_release_tag_script_checks_expected_files(self) -> None:
        content = VERIFY_RELEASE_TAG.read_text(encoding="utf-8")

        self.assertIn('scripts/sync_versions.py "$TAG" --print-field package_version', content)
        self.assertIn('scripts/sync_versions.py "$TAG" --print-field npm_version', content)
        self.assertIn('pyproject.toml', content)
        self.assertIn('qiongli/__init__.py', content)
        self.assertIn('skills/registry.yaml', content)
        self.assertIn('qiongli-workflow/VERSION', content)
        self.assertNotIn('actual_workflow_registry_version', content)
        self.assertNotIn('Path("qiongli-workflow/skills/registry.yaml")', content)
        self.assertNotIn('echo "[verify-release-tag] qiongli-workflow/skills/registry.yaml mismatch', content)
        self.assertIn('qiongli/payload/qiongli-workflow/VERSION', content)
        self.assertIn('qiongli/payload/qiongli-workflow/skills/registry.yaml', content)
        self.assertIn('qiongli/payload/skills/registry.yaml', content)
        self.assertIn('packages/npm-qiongli/package.json', content)
        self.assertIn('package-lock.json', content)
        self.assertIn('packages/npm-qiongli/payload/qiongli-workflow/VERSION', content)
        self.assertIn('packages/npm-qiongli/payload/qiongli-workflow/skills/registry.yaml', content)
        self.assertIn('packages/npm-qiongli/python-runtime/qiongli/__init__.py', content)
        self.assertIn('packages/npm-qiongli/python-runtime/skills/registry.yaml', content)
        self.assertIn('plugins/qiongli/.codex-plugin/plugin.json', content)
        self.assertIn('plugins/qiongli/skills/qiongli-workflow/VERSION', content)
        self.assertIn('plugins/qiongli/skills/qiongli-workflow/skills/registry.yaml', content)
        self.assertIn('plugins/qiongli/.claude-plugin/plugin.json', content)
        self.assertIn('plugins/qiongli/gemini-extension.json', content)


if __name__ == "__main__":
    unittest.main()
