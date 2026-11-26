---
description: "Turn reviewer comments into a revision plan, response matrix, and response letter."
---

# Rebuttal / Revision Response

Draft the response the user requested from the actual reviewer comments,
available manuscript text and author decisions. For a single comment, return
the reviewer-facing reply in chat. Give the reader the reason for the answer and
identify any verified change with its location. Describe a proposed change as
proposed, and ask only when an unresolved author decision affects the response.

## Request

$ARGUMENTS

## Formal H2 package

Use `rebuttal-assistant` and the H2 section of `references/stage-H-submission.md`.
Reuse the selected project, reviewer/round IDs, any editor letter and stated
constraints. Address each supplied point and check the response against the
matching manuscript version. Keep unsupported change locations and unperformed
analyses pending. For an explicit neighboring review focus (H2_5, H3 or H4),
use that task's card and outputs in the same stage reference.

- `RESEARCH/[topic]/revision/response_matrix.md` uses `templates/rebuttal-response-matrix.md`.
- `RESEARCH/[topic]/revision/response_letter.md` uses `templates/rebuttal-letter.md`.
- Include `RESEARCH/[topic]/revision/revision_plan.md` when a separate plan is needed.

For a requested save, resolve the destination and use preview, explicit approval
and revision-checked apply. Preparing a letter does not send it to the editor.
