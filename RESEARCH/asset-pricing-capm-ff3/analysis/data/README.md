# Analysis Inputs

Raw Kenneth French ZIP/CSV inputs belong under `raw/`, which is ignored by Git and must not be committed. `run_analysis.py` downloads missing archives only from the two URLs pinned in `design/dataset_plan.md`, verifies their SHA-256 digests and exact ZIP member names, and reads them without extracting files to the repository.
