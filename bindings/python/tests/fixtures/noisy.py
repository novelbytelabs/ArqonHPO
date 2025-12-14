import random

# Synthetic "Noisy Cheap" function
# Goal: Minimize f(x)
# f(x) = Sphere + Noise

def noisy_cheap(params):
    """
    Evaluates a noisy function.
    Target: 0.0 at [0, 0] with noise.
    """
    # f(x) = sum(x_i^2) + noise
    # Noise: Uniform(-1.0, 1.0) * level? Or Gaussian?
    # Spec says "high variance".
    
    val = sum(x**2 for x in params.values())
    
    # Add noise. 
    # To force Classifier to see "Chaotic", we need Variance/Mean > 2.0 (our new threshold).
    # If val is small (near optimal), mean is small.
    # If noise is large, Variance is large.
    # Add random noise in [-5, 5].
    noise = random.uniform(-5.0, 5.0)
    
    return val + noise

def get_noisy_config():
    return {
        "seed": 202,
        "budget": 50,
        "probe_ratio": 0.4, # More probe samples to detect noise
        "bounds": {
            "x": {"min": -5.0, "max": 5.0, "scale": "Linear"},
            "y": {"min": -5.0, "max": 5.0, "scale": "Linear"}
        }
    }
