#!/usr/bin/env python3
"""
CI Guardrail Tests for ArqonHPO Probe Upgrade

Tests:
1. Probe-only quality (shifted instances)
2. End-to-end structured routing sanity (NM must win)
3. Multimodal guardrail (probe must not be fragile on Rastrigin)
4. Geometry regression test (fast, deterministic)

Run: conda run -n helios-gpu-118 pytest benchmarks/test_probe_guardrails.py -v
"""

import numpy as np
from scipy.spatial import KDTree
from typing import Callable, Tuple, List
import pytest

# =============================================================================
# Configuration
# =============================================================================

CI_SEED = 20251214
K_SHIFTS_STRUCTURED = 25
K_SHIFTS_MULTIMODAL = 30
B_TOTAL = 200
PROBE_RATIO = 0.2
EPSILON = 1e-12


# =============================================================================
# Probe Implementations
# =============================================================================

def get_primes(n: int) -> np.ndarray:
    if n == 0:
        return np.array([], dtype=int)
    limit = max(20, int(n * (np.log(n + 1) + np.log(np.log(n + 2)))) + 20)
    is_prime = np.ones(limit, dtype=bool)
    is_prime[:2] = False
    for i in range(2, int(limit**0.5) + 1):
        if is_prime[i]:
            is_prime[i*i::i] = False
    return np.nonzero(is_prime)[0][:n]


def probe_legacy(n_samples: int, dims: int) -> np.ndarray:
    """Legacy p/1000 probe - the flawed implementation."""
    primes = get_primes(n_samples)
    candidates = []
    base_positions = (primes / 1000.0) % 1.0
    
    for i in range(n_samples):
        point = []
        pos = base_positions[i]
        for d in range(dims):
            dim_offset = (d + 1) * 0.618033988749895
            val = (pos + dim_offset * (i / float(n_samples))) % 1.0
            point.append(val)
        candidates.append(point)
    return np.array(candidates)


def probe_new(n_samples: int, dims: int) -> np.ndarray:
    """New PrimeSqrtSlopesRotProbe - the validated implementation.
    
    Formula: x[i,d] = frac((i+1) * sqrt(primes[50+d]) + frac(primes[200+d] * (sqrt(2)-1)))
    """
    primes = get_primes(210 + dims)
    prime_offset = 50
    rot_offset = 200
    rot_alpha = np.sqrt(2) - 1
    
    candidates = []
    for i in range(n_samples):
        point = []
        for d in range(dims):
            slope = np.sqrt(primes[prime_offset + d])
            rotation = (primes[rot_offset + d] * rot_alpha) % 1.0
            val = ((i + 1) * slope + rotation) % 1.0
            point.append(val)
        candidates.append(point)
    return np.array(candidates)


def probe_random(n_samples: int, dims: int, seed: int = 0) -> np.ndarray:
    """Random uniform probe."""
    rng = np.random.default_rng(seed)
    return rng.random((n_samples, dims))


# =============================================================================
# Benchmark Functions (Unit Cube → Scaled)
# =============================================================================

def sphere(x: np.ndarray) -> float:
    x = (x * 2 - 1) * 5.0
    return float(np.sum(x**2))


def rosenbrock(x: np.ndarray) -> float:
    x = (x * 2 - 1) * 2.048
    return float(np.sum(100 * (x[1:] - x[:-1]**2)**2 + (1 - x[:-1])**2))


def rastrigin(x: np.ndarray) -> float:
    x = (x * 2 - 1) * 5.12
    d = len(x)
    return float(10 * d + np.sum(x**2 - 10 * np.cos(2 * np.pi * x)))


# =============================================================================
# Refinement Implementations
# =============================================================================

def refine_nelder_mead(init_points: np.ndarray, fn: Callable, budget: int) -> float:
    """Nelder-Mead from best initial point."""
    from scipy.optimize import minimize
    
    values = [fn(x) for x in init_points]
    best_idx = np.argmin(values)
    x0 = init_points[best_idx]
    best = values[best_idx]
    
    remaining = budget - len(init_points)
    if remaining <= 0:
        return best
    
    def objective(x):
        return fn(np.clip(x, 0, 1))
    
    try:
        result = minimize(objective, x0, method='Nelder-Mead', 
                         options={'maxfev': remaining})
        if result.fun < best:
            best = result.fun
    except:
        pass
    
    return best


