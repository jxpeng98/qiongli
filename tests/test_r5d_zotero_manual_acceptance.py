from __future__ import annotations

import hashlib
import json
import stat
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
SCRIPT = REPO_ROOT / "scripts" / "r5d_zotero_manual_acceptance.mjs"
CONFIRMATIONS = [
    "isolated-profile-no-sync",
    "missing-companion-fallback",
    "preview-cancel-no-profile-write",
    "legacy-update-handoff",
    "zotero-owned-install",
    "restart-ready-live-contract",
    "search-write-replay-duplicate-redaction",
    "disable-reenable-remove-fallback",
    "disposable-profile-removed",
]


def automated_receipt(*, product_build: str = "a" * 40) -> dict[str, object]:
    return {
        "schemaVersion": 1,
        "recordType": "qiongli-r5d-zotero-automated-acceptance",
        "status": "accepted-automated-nonpublishing",
        "publicationAllowed": False,
        "productVersion": "2.0.0-alpha.2",
        "productBuild": product_build,
        "executableSha256": "b" * 64,
        "packageManifestSha256": "c" * 64,
        "companion": {
            "version": "0.3.0",
            "endpointVersion": "2",
            "zoteroMinimumVersion": "8.0",
            "zoteroMaximumVersion": "9.0.*",
            "xpiSha256": "d" * 64,
            "artifactManifestSha256": "e" * 64,
            "releaseTag": "v2.0.0-alpha.2",
            "updateLink": (
                "https://github.com/jxpeng98/qiongli/releases/download/"
                "v2.0.0-alpha.2/qiongli-zotero-companion-0.3.0.xpi"
            ),
            "updateManifestSha256": "f" * 64,
        },
        "checks": {
            "appResourceArtifactBound": True,
            "desktopPackageManifestBound": True,
            "releaseArtifactByteIdentity": True,
            "automaticUpdateManifestBound": True,
            "startupSnapshotObservationNeutral": True,
            "isolatedHomeProfileUnchanged": True,
            "nativeStateMatrix": True,
            "legacyEndpointRequiresUpdate": True,
            "disposableSearchCollectionsTagsNotesAttachments": True,
            "approvedWriteReceiptLifecycle": True,
            "duplicateAndCuratedMetadataPreservation": True,
            "endpointShutdownRemoval": True,
            "importFileFallback": True,
        },
        "manualGates": {
            "zoteroOwnedInstallConfirmation": "not-run",
            "restartActivationObservation": "not-run",
            "displayedAppStateReview": "not-run",
            "companionDisable": "not-run",
            "companionRemoval": "not-run",
        },
    }


def run_recorder(
    automated: Path,
    output: Path,
    *,
    confirmations: list[str] | None = None,
) -> subprocess.CompletedProcess[str]:
    arguments = [
        "node",
        str(SCRIPT),
        "--automated-receipt",
        str(automated),
        "--out",
        str(output),
        "--operator-id",
        "local-r5d-operator",
    ]
    for confirmation in confirmations if confirmations is not None else CONFIRMATIONS:
        arguments.extend(["--confirm", confirmation])
    return subprocess.run(
        arguments,
        cwd=REPO_ROOT,
        text=True,
        capture_output=True,
        check=False,
    )


class R5DZoteroManualAcceptanceTests(unittest.TestCase):
    def test_lists_the_exact_manual_gate_identifiers(self) -> None:
        result = subprocess.run(
            ["node", str(SCRIPT), "--list-gates"],
            cwd=REPO_ROOT,
            text=True,
            capture_output=True,
            check=False,
        )

        self.assertEqual(result.returncode, 0, msg=result.stderr)
        self.assertEqual(result.stdout.splitlines(), CONFIRMATIONS)

    def test_records_receipt_bound_to_clean_packaged_automated_evidence(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir).resolve()
            automated = root / "automated.json"
            output = root / "manual.json"
            automated_bytes = (
                json.dumps(automated_receipt(), indent=2, sort_keys=True) + "\n"
            ).encode()
            automated.write_bytes(automated_bytes)

            result = run_recorder(automated, output)

            self.assertEqual(result.returncode, 0, msg=result.stderr)
            receipt = json.loads(output.read_text(encoding="utf-8"))
            self.assertEqual(receipt["status"], "accepted-manual-nonpublishing")
            self.assertFalse(receipt["publicationAllowed"])
            self.assertEqual(receipt["source"]["productBuild"], "a" * 40)
            self.assertEqual(
                receipt["source"]["automatedReceiptSha256"],
                hashlib.sha256(automated_bytes).hexdigest(),
            )
            self.assertEqual(
                receipt["confirmations"],
                {confirmation: "confirmed" for confirmation in CONFIRMATIONS},
            )
            self.assertNotIn(str(root), output.read_text(encoding="utf-8"))
            self.assertEqual(
                stat.S_IMODE(output.stat().st_mode),
                0o600,
            )

    def test_rejects_a_missing_confirmation_without_writing(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir).resolve()
            automated = root / "automated.json"
            output = root / "manual.json"
            automated.write_text(json.dumps(automated_receipt()), encoding="utf-8")

            result = run_recorder(
                automated,
                output,
                confirmations=CONFIRMATIONS[:-1],
            )

            self.assertNotEqual(result.returncode, 0)
            self.assertIn("r5d-zotero-manual-confirmation-missing", result.stderr)
            self.assertFalse(output.exists())

    def test_rejects_dirty_source_build_automated_receipt(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir).resolve()
            automated = root / "automated.json"
            output = root / "manual.json"
            automated.write_text(
                json.dumps(automated_receipt(product_build="source-build")),
                encoding="utf-8",
            )

            result = run_recorder(automated, output)

            self.assertNotEqual(result.returncode, 0)
            self.assertIn(
                "r5d-zotero-manual-automated-receipt-invalid",
                result.stderr,
            )
            self.assertFalse(output.exists())

    def test_rejects_existing_output(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir).resolve()
            automated = root / "automated.json"
            output = root / "manual.json"
            automated.write_text(json.dumps(automated_receipt()), encoding="utf-8")
            output.write_text("keep", encoding="utf-8")

            result = run_recorder(automated, output)

            self.assertNotEqual(result.returncode, 0)
            self.assertIn("r5d-zotero-manual-output-exists", result.stderr)
            self.assertEqual(output.read_text(encoding="utf-8"), "keep")


if __name__ == "__main__":
    unittest.main()
