# Reuse And Scope

Before adding code, search for the shared owner and every caller. Prefer, in
order: no new behavior, an existing project helper, the standard library, a
native platform feature, an installed dependency, then the smallest new code.

Fix a defect once at the shared boundary. Do not add a second registry,
materializer, project format, provider model, release ledger, or umbrella test.
Add one focused regression for non-trivial logic.

Defer work that does not improve the current Trellis task's observable outcome.
