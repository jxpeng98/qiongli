# Ritchie2013Retrieval — table and supplement continuation

DOI: https://doi.org/10.1371/journal.pone.0078976

Authors: Stuart J. Ritchie; Sergio Della Sala; Robert D. McIntosh (2013).
Creative Commons Attribution as stated in the publisher XML; version unspecified.
This packet expands access to the same public paper, not a correction or a new study.
Tables below are transcriptions of Experiment 1 cells, including printed totals.
Captions come from article XML; tables were checked against the publisher image or rendered DOC.
Text S1 is an unedited excerpt of the DOC text conversion. Formatting is lost.
Exact retrieved-byte and packet digests are in `tables-manifest.json`.

## Ritchie2013Retrieval:table-1

Publisher caption: Mean percentage scores (SDs) and sample sizes for Experiment 1.

| 原始行名 | Mind maps | No mind maps | Total score | N |
|---|---|---|---|---|
| Retrieval | 72.09 (21.58) | 77.31 (24.22) | 74.70 (22.86) | 52 |
| Non-retrieval | 68.31 (29.80) | 64.43 (28.26) | 66.23 (28.79) | 56 |
| Total score | 70.20 (25.83) | 70.41 (27.02) | 70.31 (26.33) | 108 |
| N | 52 | 56 | 108 | 空白 |

## Ritchie2013Retrieval:table-s2-condition-distribution

Publisher caption: Table S2 Distributions of each factsheet across the four cells of the experiment (upper part of table), and across year groups (lower part of table), for Experiments 1 and 2. (DOC)

| Country on Factsheet | Retrieval/Mind Maps | Retrieval/No Mind Maps | Non-retrieval/Mind Maps | Non-retrieval/No Mind Maps |
|---|---|---|---|---|
| Senegal | 8 | 8 | 7 | 8 |
| South Korea | 5 | 7 | 10 | 6 |
| Iran | 6 | 7 | 7 | 5 |
| Peru | 7 | 4 | 7 | 7 |

## Ritchie2013Retrieval:table-s2-year-group-distribution

Publisher caption: Table S2 Distributions of each factsheet across the four cells of the experiment (upper part of table), and across year groups (lower part of table), for Experiments 1 and 2. (DOC)

| Country on Factsheet | Primary 5 | Primary 7 |
|---|---|---|
| Senegal | 17 | 15 |
| South Korea | 15 | 13 |
| Iran | 13 | 11 |
| Peru | 14 | 11 |

## Ritchie2013Retrieval:table-s4

Publisher caption: Table S4 Numbers of participants in each year group in each cell of Experiments 1 and 2, only including participants who contributed data at all testing points. (DOC)

| Experiment | 条件 | Year group | Mind maps | No mind maps | Total |
|---|---|---|---|---|---|
| 1 | Retrieval | Primary 5 | 15 | 13 | 28 |
| 1 | Retrieval | Primary 7 | 11 | 13 | 24 |
| 1 | Non-retrieval | Primary 5 | 13 | 17 | 30 |
| 1 | Non-retrieval | Primary 7 | 13 | 13 | 26 |
| 1 | Total | Primary 5 | 28 | 30 | 38 |
| 1 | Total | Primary 7 | 24 | 26 | 50 |

## Ritchie2013Retrieval:table-s5-fixed-effects

Publisher caption: Table S5 Results of the generalized linear mixed model for Experiment 1. (DOC)

| Fixed effects | Coefficient | SE | z | p |
|---|---|---|---|---|
| (Intercept) | 1.82 | .23 | 8.00 | < .001 |
| Group – Non-retrieval | −1.45 | .30 | −4.75 | < .001 |
| Group – Mind Maps | −.58 | .31 | −1.85 | .06 |
| Facts recorded in learning session | 1.26 | .11 | 11.12 | < .001 |
| Retrieval × mind map interaction | 1.61 | .44 | 3.69 | < .001 |

## Ritchie2013Retrieval:table-s5-random-effects

Publisher caption: Table S5 Results of the generalized linear mixed model for Experiment 1. (DOC)

| Random effects | Variance | SD | No. observations |
|---|---|---|---|
| ID (Intercept) | .90 | .95 | 109 |

## Ritchie2013Retrieval:text-s1-supplementary-analyses-experiment-1

Publisher DOC `.s007`; heading path: Supplementary analyses → Experiment 1.

Supplementary analyses

As noted in the main article, we view the ANCOVA we used as the most transparent and straightforward way to analyze our results (especially given that the data from both experiments fit the assumptions of that method). Nevertheless, it could be argued that this analysis is not optimal since the data from the tests is in the form of binomial counts: that is, it records how many test questions, which are either correct or incorrect, each participant answered correctly [26]. In addition, since each year group were administered tests with different numbers of answers (more difficult tests for older children), we had more information for the older children, and thus more reliable results. Collapsing the scores across these tests into z-scores, as we did for the ANCOVA above, results in a loss of this extra information.

Here, we provide a supplementary analysis of the data from both of our experiments, and instead of using ANCOVA we use a generalized linear mixed-effects model that takes into account the binomial nature of the data and the different tests. For both experiments, the analysis was carried out using the ‘lmer’ command in the R package ‘lme4’ [S1]. 

Experiment 1

For the first experiment, we ran a generalized linear mixed-effects model for binomial counts. Instead of using the z-score of test ability, the dependent variables were the number of facts correctly written down, and the number of facts incorrectly written down or not written down. The model allowed for main effects of, and the interaction between, retrieval practice group and mind mapping group, and controlled for the number of facts recorded in the learning session. A random effect of participant ID number was added to indicate that participants will differ in their ability to recall facts.

The results of this analysis are shown in Table S5. The pattern of results was similar to that in the ANCOVA analysis: a significant effect of retrieval practice group, no significant effect of mind mapping group, and a significant retrieval group × mind mapping group interaction, along with a significant influence of the single covariate, number of facts recorded during the learning session.
