RPZL Validation Package
=======================

This package contains all data, artifacts, and notebook files for reproducible validation of the Recursive Prime Zoom & Lift (RPZL) methodology.

Contents:
1. data/
   - Various JSON files recording results from each phase of experiments.
   - rpzl_spec.json: Unified experiment specification.
   - rpzl_archive.tar.gz: Compressed archive of all data and notebook.
2. RPZL_Validation_Notebook.ipynb
   - The master Jupyter notebook executing all phases.

Usage:
- Extract rpzl_archive.tar.gz to reproduce experiment directories.
- Open RPZL_Validation_Notebook.ipynb in VS Code (Jupyter extension).
- Ensure Python 3.10+, install dependencies: numpy, scipy, sympy, sklearn, tqdm, torchvision, matplotlib.
- Run cells in order; all results will match those in the JSON artifacts.

All experiments use random seed = 42 for deterministic behavior, no global variables, and store intermediate state on disk in 'data/'.

