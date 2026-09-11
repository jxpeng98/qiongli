# 2.x quickstart

Download the [native archive](guide/cli-2x.md#standalone-binary-download) for your
system, extract it, and run `./qiongli --version` in a terminal. On Windows, use
`.\qiongli.exe --version`. The executable includes its research resources; no separate
Python, Node.js or Rust runtime is needed. The [installation guide](guide/cli-2x.md)
also gives the requirements and commands for each package-manager channel.

## Check the installation

After adding the executable to PATH, run:

```sh
qiongli doctor
qiongli setup
qiongli content
qiongli project
```

`setup` reviews other visible CLI copies without deleting or moving them.
`project` lists registered research projects; use `qiongli help project create`
for creation options. Preserve project files and backups before updating packages.
Removing an application package is not research-data cleanup.

## Use a Host

Development builds offer `qiongli install plugin` to export bundled content and
register it through official Codex or Claude commands. Follow the
[complete example](guide/cli-2x.md#install-and-upgrade-bundled-content) to choose an
export directory, then approve the file and Host operations separately. These
shortcuts are not included in the beta.3 release tag.

Start a new Host session after updating and confirm that its tools are available.
Then ask for the work directly, such as “Read this paper and explain its findings
and evidence limits.” New Codex Plugin builds also expose `$qiongli-paper-read`.
The entry loads shared guidance and the relevant workflow; it does not restart
the entire research lifecycle.

## Save and continue

Review proposed changes before saving. Graph rebuilds canonical records; ordinary
notes need Host-assisted normalization into source-bound records first. At a stage
boundary, request a summary that preserves findings, sources and changes.
Original files remain intact. Any cleanup requires your selection of individual
files and your own deletion action.

[Graph scope and checks](guide/cli-2x.md#research-graph) · [1.x commands](reference/cli.md)
