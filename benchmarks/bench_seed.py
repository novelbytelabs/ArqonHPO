"""Benchmark for ArqonSolver.seed() method performance."""
import json
import time
from arqonhpo import ArqonSolver


def benchmark_seed(num_points: int, num_params: int, iterations: int = 100) -> dict:
    """
    Benchmark seeding performance.
    
    Args:
        num_points: Number of seed points to inject
        num_params: Number of parameters per point
        iterations: Number of iterations to average
    
    Returns:
        dict with timing results
    """
    # Create config with specified number of parameters
    bounds = {
        f"x{i}": {"min": -5.0, "max": 5.0, "scale": "Linear"}
        for i in range(num_params)
    }
    config = {
        "seed": 42,
        "budget": 1000,
        "bounds": bounds,
        "probe_ratio": 0.2
    }
    config_json = json.dumps(config)
    
    # Generate seed data
    seed_data = [
        {
            "params": {f"x{j}": float(i * j) / num_points for j in range(num_params)},
            "value": float(i),
            "cost": 1.0
        }
        for i in range(num_points)
    ]
    seed_json = json.dumps(seed_data)
    
    # Benchmark
    times = []
    for _ in range(iterations):
        solver = ArqonSolver(config_json)
        
        start = time.perf_counter_ns()
        solver.seed(seed_json)
        end = time.perf_counter_ns()
        
        times.append(end - start)
        
        # Verify
        assert solver.get_history_len() == num_points
    
    avg_ns = sum(times) / len(times)
    min_ns = min(times)
    max_ns = max(times)
    
    return {
        "num_points": num_points,
        "num_params": num_params,
        "iterations": iterations,
        "avg_ns": avg_ns,
        "avg_us": avg_ns / 1000,
        "avg_ms": avg_ns / 1_000_000,
        "min_ns": min_ns,
        "max_ns": max_ns,
        "points_per_us": num_points / (avg_ns / 1000) if avg_ns > 0 else 0
    }


def main():
    print("=" * 60)
    print("ArqonSolver.seed() Performance Benchmark")
    print("=" * 60)
    print()
    
    # Test configurations
    test_cases = [
        (10, 1),      # Small: 10 points, 1 param
        (100, 1),     # Medium: 100 points, 1 param
        (100, 10),    # Medium: 100 points, 10 params
        (1000, 1),    # Large: 1000 points, 1 param
        (1000, 10),   # Large: 1000 points, 10 params
        (10000, 10),  # Very large: 10k points, 10 params
    ]
    
    print(f"{'Points':<10} {'Params':<10} {'Avg (µs)':<15} {'Points/µs':<15}")
    print("-" * 50)
    
    results = []
    for num_points, num_params in test_cases:
        result = benchmark_seed(num_points, num_params, iterations=50)
        results.append(result)
        print(f"{num_points:<10} {num_params:<10} {result['avg_us']:<15.2f} {result['points_per_us']:<15.2f}")
    
    print()
    print("=" * 60)
    print("Summary")
    print("=" * 60)
    
    # Calculate throughput
    for r in results:
        print(f"  {r['num_points']:>5} points × {r['num_params']:<2} params: "
              f"{r['avg_us']:>8.2f} µs ({r['points_per_us']:.1f} points/µs)")
    
    print()
    print("✓ Benchmark complete")


if __name__ == "__main__":
    main()
