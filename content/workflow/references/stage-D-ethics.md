# Stage D — Ethics / IRB / Compliance (D1–D3)

This stage produces ethics-ready materials and forces early clarity on privacy, consent, and data governance.

## Canonical outputs (contract paths)

- `D1` → `ethics_irb.md`
- `D2` → `manuscript/manuscript.md` (ethics + availability statements section)
- `D3` → `compliance/deidentification_plan.md`

## Quality gate focus

- `Q4` (reproducibility baseline): governance + availability statements should match the actual planned artifacts.

## Match permission to the actual use

Distinguish planning, recruitment, collection, secondary use, linkage, external
processing and sharing. Reuse an existing decision only when it covers the
population, data, use and destination now proposed. Record the deciding body,
reference/date and status from supplied evidence; do not invent approval or
self-award an exemption. Missing permission blocks the affected action while
authorized planning can continue.

Use the relevant discipline guide for minors, dependent relationships, patient
records, politically sensitive participants, restricted archives or protected
locations. Public availability is not blanket consent for reuse or disclosure.
Check whether consent covers quotations, images, recordings and external AI
processing. Explain feasible withdrawal limits once data are irreversibly
anonymized or incorporated into shared outputs; do not promise impossible removal.

Separate pseudonymization from anonymization. Keep linkage keys apart from
analysis data, specify who can access each, and assess combinations of indirect
identifiers. Carry approved sharing restrictions into reproducibility materials
and the D2/H1 statements; synthetic data do not prove that real data are anonymous.

---

## D1 — Ethics / IRB Pack

**Definition of done**
- Participant population and recruitment are clearly described
- Risks are identified and mitigations are stated
- Consent/withdrawal process is documented (or justified as not applicable)
- Data security + access control + retention are specified

Suggested structure: `ethics_irb.md`

```markdown
# Ethics / IRB Pack

## Study overview
- Purpose:
- Population:
- Procedures:

## Risk assessment
| Risk | Likelihood | Impact | Mitigation |
|---|---:|---:|---|

## Consent
- Consent process:
- Withdrawal:

## Privacy & security
- Data minimization:
- Storage:
- Access control:
- Retention:

## Sensitive data handling
- Identifiers collected:
- De-identification linkage (D3):

## Recruitment materials (if applicable)
- ...
```

---

## D2 — Manuscript-Ready Ethics & Availability Statements

Write statements that match the venue’s required disclosure format.

**Definition of done**
- Evidenced ethics approval or exemption status (or a clearly unresolved decision)
- Informed consent statement (or not applicable)
- Data availability statement
- Code availability statement
- Competing interests / funding (if applicable)

Write into: `manuscript/manuscript.md` (usually Methods end or a dedicated section).

---

## D3 — De-identification Plan

Treat as a technical privacy plan, not a vague promise.

**Definition of done**
- Data classification (direct identifiers / quasi-identifiers / sensitive attributes)
- Threat model (who could re-identify and how)
- Concrete transformation plan (suppression/generalization/aggregation/perturbation)
- Re-identification risk evaluation approach

Write into: `compliance/deidentification_plan.md`.

Suggested table:

```markdown
| Field | Type (direct/quasi/sensitive) | Action | Rationale |
|---|---|---|---|
```
