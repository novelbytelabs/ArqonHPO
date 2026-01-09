#!/usr/bin/env bash
python - <<'PY'
import os

x = float(os.environ["ARQON_x"])
y = float(os.environ["ARQON_y"])
value = (x - 2) ** 2 + (y + 1) ** 2
print(f"RESULT={value}")
PY
