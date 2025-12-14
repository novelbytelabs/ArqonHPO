import time
import math

# Synthetic "Smooth Expensive" function
# Resembles a clean Rosenbrock or Sphere with sleep injection.
# Goal: Minimize f(x)

def smooth_expensive(params, sleep_time=0.01):
    """
    Evaluates a smooth function (Sphere) with simulated expense.
    Target: 0.0 at [0, 0, ...]
    """
    # Sleep to simulate "expensive" sim
    if sleep_time > 0:
        time.sleep(sleep_time)
        
    # f(x) = sum(x_i^2)
    val = sum(x**2 for x in params.values())
    return val

def get_smooth_config():
    return {
        "seed": 101,
        "budget": 50,
        "probe_ratio": 0.2, # 10 probe points
        "bounds": {
            "x": {"min": -5.0, "max": 5.0, "scale": "Linear"},
            "y": {"min": -5.0, "max": 5.0, "scale": "Linear"}
        }
    }
