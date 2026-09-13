# /// script
# requires-python = ">=3.11"
# dependencies = [
#   "numpy==2.3.2",
#   "pandas==2.3.2",
#   "statsmodels==0.14.5",
# ]
# ///
"""Run the pinned CAPM-versus-FF3 portfolio comparison."""

from __future__ import annotations

import argparse
import cProfile
import csv
import hashlib
import io
import json
import os
import pstats
import re
import sys
import urllib.request
import zipfile
from pathlib import Path, PurePosixPath
from typing import Any

import numpy as np
import statsmodels.api as sm
from statsmodels.stats.diagnostic import acorr_ljungbox, het_breuschpagan
from statsmodels.stats.stattools import durbin_watson


ANALYSIS_DIR = Path(__file__).resolve().parent
RAW_DIR = ANALYSIS_DIR / "data" / "raw"
RESULTS_DIR = ANALYSIS_DIR / "results"
START_MONTH = 196307
HAC_LAGS = 6
PIN_REVIEWED_ON = "2026-08-24"
MISSING_SENTINELS = {-99.99, -999.0}
MONTH_RE = re.compile(r"^[0-9]{6}$")
MAX_MEMBER_BYTES = 10 * 1024 * 1024

INPUTS = (
    {
        "input_id": "factors",
        "filename": "F-F_Research_Data_Factors_CSV.zip",
        "url": "https://mba.tuck.dartmouth.edu/pages/faculty/ken.french/ftp/F-F_Research_Data_Factors_CSV.zip",
        "sha256": "cd6d8e0d175b6f423862a6ad15a3073a6e4264b52b2ac9262396c79f707c6bcb",
        "member": "F-F_Research_Data_Factors.csv",
    },
    {
        "input_id": "portfolios",
        "filename": "25_Portfolios_5x5_CSV.zip",
        "url": "https://mba.tuck.dartmouth.edu/pages/faculty/ken.french/ftp/25_Portfolios_5x5_CSV.zip",
        "sha256": "43cfc360fca14e7d50766e8432fb8b6151c47078512efe74bd0f5d3804789a2a",
        "member": "25_Portfolios_5x5.csv",
    },
)


class AnalysisError(RuntimeError):
    """A stable, user-actionable analysis failure."""


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def canonical_json(value: Any) -> bytes:
    return (json.dumps(value, ensure_ascii=False, indent=2, sort_keys=True) + "\n").encode()


def csv_bytes(fieldnames: list[str], rows: list[dict[str, str]]) -> bytes:
    output = io.StringIO(newline="")
    writer = csv.DictWriter(output, fieldnames=fieldnames, lineterminator="\n")
    writer.writeheader()
    writer.writerows(rows)
    return output.getvalue().encode()


def format_float(value: float) -> str:
    if not np.isfinite(value):
        raise AnalysisError("analysis produced a non-finite result")
    return f"{value:.10f}"


def rounded(value: float) -> float:
    if not np.isfinite(value):
        raise AnalysisError("analysis produced a non-finite result")
    return round(float(value), 10)


def validate_month(value: str) -> int:
    candidate = value.strip()
    if not MONTH_RE.fullmatch(candidate):
        raise AnalysisError(f"invalid monthly date: {candidate!r}")
    numeric = int(candidate)
    month = numeric % 100
    if numeric // 100 < 1900 or not 1 <= month <= 12:
        raise AnalysisError(f"invalid monthly date: {candidate!r}")
    return numeric


def monthly_range(start: int, end: int) -> list[int]:
    values: list[int] = []
    current = start
    while current <= end:
        values.append(current)
        year, month = divmod(current, 100)
        current = (year + 1) * 100 + 1 if month == 12 else year * 100 + month + 1
    return values


def parse_numeric(value: str, *, source: str, month: int) -> float:
    try:
        number = float(value.strip())
    except ValueError as exc:
        raise AnalysisError(f"{source} has a non-numeric value at {month}") from exc
    if not np.isfinite(number) or number in MISSING_SENTINELS:
        raise AnalysisError(f"{source} has missing or non-finite data at {month}")
    return number


