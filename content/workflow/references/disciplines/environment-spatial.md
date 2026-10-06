# Environment, ecology and spatial research

Use for ecology, environmental science, geography, agriculture, conservation,
remote sensing and spatial policy studies. Choose the spatial/temporal unit and
observation process before choosing a model.

## Question and sources — A/B

- Define the process, organism/system, spatial extent, time scale and intended
  inference. A site-level association may not describe a region or predict a
  future climate. Distinguish ecological process from how it was observed.
- For field observations, inventories, satellite products and administrative
  maps, record provider/product/version, coordinate system, resolution, dates,
  units, detection limits and sampling footprint. Document access and licenses.
- Search local names, taxonomic synonyms, habitat/process terms and spatial scale.
  Compare studies only after checking seasonal coverage and measurement methods.

## Common methods — C/I

| Work | Decisions and diagnostics | Risk to inference |
|---|---|---|
| Field / agricultural experiment | Treatment unit, plots/blocks, randomization, repeated measures, season and site replication | Counting subsamples as independent treatments; site or season confounded with treatment |
| Species occurrence / occupancy | Sampling effort, repeat visits, detectability, presence/background or absence definition | Nondetection treated as absence; convenience records treated as a random sample |
| Spatial regression / mapping | Coordinate reference system, support, spatial weights, dependence and boundary definition | Ecological fallacy, scale/aggregation effects and residual spatial dependence |
| Remote sensing / classification | Sensor/product, cloud/missing-data treatment, reference labels, spatial/time split | Adjacent pixels from the same site leaking across train/test; resampling creating apparent precision |
| Environmental time series | Time alignment, seasonality, lag/exposure window, trends and autocorrelation | Searching many lags and reporting only significance; ignoring shared trends |
| Simulation / scenarios | Initial/boundary conditions, parameter calibration, independent validation and sensitivity | Describing a conditional scenario as a forecast or calibrating and validating on the same data |
| Environmental policy / impact | Comparator, before/after coverage, implementation, spillovers and co-occurring events | Calling a before/after change causal without a supported counterfactual |

Account for clustered/spatial/temporal dependence in uncertainty and validation.
Select blocking distances and holdouts from the intended generalization task;
random splits are not automatically adequate for spatial prediction.

## Common content to organize

Keep a sample/site/date table linked to the raw source; distinguish missing,
below-detection-limit, nondetection and structural zero. Preserve coordinate
transformations, resampling, quality flags and exclusion rules in the pipeline.
Map layers need compatible units, projections and temporal support before joins.
Do not silently impute unavailable measurements or claim pixel-level precision
from a coarse underlying product.

For field instruments, carry calibration, maintenance, drift and measurement
uncertainty into results. The [NIST measurement-process guide](https://www.nist.gov/publications/nistsematech-engineering-statistics-handbook-chapter-2-measurement-process)
provides a starting point; instrument/product documentation owns its actual
calibration and quality rules.

## Ethics, interpretation and delivery — D/E/F/G/H

Respect field access, collection permits, community knowledge restrictions and
sensitive-species locations. Public maps can expose protected places or households;
make disclosure choices from actual permissions, not a presumed open-data norm.

Report uncertainty and coverage on maps and trend estimates. Separate observed
change, modeled attribution and scenario projection. Carry unobserved regions,
seasonal gaps and extrapolation into the discussion and policy implications.
Use the existing ecology-environmental profile for applicable method checks and
the dataset's own technical documentation for product-specific decisions.
