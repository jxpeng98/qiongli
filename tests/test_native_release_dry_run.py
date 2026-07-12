from __future__ import annotations

import copy
import io
import json
import os
from contextlib import redirect_stdout
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

from tooling.scripts import native_release_dry_run as dry_run
from tooling.scripts.validate_capability_contract import validate_instance


REPO_ROOT = Path(__file__).resolve().parents[1]
SCHEMA_PATH = REPO_ROOT / "tooling/release/native-release-plan.schema.json"
SOURCE_COMMIT = "a" * 40


def _write_native_fixture(
    root: Path,
    *,
    version: str = "2.0.0-alpha.1",
    channel: str = "alpha",
    lock_version: str | None = None,
) -> None:
    native = root / "packages/qiongli-native"
    native.mkdir(parents=True)
    (native / "Cargo.toml").write_text(
        "[workspace]\n"
        'resolver = "3"\n'
        'members = ["apps/qiongli"]\n'
        "\n"
        "[workspace.package]\n"
        f'version = "{version}"\n'
        'edition = "2024"\n'
        "\n"
        "[workspace.metadata.qiongli]\n"
        'product = "qiongli"\n'
        f'channel = "{channel}"\n',
        encoding="utf-8",
    )
    (native / "Cargo.lock").write_text(
        "version = 4\n"
        "\n"
        "[[package]]\n"
        'name = "qiongli"\n'
        f'version = "{lock_version or version}"\n',
        encoding="utf-8",
    )


class NativeReleaseDryRunTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.schema = json.loads(SCHEMA_PATH.read_text(encoding="utf-8"))

    def _plan(self) -> dict[str, object]:
        return dry_run.build_plan(
            REPO_ROOT,
            "v2.0.0-alpha.1",
            target_os="linux",
            target_arch="amd64",
            source_ref="2.x",
            source_ref_type="branch",
            worktree_state="clean",
            source_commit=SOURCE_COMMIT,
        )

    def _run_main(self, arguments: list[str]) -> tuple[int, str]:
        stdout = io.StringIO()
        with redirect_stdout(stdout):
            exit_code = dry_run.main(arguments)
        return exit_code, stdout.getvalue()

    def test_plan_matches_closed_schema_and_native_source(self) -> None:
        plan = self._plan()

        self.assertEqual(validate_instance(plan, self.schema), [])
        self.assertEqual(plan["record_type"], "qiongli-native-release-dry-run-plan")
        self.assertEqual(plan["task_id"], "REL-201")
        self.assertEqual(plan["status"], "planned-only")
        self.assertEqual(
            plan["identity"],
            {
                "product": "qiongli",
                "version": "2.0.0-alpha.1",
                "repo_tag": "v2.0.0-alpha.1",
                "channel": "alpha",
                "release_line": "native-2x",
            },
        )
        self.assertEqual(plan["source"]["required_branch"], "2.x")
        self.assertEqual(plan["source"]["required_ref_type"], "branch")
        self.assertEqual(plan["source"]["observed_ref"], "2.x")
        self.assertEqual(plan["source"]["observed_ref_type"], "branch")
        self.assertEqual(plan["source"]["worktree_state"], "clean")
        self.assertEqual(plan["source"]["source_commit"], SOURCE_COMMIT)
        self.assertTrue(plan["source"]["release_source_eligible"])
        self.assertEqual(
            plan["source"]["version_source"],
            "packages/qiongli-native/Cargo.toml#workspace.package.version",
        )
        self.assertEqual(
            plan["integrity"]["payload_sha256"],
            dry_run.canonical_payload_sha256(plan),
        )

    def test_every_object_schema_is_closed_and_requires_all_properties(self) -> None:
        def visit(value: object, path: str) -> None:
            if isinstance(value, dict):
                if value.get("type") == "object":
                    self.assertFalse(value.get("additionalProperties", True), path)
                    properties = value.get("properties")
                    self.assertIsInstance(properties, dict, path)
                    self.assertEqual(set(value.get("required", [])), set(properties), path)
                for key, item in value.items():
                    visit(item, f"{path}.{key}")
            elif isinstance(value, list):
                for index, item in enumerate(value):
                    visit(item, f"{path}[{index}]")

        visit(self.schema, "$")

    def test_target_identity_is_concrete_normalized_and_planned_only(self) -> None:
        artifact = self._plan()["planned_artifacts"][0]

        self.assertEqual(artifact["status"], "planned-only")
        self.assertEqual(
            artifact["identity"],
            {
                "product": "qiongli",
                "version": "2.0.0-alpha.1",
                "channel": "alpha",
                "profile": "bootstrap",
                "os": "linux",
                "arch": "x86_64",
                "installer_kind": "portable-archive",
            },
        )
        self.assertEqual(artifact["target_source"], "explicit")
        self.assertFalse(artifact["artifact_created"])
        self.assertFalse(artifact["target_native_startup_verified"])
        self.assertFalse(artifact["signed"])

    def test_channel_and_legacy_registry_isolation_are_explicit(self) -> None:
        plan = self._plan()
        isolation = plan["channel_isolation"]
        publication = plan["publication"]

        self.assertEqual(isolation["canonical_channels"], ["alpha", "beta", "stable"])
        self.assertEqual(isolation["selected_channel"], "alpha")
        self.assertFalse(isolation["mutable_alias_is_canonical"])
        self.assertFalse(isolation["cross_channel_fallback"])
        self.assertFalse(isolation["legacy_1x_feed_included"])
        self.assertEqual(isolation["pypi_publication"], "not-applicable")
        self.assertEqual(isolation["npm_publication"], "not-applicable")
        self.assertEqual(publication["mode"], "dry-run")
        self.assertFalse(publication["publication_performed"])
        self.assertFalse(publication["publication_allowed"])
        self.assertEqual(publication["publication_network_access"], "forbidden")
        self.assertEqual(publication["git_mutation"], "forbidden")
        self.assertEqual(len(publication["future_blockers"]), 5)

    def test_schema_rejects_cross_field_channel_mismatch(self) -> None:
        plan = copy.deepcopy(self._plan())
        plan["identity"]["channel"] = "stable"
        plan["planned_artifacts"][0]["identity"]["channel"] = "stable"
        failures = validate_instance(plan, self.schema)

        self.assertTrue(failures)
        self.assertTrue(any("oneOf" in failure for failure in failures))

    def test_bundle_is_byte_deterministic_and_does_not_mutate_source(self) -> None:
        manifest = REPO_ROOT / dry_run.NATIVE_MANIFEST_RELATIVE
        lock = REPO_ROOT / dry_run.NATIVE_LOCK_RELATIVE
        before = {manifest: manifest.read_bytes(), lock: lock.read_bytes()}
        plan = self._plan()

        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory)
            first_paths = dry_run.write_bundle(REPO_ROOT, base / "first", plan)
            second_paths = dry_run.write_bundle(REPO_ROOT, base / "second", plan)
            self.assertEqual(
                [path.name for path in first_paths],
                [path.name for path in second_paths],
            )
            self.assertEqual(
                [path.read_bytes() for path in first_paths],
                [path.read_bytes() for path in second_paths],
            )
            self.assertEqual(len(list((base / "first").iterdir())), 3)

        self.assertEqual({path: path.read_bytes() for path in before}, before)

    def test_notes_and_rollback_do_not_overclaim_reality(self) -> None:
        plan = self._plan()
        notes = dry_run._notes_text(plan)
        rollback = dry_run._rollback_text(plan)

        for text in (notes, rollback):
            self.assertIn("no", text.lower())
            self.assertNotIn("publication succeeded", text.lower())
            self.assertNotIn("artifact built", text.lower())
        self.assertIn("planned only; publication is not allowed", notes)
        self.assertIn("Release Notes", notes)
        self.assertIn("Stage: Alpha", notes)
        self.assertIn("Artifact produced: `false`", notes)
        self.assertIn("PyPI publication: `not-applicable`", notes)
        self.assertIn("npm publication: `not-applicable`", notes)
        self.assertIn("Validation Evidence", notes)
        self.assertIn("Publish Steps", notes)
        self.assertIn("rollback.md", notes)
        self.assertIn("No release was published", rollback)
        self.assertIn("no Git ref was created or moved", rollback)
        self.assertIn("three generated `qiongli-native-release-*` bundle files", rollback)
        self.assertIn("preserve the containing directory", rollback.lower())
        self.assertIn("Promotion never relabels", rollback)

    def test_source_version_channel_and_lock_mismatches_fail_closed(self) -> None:
        cases = (
            {"version": "2.0.0-alpha.2", "channel": "alpha", "lock_version": None},
            {"version": "2.0.0-alpha.1", "channel": "beta", "lock_version": None},
            {
                "version": "2.0.0-alpha.1",
                "channel": "alpha",
                "lock_version": "2.0.0-alpha.2",
            },
        )
        for index, case in enumerate(cases):
            with self.subTest(case=index), tempfile.TemporaryDirectory() as directory:
                root = Path(directory) / "repo"
                _write_native_fixture(root, **case)
                with self.assertRaises(dry_run.SourceMismatch):
                    dry_run.build_plan(
                        root,
                        "v2.0.0-alpha.1",
                        target_os="linux",
                        target_arch="x86_64",
                    )

    def test_only_native_2x_canonical_tags_are_allowed(self) -> None:
        for tag in (
            "v1.19.0-beta.1",
            "1.19.0-beta.1",
            "2.0.0-alpha.1",
            "v2.0.0-rc.1",
            "v2.0.0-alpha",
            "v2.0.0-alpha.0",
            "v2.0.0-alpha.1/escape",
        ):
            with self.subTest(tag=tag), self.assertRaises(dry_run.DryRunError):
                dry_run.build_plan(
                    REPO_ROOT,
                    tag,
                    target_os="linux",
                    target_arch="x86_64",
                )

    def test_target_aliases_normalize_and_ambiguous_values_are_rejected(self) -> None:
        self.assertEqual(dry_run.normalise_os("Darwin"), "macos")
        self.assertEqual(dry_run.normalise_os("win32"), "windows")
        self.assertEqual(dry_run.normalise_arch("arm64"), "aarch64")
        self.assertEqual(dry_run.normalise_arch("AMD64"), "x86_64")

        for value in ("", "any", "current-host", "unknown", "../linux", "linux/amd64"):
            with self.subTest(os=value), self.assertRaises(dry_run.DryRunError):
                dry_run.normalise_os(value)
        for value in ("", "any", "current-host", "unknown", "../x86_64", "x86_64/escape"):
            with self.subTest(arch=value), self.assertRaises(dry_run.DryRunError):
                dry_run.normalise_arch(value)
        for raw_os, raw_arch in (("linux", None), (None, "x86_64")):
            with self.subTest(raw_os=raw_os, raw_arch=raw_arch), self.assertRaises(
                dry_run.DryRunError
            ):
                dry_run.resolve_target(raw_os, raw_arch)

    def test_source_commit_rejects_paths_uppercase_and_non_object_ids(self) -> None:
        for value in ("main", "A" * 40, "../HEAD", "a" * 39, "a" * 41, "g" * 40):
            with self.subTest(value=value), self.assertRaises(dry_run.DryRunError):
                dry_run.build_plan(
                    REPO_ROOT,
                    "v2.0.0-alpha.1",
                    target_os="linux",
                    target_arch="x86_64",
                    source_ref="2.x",
                    source_ref_type="branch",
                    worktree_state="clean",
                    source_commit=value,
                )

    def test_source_binding_is_truthful_for_feature_dirty_and_unassessed_runs(self) -> None:
        dirty = dry_run.build_plan(
            REPO_ROOT,
            "v2.0.0-alpha.1",
            target_os="linux",
            target_arch="x86_64",
            source_ref="feat/rel-201-native-alpha-release",
            source_ref_type="branch",
            worktree_state="dirty",
        )
        self.assertEqual(dirty["source"]["required_branch"], "2.x")
        self.assertEqual(
            dirty["source"]["observed_ref"], "feat/rel-201-native-alpha-release"
        )
        self.assertIsNone(dirty["source"]["source_commit"])
        self.assertFalse(dirty["source"]["release_source_eligible"])

        unassessed = dry_run.build_plan(
            REPO_ROOT,
            "v2.0.0-alpha.1",
            target_os="linux",
            target_arch="x86_64",
        )
        self.assertEqual(unassessed["source"]["worktree_state"], "unknown")
        self.assertIsNone(unassessed["source"]["observed_ref"])
        self.assertEqual(unassessed["source"]["observed_ref_type"], "unknown")
        self.assertIsNone(unassessed["source"]["source_commit"])
        self.assertFalse(unassessed["source"]["release_source_eligible"])

        same_name_tag = dry_run.build_plan(
            REPO_ROOT,
            "v2.0.0-alpha.1",
            target_os="linux",
            target_arch="x86_64",
            source_ref="2.x",
            source_ref_type="tag",
            worktree_state="clean",
            source_commit=SOURCE_COMMIT,
        )
        self.assertFalse(same_name_tag["source"]["release_source_eligible"])

    def test_dirty_or_unassessed_source_cannot_bind_a_commit(self) -> None:
        for source_ref, source_ref_type, state in (
            ("2.x", "branch", "dirty"),
            (None, "unknown", "unknown"),
        ):
            with self.subTest(state=state), self.assertRaises(dry_run.DryRunError):
                dry_run.build_plan(
                    REPO_ROOT,
                    "v2.0.0-alpha.1",
                    target_os="linux",
                    target_arch="x86_64",
                    source_ref=source_ref,
                    source_ref_type=source_ref_type,
                    worktree_state=state,
                    source_commit=SOURCE_COMMIT,
                )

    def test_source_ref_and_type_must_be_assessed_together(self) -> None:
        for source_ref, source_ref_type in ((None, "tag"), ("2.x", "unknown")):
            with self.subTest(source_ref=source_ref, source_ref_type=source_ref_type):
                with self.assertRaises(dry_run.DryRunError):
                    dry_run.build_plan(
                        REPO_ROOT,
                        "v2.0.0-alpha.1",
                        target_os="linux",
                        target_arch="x86_64",
                        source_ref=source_ref,
                        source_ref_type=source_ref_type,
                    )

    def test_output_must_be_external_and_cannot_follow_symlink(self) -> None:
        plan = self._plan()
        internal = REPO_ROOT / "build/native-release-dry-run-test-must-not-exist"
        self.assertFalse(internal.exists())
        with self.assertRaises(dry_run.DryRunError):
            dry_run.write_bundle(REPO_ROOT, internal, plan)
        self.assertFalse(internal.exists())

        if hasattr(os, "symlink"):
            with tempfile.TemporaryDirectory() as directory:
                base = Path(directory)
                target = base / "target"
                target.mkdir()
                link = base / "link"
                try:
                    link.symlink_to(target, target_is_directory=True)
                except OSError:
                    return
                with self.assertRaises(dry_run.DryRunError):
                    dry_run.write_bundle(REPO_ROOT, link, plan)

                dangling = base / "dangling"
                dangling.symlink_to(base / "missing", target_is_directory=True)
                with self.assertRaises(dry_run.DryRunError):
                    dry_run.write_bundle(REPO_ROOT, dangling, plan)

    def test_output_directory_must_be_empty(self) -> None:
        plan = self._plan()
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "bundle"
            output.mkdir()
            unrelated = output / "unrelated.txt"
            unrelated.write_text("preserve\n", encoding="utf-8")

            with self.assertRaises(dry_run.DryRunError):
                dry_run.write_bundle(REPO_ROOT, output, plan)

            self.assertEqual(unrelated.read_text(encoding="utf-8"), "preserve\n")
            self.assertEqual(list(output.iterdir()), [unrelated])

    def test_bundle_commit_failure_leaves_no_partial_files(self) -> None:
        plan = self._plan()
        original_replace = Path.replace
        calls = 0

        def fail_second_replace(path: Path, target: Path) -> Path:
            nonlocal calls
            calls += 1
            if calls == 2:
                raise OSError("simulated atomic commit failure")
            return original_replace(path, target)

        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "bundle"
            with patch.object(Path, "replace", new=fail_second_replace):
                with self.assertRaises(dry_run.DryRunError):
                    dry_run.write_bundle(REPO_ROOT, output, plan)
            self.assertFalse(output.exists())

    def test_unsafe_later_output_is_rejected_before_any_bundle_file_is_written(self) -> None:
        if not hasattr(os, "symlink"):
            self.skipTest("symbolic links are unavailable")
        plan = self._plan()
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "bundle"
            output.mkdir()
            notes = output / "qiongli-native-release-v2.0.0-alpha.1-notes.md"
            external = Path(directory) / "external.md"
            external.write_text("unchanged\n", encoding="utf-8")
            try:
                notes.symlink_to(external)
            except OSError:
                self.skipTest("symbolic links cannot be created")

            with self.assertRaises(dry_run.DryRunError):
                dry_run.write_bundle(REPO_ROOT, output, plan)

            self.assertFalse(
                (output / "qiongli-native-release-v2.0.0-alpha.1.json").exists()
            )
            self.assertEqual(external.read_text(encoding="utf-8"), "unchanged\n")

    def test_untrusted_plan_tag_cannot_escape_the_explicit_output_directory(self) -> None:
        plan = copy.deepcopy(self._plan())
        plan["identity"]["repo_tag"] = "../../escape"
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "bundle"
            with self.assertRaises(dry_run.DryRunError):
                dry_run.write_bundle(REPO_ROOT, output, plan)
            self.assertFalse(output.exists())

    def test_semantic_validator_rejects_cross_field_drift_even_with_new_digest(self) -> None:
        mutations = (
            lambda plan: plan["identity"].__setitem__("version", "2.0.0-alpha.2"),
            lambda plan: plan["channel_isolation"].__setitem__(
                "selected_channel", "beta"
            ),
            lambda plan: plan["planned_artifacts"][0]["identity"].__setitem__(
                "version", "2.0.0-alpha.2"
            ),
            lambda plan: plan["planned_artifacts"][0].__setitem__(
                "artifact_id", "qiongli-tampered"
            ),
            lambda plan: plan["publication"].__setitem__(
                "publication_allowed", True
            ),
        )
        for index, mutate in enumerate(mutations):
            plan = copy.deepcopy(self._plan())
            mutate(plan)
            plan["integrity"]["payload_sha256"] = dry_run.canonical_payload_sha256(plan)
            with self.subTest(index=index), tempfile.TemporaryDirectory() as directory:
                output = Path(directory) / "bundle"
                with self.assertRaises(dry_run.DryRunError):
                    dry_run.write_bundle(REPO_ROOT, output, plan)
                self.assertFalse(output.exists())

    def test_cli_writes_only_three_outputs_and_emits_stable_json(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "bundle"
            exit_code, stdout = self._run_main(
                [
                    "--tag",
                    "v2.0.0-alpha.1",
                    "--root",
                    str(REPO_ROOT),
                    "--out-dir",
                    str(output),
                    "--os",
                    "macos",
                    "--arch",
                    "arm64",
                    "--source-commit",
                    SOURCE_COMMIT,
                    "--source-ref",
                    "2.x",
                    "--source-ref-type",
                    "branch",
                    "--worktree-state",
                    "clean",
                    "--json",
                ]
            )
            payload = json.loads(stdout)

            self.assertEqual(exit_code, 0)
            self.assertEqual(payload["status"], "pass")
            self.assertEqual(payload["code"], "native-release-dry-run-written")
            self.assertEqual(payload["repo_tag"], "v2.0.0-alpha.1")
            self.assertEqual(payload["channel"], "alpha")
            self.assertFalse(payload["publication_performed"])
            self.assertFalse(payload["publication_allowed"])
            self.assertEqual(
                sorted(path.name for path in output.iterdir()),
                [
                    "qiongli-native-release-v2.0.0-alpha.1-notes.md",
                    "qiongli-native-release-v2.0.0-alpha.1-rollback.md",
                    "qiongli-native-release-v2.0.0-alpha.1.json",
                ],
            )

    def test_cli_errors_are_redacted_and_do_not_create_output(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "credential-canary-must-not-echo"
            exit_code, stdout = self._run_main(
                [
                    "--tag",
                    "credential-canary-must-not-echo",
                    "--root",
                    str(REPO_ROOT),
                    "--out-dir",
                    str(output),
                    "--json",
                ]
            )
            payload = json.loads(stdout)

            self.assertEqual(exit_code, 2)
            self.assertEqual(payload["status"], "error")
            self.assertEqual(payload["code"], "native-release-dry-run-unavailable")
            self.assertNotIn("credential-canary", stdout)
            self.assertFalse(output.exists())

    def test_integrity_changes_when_semantic_payload_changes(self) -> None:
        plan = self._plan()
        changed = copy.deepcopy(plan)
        changed["source"]["source_commit"] = "b" * 40

        self.assertNotEqual(
            dry_run.canonical_payload_sha256(plan),
            dry_run.canonical_payload_sha256(changed),
        )


if __name__ == "__main__":
    unittest.main()