def ensure_input(spec: dict[str, str], *, allow_download: bool) -> bytes:
    RAW_DIR.mkdir(parents=True, exist_ok=True)
    path = RAW_DIR / spec["filename"]
    if not path.exists():
        if not allow_download:
            raise AnalysisError(f"missing cached input for --check: {spec['filename']}")
        request = urllib.request.Request(spec["url"], headers={"User-Agent": "qiongli-research/1"})
        try:
            with urllib.request.urlopen(request, timeout=60) as response:
                payload = response.read(MAX_MEMBER_BYTES)
        except OSError as exc:
            raise AnalysisError(f"failed to download {spec['input_id']}: {exc}") from exc
        if len(payload) >= MAX_MEMBER_BYTES:
            raise AnalysisError(f"download exceeds size bound: {spec['input_id']}")
        if sha256_bytes(payload) != spec["sha256"]:
            raise AnalysisError(f"download digest changed: {spec['input_id']}")
        atomic_write(path, payload)
    payload = path.read_bytes()
    actual = sha256_bytes(payload)
    if actual != spec["sha256"]:
        raise AnalysisError(
            f"input digest changed for {spec['input_id']}: expected {spec['sha256']}, got {actual}"
        )
    return payload


def read_zip_member(payload: bytes, spec: dict[str, str]) -> str:
    try:
        with zipfile.ZipFile(io.BytesIO(payload)) as archive:
            names = archive.namelist()
            if names != [spec["member"]]:
                raise AnalysisError(
                    f"{spec['input_id']} archive members changed: expected only {spec['member']}"
                )
            member_path = PurePosixPath(names[0])
            if member_path.is_absolute() or ".." in member_path.parts or len(member_path.parts) != 1:
                raise AnalysisError(f"unsafe ZIP member for {spec['input_id']}")
            info = archive.getinfo(names[0])
            if info.is_dir() or info.file_size > MAX_MEMBER_BYTES:
                raise AnalysisError(f"invalid ZIP member for {spec['input_id']}")
            member = archive.read(names[0])
    except zipfile.BadZipFile as exc:
        raise AnalysisError(f"invalid ZIP archive: {spec['input_id']}") from exc
    try:
        return member.decode("utf-8-sig")
    except UnicodeDecodeError as exc:
        raise AnalysisError(f"non-UTF-8 CSV: {spec['input_id']}") from exc


def parse_factor_monthly(text: str) -> dict[int, np.ndarray]:
    lines = text.replace("\r\n", "\n").replace("\r", "\n").split("\n")
    header_index: int | None = None
    for index, line in enumerate(lines):
        row = [cell.strip() for cell in next(csv.reader([line]))]
        if row == ["", "Mkt-RF", "SMB", "HML", "RF"]:
            header_index = index
            break
    if header_index is None:
        raise AnalysisError("factor monthly header is missing")
    rows: dict[int, np.ndarray] = {}
    for line in lines[header_index + 1 :]:
        cells = next(csv.reader([line]))
        if not cells or not MONTH_RE.fullmatch(cells[0].strip()):
            if rows:
                break
            continue
        if len(cells) != 5:
            raise AnalysisError("factor monthly row width changed")
        month = validate_month(cells[0])
        if month in rows:
            raise AnalysisError(f"duplicate factor month: {month}")
        rows[month] = np.array(
            [parse_numeric(value, source="factors", month=month) for value in cells[1:]],
            dtype=float,
        )
    if not rows:
        raise AnalysisError("factor monthly section is empty")
    return rows


def parse_portfolio_section(text: str, marker: str) -> tuple[list[str], dict[int, np.ndarray]]:
    lines = text.replace("\r\n", "\n").replace("\r", "\n").split("\n")
    matches = [index for index, line in enumerate(lines) if line.strip() == marker]
    if len(matches) != 1:
        raise AnalysisError(f"portfolio section count changed for {marker!r}")
    header_index = matches[0] + 1
    while header_index < len(lines) and not lines[header_index].strip():
        header_index += 1
    header = [cell.strip() for cell in next(csv.reader([lines[header_index]]))]
    names = header[1:]
    if len(header) != 26 or header[0] or len(set(names)) != 25 or any(not name for name in names):
        raise AnalysisError(f"portfolio header changed for {marker!r}")
    rows: dict[int, np.ndarray] = {}
    for line in lines[header_index + 1 :]:
        cells = next(csv.reader([line]))
        if not cells or not MONTH_RE.fullmatch(cells[0].strip()):
            if rows:
                break
            continue
        if len(cells) != 26:
            raise AnalysisError(f"portfolio monthly row width changed for {marker!r}")
        month = validate_month(cells[0])
        if month in rows:
            raise AnalysisError(f"duplicate portfolio month: {month}")
        rows[month] = np.array(
            [parse_numeric(value, source=marker, month=month) for value in cells[1:]],
            dtype=float,
        )
    if not rows:
        raise AnalysisError(f"portfolio monthly section is empty for {marker!r}")
    return names, rows


