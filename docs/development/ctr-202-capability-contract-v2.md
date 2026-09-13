# CTR-202 Capability Contract v2 completion

Status: **working-tree implementation complete and locally validated; commit,
review, and exact-head CI remain open**

CTR-202 expands the Capability Contract v2 pilot into a closed contract for
the MCP public-name union frozen by CTR-201. The checked registry is contract
version `2.0.0-preview.4`, registry status `preview`, and candidate coverage
mode `complete` with exactly **23 canonical tools** and **24 public names**.
The July 12 review found runtime and metadata defects behind those mechanical
fields. The current working tree fixes those defects with profile-scoped
sensitive-output metadata, bounded external-adapter environment/error handling,
managed-path checks, shared input validation, and recursive credential-key
redaction. It is not integrated evidence until committed and accepted by
exact-head CI.

## Dependency and ownership

CTR-202 starts only after the CTR-201 source-oracle gate. It owns the checked
registry, per-tool input and output schemas, complete-mode validator, contract
fixtures, tests, and CI gate. The frozen CTR-201 inventory is the target
identity; the mutable Full and Lite runtime declarations are conformance
subjects and must not be allowed to silently shrink that target.

FND-202 is a separate successor to CTR-201. CTR-202 does not build the Rust
resource pack, and FND-202 is not evidence that this contract is complete.

## Exact profile matrix

`qiongli_open_config_wizard` is the single compatibility alias. It resolves to
canonical tool `qiongli_configure_provider` and accounts for the difference
between canonical-tool and public-name counts.

| Surface category | Canonical / public | Canonical tools | `skill-only` | `marketplace-lite` | `full` |
|---|---:|---|---|---|---|
| Shared | 9 / 10 | `qiongli_config_status`, `qiongli_save_provider_config`, `qiongli_configure_provider`, `qiongli_literature_status`, `qiongli_search_plan`, `qiongli_literature_search`, `qiongli_literature_export_evidence`, `qiongli_orchestrator_route`, `qiongli_task_plan` | metadata-only | tool | tool |
| Marketplace Lite only | 2 / 2 | `qiongli_zotero_status`, `qiongli_zotero_export_import_files` | metadata-only | tool | unavailable |
| Full only | 12 / 12 | `qiongli_collect_evidence`, `qiongli_list_provider_env`, `qiongli_test_provider`, `qiongli_subject_status`, `qiongli_subject_update`, `qiongli_orchestrator_doctor`, `qiongli_lifecycle_plan`, `qiongli_journal_fit_recommend`, `qiongli_experience_query`, `qiongli_experience_show`, `qiongli_experience_lessons`, `qiongli_task_run` | metadata-only | unavailable | tool |
| Total union | **23 / 24** | frozen CTR-201 union | **23 canonical / 24 names metadata-only** | **11 canonical / 12 names tool-exposed** | **21 canonical / 22 names tool-exposed** |

An unavailable profile must not carry input or output schema references. A
tool-exposed profile must use the checked canonical schemas and its declared
transport. MCPB and Lite manifests close only the `marketplace-lite` tool set;
Full-only records must not be added to those manifests to make validation pass.

## Safety and conformance strategy

- Anchor the exact canonical and public-name set to CTR-201 evidence, then
  reject missing, extra, reordered, duplicated, or remapped names.
- Validate reverse closure as well as registry-to-runtime lookup: every Full or
  Lite runtime declaration must have exactly one truthful contract record for
  the profile that exposes it.
- Compare every exposed runtime input declaration with its profile schema. All
  declarations now match their checked profile schemas. Full
  `qiongli_literature_search` intentionally preserves its shipped
  `additionalProperties: true` behavior as explicit compatibility debt; it
  must not be described as recursively closed. Other declarations retain the
  strictness encoded by their checked schemas.
- Close and validate each output's top-level envelope rather than using an
  unconstrained placeholder. Some nested legacy payloads remain intentionally
  dynamic, so this is not a claim of recursively exact output shapes.
- Cover all 34 tool-exposed profile/public-name combinations. The fixture has
  29 positive calls whose structured outputs validate against their profile
  schemas and five intentional input-rejection calls: Full and Lite
  `qiongli_configure_provider`, their `qiongli_open_config_wizard` aliases, and
  Lite `qiongli_zotero_status`. Those five fail before handler dispatch and
  avoid opening a listener or running a loopback probe. The
  `qiongli_experience_show` positive case uses an isolated seeded record.
