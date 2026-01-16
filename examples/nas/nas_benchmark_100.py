#!/usr/bin/env python3
"""
NAS Benchmark: ArqonHPO vs Random Search (v5 - Audit Hardened)
==============================================================
Compares ArqonHPO-guided NAS against Random Search baseline.

Audit Hardening (v5):
- State Integrity: Duplicates are now explicitly `tell()`-ed as pruned to maintain solver state contract.
- Budget Enforcement: Strict check that both methods reach NAS_BUDGET_UNIQUE.
- Evaluation Safety: Evaluating directly from discretized config avoids double-discretization risk.
- Explicit Reproducibility: DataLoader num_workers=0.
- Cleanup: Unused imports removed.

Audit fixes (v4 preserved):
- Budget defined as UNIQUE architectures
- Strict validation isolation (fork_rng + NO numpy seed mutation)
- Dropout removed from search space (proxy blind to it)
- Deterministic scoring per (seed, discretized config)
"""

import json
import time
import os
import sys
import hashlib

os.environ["CUDA_VISIBLE_DEVICES"] = ""
os.environ["OMP_NUM_THREADS"] = "1"

import numpy as np
import torch
import torch.nn as nn
from torch.utils.data import DataLoader
from torchvision import datasets, transforms

torch.set_num_threads(1)

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
from arqonhpo import ArqonSolver

# =============================================================================
# CONFIGURATION
# =============================================================================
# Budget is now UNIQUE architectures found
NAS_BUDGET_UNIQUE = 100
MAX_ATTEMPTS_FACTOR = 20  # Increased to allow finding uniques deep in search
SEEDS = [42, 123, 456]
VALIDATION_EPOCHS = 5

# Search space bounds (INCLUSIVE for integers)
BOUNDS = {
    "num_layers": (2, 5),
    "channels_0": (16, 64),
    "channels_1": (32, 128),
    "channels_2": (64, 256),
    "kernel_sizes": [3, 5, 7],  # Categorical
    "use_batchnorm": (0, 1),    # Boolean
}
VALIDATION_DROPOUT = 0.25 


# =============================================================================
# DISCRETIZATION & SEEDING
# =============================================================================
def discretize_config(cfg):
    """Convert continuous config to discrete architecture. Returns hashable dict."""
    nl = int(round(cfg["num_layers"]))
    nl = max(BOUNDS["num_layers"][0], min(BOUNDS["num_layers"][1], nl))
    
    c0 = int(round(cfg["channels_0"]))
    c0 = max(BOUNDS["channels_0"][0], min(BOUNDS["channels_0"][1], c0))
    
    c1 = int(round(cfg["channels_1"]))
    c1 = max(BOUNDS["channels_1"][0], min(BOUNDS["channels_1"][1], c1))
    
    c2 = int(round(cfg["channels_2"]))
    c2 = max(BOUNDS["channels_2"][0], min(BOUNDS["channels_2"][1], c2))
    
    k = float(cfg["kernel_size"])
    k = 3 if k < 4 else (5 if k < 6 else 7)
    
    bn = 1 if float(cfg["use_batchnorm"]) > 0.5 else 0
    
    return {
        "num_layers": nl,
        "channels_0": c0,
        "channels_1": c1,
        "channels_2": c2,
        "kernel_size": k,
        "use_batchnorm": bn,
    }


def config_seed(base_seed, disc_cfg):
    """Generate deterministic seed for a specific (base_seed, config) pair."""
    s = json.dumps(disc_cfg, sort_keys=True).encode()
    h = hashlib.blake2b(s, digest_size=8).digest()
    return (int(base_seed) ^ int.from_bytes(h, "little")) & 0x7fffffff


