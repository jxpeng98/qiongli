---
id: ethics-irb-helper
stage: D_ethics
description: "Prepare IRB-ready materials: risk assessment, consent, recruitment, data governance."
inputs:
  - type: DesignSpec
    description: "Study design with participant/data types"
outputs:
  - type: EthicsPackage
    artifact: "ethics_irb.md"
constraints:
  - "Document study risks and the evidenced institutional review status"
  - "Draft consent materials when applicable to the protocol"
  - "Must address applicable data governance frameworks (GDPR, HIPAA, etc.)"
  - "Must document any deception, sensitive topics, or vulnerable populations"
failure_modes:
  - "Study type does not involve human subjects but review is still needed"
  - "Cross-jurisdiction research with conflicting ethics requirements"
  - "Secondary data re-analysis with unclear original consent scope"
tools: [filesystem]
tags: [ethics, IRB, consent, privacy, GDPR, compliance, human-subjects]
domain_aware: false
---

# Ethics & IRB Helper Skill

Prepare the requested parts of an ethics application from the protocol and documented decisions.

## Purpose

Prepare IRB-ready materials: risk assessment, consent, recruitment, data governance.

## Related Task IDs

- `D1` (ethics/IRB pack)

## Output (contract path)

- `RESEARCH/[topic]/ethics_irb.md`

## When to Use

- Before data collection involving human participants or sensitive data
- Before using secondary datasets (verify original consent covers your use case)
- When the venue or institution requires an ethics review statement
- When research touches vulnerable populations, deception, or sensitive topics

## Inputs

- `DesignSpec`: Study design with participant/data types
- If a required input is missing or insufficient, write a gap note under `RESEARCH/[topic]/context/gap_notes.md` and ask for the missing artifact instead of inventing content.
- Treat literature, data, citations, and project files as evidence sources; keep unsupported assumptions visibly marked.

## Process

### Step 1: Establish the review status

Use `references/stage-D-ethics.md` to match the proposed population, data,
activity and destination to the governing decision. Reuse its issuing body,
scope, date and reference when supplied. If a determination is missing, draft
the study description and flag the open determination for the responsible body.

### Step 2: Assess risks and mitigations

Describe how the procedures, identifiability, sensitive information or dependency
relationships create risk for these participants. Match each material risk to a
specific mitigation. Use the applicable institutional criteria to prepare a
review request. An activity label, an anonymous survey or public access alone
cannot establish exemption, an approval or a review category.

### Step 3: Prepare Consent Document

Use the governing protocol and institutional requirements to select and verify
the applicable consent elements.

#### Informed Consent Required Elements

| Element | Content | Required For |
|---------|---------|-------------|
| **Study title** | Full title as submitted | All |
| **PI and contact** | Name, institution, email, phone | All |
| **Purpose** | Why the study is being done, in plain language | All |
| **Procedures** | What participation involves, step by step | All |
| **Duration** | Expected time commitment (per session and total) | All |
| **Risks** | Physical, psychological, social, economic, legal risks | All |
| **Benefits** | To participant (if any) and to society/science | All |
| **Alternatives** | Other options available to participant | Clinical |
| **Confidentiality** | How data stored, who has access, retention period | All |
| **Compensation** | Amount, form, and conditions | If applicable |
| **Voluntary participation** | Right to withdraw at any time without penalty | All |
| **Data sharing** | Whether data may be shared or reused | When applicable |
| **Future use** | Whether data may be used for future research | Best practice |
| **Contact for questions** | PI + independent contact (ombudsperson/IRB office) | All |
| **Signature/consent mechanism** | Physical signature, electronic consent, or verbal consent documentation | All |

**For online/survey research**, check the actual procedures:
- Describe the approved consent mechanism, such as a checkbox if applicable
- Include data collection technology disclosure (cookies, IP logging)
- State the supported withdrawal and deletion options, including limits after anonymization

**For interview research**, add:
- Recording consent (audio/video separately)
- Transcript review opportunity
- Pseudonym/anonymization procedure
- Data excerpt usage in publications

### Step 4: Prepare Recruitment Materials

| Material | Purpose | Key Requirements |
|----------|---------|-----------------|
| Recruitment script/email | Initial contact | IRB-approved language; no coercion |
| Flyer/posting | Broad recruitment | Must include PI contact; avoid guaranteeing benefits |
| Screening questionnaire | Eligibility determination | Keep minimal; destroy ineligible data |
| Social media post | Online recruitment | Platform-specific rules; screenshot for records |

**Anti-coercion checklist**:
- [ ] No academic credit as sole incentive (offer alternatives)
- [ ] No employer/supervisor in recruitment chain
- [ ] Compensation is not so high as to constitute undue inducement
- [ ] Power dynamics addressed (professor↔student, doctor↔patient)

### Step 5: Data Governance Plan

| Aspect | What to Document |
|--------|-----------------|
| **Storage** | Where (institution server, cloud, local), encryption standard |
| **Access control** | Who can access raw vs de-identified data |
| **De-identification** | Method (see `deidentification-planner` D3) |
| **Retention** | How long, per institutional/funder requirements |
| **Destruction** | When and how data will be destroyed |
| **Transfer** | Rules for sharing with collaborators, especially cross-border |
| **Breach protocol** | Steps if data is compromised |

Identify applicable governance requirements from the actual institution,
jurisdiction, data agreement and planned use. Record the source and any
unresolved question alongside the affected procedure.

### Step 6: Manuscript disclosures

Use `statement-generator` for requested manuscript declarations. Report approval,
consent, sharing conditions and author verification only when the supplied
records support them. If review is pending, use that status in the draft.

## Output Contract

- `EthicsPackage`: write `RESEARCH/[topic]/ethics_irb.md`.
- When the pack includes participant materials, use
  `RESEARCH/[topic]/instruments/consent_form.md` and
  `RESEARCH/[topic]/instruments/recruitment_script.md` for the applicable drafts.
- Separate finding, interpretation, and implication in the final artifact.
- Do not invent citations, data, sample sizes, statistical results, or reviewer comments.
- Apply `references/academic-output-rubric.md` before finalizing scholarly prose or review artifacts.

## Quality Bar

For a complete application draft, check the following against the protocol:

- [ ] Study risks and mitigations are documented, with institutional review status evidenced or identified as pending
- [ ] Consent materials cover the applicable protocol and institutional requirements
- [ ] Recruitment materials are non-coercive and IRB-appropriate
- [ ] Data governance plan covers storage, access, retention, and destruction
- [ ] Applicable regulatory frameworks identified and addressed
- [ ] Manuscript ethics statement drafted
- [ ] If vulnerable populations: specific protections documented
- [ ] If deception: debriefing procedure included

## Common Pitfalls

| Pitfall | Problem | Fix |
|---------|---------|-----|
| "No human subjects" assumption for online data | Social media users may be identifiable | Check institutional policy on internet research |
| Consent form too technical | Participants don't understand risks | Use language participants can understand and check the institution’s requirements |
| Missing cross-border considerations | The destination may introduce additional obligations | Verify applicable institutional and transfer requirements for the proposed use |
| Using a draft recruitment script as approved | Its status may differ from the protocol | Check the decision and identify which participant materials it covers |
| Not planning for incidental findings | Sensitive data may reveal concerning information | Pre-specify what constitutes a reportable finding and the response protocol |

## Formal pack template

Use `templates/ethics-irb-pack.md` for the formal pack. Record the basis and
status of any institutional decision alongside the risk assessment.
