# Repository Source Code Guidance

Status: optional diagnostic reference. RC1 is not a required migration, CI,
pull-request, or release gate for Qiongli 2.x.

Qiongli repository code follows the same lightweight development policy used
by the Python line: implement a coherent useful behavior, run the normal tools
for the changed component, review the change, and fix concrete defects. This
document does not judge academic analysis code produced for a paper; academic
workflow guidance remains under Stage I and the historical AC1 design.

## Native development policy

Maintainers may add the Rust crates, locked dependencies, build scripts,
desktop libraries, operating-system APIs, FFI, and typed process adapters
needed by the approved architecture. The installed product must not require a
user to install Python, Node, or Rust in order to start or use supported
features.

Run the language-native checks relevant to the changed component. For the
native workspace, the normal baseline is:

```bash
cargo fmt --manifest-path packages/qiongli-native/Cargo.toml --all -- --check
cargo clippy --manifest-path packages/qiongli-native/Cargo.toml \
  --workspace --all-targets --all-features --locked -- -D warnings
cargo test --manifest-path packages/qiongli-native/Cargo.toml \
  --workspace --all-targets --all-features --locked
```

Run broader integration, operating-system, packaging, signing, updater, and
clean-machine checks when preparing the corresponding feature or artifact.

Concrete credential disclosure, private-data leakage, path traversal, command
injection, unauthorized writes, destructive side effects, or data loss remain
bugs and must be fixed in the affected component. They do not require a
separate repository-governance program before unrelated migration work can
continue.

## Optional RC1 diagnostic

The historical machine policy remains at
`tooling/quality/repository-source-code-contract.yaml`, with its wrapper at
`scripts/validate_repository_source.py`. Maintainers may run it manually when
useful:

```bash
python scripts/validate_repository_source.py --base-ref <base-commit>
python scripts/validate_repository_source.py --full-tree --json
```

The validator can evaluate whole-native-tree rules for a selected file and can
classify context-free portable path text as a finding. Its result is therefore
diagnostic only and is not evidence that an unrelated change introduced a
cybersecurity defect. No GOV-201 applicability system, exception framework, or
RC1 release enforcement is required for the 2.x migration.

Generated distribution payloads remain protected by
`scripts/check_generated_payload_edits.py` because that check enforces the
canonical-source boundary rather than a general coding-style policy.
