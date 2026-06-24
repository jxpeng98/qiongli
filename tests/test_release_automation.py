from __future__ import annotations

import unittest
from pathlib import Path

from qiongli.source_layout import RepoLayout


REPO_ROOT = Path(__file__).resolve().parents[1]
LAYOUT = RepoLayout(REPO_ROOT)
RELEASE_AUTOMATION = LAYOUT.scripts / "release_automation.sh"
RELEASE_READY = LAYOUT.scripts / "release_ready.sh"
RELEASE_PREFLIGHT = LAYOUT.scripts / "release_preflight.sh"
RELEASE_POSTFLIGHT = LAYOUT.scripts / "release_postflight.sh"
PYPI_PREFLIGHT = LAYOUT.scripts / "pypi_preflight.sh"
RELEASE_WORKFLOW = REPO_ROOT / ".github" / "workflows" / "release-automation.yml"
INSTALL_CHECK_WORKFLOW = REPO_ROOT / ".github" / "workflows" / "install-check.yml"
MACOS_INSTALL_CHECK_WORKFLOW = REPO_ROOT / ".github" / "workflows" / "install-check-macos.yml"
AUTO_RERUN_WORKFLOW = REPO_ROOT / ".github" / "workflows" / "auto-rerun-failed-actions.yml"
PUBLISH_PYPI_WORKFLOW = REPO_ROOT / ".github" / "workflows" / "publish-pypi.yml"
PUBLISH_NPM_WORKFLOW = REPO_ROOT / ".github" / "workflows" / "publish-npm.yml"
VERIFY_RELEASE_TAG = LAYOUT.scripts / "verify_release_tag_version.sh"
CHANGELOG_SECTION = LAYOUT.scripts / "changelog_section.py"
RELEASE_AUTOMATION_DOC = REPO_ROOT / "tooling" / "release" / "automation.md"
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
        self.assertIn('./scripts/release_ready.sh "${release_ready_args[@]}"', content)
        self.assertIn("publish mode requires --version or --tag", content)
        self.assertIn('version_from_tag="$2"', content)
        self.assertIn('repo_tag_from_version="$(normalize_field "$version" repo_version)"', content)
        self.assertIn('repo_tag_from_tag="$(normalize_field "$version_from_tag" repo_version)"', content)
        self.assertIn('if [[ -n "$version" && -n "$version_from_tag" && "$repo_tag_from_version" != "$repo_tag_from_tag" ]]; then', content)
        self.assertIn("--maintainer-smoke", content)
        self.assertIn("git add CHANGELOG.md", content)
        self.assertIn('git add "tooling/release/${repo_tag}.md"', content)
        self.assertIn('git tag -a "$repo_tag"', content)
        self.assertIn('git push "$push_remote" "$push_branch"', content)
        self.assertIn('git push "$push_remote" "$repo_tag"', content)
        self.assertNotIn('git push "$push_remote" "$push_branch" "$repo_tag"', content)
        self.assertIn('acceptance_out="tooling/release/acceptance/${repo_tag}-receipt.md"', content)
        self.assertIn('./scripts/release_postflight.sh --tag "$repo_tag" --acceptance-out "$acceptance_out"', content)
        self.assertIn('git add "$acceptance_out"', content)
        self.assertIn('chore: record release ${repo_tag} acceptance', content)
        self.assertIn('git push "$push_remote" "$push_branch"', content)
        self.assertIn('content/distribution/plugins.yaml', content)
        self.assertIn('tooling/scripts/build_plugin_artifacts.py', content)
        self.assertIn('tooling/scripts/materialize_distribution_payloads.py', content)
        self.assertNotIn('packages/qiongli-plugin/.codex-plugin/plugin.json', content)
        self.assertNotIn('packages/qiongli-next-plugin', content)
        self.assertIn('content/workflow/SKILL.md', content)
        self.assertIn('content/workflow/VERSION', content)
        self.assertIn('content/skills/registry.yaml', content)
        self.assertIn('docs/reference/skills.md', content)
        self.assertIn('docs/zh/reference/skills.md', content)
        self.assertIn('uv.lock', content)
        self.assertNotIn('qiongli-workflow/VERSION', content)
        self.assertNotIn('qiongli-workflow/skills/registry.yaml', content)
        self.assertNotIn('      skills/registry.yaml \\', content)
        self.assertIn('packages/npm-qiongli', content)
        self.assertIn('package-lock.json', content)
        self.assertIn('npm_preflight.sh', content)
        self.assertIn('release_ready_args=(--version "$version_input")', content)
        self.assertNotIn("python3 scripts/materialize_distribution_payloads.py --target all --in-place", content)
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
        self.assertIn("before tag creation", docs)
        self.assertIn("创建 tag 前", docs)

    def test_publish_mode_allows_beta_release_from_dev_only(self) -> None:
        content = RELEASE_AUTOMATION.read_text(encoding="utf-8")

        self.assertIn('DEV_PRERELEASE_BRANCH="dev"', content)
        self.assertIn('release_branch="$primary_branch"', content)
        self.assertIn('if is_prerelease_tag "$repo_tag" && [[ "$current_branch" == "$DEV_PRERELEASE_BRANCH" ]]; then', content)
        self.assertIn('release_branch="$DEV_PRERELEASE_BRANCH"', content)
        self.assertIn('Current branch: $current_branch; push branch: $push_branch; expected release branch: $release_branch', content)

    def test_publish_mode_uses_release_ready_staging_before_commit_and_tag(self) -> None:
        content = RELEASE_AUTOMATION.read_text(encoding="utf-8")

        release_ready = '"${release_ready_args[@]}"'
        git_add = "git add \\"
        tag = 'git tag -a "$repo_tag"'

        self.assertNotIn("sync_generated_distribution_payloads", content)
        self.assertNotIn("materialize_distribution_payloads.py --target all --in-place", content)
        self.assertIn('release_ready_args=(--version "$version_input")', content)
        self.assertIn(release_ready, content)
        self.assertLess(content.index(release_ready), content.index(git_add))
        self.assertLess(content.index(release_ready), content.index(tag))

    def test_publish_mode_gates_tag_publish_on_branch_checks(self) -> None:
        content = RELEASE_AUTOMATION.read_text(encoding="utf-8")

        branch_push = 'git push "$push_remote" "$push_branch"'
        branch_gate = 'wait_for_required_workflows "$repo_slug" "$push_branch" "$release_commit" "$ci_timeout_seconds" "$ci_poll_interval_seconds" "${BRANCH_REQUIRED_WORKFLOWS[@]}"'
        tag_create = 'git tag -a "$repo_tag" -m "$tag_message"'
        tag_push = 'git push "$push_remote" "$repo_tag"'
        postflight = './scripts/release_postflight.sh --tag "$repo_tag" --acceptance-out "$acceptance_out"'

        self.assertIn('BRANCH_REQUIRED_WORKFLOWS=("CI" "Checkout Install Check")', content)
        self.assertIn('release_commit="$(git rev-parse HEAD)"', content)
        self.assertIn('repo_slug="$(derive_repo_slug || true)"', content)
        self.assertIn(branch_push, content)
        self.assertIn(branch_gate, content)
        self.assertIn(tag_create, content)
        self.assertIn(tag_push, content)
        self.assertIn(postflight, content)
        self.assertIn('publish mode cannot skip CI status checks before tag creation', content)
        self.assertIn('publish mode cannot disable CI waiting before tag creation', content)
        self.assertIn('publish mode requires hard CI timeout mode before tag creation', content)
        self.assertLess(content.index(branch_push), content.index(branch_gate))
        self.assertLess(content.index(branch_gate), content.index(tag_create))
        self.assertLess(content.index(tag_create), content.index(tag_push))
        self.assertLess(content.index(tag_push), content.index(postflight))

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
        self.assertIn("python3 scripts/generate_stable_release_notes.py \\", content)
        self.assertIn('--repo "${REPO_SLUG:-jxpeng98/qiongli}" \\', content)
        self.assertIn('--output "$TEMP_RELEASE_NOTES"', content)
        self.assertIn('RELEASE_NOTES_LABEL="stable notes: CHANGELOG.md [${version}] + download guide"', content)
        self.assertIn('POSTFLIGHT_STAGING_DIR=""', content)
        self.assertIn('python3 scripts/materialize_distribution_payloads.py --target all --out "$POSTFLIGHT_STAGING_DIR" --force', content)
        self.assertIn('bash ./scripts/verify_release_tag_version.sh --root "$POSTFLIGHT_STAGING_DIR" --tag "$TAG"', content)
        self.assertIn("gh release view", content)
        self.assertIn("--prerelease", content)
        self.assertIn('scripts/build_plugin_artifacts.py --root "$POSTFLIGHT_STAGING_DIR" --tag "$TAG" --dist-dir dist', content)
        self.assertIn('PLUGIN_ARTIFACTS=(', content)
        self.assertIn('if [[ "${TAG#v}" == *-* ]]; then', content)
        self.assertIn('"dist/qiongli-next-codex-plugin-${TAG}.tar.gz"', content)
        self.assertIn('"dist/qiongli-next-claude-plugin-${TAG}.tar.gz"', content)
        self.assertIn('"dist/qiongli-next-claude-plugin-${TAG}.zip"', content)
        self.assertIn('"dist/qiongli-next-claude-desktop-skill-core-${TAG}.zip"', content)
        self.assertIn('"dist/qiongli-claude-plugin-${TAG}.zip"', content)
        self.assertIn('"dist/qiongli-core-codex-plugin-${TAG}.tar.gz"', content)
        self.assertIn('"dist/qiongli-economics-codex-plugin-${TAG}.tar.gz"', content)
        self.assertIn('"dist/qiongli-accounting-codex-plugin-${TAG}.tar.gz"', content)
        self.assertIn('"dist/qiongli-business-codex-plugin-${TAG}.tar.gz"', content)
        self.assertIn('"dist/qiongli-finance-codex-plugin-${TAG}.tar.gz"', content)
        self.assertIn('"dist/qiongli-political-economy-codex-plugin-${TAG}.tar.gz"', content)
        self.assertIn('"dist/qiongli-geoeconomics-codex-plugin-${TAG}.tar.gz"', content)
        self.assertIn('"dist/qiongli-economics-accounting-codex-plugin-${TAG}.tar.gz"', content)
        self.assertIn('"dist/qiongli-core-claude-plugin-${TAG}.tar.gz"', content)
        self.assertIn('"dist/qiongli-core-claude-plugin-${TAG}.zip"', content)
        self.assertIn('"dist/qiongli-economics-claude-plugin-${TAG}.tar.gz"', content)
        self.assertIn('"dist/qiongli-economics-claude-plugin-${TAG}.zip"', content)
        self.assertIn('"dist/qiongli-accounting-claude-plugin-${TAG}.tar.gz"', content)
        self.assertIn('"dist/qiongli-accounting-claude-plugin-${TAG}.zip"', content)
        self.assertIn('"dist/qiongli-business-claude-plugin-${TAG}.tar.gz"', content)
        self.assertIn('"dist/qiongli-business-claude-plugin-${TAG}.zip"', content)
        self.assertIn('"dist/qiongli-finance-claude-plugin-${TAG}.tar.gz"', content)
        self.assertIn('"dist/qiongli-finance-claude-plugin-${TAG}.zip"', content)
        self.assertIn('"dist/qiongli-political-economy-claude-plugin-${TAG}.tar.gz"', content)
        self.assertIn('"dist/qiongli-political-economy-claude-plugin-${TAG}.zip"', content)
        self.assertIn('"dist/qiongli-geoeconomics-claude-plugin-${TAG}.tar.gz"', content)
        self.assertIn('"dist/qiongli-geoeconomics-claude-plugin-${TAG}.zip"', content)
        self.assertIn('"dist/qiongli-economics-accounting-claude-plugin-${TAG}.tar.gz"', content)
        self.assertIn('"dist/qiongli-economics-accounting-claude-plugin-${TAG}.zip"', content)
        self.assertIn('"dist/qiongli-claude-desktop-skill-core-${TAG}.zip"', content)
        self.assertIn('"dist/qiongli-claude-desktop-skill-economics-${TAG}.zip"', content)
        self.assertIn('"dist/qiongli-claude-desktop-skill-business-${TAG}.zip"', content)
        self.assertIn('"dist/qiongli-claude-desktop-skill-finance-${TAG}.zip"', content)
        self.assertIn('"dist/qiongli-claude-desktop-skill-political-economy-${TAG}.zip"', content)
        self.assertIn('"dist/qiongli-claude-desktop-skill-geoeconomics-${TAG}.zip"', content)
        self.assertIn('"dist/qiongli-claude-desktop-skill-${TAG}.zip"', content)
        self.assertIn('MCPB_ARTIFACT="$(python3 scripts/build_literature_mcpb.py --dist-dir dist | tail -n 1)"', content)
        self.assertIn('"$MCPB_ARTIFACT"', content)
        self.assertIn("python3 scripts/generate_release_downloads.py --tag \"$TAG\" --out-dir dist", content)
        self.assertIn('"dist/qiongli-downloads-${TAG}.md"', content)
        self.assertIn('"dist/qiongli-downloads-${TAG}.json"', content)
        self.assertIn('gh release upload "$TAG" --repo "$REPO_SLUG" --clobber "${PLUGIN_ARTIFACTS[@]}"', content)
        self.assertIn('release_args+=("${PLUGIN_ARTIFACTS[@]}")', content)

    def test_release_postflight_uploads_zotero_companion(self) -> None:
        content = RELEASE_POSTFLIGHT.read_text(encoding="utf-8")

        self.assertIn('ZOTERO_COMPANION_ARTIFACT="$(python3 scripts/build_zotero_companion.py --dist-dir dist | tail -n 1)"', content)
        self.assertIn('"$ZOTERO_COMPANION_ARTIFACT"', content)
        self.assertLess(
            content.index('ZOTERO_COMPANION_ARTIFACT="$(python3 scripts/build_zotero_companion.py --dist-dir dist | tail -n 1)"'),
            content.index("python3 scripts/generate_release_downloads.py --tag \"$TAG\" --out-dir dist"),
        )

    def test_release_postflight_publishes_codex_dist_refs(self) -> None:
        content = RELEASE_POSTFLIGHT.read_text(encoding="utf-8")

        self.assertIn("publish_codex_dist_ref()", content)
        self.assertIn('codex_slug="qiongli"', content)
        self.assertIn('codex_slug="qiongli-next"', content)
        self.assertIn('node scripts/publish-codex-dist-ref.mjs \\', content)
        self.assertIn('--version "${TAG#v}" \\', content)
        self.assertIn('--slug "$codex_slug" \\', content)
        self.assertIn('--source "$POSTFLIGHT_STAGING_DIR/plugins/$codex_slug"', content)
        self.assertIn('publish_codex_dist_ref "$TAG"', content)
        self.assertLess(
            content.index('python3 scripts/build_plugin_artifacts.py --root "$POSTFLIGHT_STAGING_DIR" --tag "$TAG" --dist-dir dist'),
            content.index('publish_codex_dist_ref "$TAG"'),
        )
        self.assertLess(
            content.index('publish_codex_dist_ref "$TAG"'),
            content.index('gh release upload "$TAG" --repo "$REPO_SLUG" --clobber "${PLUGIN_ARTIFACTS[@]}"'),
        )

    def test_checkout_install_check_runs_all_platforms_on_push_and_pr(self) -> None:
        main_content = INSTALL_CHECK_WORKFLOW.read_text(encoding="utf-8")

        self.assertIn("name: Checkout Install Check", main_content)
        self.assertIn("push:", main_content)
        self.assertIn("pull_request:", main_content)
        self.assertIn("workflow_dispatch:", main_content)
        self.assertIn("os: [ubuntu-latest, macos-latest]", main_content)
        self.assertIn("runs-on: windows-latest", main_content)

        self.assertFalse(
            MACOS_INSTALL_CHECK_WORKFLOW.exists(),
            msg="macOS checkout checks should stay in the main workflow to avoid duplicate checks.",
        )

    def test_failed_ci_and_checkout_runs_are_rerun_once(self) -> None:
        content = AUTO_RERUN_WORKFLOW.read_text(encoding="utf-8")

        self.assertIn("name: Auto Rerun Failed Actions", content)
        self.assertIn("workflow_run:", content)
        self.assertIn("workflows:", content)
        self.assertIn("- CI", content)
        self.assertIn("- Checkout Install Check", content)
        self.assertNotIn("- Release Automation", content)
        self.assertNotIn("- Publish to PyPI", content)
        self.assertIn("actions: write", content)
        self.assertIn("github.event.workflow_run.conclusion == 'failure'", content)
        self.assertIn("github.event.workflow_run.run_attempt < 2", content)
        self.assertIn('gh run rerun "$RUN_ID" --repo "$REPO" --failed', content)

    def test_release_postflight_supports_soft_ci_timeout_and_gh_api_fallback(self) -> None:
        content = RELEASE_POSTFLIGHT.read_text(encoding="utf-8")

        self.assertIn('CI_TIMEOUT_MODE="hard"', content)
        self.assertIn("--ci-timeout-mode <hard|soft>", content)
        self.assertIn('fetch_actions_runs()', content)
        self.assertIn('gh api "repos/${repo_slug}/actions/runs?branch=${ref_name}&per_page=20"', content)
        self.assertIn('curl -fsSL -H "Authorization: Bearer ${GH_TOKEN}" "$api_url"', content)
        self.assertIn('[[ "$CI_TIMEOUT_MODE" == "hard" || "$CI_TIMEOUT_MODE" == "soft" ]]', content)
        self.assertIn('CI_STATUS="pending:timeout-after-${CI_TIMEOUT_SECONDS}s"', content)
        self.assertIn('CI_STATUS="skipped:query-unavailable"', content)
        self.assertIn('if [[ "$CI_TIMEOUT_MODE" == "soft" ]]; then', content)

    def test_publish_mode_passes_ci_timeout_mode_to_postflight(self) -> None:
        content = RELEASE_AUTOMATION.read_text(encoding="utf-8")

        self.assertIn('ci_timeout_mode="hard"', content)
        self.assertIn("--ci-timeout-mode", content)
        self.assertIn('post_args+=(--wait-ci --ci-timeout-seconds "$ci_timeout_seconds" --ci-timeout-mode hard --ci-poll-interval-seconds "$ci_poll_interval_seconds")', content)

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

        self.assertIn('content/distribution/plugins.yaml', content)
        self.assertIn('tooling/scripts/build_plugin_artifacts.py', content)
        self.assertIn('tooling/scripts/materialize_distribution_payloads.py', content)
        self.assertNotIn('packages/qiongli-plugin/.codex-plugin/plugin.json', content)
        self.assertNotIn('packages/qiongli-next-plugin|packages/qiongli-next-plugin/*', content)
        self.assertIn('content/workflow/VERSION', content)
        self.assertIn('content/skills/registry.yaml', content)
        self.assertIn('packages/npm-qiongli/package.json', content)
        self.assertIn('package-lock.json', content)
        self.assertIn('uv.lock', content)
        self.assertIn('docs/reference/skills.md', content)
        self.assertIn('docs/zh/reference/skills.md', content)
        self.assertNotIn('qiongli-workflow/VERSION', content)
        self.assertNotIn('qiongli-workflow/VERSION|skills/registry.yaml', content)
        self.assertNotIn('skills/*)', content)
        self.assertNotIn('packages/python-qiongli/src/qiongli/payload|packages/python-qiongli/src/qiongli/payload/*', content)
        self.assertNotIn('plugins/qiongli/skills/qiongli-workflow|plugins/qiongli/skills/qiongli-workflow/*', content)
        self.assertNotIn('qiongli-workflow/skills/registry.yaml', content)

    def test_release_ready_runs_package_preflights_from_staging_root(self) -> None:
        content = RELEASE_READY.read_text(encoding="utf-8")

        self.assertNotIn("python3 scripts/materialize_distribution_payloads.py --target next-plugin --in-place", content)
        preflight = './scripts/release_automation.sh pre "${PRE_ARGS[@]}" --materialize-out "$RELEASE_STAGING_DIR"'
        verify = 'bash ./scripts/verify_release_tag_version.sh --root "$RELEASE_STAGING_DIR" --tag "$REPO_TAG"'
        pypi = 'bash ./scripts/pypi_preflight.sh --root "$RELEASE_STAGING_DIR" "${PYPI_ARGS[@]}"'
        npm = 'bash ./scripts/npm_preflight.sh --root "$RELEASE_STAGING_DIR"'

        self.assertIn('RELEASE_STAGING_DIR=""', content)
        self.assertIn('--staging-dir <dir>', content)
        self.assertIn('mktemp -d "${TMPDIR:-/tmp}/qiongli-release-ready.XXXXXX"', content)
        self.assertIn(preflight, content)
        self.assertIn(verify, content)
        self.assertIn(pypi, content)
        self.assertIn(npm, content)
        self.assertLess(content.index(preflight), content.index(verify))
        self.assertLess(content.index(verify), content.index(pypi))
        self.assertLess(content.index(pypi), content.index(npm))

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

    def test_release_preflight_supports_quick_ci_gate(self) -> None:
        content = RELEASE_PREFLIGHT.read_text(encoding="utf-8")

        self.assertIn("QUICK_MODE=0", content)
        self.assertIn("RUN_UNIT_TESTS=1", content)
        self.assertIn("RUN_CONTROLLER_EVALS=1", content)
        self.assertIn("--quick", content)
        self.assertIn("--skip-unit-tests", content)
        self.assertIn("--skip-controller-evals", content)
        self.assertIn('echo "[preflight] unit tests skipped"', content)
        self.assertIn('echo "[preflight] controller-mode evals skipped"', content)
        self.assertIn('unittest_summary="skipped"', content)

    def test_release_preflight_syncs_npm_payload_before_tests(self) -> None:
        content = RELEASE_PREFLIGHT.read_text(encoding="utf-8")

        materialize = 'python3 scripts/materialize_distribution_payloads.py --target all --out "$MATERIALIZE_OUT" --force'
        self.assertIn('echo "[preflight] materialize distribution payloads"', content)
        self.assertIn(materialize, content)
        self.assertIn('MATERIALIZE_OUT="$(mktemp -d "${TMPDIR:-/tmp}/qiongli-release-preflight.XXXXXX")"', content)
        self.assertIn('if [[ "$MATERIALIZE_IN_PLACE" -eq 1 ]]; then', content)
        self.assertIn('echo "[preflight] sync skill reference docs"', content)
        self.assertIn("python3 scripts/generate_skill_docs.py", content)
        self.assertLess(
            content.index(materialize),
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

    def test_release_preflight_supports_staged_materialization_for_ci(self) -> None:
        content = RELEASE_PREFLIGHT.read_text(encoding="utf-8")

        staged_materialize = 'python3 scripts/materialize_distribution_payloads.py --target all --out "$MATERIALIZE_OUT" --force'
        staged_validate = 'validate_cmd=(python3 scripts/validate_research_standard.py --root "$PREFLIGHT_ROOT")'

        self.assertIn("--materialize-out <dir>", content)
        self.assertIn("--in-place", content)
        self.assertIn("MATERIALIZE_OUT=\"\"", content)
        self.assertIn("MATERIALIZE_IN_PLACE=0", content)
        self.assertIn("PREFLIGHT_ROOT=\"$ROOT_DIR\"", content)
        self.assertIn(staged_materialize, content)
        self.assertIn('PREFLIGHT_ROOT="$MATERIALIZE_OUT"', content)
        self.assertIn('if [[ "$MATERIALIZE_IN_PLACE" -eq 1 ]]; then', content)
        self.assertIn('[preflight] in-place materialization requires explicit --in-place', content)
        self.assertIn(staged_validate, content)
        self.assertLess(content.index(staged_materialize), content.index(staged_validate))

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

    def test_pypi_preflight_materializes_payloads_before_build(self) -> None:
        content = PYPI_PREFLIGHT.read_text(encoding="utf-8")

        materialize = 'python3 scripts/materialize_distribution_payloads.py --target all --out "$PREFLIGHT_ROOT" --force'
        build = "python3 -m build"

        self.assertIn("--root <dir>", content)
        self.assertIn('ROOT_DIR="$(cd "$2" && pwd)"', content)
        self.assertIn("--in-place", content)
        self.assertIn("PREFLIGHT_ROOT=\"\"", content)
        self.assertIn('PREFLIGHT_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/qiongli-pypi-preflight-root.XXXXXX")"', content)
        self.assertIn('if [[ "$PREFLIGHT_IN_PLACE" -eq 1 ]]; then', content)
        self.assertIn(materialize, content)
        self.assertIn(build, content)
        self.assertIn('cd "$PREFLIGHT_ROOT"', content)
        self.assertLess(content.index(materialize), content.index(build))

    def test_npm_preflight_accepts_staging_root(self) -> None:
        content = (LAYOUT.scripts / "npm_preflight.sh").read_text(encoding="utf-8")

        self.assertIn("--root <dir>", content)
        self.assertIn('ROOT_DIR="$(cd "$2" && pwd)"', content)
        self.assertIn("--in-place", content)
        self.assertIn("PREFLIGHT_ROOT=\"\"", content)
        self.assertIn('PREFLIGHT_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/qiongli-npm-preflight-root.XXXXXX")"', content)
        self.assertIn('if [[ "$PREFLIGHT_IN_PLACE" -eq 1 ]]; then', content)
        self.assertIn('PKG_DIR="$ROOT_DIR/packages/npm-qiongli"', content)
        self.assertIn('PKG_DIR="$PREFLIGHT_ROOT/packages/npm-qiongli"', content)
        self.assertIn('python3 scripts/materialize_distribution_payloads.py --target all --out "$PREFLIGHT_ROOT" --force', content)
        self.assertIn('cd "$PREFLIGHT_ROOT"', content)

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
        content = (LAYOUT.scripts / "generate_release_notes.sh").read_text(encoding="utf-8")

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
        self.assertIn('bash scripts/verify_release_tag_version.sh --root "$RUNNER_TEMP/qiongli-release-dist" --tag "$tag"', content)
        self.assertIn('git config user.name "github-actions[bot]"', content)
        self.assertIn("python -m pip install -e . build twine", content)
        self.assertIn("./scripts/release_automation.sh \"$mode\" \"${args[@]}\"", content)

    def test_publish_pypi_workflow_verifies_tag_matches_repo_version(self) -> None:
        content = PUBLISH_PYPI_WORKFLOW.read_text(encoding="utf-8")

        self.assertIn('bash scripts/verify_release_tag_version.sh --root "$RUNNER_TEMP/qiongli-dist" --tag "${RELEASE_TAG}"', content)
        self.assertIn('packages-dir: ${{ runner.temp }}/qiongli-dist/dist', content)
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
                self.assertIn('bash scripts/verify_release_tag_version.sh --root "$RUNNER_TEMP/qiongli-dist" --tag "${RELEASE_TAG}"', content)

    def test_tag_publish_workflows_materialize_staging_before_version_verify(self) -> None:
        for workflow in (PUBLISH_PYPI_WORKFLOW, PUBLISH_NPM_WORKFLOW):
            with self.subTest(workflow=workflow.name):
                content = workflow.read_text(encoding="utf-8")

                verify = 'bash scripts/verify_release_tag_version.sh --root "$RUNNER_TEMP/qiongli-dist" --tag "${RELEASE_TAG}"'
                install = "python -m pip install -e ."
                materialize = 'python3 scripts/materialize_distribution_payloads.py --target all --out "$RUNNER_TEMP/qiongli-dist" --force'
                self.assertIn(materialize, content)
                self.assertIn(install, content)
                self.assertNotIn("python3 scripts/materialize_distribution_payloads.py --target all --in-place", content)
                self.assertLess(content.index(install), content.index(materialize))
                self.assertLess(content.index(materialize), content.index(verify))

    def test_release_workflow_materializes_staging_before_version_verify(self) -> None:
        content = RELEASE_WORKFLOW.read_text(encoding="utf-8")

        verify = 'bash scripts/verify_release_tag_version.sh --root "$RUNNER_TEMP/qiongli-release-dist" --tag "$tag"'
        install = "python -m pip install -e . build twine"
        materialize = 'python3 scripts/materialize_distribution_payloads.py --target all --out "$RUNNER_TEMP/qiongli-release-dist" --force'
        self.assertIn(materialize, content)
        self.assertIn(install, content)
        self.assertNotIn("python3 scripts/materialize_distribution_payloads.py --target all --in-place", content)
        self.assertLess(content.index(install), content.index(materialize))
        self.assertLess(content.index(materialize), content.index(verify))

    def test_verify_release_tag_script_checks_expected_files(self) -> None:
        content = VERIFY_RELEASE_TAG.read_text(encoding="utf-8")

        self.assertIn("--root <dir>", content)
        self.assertIn('ROOT_DIR="$(cd "$2" && pwd)"', content)
        self.assertIn('cd "$ROOT_DIR"', content)
        self.assertIn('scripts/sync_versions.py "$TAG" --print-field package_version', content)
        self.assertIn('scripts/sync_versions.py "$TAG" --print-field npm_version', content)
        self.assertIn('pyproject.toml', content)
        self.assertIn('packages/python-qiongli/src/qiongli/__init__.py', content)
        self.assertIn('content/skills/registry.yaml', content)
        self.assertIn('content/workflow/VERSION', content)
        self.assertNotIn('Path("skills/registry.yaml")', content)
        self.assertNotIn('< qiongli-workflow/VERSION', content)
        self.assertNotIn('actual_workflow_registry_version', content)
        self.assertNotIn('Path("qiongli-workflow/skills/registry.yaml")', content)
        self.assertNotIn('echo "[verify-release-tag] qiongli-workflow/skills/registry.yaml mismatch', content)
        self.assertIn('packages/python-qiongli/src/qiongli/payload/qiongli-workflow/VERSION', content)
        self.assertIn('packages/python-qiongli/src/qiongli/payload/qiongli-workflow/skills/registry.yaml', content)
        self.assertIn('packages/python-qiongli/src/qiongli/payload/skills/registry.yaml', content)
        self.assertIn('packages/npm-qiongli/package.json', content)
        self.assertIn('package-lock.json', content)
        self.assertIn('packages/npm-qiongli/payload/qiongli-workflow/VERSION', content)
        self.assertIn('packages/npm-qiongli/payload/qiongli-workflow/skills/registry.yaml', content)
        self.assertIn('packages/npm-qiongli/python-runtime/qiongli/__init__.py', content)
        self.assertIn('packages/npm-qiongli/python-runtime/skills/registry.yaml', content)
        self.assertIn('plugins/qiongli/.codex-plugin/plugin.json', content)
        self.assertIn('plugins/qiongli/skills/qiongli-workflow/VERSION', content)
        self.assertIn('plugins/qiongli/skills/qiongli-workflow/skills/registry.yaml', content)
        self.assertIn('plugins/qiongli-next/.codex-plugin/plugin.json', content)
        self.assertIn('plugins/qiongli-next/skills/qiongli-workflow/VERSION', content)
        self.assertIn('plugins/qiongli-next/skills/qiongli-workflow/skills/registry.yaml', content)
        self.assertIn('plugins/qiongli/.claude-plugin/plugin.json', content)
        self.assertNotIn('plugins/qiongli/gemini-extension.json', content)
        self.assertIn('python3 scripts/audit_distribution_payloads.py --root "$ROOT_DIR"', content)


if __name__ == "__main__":
    unittest.main()
