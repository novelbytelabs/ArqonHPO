import time
from typing import Dict

# ============================================================================
# Structured (Smooth) Objective Functions for US1 Testing
# ============================================================================

def sphere(params: Dict[str, float]) -> float:
    """
    Sphere function: f(x) = sum(x_i^2)
    Global minimum: 0.0 at origin
    Landscape: Smooth, unimodal, convex → Structured
    """
    return sum(x**2 for x in params.values())


def rosenbrock(params: Dict[str, float]) -> float:
    """
    Rosenbrock function: f(x,y) = (a-x)^2 + b*(y-x^2)^2
    Global minimum: 0.0 at (a, a^2) = (1, 1)
    Landscape: Narrow valley, smooth → Structured
    
    Standard constants: a=1, b=100
    """
    keys = sorted(params.keys())
    if len(keys) < 2:
        # Fallback for 1D: just return sphere
        return sphere(params)
    
    result = 0.0
    for i in range(len(keys) - 1):
        x = params[keys[i]]
        y = params[keys[i + 1]]
        result += (1 - x)**2 + 100 * (y - x**2)**2
    return result


def smooth_expensive(params: Dict[str, float], sleep_time: float = 0.01) -> float:
    """
    Sphere function with simulated expensive evaluation.
    Target: 0.0 at [0, 0, ...]
    """
    if sleep_time > 0:
        time.sleep(sleep_time)
    return sphere(params)


def rosenbrock_expensive(params: Dict[str, float], sleep_time: float = 0.01) -> float:
    """
    Rosenbrock function with simulated expensive evaluation.
    Target: 0.0 at [1, 1, ...]
    """
    if sleep_time > 0:
        time.sleep(sleep_time)
    return rosenbrock(params)


# ============================================================================
# Configuration Templates
# ============================================================================

def get_smooth_config(seed: int = 101) -> dict:
    """Standard 2D Sphere configuration."""
    return {
        "seed": seed,
        "budget": 50,
        "probe_ratio": 0.2,  # 10 probe points
        "bounds": {
            "x": {"min": -5.0, "max": 5.0, "scale": "Linear"},
            "y": {"min": -5.0, "max": 5.0, "scale": "Linear"}
        }
    }


def get_rosenbrock_config(seed: int = 102) -> dict:
    """Standard 2D Rosenbrock configuration with shifted bounds."""
    return {
        "seed": seed,
        "budget": 100,
        "probe_ratio": 0.2,  # 20 probe points
        "bounds": {
            "x": {"min": -2.0, "max": 2.0, "scale": "Linear"},
            "y": {"min": -1.0, "max": 3.0, "scale": "Linear"}
        }
    }

