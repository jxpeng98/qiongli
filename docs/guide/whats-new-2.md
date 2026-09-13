# Qiongli 2.0: what changes from 1.x

Qiongli 2.0.0 is the first stable release of the Rust-native CLI. It brings the
program, research content and MCP implementations into one executable, while
keeping the Host and model of your choice. Reading, literature review, study
design and writing remain the familiar starting points.

The main change is how those capabilities fit together: fewer runtime pieces to
install, one native implementation behind the CLI and MCP, and research records
that can be inspected beyond the current conversation. This release covers the
CLI and its bundled integrations. It does not mark the broader Desktop replacement,
all 1.x migrations or live collaboration acceptance complete.

## 1.x and 2.0 at a glance

The comparison uses the retained late-1.x implementation, including the 1.19
compatibility baseline. Earlier 1.x releases had different packaging and features;
not every difference below applies to every old installation.

| Area | Late 1.x | 2.0 |
|---|---|---|
| Main runtime | Python application; npm also provided an asset-management entry | Rust-native CLI with embedded research content and Lite/Full MCP |
| Direct installation | Requirements varied by entry; native Lite Marketplace bundles already existed | Complete CLI archives for three targets, with no separately installed Python, Node.js or Rust |
| Package channels | Python and npm entries had different runtime responsibilities | GitHub, npm, PyPI and Cargo deliver the same native CLI contract; each keeps its own installer prerequisites |
| Plugin and MCP | Native Lite and Python Full followed different runtime paths | Both profiles run natively; Marketplace starts Lite, CLI-installed local Plugins start Full |
| Installation flow | A larger set of surface, part and provider options | Guided `install`; `install plugin` also updates an existing verified export and Host registration |
| Research guidance | More prescribed stage sequences, roles and generic quotas | Guidance follows the requested outcome, available evidence and applicable protocol; narrow requests can stay narrow |
| Graph | Citation discovery and expansion supported literature work | Research Graph also links claims, evidence locations, decisions and gaps in canonical project records |
| Continuity | Workflow artifacts and handoff records | Structured capture, revision checks, Graph snapshots and stage summaries share source identities |
| Agent work | Workflow roles and orchestration conventions | Explicit task/source scope for Host subagents and handoff packets; the Host still supplies execution |
| Maintenance | Python runtime plus multiple distribution paths | Shared Rust services and canonical content; package builders bind versions, targets and embedded resources |

## Install the program, then connect your Host

