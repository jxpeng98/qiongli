# Cross-Layer Consistency

For a public contract change, trace this chain before editing:

`canonical content/schema -> native owner -> App API/CLI/MCP -> Desktop or Host -> package -> docs/receipt`

Update only the links affected by the change, but verify the complete chain.
In particular:

- provider fields come from the native provider model, not UI conditionals;
- MCP tool names must agree across registry, schema, native dispatch, Skills,
  packaged profile, and release claims;
- project writes remain previewed, digest/revision bound, and explicit;
- installed Plugin/Skill payloads are outputs, never canonical sources;
- a receipt from an older source or package cannot qualify a changed candidate.

The last check is one packaged vertical journey when package inputs changed,
not a second set of per-layer acceptance suites.
