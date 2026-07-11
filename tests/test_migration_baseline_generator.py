from __future__ import annotations

import json
import os
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tooling.scripts import generate_migration_baseline as generator
from tooling.scripts.validate_capability_contract import validate_instance


REPO_ROOT = Path(__file__).resolve().parents[1]
BASELINE_ROOT = REPO_ROOT / "tooling/migration/baselines/v1.19.0-beta.1"
MANIFEST_PATH = BASELINE_ROOT / "manifest.json"
EXPECTED_TAG = "v1.19.0-beta.1"
EXPECTED_COMMIT = "8d2e99866ce4c4efb8b3b5e0265c0c1f89a36b0f"
EXPECTED_TAG_OBJECT = "e68e3af4c879d8e9053124d1aed625bfcddfdbb4"
ACCEPTED_ASSET_DIR = os.environ.get("QIONGLI_ACCEPTED_RELEASE_ASSET_DIR")


def _load(path: Path) -> dict[str, object]:
    return json.loads(path.read_text(encoding="utf-8"))


def _tree_bytes(root: Path) -> dict[str, bytes]:
    return {
        path.relative_to(root).as_posix(): path.read_bytes()
        for path in sorted(root.rglob("*"))
        if path.is_file()
    }


class MigrationBaselineGeneratorTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = _load(MANIFEST_PATH)
        cls.manifest_schema = _load(
            REPO_ROOT / "tooling/migration/baseline-manifest.schema.json"
        )
        cls.oracle_schema = _load(
            REPO_ROOT / "tooling/migration/oracle-fixture.schema.json"
        )

    def _copy_baseline(self, parent: Path) -> Path:
        destination = parent / "baseline"
        shutil.copytree(BASELINE_ROOT, destination)
        return destination

    def _recorded_oracle_documents(self) -> dict[str, dict[str, object]]:
        return {
            f"oracles/{name}": _load(BASELINE_ROOT / "oracles" / name)
            for name in ("python-full.json", "rust-lite.json", "node-mcpb.json")
        }

    def _rewrite_oracle_with_valid_hashes(
        self, baseline: Path, *, leak: str, as_key: bool = False
    ) -> None:
        oracle_path = baseline / "oracles/python-full.json"
        oracle = _load(oracle_path)
        if as_key:
            oracle["cases"][0]["outcome"]["value"][leak] = "leaked-key"
        else:
            oracle["cases"][0]["outcome"]["value"]["portable_value"] = leak
        oracle_bytes = generator._canonical_bytes(oracle)
        oracle_path.write_bytes(oracle_bytes)

        manifest_path = baseline / "manifest.json"
        manifest = _load(manifest_path)
        descriptor = next(
            item
            for item in manifest["oracle_fixtures"]
            if item["path"] == "oracles/python-full.json"
        )
        descriptor["sha256"] = generator._sha256(oracle_bytes)
        descriptor["size_bytes"] = len(oracle_bytes)
        manifest["integrity"]["corpus_sha256"] = generator._sha256(
            generator._compact_canonical_bytes(generator._corpus_payload(manifest))
        )
        manifest_path.write_bytes(generator._canonical_bytes(manifest))

    def test_checked_in_manifest_and_oracles_are_schema_valid_and_nonempty(self) -> None:
        self.assertEqual(validate_instance(self.manifest, self.manifest_schema), [])
        self.assertEqual(
            (MANIFEST_PATH.parent / self.manifest["$schema"]).resolve(),
            (REPO_ROOT / "tooling/migration/baseline-manifest.schema.json").resolve(),
        )
        for descriptor in self.manifest["oracle_fixtures"]:
            path = BASELINE_ROOT / descriptor["path"]
            oracle = _load(path)
            with self.subTest(oracle=descriptor["oracle_id"]):
                self.assertEqual(validate_instance(oracle, self.oracle_schema), [])
                self.assertTrue(oracle["cases"])
                self.assertEqual(
                    (path.parent / oracle["$schema"]).resolve(),
                    (REPO_ROOT / "tooling/migration/oracle-fixture.schema.json").resolve(),
                )
                self.assertEqual(descriptor["case_count"], len(oracle["cases"]))
                data = path.read_bytes()
                self.assertEqual(descriptor["size_bytes"], len(data))
                self.assertEqual(descriptor["sha256"], generator._sha256(data))

    def test_manifest_pins_tag_receipt_domains_packages_assets_and_identities(self) -> None:
        source = self.manifest["source"]
        self.assertEqual(source["tag"], EXPECTED_TAG)
        self.assertEqual(source["tag_object_oid"], EXPECTED_TAG_OBJECT)
        self.assertEqual(source["tag_type"], "annotated")
        self.assertEqual(source["peeled_commit"], EXPECTED_COMMIT)
        self.assertEqual(source["tree_access"], "git-ls-tree-and-cat-file")
        receipt = self.manifest["acceptance_receipt"]
        self.assertEqual(
            receipt["path"],
            "tooling/release/acceptance/v1.19.0-beta.1-receipt.md",
        )
        self.assertEqual(receipt["status"], "finalized")
        plan = _load(REPO_ROOT / "tooling/migration/qiongli-1x-baseline-plan.json")
        capture_inputs = plan["capture_inputs"]
        self.assertEqual(
            receipt["source_commit"], capture_inputs["finalized_receipt_commit"]
        )
        self.assertEqual(receipt["git_blob_oid"], capture_inputs["finalized_receipt_git_blob_oid"])
        self.assertEqual(receipt["sha256"], capture_inputs["finalized_receipt_sha256"])
        self.assertEqual(receipt["size_bytes"], capture_inputs["finalized_receipt_size_bytes"])
        self.assertEqual(len(self.manifest["domains"]), 11)
        self.assertTrue(all(domain["files"] for domain in self.manifest["domains"]))
        self.assertTrue(
            all(
                file["mode"] in {"100644", "100755"}
                and file["size_bytes"] >= 0
                and len(file["sha256"]) == 64
                for domain in self.manifest["domains"]
                for file in domain["files"]
            )
        )
        self.assertEqual(
            [tree["root"] for tree in self.manifest["package_trees"]],
            [
                "content/",
                "packages/python-qiongli/",
                "packages/qiongli-lite-mcp/",
                "packages/qiongli-literature-mcpb/",
                "packages/npm-qiongli/",
            ],
        )
        self.assertEqual(len(self.manifest["release_assets"]), 10)
        self.assertEqual(len(self.manifest["native_identities"]), 5)
        for native in self.manifest["native_identities"]:
            with self.subTest(container=native["container"]):
                identity = native["identity"]
                self.assertEqual(identity["target_triple"], "aarch64-apple-darwin")
                self.assertEqual(identity["runtime_implementation"], "rust")
                self.assertEqual(identity["target_policy"], "current-host-only")
                self.assertEqual(
                    native["identity_document_sha256"],
                    generator._sha256(generator._canonical_bytes(identity)),
                )
                self.assertEqual(
                    {
                        "identity_member": native["identity_member"],
                        "binary_member": native["binary_member"],
                        "identity_document_sha256": native["identity_document_sha256"],
                    },
                    capture_inputs["native_member_bindings"][native["container"]],
                )
        for tree in self.manifest["package_trees"]:
            self.assertEqual(
                tree["tree_sha256"],
                generator._sha256(
                    generator._compact_canonical_bytes(tree["files"])
                ),
            )
        self.assertEqual(
            self.manifest["integrity"]["corpus_sha256"],
            generator._sha256(
                generator._compact_canonical_bytes(
                    generator._corpus_payload(self.manifest)
                )
            ),
        )

    def test_runtime_oracles_capture_planned_capabilities_without_registry_duplication(
        self,
    ) -> None:
        plan = _load(REPO_ROOT / "tooling/migration/qiongli-1x-baseline-plan.json")
        planned = {oracle["id"]: oracle for oracle in plan["oracles"]}
        expected_source_kinds = {
            "node-mcpb": "peeled-tag-materialization",
            "python-full": "peeled-tag-materialization",
            "rust-lite": "accepted-release-binary",
        }
        for oracle_id, source_kind in expected_source_kinds.items():
            oracle = _load(BASELINE_ROOT / f"oracles/{oracle_id}.json")
            with self.subTest(oracle_id=oracle_id):
                self.assertEqual(
                    oracle["capture_kind"], "accepted-runtime-outcomes"
                )
                self.assertEqual(oracle["schema_version"], "2.0")
                self.assertEqual(
                    oracle["source"]["runtime_source"]["kind"], source_kind
                )
                self.assertEqual(
                    oracle["coverage"]["captured_capabilities"],
                    planned[oracle_id]["required_coverage"],
                )
                captured = {
                    capability
                    for case in oracle["cases"]
                    for capability in case["coverage"]
                }
                self.assertEqual(
                    captured, set(planned[oracle_id]["required_coverage"])
                )
                self.assertTrue(
                    all(case["invocation"]["operation"] for case in oracle["cases"])
                )
                self.assertTrue(
                    all(case["source_paths"] for case in oracle["cases"])
                )
                self.assertTrue(
                    all(case["outcome"]["exit_code"] == 0 for case in oracle["cases"])
                )
                self.assertNotIn("tools", oracle)
                self.assertNotIn("error_taxonomy", oracle)
                self.assertNotIn("profiles", oracle)
                serialized = json.dumps(oracle, sort_keys=True)
                self.assertNotIn("QIONGLI_CANARY_DO_NOT_ECHO", serialized)

    def test_package_and_domain_file_paths_are_sorted_and_tag_derived(self) -> None:
        snapshot = generator.TagSnapshot(REPO_ROOT, EXPECTED_TAG, EXPECTED_COMMIT)
        for collection in (self.manifest["domains"], self.manifest["package_trees"]):
            for item in collection:
                paths = [file["path"] for file in item["files"]]
                self.assertEqual(paths, sorted(paths))
                for file in item["files"]:
                    entry = snapshot.exact(file["path"])
                    self.assertEqual(file, entry.manifest_record())

    def test_tag_ref_change_cannot_change_the_tree_snapshot(self) -> None:
        real_git = generator._git
        tree_targets: list[str] = []

        def guarded_git(
            repo_root: Path,
            args: list[str],
            *,
            input_bytes: bytes | None = None,
        ) -> bytes:
            if args and args[0] == "ls-tree":
                tree_targets.append(args[-1])
                if args[-1] == EXPECTED_TAG:
                    raise AssertionError("mutable tag ref was used after peeling")
            return real_git(repo_root, args, input_bytes=input_bytes)

        with mock.patch.object(generator, "_git", side_effect=guarded_git):
            snapshot = generator.TagSnapshot(REPO_ROOT, EXPECTED_TAG, EXPECTED_COMMIT)
        self.assertEqual(tree_targets, [EXPECTED_COMMIT])
        self.assertTrue(snapshot.entries)

    def test_git_snapshot_ignores_repository_selection_environment(self) -> None:
        with tempfile.TemporaryDirectory() as temp_name:
            fake_git_dir = Path(temp_name) / "not-the-repository"
            fake_git_dir.mkdir()
            with mock.patch.dict(
                os.environ,
                {
                    "GIT_DIR": str(fake_git_dir),
                    "GIT_WORK_TREE": str(fake_git_dir),
                    "GIT_OBJECT_DIRECTORY": str(fake_git_dir),
                },
            ):
                snapshot = generator.TagSnapshot(
                    REPO_ROOT, EXPECTED_TAG, EXPECTED_COMMIT
                )
        self.assertEqual(snapshot.peeled_commit, EXPECTED_COMMIT)
        self.assertTrue(snapshot.entries)

    def test_selector_semantics_distinguish_segments_and_double_star(self) -> None:
        self.assertTrue(generator.selector_matches("content/*.md", "content/README.md"))
        self.assertFalse(
            generator.selector_matches("content/*.md", "content/workflow/README.md")
        )
        self.assertTrue(
            generator.selector_matches(
                "content/**/README.?d", "content/workflow/references/README.md"
            )
        )
        self.assertTrue(
            generator.selector_matches("**/target/**", "packages/tool/target/release/bin")
        )
        self.assertFalse(
            generator.selector_matches("packages/*/package.json", "packages/a/lib/package.json")
        )
        with self.assertRaises(generator.BaselineError):
            generator.selector_matches("content/[ab].json", "content/a.json")
        with self.assertRaises(generator.BaselineError):
            generator.selector_matches("../content/**", "content/a.json")
        with self.assertRaises(generator.BaselineError):
            generator.selector_matches("content//**", "content/a.json")

    def test_two_temp_output_roots_are_byte_for_byte_identical(self) -> None:
        evidence = (
            self.manifest["release_assets"],
            self.manifest["native_identities"],
        )
        first = generator.build_capture(
            repo_root=REPO_ROOT,
            recorded_release_evidence=evidence,
            recorded_oracle_documents=self._recorded_oracle_documents(),
        )
        second = generator.build_capture(
            repo_root=REPO_ROOT,
            recorded_release_evidence=evidence,
            recorded_oracle_documents=self._recorded_oracle_documents(),
        )
        self.assertEqual(first, second)
        with (
            tempfile.TemporaryDirectory() as first_temp,
            tempfile.TemporaryDirectory() as second_temp,
        ):
            first_root = Path(first_temp) / "one" / "baseline"
            second_root = Path(second_temp) / "two" / "baseline"
            generator.write_capture(first_root, first)
            generator.write_capture(second_root, second)
            self.assertEqual(_tree_bytes(first_root), _tree_bytes(second_root))
            serialized = b"".join(_tree_bytes(first_root).values())
            self.assertNotIn(str(first_root).encode(), serialized)
            self.assertNotIn(str(second_root).encode(), serialized)
            if ACCEPTED_ASSET_DIR:
                self.assertNotIn(ACCEPTED_ASSET_DIR.encode(), serialized)

    def test_verify_recomputes_without_launching_oracle_runtimes(self) -> None:
        commands: list[list[str]] = []
        real_run = subprocess.run

        def recording_run(command: list[str], *args: object, **kwargs: object) -> object:
            commands.append(list(command))
            return real_run(command, *args, **kwargs)

        with mock.patch.object(generator.subprocess, "run", side_effect=recording_run):
            generator.verify_capture(output_dir=BASELINE_ROOT, repo_root=REPO_ROOT)
        self.assertTrue(commands)
        self.assertTrue(all(command[0] == "git" for command in commands))
        flattened = {part for command in commands for part in command}
        self.assertTrue({"ls-tree", "cat-file"}.issubset(flattened))
        self.assertTrue(
            {"python", "python3", "node", "npm", "cargo", "rustc"}.isdisjoint(flattened)
        )

    def test_verify_rejects_oracle_mutation(self) -> None:
        with tempfile.TemporaryDirectory() as temp_name:
            baseline = self._copy_baseline(Path(temp_name))
            oracle_path = baseline / "oracles/rust-lite.json"
            oracle = _load(oracle_path)
            oracle["cases"][0]["outcome"]["value"]["mutated"] = True
            oracle_path.write_bytes(generator._canonical_bytes(oracle))
            with self.assertRaisesRegex(generator.BaselineError, "hash/size drift"):
                generator.verify_capture(output_dir=baseline, repo_root=REPO_ROOT)

    def test_verify_rejects_secret_and_machine_path_even_with_rehashed_manifest(self) -> None:
        leaks = (
            ("QIONGLI_CANARY_" + "DO_NOT_ECHO_7f3a9c", False),
            ("/" + "Users/example/private/baseline.json", False),
            ("error at /" + "tmp/private/baseline.json", False),
            ("C:" + "\\tmp\\private\\baseline.json", False),
            ("error at C:" + "\\Users\\example\\baseline.json", False),
            ("sk-" + "not-a-real-token-value", False),
            ("error at /" + "home/example/private.json", True),
            ("QIONGLI_CANARY_" + "DO_NOT_ECHO_key", True),
        )
        for leak, as_key in leaks:
            with (
                self.subTest(leak=leak, as_key=as_key),
                tempfile.TemporaryDirectory() as temp_name,
            ):
                baseline = self._copy_baseline(Path(temp_name))
                self._rewrite_oracle_with_valid_hashes(
                    baseline, leak=leak, as_key=as_key
                )
                with self.assertRaisesRegex(
                    generator.BaselineError, "secret-shaped|machine-local"
                ):
                    generator.verify_capture(output_dir=baseline, repo_root=REPO_ROOT)

    def test_portability_scan_allows_portable_https_urls(self) -> None:
        generator._scan_portability(
            {
                "homepage": "https://example.org",
                "fixture": "https://example.org/tmp/baseline.json",
            }
        )

    def test_verify_rejects_release_asset_hash_drift_against_receipt(self) -> None:
        with tempfile.TemporaryDirectory() as temp_name:
            baseline = self._copy_baseline(Path(temp_name))
            manifest_path = baseline / "manifest.json"
            manifest = _load(manifest_path)
            manifest["release_assets"][0]["sha256"] = "0" * 64
            manifest["integrity"]["corpus_sha256"] = generator._sha256(
                generator._compact_canonical_bytes(generator._corpus_payload(manifest))
            )
            manifest_path.write_bytes(generator._canonical_bytes(manifest))
            with self.assertRaisesRegex(
                generator.BaselineError, "finalized acceptance evidence"
            ):
                generator.verify_capture(output_dir=baseline, repo_root=REPO_ROOT)

    def test_verify_rejects_forged_native_archive_member_paths(self) -> None:
        with tempfile.TemporaryDirectory() as temp_name:
            baseline = self._copy_baseline(Path(temp_name))
            manifest_path = baseline / "manifest.json"
            manifest = _load(manifest_path)
            native = manifest["native_identities"][0]
            native["identity_member"] = (
                "forged/qiongli-literature-provider.target.json"
            )
            native["binary_member"] = "forged/qiongli-literature-provider"
            manifest["integrity"]["corpus_sha256"] = generator._sha256(
                generator._compact_canonical_bytes(generator._corpus_payload(manifest))
            )
            manifest_path.write_bytes(generator._canonical_bytes(manifest))
            with self.assertRaisesRegex(
                generator.BaselineError, "native member binding drift"
            ):
                generator.verify_capture(output_dir=baseline, repo_root=REPO_ROOT)

    def test_verify_rejects_extra_files_and_symlinks(self) -> None:
        with tempfile.TemporaryDirectory() as temp_name:
            baseline = self._copy_baseline(Path(temp_name))
            (baseline / "unexpected.json").write_text("{}\n", encoding="utf-8")
            with self.assertRaisesRegex(generator.BaselineError, "file set drift"):
                generator.verify_capture(output_dir=baseline, repo_root=REPO_ROOT)
        with tempfile.TemporaryDirectory() as temp_name:
            baseline = self._copy_baseline(Path(temp_name))
            (baseline / "leak").symlink_to(baseline / "manifest.json")
            with self.assertRaisesRegex(generator.BaselineError, "symlink"):
                generator.verify_capture(output_dir=baseline, repo_root=REPO_ROOT)

    def test_capture_refuses_destructive_or_unrecognized_output_directories(self) -> None:
        evidence = (
            self.manifest["release_assets"],
            self.manifest["native_identities"],
        )
        documents = generator.build_capture(
            repo_root=REPO_ROOT,
            recorded_release_evidence=evidence,
            recorded_oracle_documents=self._recorded_oracle_documents(),
        )
        for protected in (REPO_ROOT / ".git", REPO_ROOT / "tooling"):
            with self.subTest(protected=protected):
                with self.assertRaisesRegex(
                    generator.BaselineError, "canonical versioned directory"
                ):
                    generator.write_capture(
                        protected, documents, repo_root=REPO_ROOT
                    )
        with tempfile.TemporaryDirectory() as temp_name:
            victim = Path(temp_name) / "nested" / "victim"
            victim.mkdir(parents=True)
            marker = victim / "keep.txt"
            marker.write_text("must survive\n", encoding="utf-8")
            with self.assertRaisesRegex(
                generator.BaselineError, "non-baseline non-empty"
            ):
                generator.write_capture(victim, documents, repo_root=REPO_ROOT)
            self.assertEqual(marker.read_text(encoding="utf-8"), "must survive\n")
        outside_temp = Path.home() / "qiongli-baseline-do-not-write"
        with self.assertRaisesRegex(generator.BaselineError, "temporary root"):
            generator.write_capture(outside_temp, documents, repo_root=REPO_ROOT)

    def test_capture_rejects_canonical_output_with_symlink_ancestor(self) -> None:
        with tempfile.TemporaryDirectory() as temp_name:
            root = Path(temp_name)
            repo = root / "repo"
            outside = root / "outside"
            (repo / "tooling/migration").mkdir(parents=True)
            outside.mkdir()
            (repo / "tooling/migration/baselines").symlink_to(
                outside, target_is_directory=True
            )
            output = repo / generator.CANONICAL_OUTPUT_RELATIVE
            with self.assertRaisesRegex(generator.BaselineError, "symlink"):
                generator._safe_output_dir(output, repo_root=repo)

    def test_portability_scan_rejects_extended_paths_timestamps_and_pids(self) -> None:
        leaks = (
            "/private/var/folders/aa/bb/T/result.json",
            "/Volumes/External/result.json",
            "/root/private/result.json",
            "\\\\server\\share\\result.json",
            "2026-07-11T03:12:45Z",
            "process_id=48291",
        )
        for leak in leaks:
            with self.subTest(leak=leak):
                with self.assertRaises(generator.BaselineError):
                    generator._scan_portability({"value": leak})

    @unittest.skipUnless(
        ACCEPTED_ASSET_DIR and Path(ACCEPTED_ASSET_DIR).is_dir(),
        "set QIONGLI_ACCEPTED_RELEASE_ASSET_DIR to replay release assets",
    )
    def test_capture_check_replays_the_accepted_release_assets(self) -> None:
        self.assertEqual(
            generator.main(
                [
                    "capture",
                    "--check",
                    "--asset-dir",
                    str(ACCEPTED_ASSET_DIR),
                    "--output-dir",
                    str(BASELINE_ROOT),
                ]
            ),
            0,
        )


if __name__ == "__main__":
    unittest.main()