def validate_common_sample(
    factors: dict[int, np.ndarray],
    portfolio_sets: dict[str, tuple[list[str], dict[int, np.ndarray]]],
) -> list[int]:
    common = set(factors)
    for _, rows in portfolio_sets.values():
        common &= set(rows)
    months = sorted(month for month in common if month >= START_MONTH)
    if not months or months[0] != START_MONTH or len(months) < 600:
        raise AnalysisError("common sample does not satisfy the locked start and minimum length")
    if months != monthly_range(months[0], months[-1]):
        raise AnalysisError("common sample has a missing month")
    return months


def fit_models(
    factors: dict[int, np.ndarray],
    portfolio_sets: dict[str, tuple[list[str], dict[int, np.ndarray]]],
    months: list[int],
) -> tuple[list[dict[str, Any]], list[dict[str, Any]], dict[str, Any]]:
    factor_matrix = np.vstack([factors[month] for month in months]) / 100.0
    if factor_matrix.shape != (len(months), 4) or not np.isfinite(factor_matrix).all():
        raise AnalysisError("factor matrix invariant failed")
    model_rows: list[dict[str, Any]] = []
    diagnostic_rows: list[dict[str, Any]] = []
    model_specs = (("capm", [0]), ("ff3", [0, 1, 2]))
    for weighting, (portfolio_names, source_rows) in portfolio_sets.items():
        returns = np.vstack([source_rows[month] for month in months]) / 100.0
        if returns.shape != (len(months), 25) or not np.isfinite(returns).all():
            raise AnalysisError(f"portfolio matrix invariant failed: {weighting}")
        excess = returns - factor_matrix[:, [3]]
        for portfolio_index, portfolio in enumerate(portfolio_names):
            y = excess[:, portfolio_index]
            for model_name, factor_indices in model_specs:
                x = sm.add_constant(factor_matrix[:, factor_indices], has_constant="add")
                if np.linalg.matrix_rank(x) != x.shape[1]:
                    raise AnalysisError(f"rank-deficient design: {weighting}/{portfolio}/{model_name}")
                fit = sm.OLS(y, x).fit(
                    cov_type="HAC",
                    cov_kwds={"maxlags": HAC_LAGS, "use_correction": True},
                )
                params = np.asarray(fit.params, dtype=float)
                bse = np.asarray(fit.bse, dtype=float)
                tvalues = np.asarray(fit.tvalues, dtype=float)
                pvalues = np.asarray(fit.pvalues, dtype=float)
                residual = np.asarray(fit.resid, dtype=float)
                lb = acorr_ljungbox(residual, lags=[HAC_LAGS], return_df=True)
                bp_lm, bp_pvalue, _, _ = het_breuschpagan(residual, x)
                model_rows.append(
                    {
                        "weighting": weighting,
                        "portfolio": portfolio,
                        "model": model_name,
                        "start_month": months[0],
                        "end_month": months[-1],
                        "nobs": len(months),
                        "alpha_monthly": params[0],
                        "alpha_hac_se": bse[0],
                        "alpha_hac_t": tvalues[0],
                        "alpha_hac_pvalue": pvalues[0],
                        "beta_mkt": params[1],
                        "beta_smb": params[2] if model_name == "ff3" else None,
                        "beta_hml": params[3] if model_name == "ff3" else None,
                        "adjusted_r_squared": float(fit.rsquared_adj),
                    }
                )
                diagnostic_rows.append(
                    {
                        "weighting": weighting,
                        "portfolio": portfolio,
                        "model": model_name,
                        "durbin_watson": float(durbin_watson(residual)),
                        "ljung_box_lag": HAC_LAGS,
                        "ljung_box_stat": float(lb["lb_stat"].iloc[0]),
                        "ljung_box_pvalue": float(lb["lb_pvalue"].iloc[0]),
                        "breusch_pagan_lm": float(bp_lm),
                        "breusch_pagan_pvalue": float(bp_pvalue),
                    }
                )
    correlations = np.corrcoef(factor_matrix[:, :3], rowvar=False)
    factor_diagnostics = {
        "schema_version": 1,
        "sample": {"start_month": months[0], "end_month": months[-1], "nobs": len(months)},
        "factor_order": ["Mkt-RF", "SMB", "HML"],
        "correlation_matrix": [[rounded(value) for value in row] for row in correlations],
        "design_condition_numbers": {
            "capm": rounded(np.linalg.cond(sm.add_constant(factor_matrix[:, [0]], has_constant="add"))),
            "ff3": rounded(np.linalg.cond(sm.add_constant(factor_matrix[:, :3], has_constant="add"))),
        },
        "missing_sentinel_count": 0,
        "duplicate_month_count": 0,
    }
    return model_rows, diagnostic_rows, factor_diagnostics


