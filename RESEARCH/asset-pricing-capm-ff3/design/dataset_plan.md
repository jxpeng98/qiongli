# Dataset Plan

| Dataset | Coverage | Access | Key Variables | Risks |
|---|---|---|---|---|
| Fama/French Research Data Factors CSV | Monthly 1926-07 to 2026-06 in pinned archive | Official public ZIP; local download only | Mkt-RF, SMB, HML, RF | Historical revisions; digest drift; T-bill source change after 2024-05 |
| 25 Portfolios 5x5 CSV | Monthly 1926-07 to 2026-06 in pinned archive | Official public ZIP; local download only | 25 value-weighted and 25 equal-weighted returns | Historical revisions; sentinels; shared sort construction |

## Pinned Inputs

| Input ID | Official URL | Expected Member | SHA-256 |
|---|---|---|---|
| factors | `https://mba.tuck.dartmouth.edu/pages/faculty/ken.french/ftp/F-F_Research_Data_Factors_CSV.zip` | `F-F_Research_Data_Factors.csv` | `cd6d8e0d175b6f423862a6ad15a3073a6e4264b52b2ac9262396c79f707c6bcb` |
| portfolios | `https://mba.tuck.dartmouth.edu/pages/faculty/ken.french/ftp/25_Portfolios_5x5_CSV.zip` | `25_Portfolios_5x5.csv` | `43cfc360fca14e7d50766e8432fb8b6151c47078512efe74bd0f5d3804789a2a` |

The archives were downloaded and hashed on 2026-08-24. A changed digest is a review event, not an automatic update.