For the fewest prerequisites, download the matching **complete CLI archive** from
[Release v2.0.0](https://github.com/jxpeng98/qiongli/releases/tag/v2.0.0), verify
`SHA256SUMS`, extract it and run `./qiongli --help` (PowerShell:
`.\qiongli.exe --help`). Supported targets are macOS ARM64, Windows x64 and
Linux x64 with glibc 2.35+. Windows includes the C runtime in the executable.
GitHub's automatic Source code archives are for building, not this installation path.

npm needs Node.js 18+, PyPI needs Python 3.9+, and Cargo compiles the CLI with
Rust 1.97+ and a native linker. Those are channel requirements. They are not
requirements for the standalone binary or its exported native Plugin. See the
[download and package commands](cli-2x.md#standalone-binary-download).

Run `qiongli install` to choose a Plugin, standalone Skills or MCP configuration.
The recommended local Plugin contains the Skills, its own copy of the program
and Full MCP configuration. The Host loads its Plugin cache; copying Skills into
`~/.agents/skills` is unnecessary. There is no separate Qiongli MCP package or
background terminal to maintain.

`qiongli install plugin` discovers the registered source and offers an update.
`upgrade plugin` and `update plugin` use the same flow. File changes and official
Host registration have separate previews and confirmations. A second Host uses
a separate source directory. The completion summary shows the version, source,
cache, verified registration and remaining session or Hook checks.

For known Codex Qiongli conflicts, 2.0 can migrate after showing the exact entries
to disable and obtaining Host confirmation. It preserves previous sources,
caches, unrelated Plugins and model settings. Unsupported identities or changed
configuration stop the operation. Claude retains manual conflict handling. If
registration fails after an old Plugin is disabled, retry or re-enable it in
Codex; the installer reports that state.

## Know which tools your Plugin provides

| Installation | MCP profile | What it provides |
|---|---|---|
| CLI-installed local Plugin | Full, 32 tools | Literature and Zotero operations plus projects, Graph and Host handoffs |
| Native Marketplace platform Plugin | Lite, 14 tools | Bounded literature, configuration, search-planning and Zotero operations |
| Standalone Skills export | None automatically | Research instructions and references |

Both MCP profiles are compiled into the program. Native Marketplace bundles need
no Node/npm bridge, Python interpreter or executable download at launch. Late
1.x already offered independent native Lite bundles; 2.0 extends native delivery
to the complete CLI and Full MCP. A common Skill name does not make unavailable
tools appear in a Host.

`qiongli doctor` and `qiongli install list` inspect local installation state.
`qiongli mcp check` exercises the current CLI's protocol. Open a new Host session
to verify actual tool discovery and call `qiongli_config_status`; check online
literature setup separately with `qiongli_literature_status`. A successful file
export alone does not establish a working session. Host applications, provider
credentials and Zotero configuration remain separate setup choices.

## Give the model direction without prescribing every step

The Skills retain source attribution, method limits and reviewable changes, while
removing generic pass counts and compulsory long workflows from small requests.
Formal reviews still follow the chosen protocol; a reply about one paragraph
does not need a whole-project artifact set.

Journal work keeps two distinct routes: assess and adapt to a journal you have
chosen, or assess a manuscript before finding suitable journals. Recommendations
must use the actual article type, evidence and current venue requirements.
Humanizer guidance improves English and Chinese flow without changing claims,
numbers, citations or uncertainty.

Use `$no-qiongli` in Codex, or say “reply only”, “NoQ问理” or “仅回复”, when you
want a conversational answer without model-invoked tools, agents or file work.
This is an instruction to the model, not a runtime permission lock. Optional
context hooks can remind the Host of continuity and approval boundaries. They
are off on first local Plugin installation; enabling them still requires Host
trust and a real event check. See [collaboration and hooks](../advanced/agent-skill-collaboration.md).

## Follow a claim through the Research Graph

The Host reads authorized material and proposes structured research records.
After review and approval, Qiongli can rebuild the Graph from those records,
retaining claim IDs, citation keys and source locations. Reviewed, proposed and
rejected relations remain distinct. Missing evidence stays visible instead of
becoming an inferred supporting link.

```sh
qiongli project graph view --project-id <prj_id> --save
qiongli project graph view --project-id <prj_id> --open
```

The first command saves a new private HTML snapshot; the second also asks the
operating system to open it. Search and filter records, follow directed links,
inspect gaps and copy source commands tied to the exported revision. The page
works offline without a server. Export again after changes: it is a snapshot,
not a live editor. Exported research may be private; filtering the view does not
remove other records from the exported snapshot.

This complements literature citation discovery. It does not automatically
understand arbitrary PDFs, prove a claim correct or replace the model's reading.
Try the [reproducible Graph example](../examples/research-graph.md), including its
unsupported claim, to see the intended behavior.

Stage summaries preserve substantial findings, source links, earlier stages and
what changed. They help a later session resume without treating a summary as new
evidence. Any retention review lists individual files for the user to select and
delete personally; summarizing never silently removes the originals.

## Collaborate through the Host you already use

Qiongli describes bounded tasks, source/candidate revisions and how to return
findings to a coordinator. Actual subagents come from the Host. A second role in
the same conversation is self-review, not independent review. Cross-Host work
uses an available authorized communication tool or a packet that you transfer.
There is no new background agent scheduler or automatic cross-device service.

The coordinator checks returned evidence and revision identity before integrating
changes. Existing preview, approval and compare-and-swap checks still control
canonical project writes. Delegation does not grant another agent permission to
overwrite research files.

## Move from 1.x without losing your way back

1. Keep a backup of research files, configuration and the previous install version.
   A new executable does not make a data migration reversible.
2. Choose one CLI installation channel. Use `type -a qiongli ql` on Unix or
   `Get-Command qiongli,ql -All` in PowerShell to see which command runs. npm, pip
   and Cargo do not overwrite one another's installations. `qiongli setup` offers
   manual cleanup guidance; you decide which files or packages to remove.
3. Install 2.0.0, check `qiongli --version`, then run `qiongli install plugin`.
   Keep the registered source directory when refreshing a Plugin.
4. Open a new Host session and check the tools. Try a copied project before moving
   important work. Old shell commands are not universally compatible; use the
   [2.x command guide](cli-2x.md) and the retained [1.x reference](../reference/cli.md).
5. For rollback, use the saved archive or reinstall the exact previous package,
   then restore its matching Plugin through the supported Host flow. Research
   files stay in place; restore a data backup separately if a migration changed them.

Stable npm installation uses `qiongli@latest`; `next` can remain on the earlier
Beta. PyPI and Cargo can pin `2.0.0`. `upgrade cli` explains the original channel's
update command; it does not run a package manager or convert installations.

## What the release checks establish

The release pipeline builds and installs the same source on all three supported
targets, verifies versions and hashes, exercises extracted binaries with an empty
PATH, and checks packaged Lite/Full MCP and registry installs. Cargo source
publication has its own checks. Public asset verification belongs to the release
workflow; submitting that workflow is not evidence that publication finished.

Rust removes the Python runtime from the native execution path, and shared owners
reduce duplicated implementation. These are architectural improvements, not a
measured speedup or maintenance-cost result. Full 1.x migration acceptance,
live Hook delivery, real browser interactions, all Host/model combinations and
cross-Host research journeys retain their own evidence requirements. The retained
Desktop and 1.x sources are not retired by this CLI release.
