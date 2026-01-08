"""Tests for ArqonSolver.seed() method - warm-starting with historical data."""
import json
import pytest
from arqonhpo import ArqonSolver


def test_seed_basic():
    """Verify seed() injects data and increases history length."""
    config = {
        "seed": 42,
        "budget": 100,
        "bounds": {"x": {"min": -5.0, "max": 5.0, "scale": "Linear"}},
        "probe_ratio": 0.2
    }
    solver = ArqonSolver(json.dumps(config))
    
    # Initially empty history
    assert solver.get_history_len() == 0
    
    # Seed with historical data
    seed_data = [{"params": {"x": 0.5}, "value": 10.0, "cost": 1.0}]
    solver.seed(json.dumps(seed_data))
    
    # Verify history length increased
    assert solver.get_history_len() == 1


def test_seed_multiple_points():
    """Seed with multiple points at once."""
    config = {
        "seed": 42,
        "budget": 100,
        "bounds": {"x": {"min": -5.0, "max": 5.0, "scale": "Linear"}},
        "probe_ratio": 0.2
    }
    solver = ArqonSolver(json.dumps(config))
    
    # Seed with 5 historical evaluations
    seed_data = [
        {"params": {"x": float(i)}, "value": float(i**2), "cost": 1.0}
        for i in range(-2, 3)
    ]
    solver.seed(json.dumps(seed_data))
    
    assert solver.get_history_len() == 5


def test_seed_incremental():
    """Seed can be called multiple times incrementally."""
    config = {
        "seed": 42,
        "budget": 100,
        "bounds": {"x": {"min": -5.0, "max": 5.0, "scale": "Linear"}},
        "probe_ratio": 0.2
    }
    solver = ArqonSolver(json.dumps(config))
    
    # First seed
    solver.seed(json.dumps([{"params": {"x": 0.0}, "value": 1.0, "cost": 1.0}]))
    assert solver.get_history_len() == 1
    
    # Second seed
    solver.seed(json.dumps([{"params": {"x": 1.0}, "value": 2.0, "cost": 1.0}]))
    assert solver.get_history_len() == 2
    
    # Third seed with multiple points
    solver.seed(json.dumps([
        {"params": {"x": 2.0}, "value": 3.0, "cost": 1.0},
        {"params": {"x": 3.0}, "value": 4.0, "cost": 1.0}
    ]))
    assert solver.get_history_len() == 4


def test_seed_then_ask():
    """Seeded data should influence subsequent ask() calls.
    
    When seeding enough data to meet the probe budget, the solver
    should transition to Classify->Refine and return new candidates.
    """
    config = {
        "seed": 42,
        "budget": 100,
        "bounds": {"x": {"min": -5.0, "max": 5.0, "scale": "Linear"}},
        "probe_ratio": 0.2  # 20% of 100 = 20 points needed for probe phase
    }
    solver = ArqonSolver(json.dumps(config))
    
    # Seed with enough historical data to meet probe budget (20 points)
    seed_data = [
        {"params": {"x": float(i) / 4.0}, "value": float(i**2), "cost": 1.0}
        for i in range(-10, 10)  # 20 points
    ]
    solver.seed(json.dumps(seed_data))
    
    # Verify we have enough data
    assert solver.get_history_len() == 20
    
    # Ask should now proceed to Classify->Refine and return new candidates
    batch = solver.ask()
    assert batch is not None


def test_seed_multidimensional():
    """Seed works with multiple parameters."""
    config = {
        "seed": 42,
        "budget": 100,
        "bounds": {
            "x": {"min": -5.0, "max": 5.0, "scale": "Linear"},
            "y": {"min": 0.0, "max": 10.0, "scale": "Log"},
            "z": {"min": -1.0, "max": 1.0, "scale": "Linear"}
        },
        "probe_ratio": 0.2
    }
    solver = ArqonSolver(json.dumps(config))
    
    # Seed with multi-dimensional points
    seed_data = [
        {"params": {"x": 1.0, "y": 5.0, "z": 0.0}, "value": 1.5, "cost": 1.0},
        {"params": {"x": -1.0, "y": 2.0, "z": 0.5}, "value": 2.5, "cost": 1.0}
    ]
    solver.seed(json.dumps(seed_data))
    
    assert solver.get_history_len() == 2


def test_seed_invalid_json():
    """Invalid JSON should raise ValueError."""
    config = {
        "seed": 42,
        "budget": 100,
        "bounds": {"x": {"min": -5.0, "max": 5.0, "scale": "Linear"}},
        "probe_ratio": 0.2
    }
    solver = ArqonSolver(json.dumps(config))
    
    with pytest.raises(ValueError) as exc_info:
        solver.seed("not valid json")
    
    assert "Invalid seed data" in str(exc_info.value)


def test_seed_empty_array():
    """Seeding with empty array is allowed (no-op)."""
    config = {
        "seed": 42,
        "budget": 100,
        "bounds": {"x": {"min": -5.0, "max": 5.0, "scale": "Linear"}},
        "probe_ratio": 0.2
    }
    solver = ArqonSolver(json.dumps(config))
    
    solver.seed(json.dumps([]))
    
    # Should not raise, history stays at 0
    assert solver.get_history_len() == 0


def test_seed_missing_field():
    """Missing required field should raise ValueError."""
    config = {
        "seed": 42,
        "budget": 100,
        "bounds": {"x": {"min": -5.0, "max": 5.0, "scale": "Linear"}},
        "probe_ratio": 0.2
    }
    solver = ArqonSolver(json.dumps(config))
    
    # Missing "value" field
    with pytest.raises(ValueError) as exc_info:
        solver.seed(json.dumps([{"params": {"x": 0.5}, "cost": 1.0}]))
    
    assert "Invalid seed data" in str(exc_info.value)
