# FastAPI Integration

Expose ArqonHPO as a REST API service using FastAPI.

---

## Basic Setup

```python
from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
from arqonhpo import ArqonSolver
import json
from typing import Optional

app = FastAPI(title="ArqonHPO Service")

# Global solver state
solver: Optional[ArqonSolver] = None

class ConfigRequest(BaseModel):
    seed: int = 42
    budget: int = 100
    bounds: dict

class TellRequest(BaseModel):
    params: dict
    value: float
    cost: float = 1.0

@app.post("/init")
def init_solver(config: ConfigRequest):
    global solver
    config_dict = {
        "seed": config.seed,
        "budget": config.budget,
        "bounds": config.bounds
    }
    solver = ArqonSolver(json.dumps(config_dict))
    return {"status": "initialized", "budget": config.budget}

@app.get("/ask")
def ask():
    if solver is None:
        raise HTTPException(status_code=400, detail="Solver not initialized")
    
    candidates = solver.ask()
    if candidates is None:
        return {"done": True, "candidates": []}
    return {"done": False, "candidates": candidates}

@app.post("/tell")
def tell(results: list[TellRequest]):
    if solver is None:
        raise HTTPException(status_code=400, detail="Solver not initialized")
    
    results_list = [
        {"params": r.params, "value": r.value, "cost": r.cost}
        for r in results
    ]
    solver.tell(json.dumps(results_list))
    return {"status": "ok", "history_len": solver.get_history_len()}

@app.get("/status")
def status():
    if solver is None:
        return {"initialized": False}
    return {
        "initialized": True,
        "history_len": solver.get_history_len()
    }
```

---

## Run the Service

```bash
pip install fastapi uvicorn arqonhpo
uvicorn main:app --host 0.0.0.0 --port 8000
```

---

## Client Usage

```python
import requests

# Initialize
resp = requests.post("http://localhost:8000/init", json={
    "seed": 42,
    "budget": 100,
    "bounds": {"x": {"min": -5, "max": 5}}
})

# Optimization loop
while True:
    # Ask for candidates
    resp = requests.get("http://localhost:8000/ask")
    data = resp.json()
    
    if data["done"]:
        break
    
    # Evaluate candidates
    results = []
    for params in data["candidates"]:
        value = evaluate(params)  # Your function
        results.append({"params": params, "value": value, "cost": 1.0})
    
    # Tell results
    requests.post("http://localhost:8000/tell", json=results)
```

---

## With Background Tasks

For long-running evaluations:

```python
from fastapi import BackgroundTasks
import asyncio

pending_evals = {}

@app.post("/start-eval/{eval_id}")
async def start_eval(eval_id: str, params: dict, background_tasks: BackgroundTasks):
    background_tasks.add_task(run_evaluation, eval_id, params)
    return {"status": "started", "eval_id": eval_id}

async def run_evaluation(eval_id: str, params: dict):
    # Long-running evaluation
    value = await expensive_evaluation(params)
    pending_evals[eval_id] = {"params": params, "value": value}

@app.get("/result/{eval_id}")
def get_result(eval_id: str):
    if eval_id not in pending_evals:
        raise HTTPException(status_code=404, detail="Eval not found")
    return pending_evals.pop(eval_id)
```

---

## Docker Deployment

```dockerfile
FROM python:3.11-slim

RUN pip install fastapi uvicorn arqonhpo

COPY main.py .

CMD ["uvicorn", "main:app", "--host", "0.0.0.0", "--port", "8000"]
```

```bash
docker build -t arqon-api .
docker run -p 8000:8000 arqon-api
```

---

## OpenAPI Docs

FastAPI auto-generates OpenAPI docs at:
- Swagger UI: `http://localhost:8000/docs`
- ReDoc: `http://localhost:8000/redoc`

---

## Next Steps

- [Python API](../reference/python.md) — ArqonSolver reference
- [Kubernetes](kubernetes.md) — Deploy to K8s
- [Observability](../reference/observability.md) — Add metrics