def build_comparisons(model_rows: list[dict[str, Any]]) -> tuple[list[dict[str, Any]], list[dict[str, Any]]]:
    indexed = {(row["weighting"], row["portfolio"], row["model"]): row for row in model_rows}
    comparisons: list[dict[str, Any]] = []
    summaries: list[dict[str, Any]] = []
    for weighting in ("value_weighted", "equal_weighted"):
        capm_rows = [row for row in model_rows if row["weighting"] == weighting and row["model"] == "capm"]
        ff3_rows = [row for row in model_rows if row["weighting"] == weighting and row["model"] == "ff3"]
        if len(capm_rows) != 25 or len(ff3_rows) != 25:
            raise AnalysisError(f"model grid is incomplete for {weighting}")
        for capm in capm_rows:
            ff3 = indexed[(weighting, capm["portfolio"], "ff3")]
            denominator = abs(capm["alpha_monthly"])
            comparisons.append(
                {
                    "weighting": weighting,
                    "portfolio": capm["portfolio"],
                    "capm_alpha_monthly": capm["alpha_monthly"],
                    "ff3_alpha_monthly": ff3["alpha_monthly"],
                    "absolute_alpha_change": abs(ff3["alpha_monthly"]) - denominator,
                    "absolute_alpha_reduction_ratio": None
                    if denominator == 0
                    else 1.0 - abs(ff3["alpha_monthly"]) / denominator,
                    "adjusted_r_squared_change": ff3["adjusted_r_squared"] - capm["adjusted_r_squared"],
                }
            )
        capm_mean = float(np.mean([abs(row["alpha_monthly"]) for row in capm_rows]))
        ff3_mean = float(np.mean([abs(row["alpha_monthly"]) for row in ff3_rows]))
        capm_median = float(np.median([abs(row["alpha_monthly"]) for row in capm_rows]))
        ff3_median = float(np.median([abs(row["alpha_monthly"]) for row in ff3_rows]))
        if capm_mean == 0 or capm_median == 0:
            raise AnalysisError(f"CAPM alpha summary denominator is zero for {weighting}")
        for model_name, rows in (("capm", capm_rows), ("ff3", ff3_rows)):
            summaries.append(
                {
                    "weighting": weighting,
                    "model": model_name,
                    "portfolio_count": 25,
                    "mean_abs_alpha_monthly": float(np.mean([abs(row["alpha_monthly"]) for row in rows])),
                    "median_abs_alpha_monthly": float(np.median([abs(row["alpha_monthly"]) for row in rows])),
                    "alpha_abs_t_gt_1_96_count": int(
                        sum(bool(abs(row["alpha_hac_t"]) > 1.96) for row in rows)
                    ),
                    "mean_adjusted_r_squared": float(np.mean([row["adjusted_r_squared"] for row in rows])),
                    "mean_abs_alpha_reduction_vs_capm": None if model_name == "capm" else 1.0 - ff3_mean / capm_mean,
                    "median_abs_alpha_reduction_vs_capm": None
                    if model_name == "capm"
                    else 1.0 - ff3_median / capm_median,
                }
            )
    if len(model_rows) != 100 or len(comparisons) != 50 or len(summaries) != 4:
        raise AnalysisError("final model-grid counts are invalid")
    return comparisons, summaries


