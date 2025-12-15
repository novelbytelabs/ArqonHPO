
import json
import arqonhpo
import hashlib
import time

def demo_sharding():
    print("=== Phase 7 Demo: Deterministic Parallel Sharding ===")
    
    # 1. Setup Config
    dim = 5
    config = {
        "bounds": {f"x{i}": {"min": 0.0, "max": 1.0, "scale": "Linear"} for i in range(dim)},
        "budget": 10000,
        "seed": 42,
        "probe_ratio": 0.2
    }
    config_json = json.dumps(config)
    
    # 2. Simulate 100 Workers
    num_workers = 100
    points_per_worker = 10
    total_expected = num_workers * points_per_worker
    
    print(f"Simulating {num_workers} workers, each generating {points_per_worker} points.")
    print("Protocol: ZERO COORDINATION. Each worker computes its own shard.")
    
    all_points = []
    
    start_time = time.time()
    
    # In a real distributed system, this loop runs on different machines
    # Each machine initializes its own Probe instance
    for worker_id in range(num_workers):
        # Worker initializes Probe locally (stateless!)
        # Note: Workers use SAME seed 42 to share the same sequence universe.
        probe = arqonhpo.ArqonProbe(config_json, seed=42)
        
        # Calculate shard
        start_idx = worker_id * points_per_worker
        
        # Generate range
        # API: sample_range(start, count)
        points = probe.sample_range(start_idx, points_per_worker)
        
        # In real system, worker evaluates these points.
        # Here we collect them to verify uniqueness.
        all_points.extend(points)
        
        # Also verify sample_at equivalence for first point
        # Just to prove API consistency
        single_point = probe.sample_at(start_idx)
        # Check equality (float comparison might be tricky, use almost equal or string rep)
        # Using string set logic below, so string comparison is fine.
        p1_str = json.dumps(points[0], sort_keys=True)
        p2_str = json.dumps(single_point, sort_keys=True)
        assert p1_str == p2_str, f"Mismatch at index {start_idx}: range[0] vs at()"
        
    duration = time.time() - start_time
    print(f"Generation complete in {duration:.4f}s")
    
    # 3. Verification
    print("\nVerifying results...")
    
    # Hash each point to check for duplicates
    # We round to high precision to avoid float noise if any (should be exact bits though)
    # Using JSON dump as canonical representation for now
    hashes = set()
    collisions = 0
    
    for p in all_points:
        # Sort keys to ensure canonical JSON
        s = json.dumps(p, sort_keys=True)
        h = hashlib.md5(s.encode()).hexdigest()
        if h in hashes:
            collisions += 1
        else:
            hashes.add(h)
            
    print(f"Total Points: {len(all_points)}")
    print(f"Unique Points: {len(hashes)}")
    print(f"Collisions: {collisions}")
    
    if len(all_points) == total_expected and collisions == 0:
        print("\n✅ SUCCESS: 100% Unique, Collision-Free Sharding!")
    else:
        print("\n❌ FAILED: Collisions or missing points.")
        
    # 4. Reproducibility Test
    print("\nVerifying Reproducibility (Run 2)...")
    probe2 = arqonhpo.ArqonProbe(config_json, seed=42)
    # Pick arbitrary range
    run2_points = probe2.sample_range(500, 10)
    
    # Compare with collected points [500..510]
    run1_slice = all_points[500:510]
    
    match = True
    for p1, p2 in zip(run1_slice, run2_points):
        s1 = json.dumps(p1, sort_keys=True)
        s2 = json.dumps(p2, sort_keys=True)
        if s1 != s2:
            match = False
            break
            
    if match:
        print("✅ SUCCESS: Run 2 matches Run 1 exactly.")
    else:
        print("❌ FAILED: Reproducibility error.")

if __name__ == "__main__":
    demo_sharding()