def refine_tpe(init_points: np.ndarray, fn: Callable, budget: int, seed: int) -> float:
    """Optuna TPE refinement."""
    try:
        import optuna
        optuna.logging.set_verbosity(optuna.logging.WARNING)
    except ImportError:
        pytest.skip("Optuna not available")
    
    dims = init_points.shape[1]
    init_values = [fn(x) for x in init_points]
    best = min(init_values)
    remaining = budget - len(init_points)
    
    if remaining <= 0:
        return best
    
    def objective(trial):
        nonlocal best
        x = np.array([trial.suggest_float(f"x{i}", 0.0, 1.0) for i in range(dims)])
        val = fn(x)
        if val < best:
            best = val
        return val
    
    sampler = optuna.samplers.TPESampler(seed=seed)
    study = optuna.create_study(direction="minimize", sampler=sampler)
    
    for x, val in zip(init_points, init_values):
        study.add_trial(
            optuna.trial.create_trial(
                params={f"x{i}": x[i] for i in range(dims)},
                distributions={f"x{i}": optuna.distributions.FloatDistribution(0.0, 1.0) 
                              for i in range(dims)},
                values=[val],
            )
        )
    
    study.optimize(objective, n_trials=remaining, show_progress_bar=False)
    return min(best, study.best_value)


# =============================================================================
# Test Helpers
# =============================================================================

def generate_shifts(k: int, dims: int, seed: int) -> np.ndarray:
    """Generate K random torus shifts."""
    rng = np.random.default_rng(seed)
    return rng.random((k, dims))


def evaluate_probe(probe_fn: Callable, fn: Callable, shift: np.ndarray, 
                   n_samples: int, dims: int) -> float:
    """Evaluate probe quality on shifted function."""
    points = probe_fn(n_samples, dims)
    shifted = (points + shift) % 1.0
    return min(fn(x) for x in shifted)


def compute_improvement(legacy: float, new: float) -> float:
    """Compute relative improvement."""
    return (legacy - new) / (abs(legacy) + EPSILON)


# =============================================================================
# TEST 1: Probe-only quality (shifted instances)
# =============================================================================

class TestProbeOnlyQuality:
    """Ensure new probe stays a strict upgrade over legacy."""
    
    @pytest.fixture
    def setup(self):
        dims = 5
        n_probe = int(np.ceil(PROBE_RATIO * B_TOTAL))
        shifts = generate_shifts(K_SHIFTS_STRUCTURED, dims, CI_SEED)
        return dims, n_probe, shifts
    
    def test_sphere_probe_quality(self, setup):
        """Sphere(5D): New probe must beat Legacy by ≥20% median, ≥60% win rate."""
        dims, n_probe, shifts = setup
        
        legacy_values = []
        new_values = []
        wins = 0
        
        for shift in shifts:
            legacy = evaluate_probe(probe_legacy, sphere, shift, n_probe, dims)
            new = evaluate_probe(probe_new, sphere, shift, n_probe, dims)
            legacy_values.append(legacy)
            new_values.append(new)
            if new < legacy:
                wins += 1
        
        win_rate = wins / len(shifts)
        improvements = [compute_improvement(l, n) for l, n in zip(legacy_values, new_values)]
        median_improvement = np.median(improvements)
        
        print(f"\nSphere(5D) Probe Quality:")
        print(f"  Win rate: {win_rate:.2%} (threshold: ≥70%)")
        print(f"  Median improvement: {median_improvement:.2%} (threshold: ≥20%)")
        
        assert win_rate >= 0.60, f"Win rate {win_rate:.2%} < 60%"
        assert median_improvement >= 0.20, f"Median improvement {median_improvement:.2%} < 20%"
    
    def test_rosenbrock_probe_quality(self, setup):
        """Rosenbrock(5D): New probe must beat Legacy by ≥15% median, ≥65% win rate."""
        dims, n_probe, shifts = setup
        
        legacy_values = []
        new_values = []
        wins = 0
        
        for shift in shifts:
            legacy = evaluate_probe(probe_legacy, rosenbrock, shift, n_probe, dims)
            new = evaluate_probe(probe_new, rosenbrock, shift, n_probe, dims)
            legacy_values.append(legacy)
            new_values.append(new)
            if new < legacy:
                wins += 1
        
        win_rate = wins / len(shifts)
        improvements = [compute_improvement(l, n) for l, n in zip(legacy_values, new_values)]
        median_improvement = np.median(improvements)
        
        print(f"\nRosenbrock(5D) Probe Quality:")
        print(f"  Win rate: {win_rate:.2%} (threshold: ≥65%)")
        print(f"  Median improvement: {median_improvement:.2%} (threshold: ≥15%)")
        
        assert win_rate >= 0.65, f"Win rate {win_rate:.2%} < 65%"
        assert median_improvement >= 0.15, f"Median improvement {median_improvement:.2%} < 15%"