def serialize_rows(rows: list[dict[str, Any]], fields: list[str]) -> bytes:
    serialized: list[dict[str, str]] = []
    for row in rows:
        output: dict[str, str] = {}
        for field in fields:
            value = row[field]
            if value is None:
                output[field] = ""
            elif isinstance(value, float):
                output[field] = format_float(value)
            else:
                output[field] = str(value)
        serialized.append(output)
    return csv_bytes(fields, serialized)


def result_summary(
    summaries: list[dict[str, Any]],
    model_rows: list[dict[str, Any]],
    diagnostic_rows: list[dict[str, Any]],
    months: list[int],
) -> dict[str, Any]:
    keyed = {(row["weighting"], row["model"]): row for row in summaries}
    value_ff3 = keyed[("value_weighted", "ff3")]
    equal_ff3 = keyed[("equal_weighted", "ff3")]
    ff3_increases = int(
        sum(
            bool(
                abs(row["alpha_monthly"])
                > abs(
                    next(
                        item["alpha_monthly"]
                        for item in model_rows
                        if item["weighting"] == row["weighting"]
                        and item["portfolio"] == row["portfolio"]
                        and item["model"] == "capm"
                    )
                )
            )
            for row in model_rows
            if row["model"] == "ff3"
        )
    )
    return {
        "schema_version": 1,
        "research_question": "How much does FF3 attenuate CAPM pricing errors across the 25 U.S. size/book-to-market portfolios?",
        "sample": {"start_month": months[0], "end_month": months[-1], "nobs": len(months)},
        "model_configuration": {
            "models": ["capm", "ff3"],
            "covariance": "HAC",
            "maxlags": HAC_LAGS,
            "small_sample_correction": True,
            "primary_weighting": "value_weighted",
            "sensitivity_weighting": "equal_weighted",
        },
        "primary_result": {
            "value_weighted_mean_abs_alpha_reduction": rounded(value_ff3["mean_abs_alpha_reduction_vs_capm"]),
            "value_weighted_median_abs_alpha_reduction": rounded(value_ff3["median_abs_alpha_reduction_vs_capm"]),
            "value_weighted_ff3_abs_t_gt_1_96_count": value_ff3["alpha_abs_t_gt_1_96_count"],
        },
        "sensitivity_result": {
            "equal_weighted_mean_abs_alpha_reduction": rounded(equal_ff3["mean_abs_alpha_reduction_vs_capm"]),
            "equal_weighted_median_abs_alpha_reduction": rounded(equal_ff3["median_abs_alpha_reduction_vs_capm"]),
            "equal_weighted_ff3_abs_t_gt_1_96_count": equal_ff3["alpha_abs_t_gt_1_96_count"],
        },
        "portfolio_model_counts": {"model_rows": len(model_rows), "diagnostic_rows": len(diagnostic_rows)},
        "ff3_portfolios_with_increased_absolute_alpha": ff3_increases,
        "hypotheses": {
            "H1": "supported" if value_ff3["mean_abs_alpha_reduction_vs_capm"] > 0 else "not_supported",
            "H2": "supported" if value_ff3["alpha_abs_t_gt_1_96_count"] > 0 else "not_supported",
            "H3": "supported" if value_ff3["mean_abs_alpha_reduction_vs_capm"] > 0 and equal_ff3["mean_abs_alpha_reduction_vs_capm"] > 0 else "not_supported",
        },
        "claim_boundary": "Descriptive in-sample benchmark comparison; no causal, universal model-validity, or investment claim.",
    }


