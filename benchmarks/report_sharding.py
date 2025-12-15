
import argparse
import json
import hashlib
import struct
import time
import sys
import os

# Binding import path
sys.path.append(os.path.join(os.path.dirname(__file__), "../bindings/python"))
try:
    import arqonhpo
except ImportError:
    print("Error: arqonhpo bindings not found.")
    sys.exit(1)

def float_bits(f):
    """Return integer representation of float64 bits"""
    # use struct pack/unpack
    return struct.unpack('>Q', struct.pack('>d', f))[0]

def point_hash(p):
    """Hash a point dictionary using bitwise representation of values"""
    # Sort keys
    keys = sorted(p.keys())
    # Collect bits
    bits = []
    for k in keys:
        bits.append(float_bits(p[k]))
    # Return tuple of bits (hashable)
    return tuple(bits)

def report_sharding(config_path, num_workers, shard_size, output_dir):
    with open(config_path) as f:
        config = json.load(f)
        
    # We need a base config for Arqon
    # Construct a simple one based on "dims" and "seed" from suite?
    # Or expect suite format?
    # The user said "use suite config".
    # Phase 8 suite has "dims", "seed".
    dims = config.get("dims", 5)
    seed = config.get("seed", 42)
    
    bounds = {f"x{i}": {"min": 0.0, "max": 1.0, "scale": "Linear"} for i in range(dims)}
    solver_config = {
        "bounds": bounds,
        "budget": num_workers * shard_size * 2, # ample budget
        "seed": seed,
        "probe_ratio": 1.0
    }
    config_json = json.dumps(solver_config)
    
    print(f"Checking sharding: {num_workers} workers x {shard_size} points. Seed={seed}, Dims={dims}")
    
    all_points = []
    start_ts = time.time()
    
    # 1. Generate points
    for i in range(num_workers):
        probe = arqonhpo.ArqonProbe(config_json, seed=seed)
        start = i * shard_size
        points = probe.sample_range(start, shard_size)
        all_points.extend(points)
        
    duration = time.time() - start_ts
    
    # 2. Check Collisions (Bitwise)
    seen_hashes = set()
    collisions = 0
    stable_hashes = []
    
    for p in all_points:
        h = point_hash(p)
        if h in seen_hashes:
            collisions += 1
        else:
            seen_hashes.add(h)
        stable_hashes.append(str(h))
        
    # 3. Compute Stable Hash of all points
    # Concatenate all point hashes and hash that
    full_str = ",".join(stable_hashes)
    final_hash = hashlib.sha256(full_str.encode()).hexdigest()
    
    total = len(all_points)
    unique = len(seen_hashes)
    
    print(f"Total: {total}, Unique: {unique}, Collisions: {collisions}")
    print(f"Points Hash: {final_hash}")
    
    # 4. Write artifacts
    os.makedirs(output_dir, exist_ok=True)
    
    summary = {
        "total_points": total,
        "unique_points": unique,
        "collisions": collisions,
        "time_ms": duration * 1000.0,
        "throughput_pts_per_ms": total / (duration * 1000.0) if duration > 0 else 0,
        "final_hash": final_hash
    }
    
    with open(os.path.join(output_dir, "sharding_summary.json"), 'w') as f:
        json.dump(summary, f, indent=2)
        
    with open(os.path.join(output_dir, "points_hash.txt"), 'w') as f:
        f.write(final_hash)
        
    if collisions == 0:
        print("✅ Sharding check passed.")
    else:
        print("❌ Sharding check FAILED (collisions detected).")

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--config", required=True)
    parser.add_argument("--workers", type=int, default=100)
    parser.add_argument("--shard-size", type=int, default=10)
    parser.add_argument("--out", required=True)
    args = parser.parse_args()
    
    report_sharding(args.config, args.workers, args.shard_size, args.out)
