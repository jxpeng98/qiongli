# Public visual evidence pilot tasks

Use a new private project and Host profile. Preserve the user's configured model
and effort. Download the public sources named in `input-manifest.json`; record
failed attempts separately. Keep source PDFs, XML, rendered images and raw Host
traces outside the checkout. Freeze actual guidance, prompts, source bytes,
page images, permissions and review criteria before each observation. Review
criteria are for the coordinator, not the performing model.
Read back actual per-tool approval policy before model execution; tool visibility
alone is insufficient. For a user-authorized isolated Codex 0.162.1 pilot, the
three project read tools can have explicit `approval_mode = "approve"` while the
enabled-tool list remains restricted to those three. Confirm it via app-server
`config/read`, not `mcp get --json`, which omits those overrides. This follows the
[official configuration reference](https://developers.openai.com/codex/config-reference).

1. Read Kestin publisher PDF pp.1–8 and Bastani author PDF pp.1–22, plus the
   separate correction notice. Attach actual rendered Kestin pp.3–5 and Bastani
   pp.10,16,18. Ask for source-specific reading notes addressing independent
   learning versus assisted performance, with design, comparison, analysis unit,
   results, limitations, stable claim IDs and original locations. Preserve
   supplements and other visual pages as unreviewed. No project writes in the
   model turn. Exact native fulltext output is a second retained read surface,
   not proof that text parsing inspected an image.
2. Review those notes against original text and images. Save exact source packets
   first, then reviewed note additions using the existing capture/consolidation
   preview, academic approval, filesystem approval and digest checks. Bind local
   packet digests separately from PDF digests. In a fresh model process, supply
   only project ID, research question and canonical reading/writing guidance.
   Ask it to discover saved materials through Full MCP and draft a connected
   Chinese literature review (600–800 characters) and faithful English version
   (about 250 words), with source locations and unresolved limits outside prose.
3. Review the candidate and save the exact reviewed unit in a `partial` stage
   summary bound to its notes and source packets. This uses the supported summary
   owner; it is not a formal manuscript/claim-ledger consolidation. In another
   fresh process, supply only project ID and guidance, ask it to
   recover the draft, trace and visually recheck a figure-supported claim, and
   add a short English continuation. Do not copy the prior conversation, answer,
   packet paths, hashes or revision into this prompt.

For the last task, retain the authorized public local rollout in the isolated
Host home (omit `--ephemeral`, but start a new process without resume). Exec JSON
may omit a nested code-mode image call even while showing its sibling MCP and
shell calls. Verify the actual `view_image` call and returned image, comparing the
delivered image bytes with the saved digest. A filename, hash command or model
statement alone does not prove image delivery. Keep the raw rollout outside Git;
export only the relevant tool/image evidence, not model reasoning. Earlier
captures lacking this evidence remain unverified, never retroactively rescored.

Use `codex_preflight.check`, `antigravity_observation.run_process` and
`plugin_baseline.inspect_trace` as applicable; do not build another model runner
or treat a trace's final-answer prose as an actual tool call. Long captures need
an owned process whose lifetime exceeds the outer command's wait window. Keep
all attempts, including interrupted drivers and permission/setup failures, in
the denominator. SIGINT/SIGTERM cancellation now retains a cancelled receipt and
cleans up the owned child group when the capture runs on the main thread.

Check missing approvals, stale revisions and missing/changed saved source bytes
against this isolated project. Verify refusal and unchanged project/config bytes;
restore only deliberately altered fixture bytes. Preserve model read failures
separately from expected negative tests. Check actual model access and continuation
against source locations; local save/read success alone is not scholarly quality.