def results_markdown(summary: dict[str, Any], summaries: list[dict[str, Any]]) -> bytes:
    keyed = {(row["weighting"], row["model"]): row for row in summaries}
    value_capm = keyed[("value_weighted", "capm")]
    value_ff3 = keyed[("value_weighted", "ff3")]
    equal_ff3 = keyed[("equal_weighted", "ff3")]
    primary = summary["primary_result"]
    sensitivity = summary["sensitivity_result"]
    text = f"""# Analysis Results

## Finding

The common sample contains {summary['sample']['nobs']} monthly observations from {summary['sample']['start_month']} through {summary['sample']['end_month']}. Across the 25 value-weighted portfolios, mean absolute monthly alpha is {value_capm['mean_abs_alpha_monthly'] * 100:.4f} percentage points under CAPM and {value_ff3['mean_abs_alpha_monthly'] * 100:.4f} under FF3. The corresponding mean attenuation is {primary['value_weighted_mean_abs_alpha_reduction'] * 100:.2f}%. Median attenuation is {primary['value_weighted_median_abs_alpha_reduction'] * 100:.2f}%.

Under equal weighting, mean absolute-alpha attenuation is {sensitivity['equal_weighted_mean_abs_alpha_reduction'] * 100:.2f}% and median attenuation is {sensitivity['equal_weighted_median_abs_alpha_reduction'] * 100:.2f}%. FF3 leaves {primary['value_weighted_ff3_abs_t_gt_1_96_count']} value-weighted and {sensitivity['equal_weighted_ff3_abs_t_gt_1_96_count']} equal-weighted portfolio intercepts with descriptive absolute HAC t-statistics above 1.96.

## Interpretation

Within this pinned sample, the result supports H1 and H3 only when their status is recorded as `supported` in `analysis_summary.json`. Lower alpha and higher fit are consistent with SMB and HML absorbing return variation omitted by a market-only benchmark. They do not establish that FF3 is a true structural model or that the factor construction is independent of the size/book-to-market test assets.

## Diagnostic And Rival Boundary

Portfolio-level estimates and paired changes are in `model_results.csv` and `model_comparison.csv`; factor design checks are in `factor_diagnostics.json`; residual dependence and heteroskedasticity screens are in `residual_diagnostics.csv`. The threshold counts are descriptive and are not a family-wise or joint pricing-model test.

## Limitations

- The factors and test assets share size/book-to-market construction.
- HAC with six lags addresses within-series covariance but is not a cross-portfolio joint test.
- Results apply to the selected U.S. portfolio grid, monthly frequency, time window, and exact archived vintage.
- Historical source revisions can change results; digest drift therefore fails closed.
"""
    return text.encode()


def build_outputs(
    model_rows: list[dict[str, Any]],
    diagnostic_rows: list[dict[str, Any]],
    comparisons: list[dict[str, Any]],
    summaries: list[dict[str, Any]],
    factor_diagnostics: dict[str, Any],
    summary: dict[str, Any],
    provenance: dict[str, Any],
) -> dict[Path, bytes]:
    model_fields = [
        "weighting", "portfolio", "model", "start_month", "end_month", "nobs",
        "alpha_monthly", "alpha_hac_se", "alpha_hac_t", "alpha_hac_pvalue",
        "beta_mkt", "beta_smb", "beta_hml", "adjusted_r_squared",
    ]
    comparison_fields = [
        "weighting", "portfolio", "capm_alpha_monthly", "ff3_alpha_monthly",
        "absolute_alpha_change", "absolute_alpha_reduction_ratio", "adjusted_r_squared_change",
    ]
    summary_fields = [
        "weighting", "model", "portfolio_count", "mean_abs_alpha_monthly",
        "median_abs_alpha_monthly", "alpha_abs_t_gt_1_96_count", "mean_adjusted_r_squared",
        "mean_abs_alpha_reduction_vs_capm", "median_abs_alpha_reduction_vs_capm",
    ]
    diagnostic_fields = [
        "weighting", "portfolio", "model", "durbin_watson", "ljung_box_lag",
        "ljung_box_stat", "ljung_box_pvalue", "breusch_pagan_lm", "breusch_pagan_pvalue",
    ]
    outputs = {
        RESULTS_DIR / "model_results.csv": serialize_rows(model_rows, model_fields),
        RESULTS_DIR / "model_comparison.csv": serialize_rows(comparisons, comparison_fields),
        RESULTS_DIR / "model_summary.csv": serialize_rows(summaries, summary_fields),
        RESULTS_DIR / "residual_diagnostics.csv": serialize_rows(diagnostic_rows, diagnostic_fields),
        RESULTS_DIR / "factor_diagnostics.json": canonical_json(factor_diagnostics),
        RESULTS_DIR / "analysis_summary.json": canonical_json(summary),
        RESULTS_DIR / "results.md": results_markdown(summary, summaries),
        ANALYSIS_DIR / "provenance.json": canonical_json(provenance),
    }
    digest_payload = {
        "schema_version": 1,
        "outputs": {
            path.relative_to(ANALYSIS_DIR).as_posix(): sha256_bytes(payload)
            for path, payload in sorted(outputs.items(), key=lambda item: item[0].as_posix())
        },
    }
    outputs[RESULTS_DIR / "output_digests.json"] = canonical_json(digest_payload)
    return outputs


