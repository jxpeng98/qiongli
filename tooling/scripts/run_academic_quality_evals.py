#!/usr/bin/env python3
from __future__ import annotations

import sys
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from evals.runner.run_suite import EvalRunResult, main, run_evals  # noqa: E402, F401


if __name__ == "__main__":
    raise SystemExit(main())
