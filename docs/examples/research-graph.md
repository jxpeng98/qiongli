---
title: Research Graph worked example
description: Follow synthetic notes into an offline evidence graph and back to their recorded sources.
---

<script setup>
import { withBase } from 'vitepress'
</script>

# From reading notes to Research Graph

This example asks whether short breaks are associated with immediate recall.
All material is invented. DemoA and DemoB are demonstration identifiers, not real
papers or findings to cite. Open the generated page without installing Qiongli,
Python or Node, or reproduce the actual CLI run from the repository.

<p><a :href="withBase('/demos/research-graph.html')" target="_blank" rel="noopener">Open the full interactive example</a> · <a :href="withBase('/demos/research-graph.html')" download>Download offline HTML</a> · <a :href="withBase('/demos/research-graph.snapshot.json')" download>Download snapshot JSON</a></p>

<iframe :src="withBase('/demos/research-graph.html')" title="Research Graph synthetic study example" loading="lazy" sandbox="allow-scripts allow-downloads" style="width:100%;height:960px;border:1px solid var(--vp-c-divider);border-radius:8px"></iframe>

The embedded page scrolls independently; use the full-page link for more room.
It uses the CLI's actual HTML template and generated data. It is a saved snapshot
and does not follow later changes to the project.

## Try three paths

1. Select **CLM-1** and inspect its two `supports` relations. Each retains a limit:
   no random assignment, one reading task or no delayed measurement. A `cites`
   relation records a citation; it does not establish support.
2. Select **DEC-1** to inspect a reviewed `informs` relation, then **DEC-2** for a
   proposed one. Decisions shape the research without becoming supporting evidence.
3. Select **CLM-2**. Lasting improvement remains untested, with no `supports`
   relation. The source checks retain that gap.

The actual output contains **9 research records and 8 non-structural relations**,
including 2 support relations. Other unfilled canonical sources remain visible.
This is an incomplete starter project, not a finished paper.

## Follow the workflow

| Step | Material or operation | Responsibility |
|---|---|---|
| Supplied notes | DemoA / Table 1 and DemoB / Table 2 in `notes/synthetic-reading-notes.md` | Invented teaching material |
| Normalized records | `evidence/claim-evidence-ledger.csv` retains claims, locations and limits | Checked normalization supplied with the example |
| Research decisions | `context/decision_log.md` and `manuscript/claims_evidence_map.md` | Accepted decisions, proposals and manuscript citations stay distinct |
| Registration and projection | CLI preview, matching digest-bound apply, then Graph reads | Existing native CLI |
| View and source inspection | HTML from the same snapshot; a source command returns the anchored ledger excerpt | Existing Graph view and source reader |

The CLI has not interpreted arbitrary prose or PDFs. In real work, you or the
model must check the material and save normalized records through the existing
approval flow. This example demonstrates projection and traceability, not reliable
automatic extraction by a model.

## Reproduce locally

These repository development commands require Rust build tools and Python 3.
Viewing the supplied HTML requires neither.

```bash
cargo build --manifest-path packages/qiongli-native/Cargo.toml --locked -p qiongli
python3 tooling/scripts/build_research_graph_example.py \
  --cli packages/qiongli-native/target/debug/qiongli \
  --destination /tmp/qiongli-graph-demo
```

On Windows, use the corresponding `qiongli.exe` and a new output directory whose
parent already exists. The script refuses existing destinations. Choose a new
directory for another run; previous results are never removed automatically.

The script copies `tests/fixtures/research_graph_example/`, registers that copy
using isolated Qiongli configuration, and confirms preview/apply only for this
new demonstration project. It leaves your normal registration and the supplied
fixture files untouched, and launches no browser.

| Output | Purpose |
|---|---|
| `research-graph.html` | Standalone offline view |
| `snapshot.json` | Complete Graph and readiness data |
| `source-excerpt.json` | Actual source read, anchored at ledger line 2 |
| `proof.json` | Binary/input hashes, relationship counts and completed checks |
| `commands.json` | Actual CLI arguments and exit codes |
| `project/`, `config/` | Retained demonstration project and isolated configuration |

The website's source commands bind to the project and snapshot used to generate
this example. They cannot be applied to your own project. After reproducing it,
use your new HTML and the same CLI, with that process's `QIONGLI_CONFIG_HOME` set
to the configuration path printed by the script. You can also read the saved
`source-excerpt.json` without executing another command.

## Check the evidence

<p><a :href="withBase('/demos/research-graph.proof.json')">View this run's verification record</a> · <a :href="withBase('/demos/research-graph.source.json')">View the actual source response</a></p>

The generator checks evidence limits, unsupported claims, decision relations,
source anchors, revision-mismatch refusal, repeatable projection, HTML/JSON parity,
and unchanged fixture/project files after reads. CLI calls run with an empty PATH,
without an external model or online service. Repository checks also detect drift
between the public example, current template and supplied inputs.

These are synthetic-data and program-behavior checks. Actual browser rendering,
clipboard permissions and file downloads still need observation; successful
generation does not establish those outcomes or research quality.