def atomic_write(path: Path, payload: bytes) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_name(f".{path.name}.tmp")
    temporary.write_bytes(payload)
    os.replace(temporary, path)


def write_or_check(outputs: dict[Path, bytes], *, check: bool) -> None:
    if check:
        for path, expected in outputs.items():
            if not path.is_file():
                raise AnalysisError(f"deterministic output is missing: {path.relative_to(ANALYSIS_DIR)}")
            if path.read_bytes() != expected:
                raise AnalysisError(f"deterministic output changed: {path.relative_to(ANALYSIS_DIR)}")
        return
    for path, payload in outputs.items():
        atomic_write(path, payload)


def execute(*, check: bool) -> None:
    payloads = {spec["input_id"]: ensure_input(spec, allow_download=not check) for spec in INPUTS}
    texts = {
        spec["input_id"]: read_zip_member(payloads[spec["input_id"]], spec)
        for spec in INPUTS
    }
    factors = parse_factor_monthly(texts["factors"])
    portfolio_names_value, portfolios_value = parse_portfolio_section(
        texts["portfolios"], "Average Value Weighted Returns -- Monthly"
    )
    portfolio_names_equal, portfolios_equal = parse_portfolio_section(
        texts["portfolios"], "Average Equal Weighted Returns -- Monthly"
    )
    if portfolio_names_value != portfolio_names_equal:
        raise AnalysisError("value- and equal-weighted portfolio names differ")
    portfolio_sets = {
        "value_weighted": (portfolio_names_value, portfolios_value),
        "equal_weighted": (portfolio_names_equal, portfolios_equal),
    }
    months = validate_common_sample(factors, portfolio_sets)
    model_rows, diagnostic_rows, factor_diagnostics = fit_models(factors, portfolio_sets, months)
    comparisons, summaries = build_comparisons(model_rows)
    summary = result_summary(summaries, model_rows, diagnostic_rows, months)
    provenance = {
        "schema_version": 1,
        "pin_reviewed_on": PIN_REVIEWED_ON,
        "publisher": "Kenneth French Data Library",
        "raw_inputs_committed": False,
        "inputs": [
            {
                "input_id": spec["input_id"],
                "filename": spec["filename"],
                "official_url": spec["url"],
                "sha256": spec["sha256"],
                "bytes": len(payloads[spec["input_id"]]),
                "expected_member": spec["member"],
            }
            for spec in INPUTS
        ],
    }
    outputs = build_outputs(
        model_rows,
        diagnostic_rows,
        comparisons,
        summaries,
        factor_diagnostics,
        summary,
        provenance,
    )
    write_or_check(outputs, check=check)
    verb = "verified" if check else "wrote"
    print(f"{verb} {len(model_rows)} model rows over {len(months)} months ({months[0]}-{months[-1]})")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="recompute and compare without writing")
    parser.add_argument("--profile", action="store_true", help="print a bounded cProfile summary")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    try:
        if args.profile:
            profiler = cProfile.Profile()
            profiler.enable()
            execute(check=args.check)
            profiler.disable()
            pstats.Stats(profiler, stream=sys.stdout).strip_dirs().sort_stats("cumulative").print_stats(15)
        else:
            execute(check=args.check)
    except (AnalysisError, OSError, ValueError) as exc:
        print(f"analysis failed: {exc}", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