# =============================================================================
# DYNAMIC CNN
# =============================================================================
class DynamicCNN(nn.Module):
    def __init__(self, num_layers, channels, kernel_size, use_batchnorm, dropout_rate):
        super().__init__()
        layers = []
        in_ch = 1
        current_size = 28
        
        for i in range(num_layers):
            out_ch = channels[min(i, len(channels) - 1)]
            layers.append(nn.Conv2d(in_ch, out_ch, kernel_size, padding=kernel_size // 2))
            if use_batchnorm:
                layers.append(nn.BatchNorm2d(out_ch))
            layers.append(nn.ReLU())
            
            if current_size >= 4:
                layers.append(nn.MaxPool2d(2))
                current_size //= 2
            
            in_ch = out_ch
        
        self.features = nn.Sequential(*layers)
        size = max(1, current_size)
        self.classifier = nn.Sequential(
            nn.Flatten(),
            nn.Dropout(dropout_rate),
            nn.Linear(in_ch * size * size, 128),
            nn.ReLU(),
            nn.Dropout(dropout_rate),
            nn.Linear(128, 10),
        )

    def forward(self, x):
        return self.classifier(self.features(x))


def build_model(disc_cfg):
    """Build model from DISCRETIZED config."""
    return DynamicCNN(
        disc_cfg["num_layers"],
        [disc_cfg["channels_0"], disc_cfg["channels_1"], disc_cfg["channels_2"]],
        disc_cfg["kernel_size"],
        disc_cfg["use_batchnorm"] == 1,
        VALIDATION_DROPOUT,
    )


# =============================================================================
# SYNFLOW SCORE
# =============================================================================
def synflow_score(model):
    """Compute SynFlow score (abs().sum())."""
    model.eval()
    for p in model.parameters():
        if p.grad is not None:
            p.grad.zero_()
    
    x = torch.ones(1, 1, 28, 28, requires_grad=False)
    y = model(x).sum()
    y.backward()
    
    total = 0.0
    for p in model.parameters():
        if p.grad is not None:
            total += (p.grad * p).abs().sum().item()
    
    return float(total)


def evaluate_disc_config(disc_cfg, base_seed):
    """
    Evaluate DISCRETIZED config directly - Patch B (Removes double-discretize risk).
    """
    eval_seed = config_seed(base_seed, disc_cfg)
    
    # Strict RNG isolation
    with torch.random.fork_rng(devices=[]):
        torch.manual_seed(eval_seed)
        model = build_model(disc_cfg)
        score = synflow_score(model)
    
    return score


# =============================================================================
# RANDOM SEARCH
# =============================================================================
def sample_random_config(rng):
    """Sample a random config using proper discrete sampling."""
    return {
        "num_layers": float(rng.integers(BOUNDS["num_layers"][0], BOUNDS["num_layers"][1] + 1)),
        "channels_0": float(rng.integers(BOUNDS["channels_0"][0], BOUNDS["channels_0"][1] + 1)),
        "channels_1": float(rng.integers(BOUNDS["channels_1"][0], BOUNDS["channels_1"][1] + 1)),
        "channels_2": float(rng.integers(BOUNDS["channels_2"][0], BOUNDS["channels_2"][1] + 1)),
        "kernel_size": float(rng.choice(BOUNDS["kernel_sizes"])),
        "use_batchnorm": float(rng.integers(0, 2)),
    }


def run_random(seed):
    """Random Search baseline with unique budget."""
    rng = np.random.default_rng(seed)
    scores = []
    cache = {}
    best_cfg, best_score = None, -float("inf")
    start = time.time()
    
    max_asks = NAS_BUDGET_UNIQUE * MAX_ATTEMPTS_FACTOR
    unique_evals = 0
    total_asks = 0
    
    for _ in range(max_asks):
        if unique_evals >= NAS_BUDGET_UNIQUE:
            break
            
        config = sample_random_config(rng)
        disc_cfg = discretize_config(config)
        cache_key = tuple(sorted(disc_cfg.items()))
        total_asks += 1
        
        if cache_key in cache:
            # Duplicate for Random is free, just retry
            continue
        
        # Patch B: Evaluate from discretized config directly
        score = evaluate_disc_config(disc_cfg, seed)
        cache[cache_key] = score
        scores.append(score)
        unique_evals += 1
        
        if score > best_score:
            best_score, best_cfg = score, disc_cfg
    
    elapsed = time.time() - start
    return {
        "best": best_score if scores else float("nan"),
        "mean": float(np.mean(scores)) if scores else float("nan"),
        "time": elapsed,
        "scores": scores,
        "best_config": best_cfg,
        "origin_seed": seed,
        "unique_evals": unique_evals,
        "total_asks": total_asks,
        "duplicate_rate": 1.0 - (unique_evals / max(total_asks, 1))
    }


# =============================================================================
# ARQON RUNNER
# =============================================================================
def run_arqon(seed):
    """ArqonHPO-guided NAS with unique budget and correct solver semantics."""
    solver_config = {
        "seed": seed,
        "budget": NAS_BUDGET_UNIQUE * MAX_ATTEMPTS_FACTOR,
        "bounds": {
            "num_layers": {"min": float(BOUNDS["num_layers"][0]), "max": float(BOUNDS["num_layers"][1]), "scale": "Linear"},
            "channels_0": {"min": float(BOUNDS["channels_0"][0]), "max": float(BOUNDS["channels_0"][1]), "scale": "Linear"},
            "channels_1": {"min": float(BOUNDS["channels_1"][0]), "max": float(BOUNDS["channels_1"][1]), "scale": "Linear"},
            "channels_2": {"min": float(BOUNDS["channels_2"][0]), "max": float(BOUNDS["channels_2"][1]), "scale": "Linear"},
            "kernel_size": {"min": 3.0, "max": 7.0, "scale": "Linear"},
            "use_batchnorm": {"min": 0.0, "max": 1.0, "scale": "Linear"},
        },
    }
    solver = ArqonSolver(json.dumps(solver_config))
    
    scores = []
    cache = {}
    best_cfg, best_score = None, -float("inf")
    start = time.time()
    
    unique_evals = 0
    total_asks = 0
    max_asks = NAS_BUDGET_UNIQUE * MAX_ATTEMPTS_FACTOR
    
    for i in range(max_asks):
        if unique_evals >= NAS_BUDGET_UNIQUE:
            break
            
        candidates = solver.ask()
        if not candidates:
            break
            
        config = candidates[0]
        disc_cfg = discretize_config(config)
        cache_key = tuple(sorted(disc_cfg.items()))
        total_asks += 1
        
        if cache_key in cache:
            # Patch A: Tell duplicate as PRUNED to keep solver state consistent
            dup_score = cache[cache_key]
            solver.tell(json.dumps([{
                "eval_id": total_asks,
                "params": disc_cfg,
                "value": -dup_score,
                "cost": 0.0, # Zero cost to avoid budget consumption
                "pruned": True,
            }]))
            continue
        
        # Patch B: Evaluate from discretized config directly
        score = evaluate_disc_config(disc_cfg, seed)
        
        cache[cache_key] = score
        scores.append(score)
        unique_evals += 1
        
        if score > best_score:
            best_score, best_cfg = score, disc_cfg
        
        # Tell new unique result
        solver.tell(json.dumps([{
            "eval_id": total_asks,
            "params": disc_cfg,
            "value": -score,
            "cost": 1.0,
            "pruned": False,
        }]))
    
    elapsed = time.time() - start
    return {
        "best": best_score if scores else float("nan"),
        "mean": float(np.mean(scores)) if scores else float("nan"),
        "time": elapsed,
        "scores": scores,
        "best_config": best_cfg,
        "origin_seed": seed,
        "unique_evals": unique_evals,
        "total_asks": total_asks,
        "duplicate_rate": 1.0 - (unique_evals / max(total_asks, 1))
    }


# =============================================================================
# VALIDATION
# =============================================================================
def validate_best(disc_cfg, seed):
    """Train best architecture on MNIST (STRICT ISOLATION)."""
    with torch.random.fork_rng(devices=[]):
        torch.manual_seed(seed)
        
        transform = transforms.Compose([transforms.ToTensor(), transforms.Normalize((0.1307,), (0.3081,))])
        
        # Patch D: Explicit num_workers=0
        train_loader = DataLoader(datasets.MNIST('./data', train=True, download=True, transform=transform), 
                                batch_size=64, shuffle=True, num_workers=0)
        test_loader = DataLoader(datasets.MNIST('./data', train=False, transform=transform), 
                                batch_size=1000, num_workers=0)
        
        model = build_model(disc_cfg)
        opt = torch.optim.Adam(model.parameters(), lr=0.001)
        criterion = nn.CrossEntropyLoss()
        
        model.train()
        for _ in range(VALIDATION_EPOCHS):
            for data, target in train_loader:
                opt.zero_grad()
                criterion(model(data), target).backward()
                opt.step()
        
        model.eval()
        correct = sum(model(d).argmax(1).eq(t).sum().item() for d, t in test_loader)
        return 100 * correct / 10000


# =============================================================================
# MAIN
# =============================================================================
def main():
    print("=" * 70)
    print("🏁  NAS BENCHMARK v6 (FINAL AUDIT GRADE): ArqonHPO vs Random")
    print("=" * 70)
    print(f"Unique Budget: {NAS_BUDGET_UNIQUE} archs | Seeds: {len(SEEDS)}")
    print("Fixes: Paired Seeds, Zero-Cost Duplicates, Safe Disc Eval")
    print("-" * 70)
    
    paired_results = {"ArqonHPO": [], "Random": []}
    
    for seed in SEEDS:
        print(f"\n>>> Seed {seed}: Running Pair...")
        
        # Run Arqon
        print("  Running ArqonHPO...", end=" ", flush=True)
        t0 = time.time()
        ra = run_arqon(seed)
        t_arq = time.time() - t0
        print(f"Done. Uniques={ra['unique_evals']}, Dups={ra['duplicate_rate']:.1%}, Best={ra['best']:.4f}")
        
        # Run Random
        print("  Running Random...  ", end=" ", flush=True)
        t0 = time.time()
        rr = run_random(seed)
        t_rnd = time.time() - t0
        print(f"Done. Uniques={rr['unique_evals']}, Dups={rr['duplicate_rate']:.1%}, Best={rr['best']:.4f}")

        # Patch C: Enforce PAIRED Success
        if ra["unique_evals"] < NAS_BUDGET_UNIQUE or rr["unique_evals"] < NAS_BUDGET_UNIQUE:
             print(f"⚠️  Seed {seed} EXCLUDED: Budget failure (Arqon: {ra['unique_evals']}, Random: {rr['unique_evals']})")
             continue
             
        paired_results["ArqonHPO"].append(ra)
        paired_results["Random"].append(rr)
        print(f"✅ Seed {seed} PAIR INCLUDED.")

    results = paired_results

    # Summary
    print("\n" + "=" * 70)
    print("📊  RESULTS SUMMARY (Paired Only)")
    print("=" * 70)
    
    # Check for empty results due to budget failures
    if not results["ArqonHPO"]:
        print("❌ CRITICAL: No valid paired runs completed.")
        return

    print(f"{'Method':<12} | {'Best (avg)':<12} | {'Mean (avg)':<12} | {'Dup Rate':<8} | {'Time (avg)':<12}")
    print("-" * 70)
    
    for name in ["ArqonHPO", "Random"]:
        bests = [r["best"] for r in results[name]]
        means = [r["mean"] for r in results[name]]
        times = [r["time"] for r in results[name]]
        dups = [r["duplicate_rate"] for r in results[name]]
        
        print(f"{name:<12} | {np.nanmean(bests):<12.4f} | {np.nanmean(means):<12.4f} | {np.mean(dups):<8.1%} | {np.mean(times):<12.2f}s")
    
    arqon_best = np.nanmean([r["best"] for r in results["ArqonHPO"]])
    random_best = np.nanmean([r["best"] for r in results["Random"]])
    
    print("-" * 70)
    denom = max(abs(random_best), abs(arqon_best), 1e-12)
    
    if not np.isfinite(arqon_best) or not np.isfinite(random_best):
        print("⚠️  Warning: Inconclusive result due to NaN/Inf scores.")
    elif arqon_best > random_best:
        improvement = (arqon_best - random_best) / denom * 100
        print(f"🏆 ArqonHPO wins! +{improvement:.1f}% better than Random Search")
    else:
        improvement = (random_best - arqon_best) / denom * 100
        print(f"❌ Random Search wins by {improvement:.1f}%")
    
    print("\n>>> Validating ArqonHPO's best architecture on MNIST...")
    valid_runs = [r for r in results["ArqonHPO"] if np.isfinite(r["best"])]
    if valid_runs:
        best_arqon_result = max(valid_runs, key=lambda x: x["best"])
        accuracy = validate_best(best_arqon_result["best_config"], best_arqon_result["origin_seed"])
        print(f"🎯  Trained Accuracy: {accuracy:.2f}% (Seed {best_arqon_result['origin_seed']})")
        print(f"    Architecture: {best_arqon_result['best_config']}")
    else:
        accuracy = 0.0
        best_arqon_result = {"best_config": {}}
        print("⚠️  No valid ArqonHPO runs to validate.")

    with open("nas_benchmark_results_v6.json", "w") as f:
        json.dump({
            "ArqonHPO": {
                "avg_best": float(arqon_best), 
                "avg_mean": float(np.nanmean([r["mean"] for r in results["ArqonHPO"]])),
                "dup_rate": float(np.mean([r["duplicate_rate"] for r in results["ArqonHPO"]]))
            },
            "Random": {
                "avg_best": float(random_best), 
                "avg_mean": float(np.nanmean([r["mean"] for r in results["Random"]])),
                "dup_rate": float(np.mean([r["duplicate_rate"] for r in results["Random"]]))
            },
            "best_accuracy": accuracy,
            "best_config": best_arqon_result.get("best_config", {}),
        }, f, indent=2)
    print("\nResults saved to nas_benchmark_results_v6.json")


if __name__ == "__main__":
    main()
