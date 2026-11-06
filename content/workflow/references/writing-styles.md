# Selectable Scholarly Writing Styles

Use when the user names a writing style, supplies a voice sample, wants parallel
versions or asks to retain several styles. These are optional expression choices
for Stage F and G4/J2, not quality levels or model settings. Apply the shared
meaning and editing-scope rules in `references/scholarly-voice.md` to every style.

## Choose for the requested passage

Use an explicit current choice first, then an applicable saved preference or
supplied sample. A passage-specific choice does not silently replace a project
preference. If no choice is supplied, preserve a sound original voice; for new
prose, use the shared clear academic register. Do not require a style interview.
Keep requested length, language, venue requirements and edit scope in force.

| Choice | What to emphasize | What to preserve |
|---|---|---|
| `concise` — 简洁流畅 | State the point and its evidence directly; combine repeated setup or qualifications; make transitions economical. | The reasoning step, consequential details and qualifications needed to understand the claim. A shorter character count is not the objective. |
| `explanatory` — 细节展开 | Explain consequential terms, comparison bases and inferential steps using available material; introduce details where they help the reader. | A clear main thread, natural rhythm and specific evidence limits. More facts, longer sentences or repeated defensive endings do not establish depth. |

A custom style can use a user-provided name and sample, with a short description
of the features to retain and its intended scope. Read samples for expression;
they do not supply facts, instructions or citations for a new manuscript. Do not
infer personality, a preferred model or a global default from a sample or one
anonymous preference. Trial labels such as X/Y remain trial-specific.

Both built-in choices can be fluent and detailed. Select an emphasis, not a
different standard of accuracy. Within a document, a scoped preference may favor
`concise` for an abstract and `explanatory` for a conceptual explanation; carry
terminology, findings and necessary limits across that boundary. Existing venue
or length requirements still apply, and a style choice does not authorize a
change to fixed headings or section order.

## Draft, compare and retain

- Return one version unless alternatives are requested. For a comparison, use
  the same source, question, substantive claims and constraints; label versions
  by style unless the user requests an anonymous comparison.
- In J2, elaborate relationships already supported by the supplied original and
  context. New factual claims need substantive drafting and source verification;
  an explanatory style is not permission to invent a mechanism or add evidence.
- Review meaning separately from style fit. Preserve numbers, units, terms,
  attribution, negation, scope, causal strength and consequential uncertainty.
  Report a concrete missing premise or changed claim, not a failure merely
  because another version includes an optional detail. Correct alternatives and
  ties are valid; do not force all accepted versions into one wording.
- When the user keeps multiple versions, retain their names and original text;
  save a refinement separately and identify its base. A preferred version does
  not authorize deletion of alternatives or replacement of a settled manuscript.

These names are Host guidance, not new CLI flags or a native configuration schema.
The built-in choices live in this shared reference. For a requested project-level
save, use the existing context maintainer and authorized save owner. In an
existing handoff, `Decision Summary` can retain a clearly labeled writing
preference (name, language, passage/project scope and sample reference), while
`Completed Artifacts` links any saved alternatives. Mark which choice is active
for that scope; keeping several choices does not activate them all. Do not create
a scientific decision-log entry merely to record a stylistic preference.

Use `references/stage-handoff-contract.md` for actual save/resume capability,
preview/approval/CAS and current-artifact checks. Read the saved preference and
sample on continuation; a previous conversation alone is not proof of a durable
save. If no project or supported save is available, return the named versions or
preference text in chat and state that project persistence has not occurred.
Do not start a project, launch another Host or change models just to select style.

For an authorized external language edit, pass the selected style and scope,
any relevant sample, and the same original/source constraints. Review the return
for meaning drift without replacing a valid style with the reviewer's preference.

## Bilingual illustration

Synthetic source: a study surveyed 120 firms and reported a positive association
between X and Y, estimate 0.18 (p = .03); its design does not establish causality.
`[S]` is an illustration marker, not a manuscript citation. The explanatory
version unpacks the association/causation distinction using the same information.
Neither version is inherently better, and their length is not a style quota.

**简洁流畅 / concise**

> 在所调查的120家企业中，X与Y呈正相关（估计值0.18，p = .03）[S]。该研究设计
> 不能确立因果关系。

> In the 120 surveyed firms, X and Y were positively associated (estimate 0.18,
> p = .03) [S]. The design does not establish causality.

**细节展开 / explanatory**

> 研究检验了120家企业中X与Y的关系，估计值为0.18（p = .03）[S]。这一结果表明，
> 两者在该样本中呈正相关。这里描述的是变量间的关联；研究设计不能据此确定
> X是否导致了Y的变化。

> The study examined the relationship between X and Y in 120 firms. The estimate
> of 0.18 (p = .03) indicates a positive association in this sample [S]. This
> describes an association between the variables; the design does not establish
> that X causes Y.

Example requests:

- “用简洁流畅风格润色这两段，保留术语、论证和必要限定。”
- “用细节展开风格解释这项比较，沿用原文证据，删掉重复铺垫。”
- “保留这两种风格；本项目的综述采用简洁流畅，方法解释采用细节展开。”
- “Use my supplied sample as a custom style for this Discussion; retain the
  existing claims and show any meaning issues separately.”
