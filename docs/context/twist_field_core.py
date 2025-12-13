# twist_field_core.py
import numpy as np
import pandas as pd
from sympy import factorint, primerange
from sklearn.decomposition import PCA
from sklearn.metrics.pairwise import cosine_similarity
from itertools import combinations
from math import gcd

def Twist(n, primes=primerange(2, 200), s=0.5, use_su2=True):
    """
    Computes the twist-field signature of a number 'n'.

    Args:
        n (int): The number to embed in the twist field.
        primes (iterable): The primes to use as basis frequencies (default: first 200 primes).
        s (float): The decay exponent (default: 0.5).
        use_su2 (bool): Whether to use SU(2) rotations in the twist (default: True). 

    Returns:
        np.ndarray: The twist vector of n.
    """
    primes_list = np.array(list(primes))
    if use_su2: # SU(2) Twist Calculation
         radii = np.array([[n]], dtype=np.float64) # Ensure correct type/shape
         twist_vectors = []
         for p in primes_list:
              theta = 2 * np.pi * radii[0,0] / p
              I = np.eye(2, dtype=complex)
              sigma = [
                  np.array([[0, 1], [1, 0]], dtype=complex),
                  np.array([[0, -1j], [1j, 0]], dtype=complex),
                  np.array([[1, 0], [0, -1]], dtype=complex)
              ]
              U = I.copy() # Accumulated rotation
              for i in range(3):
                  axis = [1,0,0] if i==0 else [0,1,0] if i==1 else [0,0,1] # Simple axis choices
                  ax_sum = np.linalg.norm(axis)
                  axis_norm = np.array(axis) / ax_sum if ax_sum > 0 else np.zeros(3)
                  for k in range(3): U -= 1j * axis_norm[k] * sigma[k] * theta / 2.0 # SU(2) formula
              trace_u = np.trace(U)
              twist = np.abs(trace_u) / (float(p)**s) # Use absolute value of trace, apply decay
              twist_vectors.append(twist)
         return np.hstack(twist_vectors).flatten() # Ensure flattened output


    else: # Simple Twist (non-SU(2))
        radii = np.array([n])
        twist = np.cos(radii * np.log(primes_list)) / (primes_list ** s) # Simplified twist definition if needed.
        return twist.flatten()



def overlap_descriptor(n):
    """
    Calculates the overlap descriptors for a composite number n, which are
    counts of how many elements of Z/nZ are congruent to zero modulo each subset of
    the distinct prime factors of n. These overlaps correspond to the geometry
    of torus intersections in the Prime Field.

    Args:
        n (int): The composite number to analyze.

    Returns:
        pandas.DataFrame: A DataFrame where each row represents a subset of prime factors, with 'subset', 'lcm', 'overlap_count' columns.
    """
    try:
        primes_n = factorint(n) # Assumes sympy is imported
        if len(primes_n) == 1 and list(primes_n.values())[0]==1: return pd.DataFrame() # n is prime
        distinct_prime_factors = sorted(primes_n.keys())
        descriptors = []
        for subset_size in range(1, len(distinct_prime_factors)+1): # Consider each subset size
            for subset in combinations(distinct_prime_factors, subset_size):
                 lcm_subset = np.lcm.reduce(subset) if len(subset) > 1 else subset[0]
                 overlap_count_subset = n // lcm_subset
                 descriptors.append({'subset': tuple(sorted(subset)), 'lcm': lcm_subset, 'overlap_count': overlap_count_subset})

        descriptor_df = pd.DataFrame(descriptors)
        return descriptor_df
    except Exception as e:
        logging.error(f"Error in overlap_descriptor({n}): {e}")
        return pd.DataFrame()

def pca_project(data, n_components=2):
    """
    Performs PCA dimensionality reduction on the given data.

    Args:
        data (np.ndarray): The data matrix to project. Shape (n_samples, n_features).
        n_components (int): Number of principal components to keep.

    Returns:
        np.ndarray: Projected data. Shape (n_samples, n_components).
    """
    pca = PCA(n_components=n_components)
    return pca.fit_transform(data)

def cosine_similarity(v1, v2):
    """
    Calculate cosine similarity between two vectors.

    Args:
        v1 (np.ndarray): First vector.
        v2 (np.ndarray): Second vector.

    Returns:
        float: Cosine similarity score in range [-1, 1].
    """
    dot_product = np.dot(v1, v2)
    mag_v1 = np.linalg.norm(v1)
    mag_v2 = np.linalg.norm(v2)
    # Handle potential zero-magnitude vectors
    denominator = (mag_v1 * mag_v2) if mag_v1 > 0 and mag_v2 >0 else 1e-12 # Very small
    return dot_product / denominator


# If using logging, add this import and a basic config at the module level.
import logging
logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(filename)s:%(lineno)d - %(message)s')