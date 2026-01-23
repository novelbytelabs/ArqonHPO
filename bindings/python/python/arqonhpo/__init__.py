from ._internal import ArqonSolver, ArqonProbe
try:
    from importlib import metadata as _metadata

    __version__ = _metadata.version("arqonhpo")
except Exception:  # pragma: no cover - fallback for editable/dev installs
    __version__ = "unknown"

from .cli import main as cli_main

__all__ = ["ArqonSolver", "ArqonProbe", "cli_main", "__version__"]
