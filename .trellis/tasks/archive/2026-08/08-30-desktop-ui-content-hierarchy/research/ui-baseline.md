# Populated Desktop UI baseline

Captured from `?fixture=source-read-only` at the default 1024x576 browser
viewport before implementation.

| Route | Visible text nodes | Median | Below 12px | At or below 12px | Cards | Page height | Horizontal overflow |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `/overview` | 109 | 13px | 41 | 43 | 7 | 576px | no |
| `/research-library` | 142 | 12px | 64 | 72 | 13 | 2104px | no |
| `/academic-graph` | 58 | 13px | 18 | 21 | 1 | 576px | no |
| `/client-integrations` | 138 | 11px | 70 | 71 | 12 | 837px | no |
| `/about` | 95 | 13px | 43 | 44 | 3 | 776px | no |

All five routes use a 13px body and a 24px page title at this viewport. The
research library and client integrations are the clearest regression surfaces:
half or more of their visible text renders at 12px or below.

## Post-change audit

Captured from the same populated fixture after implementation. The app uses a
14px body; 11px is limited to terse technical metadata and project tags.

| Route | Visible text nodes | Median | Below 12px | Page height | Horizontal overflow |
| --- | ---: | ---: | ---: | ---: | --- |
| `/overview` | 109 | 14px | 5 | 637px | no |
| `/research-library` | 144 | 13px | 12 | 1710px | no |
| `/academic-graph` | 58 | 14px | 1 | 576px | no |
| `/client-integrations` | 138 | 14px | 5 | 900px | no |
| `/about` | 95 | 14px | 19 | 935px | no |

The research-library topology remains available through a native disclosure
and expands from 1710px to 2418px when requested. Light and dark theme checks
retained solid, legible surfaces. The in-app browser viewport is fixed at
1024px; narrower breakpoints are guarded by the existing responsive source
contracts, including the 1040px/520px library and 900px/460px integration
collapses.