# =============================================================================
# TEST 2: End-to-end structured routing sanity
# =============================================================================

class TestStructuredRouting:
    """New probe + NM must win on structured functions."""
    
    @pytest.fixture
    def setup(self):
        dims = 5
        n_probe = int(np.ceil(PROBE_RATIO * B_TOTAL))
        shifts = generate_shifts(K_SHIFTS_STRUCTURED, dims, CI_SEED)
        return dims, n_probe, shifts
    
    def test_sphere_nm_wins(self, setup):
        """Sphere(5D): NM win rate ≥75% vs TPE."""
        dims, n_probe, shifts = setup
        nm_wins = 0
        
        for i, shift in enumerate(shifts):
            probe_points = probe_new(n_probe, dims)
            probe_shifted = (probe_points + shift) % 1.0
            
            def shifted_fn(x):
                return sphere((x + shift) % 1.0)
            
            nm_result = refine_nelder_mead(probe_shifted, shifted_fn, B_TOTAL)
            tpe_result = refine_tpe(probe_shifted, shifted_fn, B_TOTAL, CI_SEED + i)
            
            if nm_result < tpe_result:
                nm_wins += 1
        
        win_rate = nm_wins / len(shifts)
        print(f"\nSphere(5D) Routing: NM win rate = {win_rate:.2%} (threshold: ≥75%)")
        
        assert win_rate >= 0.75, f"NM win rate {win_rate:.2%} < 75%"
    
    def test_rosenbrock_nm_wins(self, setup):
        """Rosenbrock(5D): NM win rate ≥65% vs TPE."""
        dims, n_probe, shifts = setup
        nm_wins = 0
        
        for i, shift in enumerate(shifts):
            probe_points = probe_new(n_probe, dims)
            probe_shifted = (probe_points + shift) % 1.0
            
            def shifted_fn(x):
                return rosenbrock((x + shift) % 1.0)
            
            nm_result = refine_nelder_mead(probe_shifted, shifted_fn, B_TOTAL)
            tpe_result = refine_tpe(probe_shifted, shifted_fn, B_TOTAL, CI_SEED + i)
            
            if nm_result < tpe_result:
                nm_wins += 1
        
        win_rate = nm_wins / len(shifts)
        print(f"\nRosenbrock(5D) Routing: NM win rate = {win_rate:.2%} (threshold: ≥65%)")
        
        assert win_rate >= 0.65, f"NM win rate {win_rate:.2%} < 65%"


# =============================================================================
# TEST 3: Multimodal guardrail
# =============================================================================