- Reject profile overclaims, unresolved aliases, empty fixture IDs,
  placeholder/TODO metadata, raw-secret output, machine-local paths in checked
  contract or fixture metadata, and unredacted sensitive-path declarations.
  Runtime-local paths remain allowed only in explicitly declared sensitive
  output fields and are confined to isolated roots in executable tests.
- Keep generated plugin payloads, Marketplace catalogs, release versions,
  tags, unrelated Rust-native 2.x product behavior, and user host/cache
  locations outside this slice. Contract-required Lite sanitization is runtime
  security work under SEC-201C, not a reason to expand a CTR schema slice.

## Local validation record

The current coherent candidate satisfies these contract conditions:

1. `2.0.0-preview.4` has status `preview`, coverage mode `complete`, and exact
   `23/23` canonical plus `24/24` public coverage;
2. all three profiles match the matrix above, including the single alias and
   both unavailable-profile boundaries;
3. checked schemas, runtime declarations, Lite/MCPB manifests, and all 34
   profile/public smoke cases pass complete-mode closure; and
4. the frozen CTR-201 target binding and mutation checks reject silent target
   shrink and profile drift.

Actual local results:

| Validation scope | Result |
|---|---|
| Complete Capability Contract validator | passed |
| CTR-201 inventory validator | passed |
| Capability Contract, profile smoke, and runtime-input suites | 51 run; 50 passed, 1 listener-dependent case skipped |
| Full MCP handler, connector, experience, and literature suites | 140 passed |
| Executable contract smoke suite | 2 passed |
| Node MCPB suite | 140 run; 137 passed, 3 platform-specific cases skipped |
| Targeted Lite Rust MCP suite | 9 passed with loopback access |
| Native Rust workspace | fmt and Clippy passed; 11 tests passed |
| Full Cargo suite | not green locally; unchanged `config_wizard` code fails in this macOS sandbox and is not CTR-202 completion evidence |

The two directly recorded validator commands are:

```bash
python3 scripts/validate_capability_contract.py --require-complete
python3 scripts/validate_ctr_201_inventory.py
python3 -m unittest tests.test_capability_contract_v2 \
  tests.test_capability_contract_v2_smoke \
  tests.test_mcp_contract_runtime_input \
  tests.test_mcp_input_validation
```

## July 12 findings and closure

Review exposed these boundary classes:

- attacker-controlled keys, provider errors, and successful nested payloads
  could cross Full or Lite output boundaries without uniform sanitization;
- experience and filesystem evidence paths could traverse or follow managed
  symlinks outside their intended roots;
- external adapters had an overbroad environment/output boundary and could
  perform effects not represented by a read-only contract;
- tool-level `sensitive_output_paths` cannot truthfully describe divergent
  Full/Lite envelopes for literature, routing, planning, subject, and doctor
  outputs.

The current candidate closes them directly as concrete product defects:

1. Full and Lite remove credential-bearing nested keys and return fixed public
   errors instead of reflecting provider or filesystem exception text.
2. Experience records reject traversal and symlink redirects through managed
   trace, run, index, and record paths.
3. External command adapters receive a minimal environment, do not inherit
   stored provider credentials when user-overridden, bound output size, and do
   not return raw stderr or startup exception text.
4. Divergent Full/Lite output schemas carry profile-scoped sensitive paths and
   mutation tests require both exposed profiles.

SEC-201 and CTR-202A-K remain useful evidence labels, but no longer require a
specific PR split. Contract, runtime, fixture, and test changes may be reviewed
together when they form one coherent working behavior.

## Remaining acceptance gate

CTR-202 is not integrated until this working-tree candidate is committed,
reviewed, exact-head CI passes, and the protected branch accepts it. Do not turn
targeted Rust Lite results into a full cross-platform or release claim.

## Claim boundary

Even after the coverage gate passes, CTR-202 does not establish a stable
Contract v2, Rust MCP or CLI parity, a Rust orchestrator, FND-202 resource-pack
implementation, desktop or Marketplace installation, a signed artifact, a
published 2.x alpha, or clean-machine zero-runtime acceptance. Those claims
remain owned by their later roadmap tasks and release gates.
