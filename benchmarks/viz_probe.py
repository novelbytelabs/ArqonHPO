import matplotlib.pyplot as plt
import numpy as np
import os

def get_primes(n):
    """Generate first n primes using sieve."""
    limit = int(n * (np.log(n) + np.log(np.log(n)))) + 20
    is_prime = np.ones(limit, dtype=bool)
    is_prime[:2] = False
    for i in range(2, int(limit**0.5) + 1):
        if is_prime[i]:
            is_prime[i*i::i] = False
    primes = np.nonzero(is_prime)[0]
    return primes[:n]

def generate_probe_points(n_samples=100, dims=2):
    """Replicate ArqonHPO Rust Probe Logic."""
    primes = get_primes(n_samples)
    
    # Base positions from normalized primes
    base_positions = (primes / 1000.0) % 1.0
    
    candidates = []
    
    for i in range(n_samples):
        point = []
        pos = base_positions[i]
        
        for d in range(dims):
            # Rust: (dim_idx + 1) * 0.618...
            dim_offset = (d + 1) * 0.618033988749895
            
            # Rust: (pos + dim_offset * (i / n)) % 1.0
            val = (pos + dim_offset * (i / float(n_samples))) % 1.0
            point.append(val)
            
        candidates.append(point)
        
    return np.array(candidates)

def plot_probe():
    points = generate_probe_points(n_samples=100, dims=2)
    random_points = np.random.rand(100, 2)
    
    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(12, 6))
    
    # Plot PCR Probe
    ax1.scatter(points[:, 0], points[:, 1], c='blue', alpha=0.6, s=50)
    ax1.set_title("PCR Prime-Golden Probe (N=100)")
    ax1.set_xlabel("Dim 0")
    ax1.set_ylabel("Dim 1")
    ax1.grid(True, alpha=0.3)
    
    # Plot Random
    ax2.scatter(random_points[:, 0], random_points[:, 1], c='red', alpha=0.6, s=50)
    ax2.set_title("Uniform Random (N=100)")
    ax2.set_xlabel("Dim 0")
    ax2.set_ylabel("Dim 1")
    ax2.grid(True, alpha=0.3)
    
    output_dir = "../docs/docs/assets"
    os.makedirs(output_dir, exist_ok=True)
    output_path = os.path.join(output_dir, "probe_lattice.png")
    
    plt.tight_layout()
    plt.savefig(output_path, dpi=300)
    print(f"Saved plot to {output_path}")

if __name__ == "__main__":
    plot_probe()
