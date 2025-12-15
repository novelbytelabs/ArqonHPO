#!/usr/bin/env python3
"""
viz_probe_upgrade.py

Visual comparison of legacy PrimeIndexProbe vs new PrimeSqrtSlopesRotProbe.
Also computes discrepancy metrics to quantify the improvement.

Run: conda run -n helios-gpu-118 python benchmarks/viz_probe_upgrade.py
"""

import matplotlib.pyplot as plt
import numpy as np
from scipy.spatial import KDTree
import os

def get_primes(n):
    """Generate first n primes using sieve."""
    if n == 0:
        return np.array([], dtype=int)
    limit = max(20, int(n * (np.log(n + 1) + np.log(np.log(n + 2)))) + 20)
    is_prime = np.ones(limit, dtype=bool)
    is_prime[:2] = False
    for i in range(2, int(limit**0.5) + 1):
        if is_prime[i]:
            is_prime[i*i::i] = False
    primes = np.nonzero(is_prime)[0]
    return primes[:n]

def probe_legacy(n_samples=200, dims=2):
    """Legacy p/1000 implementation (FLAWED - for comparison only)."""
    primes = get_primes(n_samples)
    
    candidates = []
    base_positions = (primes / 1000.0) % 1.0
    
    for i in range(n_samples):
        point = []
        pos = base_positions[i]
        for d in range(dims):
            dim_offset = (d + 1) * 0.618033988749895  # Golden ratio
            val = (pos + dim_offset * (i / float(n_samples))) % 1.0
            point.append(val)
        candidates.append(point)
    return np.array(candidates)

def probe_prime_sqrt_slopes_rot(n_samples=200, dims=2, prime_offset=50, rot_offset=200):
    """
    New PrimeSqrtSlopesRotProbe implementation (VALIDATED).
    
    x_{i,d} = frac(i * sqrt(p_{d+prime_offset}) + frac(p_{d+rot_offset} * rot_alpha))
    """
    primes = get_primes(rot_offset + dims + 10)
    rot_alpha = np.sqrt(2) - 1  # Irrational
    
    candidates = []
    for i in range(n_samples):
        point = []
        for d in range(dims):
            slope = np.sqrt(primes[prime_offset + d])
            rotation = (primes[rot_offset + d] * rot_alpha) % 1.0
            # Using i+1 to avoid origin degeneracy
            val = ((i + 1) * slope + rotation) % 1.0
            point.append(val)
        candidates.append(point)
    return np.array(candidates)

def compute_mean_nn_distance(points):
    """Compute mean nearest-neighbor distance (higher = less clumping)."""
    if len(points) < 2:
        return 0.0
    tree = KDTree(points)
    distances, _ = tree.query(points, k=2)  # k=2 to get the first neighbor (not self)
    return np.mean(distances[:, 1])

def compute_collision_count(points, threshold=0.001):
    """Count near-duplicate positions."""
    count = 0
    n = len(points)
    for i in range(n):
        for j in range(i + 1, n):
            if np.linalg.norm(points[i] - points[j]) < threshold:
                count += 1
    return count

def plot_comparison():
    """Generate side-by-side comparison with quality metrics."""
    n = 256
    dims = 2
    
    legacy = probe_legacy(n, dims)
    new = probe_prime_sqrt_slopes_rot(n, dims)
    
    # Compute quality metrics
    legacy_nn = compute_mean_nn_distance(legacy)
    new_nn = compute_mean_nn_distance(new)
    
    legacy_collisions = compute_collision_count(legacy)
    new_collisions = compute_collision_count(new)
    
    # Create figure
    fig, axes = plt.subplots(1, 2, figsize=(14, 6))
    
    # Legacy probe
    ax1 = axes[0]
    ax1.scatter(legacy[:, 0], legacy[:, 1], c='red', alpha=0.6, s=30, edgecolors='darkred', linewidths=0.5)
    ax1.set_title(f"Legacy: Prime/1000 + Golden Sweep\n(DEPRECATED)", fontsize=12, fontweight='bold')
    ax1.set_xlim(0, 1)
    ax1.set_ylim(0, 1)
    ax1.set_xlabel("Dimension 0")
    ax1.set_ylabel("Dimension 1")
    ax1.grid(True, alpha=0.3)
    ax1.set_aspect('equal')
    
    # Add metrics box for legacy
    metrics_text_legacy = f"Mean NN dist: {legacy_nn:.4f}\nCollisions: {legacy_collisions}"
    ax1.text(0.02, 0.98, metrics_text_legacy, transform=ax1.transAxes, fontsize=10,
             verticalalignment='top', bbox=dict(boxstyle='round', facecolor='white', alpha=0.8))
    
    # New probe
    ax2 = axes[1]
    ax2.scatter(new[:, 0], new[:, 1], c='green', alpha=0.6, s=30, edgecolors='darkgreen', linewidths=0.5)
    ax2.set_title(f"New: PrimeSqrtSlopesRotProbe\n(RECOMMENDED)", fontsize=12, fontweight='bold')
    ax2.set_xlim(0, 1)
    ax2.set_ylim(0, 1)
    ax2.set_xlabel("Dimension 0")
    ax2.set_ylabel("Dimension 1")
    ax2.grid(True, alpha=0.3)
    ax2.set_aspect('equal')
    
    # Add metrics box for new
    metrics_text_new = f"Mean NN dist: {new_nn:.4f}\nCollisions: {new_collisions}"
    ax2.text(0.02, 0.98, metrics_text_new, transform=ax2.transAxes, fontsize=10,
             verticalalignment='top', bbox=dict(boxstyle='round', facecolor='white', alpha=0.8))
    
    # Overall title
    fig.suptitle(f"Probe Comparison (N={n}, d={dims})", fontsize=14, fontweight='bold', y=1.02)
    
    # Save
    output_dir = "../docs/docs/assets"
    os.makedirs(output_dir, exist_ok=True)
    output_path = os.path.join(output_dir, "probe_comparison.png")
    
    plt.tight_layout()
    plt.savefig(output_path, dpi=200, bbox_inches='tight')
    print(f"Saved plot to {output_path}")
    
    # Print summary
    print("\n=== Probe Quality Comparison ===")
    print(f"{'Metric':<25} {'Legacy':>12} {'New':>12} {'Winner':>12}")
    print("-" * 65)
    print(f"{'Mean NN Distance':<25} {legacy_nn:>12.4f} {new_nn:>12.4f} {'NEW' if new_nn > legacy_nn else 'LEGACY':>12}")
    print(f"{'Collisions (< 0.001)':<25} {legacy_collisions:>12} {new_collisions:>12} {'NEW' if new_collisions < legacy_collisions else 'LEGACY':>12}")
    print()
    
    # Verdict
    if new_nn > legacy_nn and new_collisions <= legacy_collisions:
        print("✅ VERDICT: New PrimeSqrtSlopesRotProbe is SUPERIOR")
    else:
        print("⚠️ VERDICT: Results mixed - manual review needed")

if __name__ == "__main__":
    plot_comparison()