class TestMultimodalGuardrail:
    """New probe must not be fragile on Rastrigin."""
    
    def test_rastrigin_not_inferior(self):
        """Rastrigin(5D): New→TPE must be non-inferior to Random→TPE."""
        dims = 5
        n_probe = int(np.ceil(PROBE_RATIO * B_TOTAL))
        shifts = generate_shifts(K_SHIFTS_MULTIMODAL, dims, CI_SEED)
        
        new_values = []
        random_values = []
        new_wins = 0
        
        for i, shift in enumerate(shifts):
            def shifted_fn(x):
                return rastrigin((x + shift) % 1.0)
            
            # New probe → TPE
            new_points = probe_new(n_probe, dims)
            new_shifted = (new_points + shift) % 1.0
            new_result = refine_tpe(new_shifted, shifted_fn, B_TOTAL, CI_SEED + i)
            new_values.append(new_result)
            
            # Random probe → TPE
            random_points = probe_random(n_probe, dims, CI_SEED + i)
            random_shifted = (random_points + shift) % 1.0
            random_result = refine_tpe(random_shifted, shifted_fn, B_TOTAL, CI_SEED + i + 1000)
            random_values.append(random_result)
            
            if new_result < random_result:
                new_wins += 1
        
        mean_new = np.mean(new_values)
        mean_random = np.mean(random_values)
        win_rate = new_wins / len(shifts)
        
        print(f"\nRastrigin(5D) Multimodal Guardrail:")
        print(f"  New mean: {mean_new:.4f}")
        print(f"  Random mean: {mean_random:.4f}")
        print(f"  Win rate: {win_rate:.2%}")
        
        # Pass if either condition met
        non_inferior = (mean_new <= mean_random * 1.05) or (win_rate >= 0.45)
        assert non_inferior, f"New probe is inferior: mean={mean_new:.4f} vs random={mean_random:.4f}, win_rate={win_rate:.2%}"


# =============================================================================
# TEST 4: Geometry regression test (fast, deterministic)
# =============================================================================

class TestGeometryRegression:
    """Fast test to catch striping/collisions regression."""
    
    def test_probe_geometry(self):
        """Probe geometry: CD ≤ 0.15, mean_NN ≥ 0.25, hard fail if NN < 0.10."""
        dims = 10
        n_samples = 256
        
        points = probe_new(n_samples, dims)
        
        # Compute mean nearest-neighbor distance
        tree = KDTree(points)
        distances, _ = tree.query(points, k=2)
        mean_nn = np.mean(distances[:, 1])
        
        # Compute centered discrepancy (simplified)
        # Using L2 star discrepancy approximation
        cd = self._compute_cd(points)
        
        print(f"\nGeometry Test (d={dims}, N={n_samples}):")
        print(f"  Centered Discrepancy: {cd:.6f} (threshold: ≤0.15)")
        print(f"  Mean NN Distance: {mean_nn:.6f} (threshold: ≥0.25, hard fail <0.10)")
        
        # Hard fail check
        assert mean_nn >= 0.10, f"HARD FAIL: mean_NN={mean_nn:.6f} < 0.10 (degeneracy detected)"
        
        # Soft thresholds
        assert cd <= 0.15, f"Centered discrepancy {cd:.6f} > 0.15"
        assert mean_nn >= 0.25, f"Mean NN distance {mean_nn:.6f} < 0.25"
    
    def _compute_cd(self, points: np.ndarray) -> float:
        """Compute centered L2 discrepancy (approximation)."""
        n, d = points.shape
        
        # Simplified CD calculation
        term1 = 0.0
        for i in range(n):
            prod = 1.0
            for j in range(d):
                x = points[i, j]
                prod *= (1 + 0.5 * abs(x - 0.5) - 0.5 * (x - 0.5)**2)
            term1 += prod
        term1 = term1 / n
        
        term2 = 0.0
        for i in range(n):
            for k in range(n):
                prod = 1.0
                for j in range(d):
                    x_i = points[i, j]
                    x_k = points[k, j]
                    prod *= (1 + 0.5 * abs(x_i - 0.5) + 0.5 * abs(x_k - 0.5) 
                            - 0.5 * abs(x_i - x_k))
                term2 += prod
        term2 = term2 / (n * n)
        
        cd = np.sqrt(abs((13.0/12.0)**d - 2 * term1 + term2))
        return cd


# =============================================================================
# Main
# =============================================================================

if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
