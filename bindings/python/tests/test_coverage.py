"""Tests for ArqonSolver.ask_one() and ArqonProbe for 100% Python binding coverage."""
import json
import pytest
from arqonhpo import ArqonSolver, ArqonProbe


class TestAskOne:
    """Tests for the ask_one() method - online/real-time optimization."""

    def test_ask_one_basic(self):
        """ask_one returns exactly one candidate."""
        config = {
            "seed": 42,
            "budget": 100,
            "bounds": {"x": {"min": -5.0, "max": 5.0, "scale": "Linear"}},
            "probe_ratio": 0.2
        }
        solver = ArqonSolver(json.dumps(config))
        
        candidate = solver.ask_one()
        assert candidate is not None
        assert "x" in candidate
        assert -5.0 <= candidate["x"] <= 5.0

    def test_ask_one_within_bounds(self):
        """ask_one respects parameter bounds."""
        config = {
            "seed": 123,
            "budget": 50,
            "bounds": {
                "alpha": {"min": 0.001, "max": 1.0, "scale": "Log"},
                "beta": {"min": 0.0, "max": 100.0, "scale": "Linear"}
            },
            "probe_ratio": 0.3
        }
        solver = ArqonSolver(json.dumps(config))
        
        for _ in range(5):
            candidate = solver.ask_one()
            if candidate is None:
                break
            assert 0.001 <= candidate["alpha"] <= 1.0
            assert 0.0 <= candidate["beta"] <= 100.0
            
            # Feed back result to advance solver
            result = [{
                "params": candidate, 
                "value": candidate["alpha"] * candidate["beta"], 
                "cost": 1.0
            }]
            solver.seed(json.dumps(result))

    def test_ask_one_online_loop(self):
        """Simulate online optimization with ask_one/seed loop."""
        config = {
            "seed": 42,
            "budget": 10,
            "bounds": {"x": {"min": -2.0, "max": 2.0, "scale": "Linear"}},
            "probe_ratio": 0.5
        }
        solver = ArqonSolver(json.dumps(config))
        
        evaluations = []
        for i in range(8):
            candidate = solver.ask_one()
            if candidate is None:
                break
                
            # Objective: x^2 (minimum at x=0)
            value = candidate["x"] ** 2
            evaluations.append(value)
            
            # Seed the result back
            result = [{"params": candidate, "value": value, "cost": 1.0}]
            solver.seed(json.dumps(result))
        
        # Should have done at least 5 evaluations
        assert len(evaluations) >= 5
        # History should reflect evaluations
        assert solver.get_history_len() >= 5


class TestArqonProbe:
    """Tests for ArqonProbe - stateless LDS sampling."""

    def test_probe_sample_at(self):
        """sample_at returns a valid parameter dict."""
        config = {
            "seed": 42,
            "budget": 100,
            "bounds": {"x": {"min": 0.0, "max": 10.0, "scale": "Linear"}},
            "probe_ratio": 0.3
        }
        probe = ArqonProbe(json.dumps(config), seed=42)
        
        point = probe.sample_at(0)
        assert isinstance(point, dict)
        assert "x" in point
        assert 0.0 <= point["x"] <= 10.0

    def test_probe_sample_range(self):
        """sample_range returns multiple valid points."""
        config = {
            "seed": 42,
            "budget": 100,
            "bounds": {
                "x": {"min": -1.0, "max": 1.0, "scale": "Linear"},
                "y": {"min": 1.0, "max": 100.0, "scale": "Log"}
            },
            "probe_ratio": 0.3
        }
        probe = ArqonProbe(json.dumps(config), seed=42)
        
        points = probe.sample_range(0, 10)
        assert len(points) == 10
        
        for point in points:
            assert "x" in point
            assert "y" in point
            assert -1.0 <= point["x"] <= 1.0
            assert 1.0 <= point["y"] <= 100.0

    def test_probe_deterministic_sampling(self):
        """Two probes with same seed produce identical samples."""
        config_json = json.dumps({
            "seed": 42,
            "budget": 100,
            "bounds": {"x": {"min": 0.0, "max": 1.0, "scale": "Linear"}},
            "probe_ratio": 0.3
        })
        
        probe1 = ArqonProbe(config_json, seed=123)
        probe2 = ArqonProbe(config_json, seed=123)
        
        for i in range(5):
            p1 = probe1.sample_at(i)
            p2 = probe2.sample_at(i)
            assert p1 == p2

    def test_probe_stateless_sharding(self):
        """Different start indices enable sharded sampling."""
        config_json = json.dumps({
            "seed": 42,
            "budget": 100,
            "bounds": {"x": {"min": 0.0, "max": 1.0, "scale": "Linear"}},
            "probe_ratio": 0.3
        })
        
        probe = ArqonProbe(config_json, seed=42)
        
        # Shard 1: indices 0-9
        shard1 = probe.sample_range(0, 10)
        # Shard 2: indices 10-19
        shard2 = probe.sample_range(10, 10)
        
        assert len(shard1) == 10
        assert len(shard2) == 10
        # Should be different points (no overlap)
        assert shard1[0] != shard2[0]

    def test_probe_default_seed(self):
        """ArqonProbe works with default seed."""
        config_json = json.dumps({
            "seed": 42,
            "budget": 100,
            "bounds": {"x": {"min": 0.0, "max": 1.0, "scale": "Linear"}},
            "probe_ratio": 0.3
        })
        
        # Default seed is 42
        probe = ArqonProbe(config_json)
        point = probe.sample_at(0)
        assert 0.0 <= point["x"] <= 1.0


class TestErrorHandling:
    """Tests for error handling in Python bindings."""

    def test_invalid_config_json(self):
        """Invalid JSON config raises ValueError."""
        with pytest.raises(ValueError) as exc_info:
            ArqonSolver("not valid json")
        
        assert "Invalid config" in str(exc_info.value)

    def test_invalid_tell_json(self):
        """Invalid JSON in tell raises ValueError."""
        config = {
            "seed": 42,
            "budget": 100,
            "bounds": {"x": {"min": 0.0, "max": 1.0, "scale": "Linear"}},
            "probe_ratio": 0.3
        }
        solver = ArqonSolver(json.dumps(config))
        
        with pytest.raises(ValueError) as exc_info:
            solver.tell("broken json")
        
        assert "Invalid results" in str(exc_info.value)

    def test_invalid_probe_config(self):
        """Invalid config for ArqonProbe raises ValueError."""
        with pytest.raises(ValueError) as exc_info:
            ArqonProbe("not valid json")
        
        assert "Invalid config" in str(exc_info.value)
