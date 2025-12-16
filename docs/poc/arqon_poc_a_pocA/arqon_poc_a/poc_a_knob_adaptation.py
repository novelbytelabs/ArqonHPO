
"""
POC-A: Knob adaptation under drift with Tier-1 safety executor.

Run:
  python poc_a_knob_adaptation.py --steps 4000 --seed 42

Files:
  out/poc_a_run.png
  out/audit_log.jsonl
  out/history.csv
"""

from __future__ import annotations
from dataclasses import dataclass, asdict
from typing import Dict, List, Tuple, Any
import argparse
import json
import math
import os
import time

import numpy as np
import pandas as pd


# -----------------------------
# Deterministic RNG
# -----------------------------
class RNG:
    def __init__(self, seed: int):
        self.rs = np.random.RandomState(seed)

    def randn(self) -> float:
        return float(self.rs.randn())

    def randint(self, lo: int, hi: int) -> int:
        return int(self.rs.randint(lo, hi))

    def uniform(self, lo: float, hi: float) -> float:
        return float(lo + (hi - lo) * self.rs.rand())


# -----------------------------
# Knobs (continuous) we adapt
# -----------------------------
KNOBS = [
    "pool_size",          # connection pool sizing (continuous proxy)
    "cache_ratio",        # cache aggressiveness
    "prefetch",           # prefetch level
    "quant_aggr",         # quantization aggressiveness (0..1): higher => faster, less accurate
    "scheduler_aggr",     # scheduler aggressiveness (0..1): higher => lower latency, more error risk under drift
    "timeout_slack",      # slack to reduce errors under drift but can increase latency
]


@dataclass(frozen=True)
class KnobConfig:
    pool_size: float
    cache_ratio: float
    prefetch: float
    quant_aggr: float
    scheduler_aggr: float
    timeout_slack: float


# -----------------------------
# Data plane (simulated service/universe)
# -----------------------------
class DataPlane:
    """
    Produces request telemetry under a drifting environment.

    Environment signals:
      - load(t): increases latency pressure
      - drift(t): increases error pressure
      - hw(t): hardware speed multiplier (changes cost/latency)

    Knobs trade off latency vs error vs cost.

    This is a proxy for a real system where:
      - latency/cost are measured from live traffic,
      - quality/error is measured via proxy score / acceptance / reward model,
      - conditions drift over time.
    """
    def __init__(self, seed: int):
        self.rng = RNG(seed)
        self.t = 0

        # accumulators for digest window
        self._lat_samples: List[float] = []
        self._err_samples: List[float] = []
        self._cost_samples: List[float] = []
        self._viol = 0

    def environment(self, t: int) -> Dict[str, float]:
        # load oscillates with occasional spikes
        load = 0.6 + 0.25 * math.sin(t / 120.0) + 0.15 * math.sin(t / 23.0)
        if (t % 900) in range(560, 680):
            load += 0.45  # spike window

        # data drift turns on in windows
        drift = 0.15 + 0.10 * (1.0 + math.sin(t / 160.0))
        if (t % 800) in range(300, 520):
            drift += 0.35

        # hardware speed multiplier (lower => slower hardware)
        hw = 1.0
        if (t % 1100) in range(740, 920):
            hw = 0.78  # "throttling" window

        # SLA / budget drift
        sla_p99_ms = 140.0 if load < 0.95 else 120.0
        cost_budget = 1.25 if load < 0.95 else 1.05

        return {"load": float(load), "drift": float(drift), "hw": float(hw),
                "sla_p99_ms": float(sla_p99_ms), "cost_budget": float(cost_budget)}

    def _request_metrics(self, cfg: KnobConfig, env: Dict[str, float]) -> Tuple[float, float, float]:
        """
        Return (latency_ms, error_rate, cost_units) for one synthetic request.
        """
        load, drift, hw = env["load"], env["drift"], env["hw"]

        # Effective capacity rises with pool_size and scheduler aggressiveness; cache helps too
        capacity = (cfg.pool_size ** 0.75) * (0.8 + 0.7 * cfg.scheduler_aggr) * (0.9 + 0.6 * cfg.cache_ratio)

        # Latency base affected by hw; prefetch reduces latency but increases cost; quant reduces latency
        base_lat = 55.0 / max(0.55, hw)
        queueing = 80.0 * max(0.0, load - 0.65) / max(1e-3, capacity)
        cache_help = 18.0 * (1.0 - cfg.cache_ratio)
        quant_help = 22.0 * (1.0 - cfg.quant_aggr)  # low quant_aggr => slower
        prefetch_help = 10.0 * (1.0 - math.tanh(cfg.prefetch / 3.0))

        # Timeout slack increases latency slightly but can reduce errors
        slack_cost_lat = 12.0 * cfg.timeout_slack

        lat_ms = base_lat + queueing + cache_help + quant_help + prefetch_help + slack_cost_lat

        # Noise in latency (tail risk grows with load)
        tail = (10.0 + 25.0 * max(0.0, load - 0.8)) * abs(self.rng.randn())
        lat_ms += tail

        # Error rate: drift increases errors; quant increases errors; aggressive scheduling increases errors under drift;
        # timeout slack reduces errors.
        base_err = 0.012
        drift_err = 0.060 * drift
        quant_err = 0.030 * (cfg.quant_aggr ** 1.5)
        sched_err = 0.020 * (cfg.scheduler_aggr ** 1.2) * drift
        timeout_help = 0.030 * math.tanh(cfg.timeout_slack / 0.7)

        err = base_err + drift_err + quant_err + sched_err - timeout_help

        # Cache and prefetch reduce errors a bit (better hit rate, fewer misses)
        err -= 0.007 * math.tanh(cfg.cache_ratio / 0.5)
        err -= 0.005 * math.tanh(cfg.prefetch / 2.5)

        # Bound error
        err = float(np.clip(err, 0.001, 0.25))

        # Cost: higher pool, cache, prefetch cost more; quant reduces cost; hardware throttling increases cost
        cost = (0.55
                + 0.16 * (cfg.pool_size ** 0.6)
                + 0.25 * cfg.cache_ratio
                + 0.18 * math.tanh(cfg.prefetch / 3.0)
                - 0.20 * cfg.quant_aggr)
        cost *= (1.05 / hw)

        # noise in cost
        cost *= (1.0 + 0.03 * self.rng.randn())
        cost = float(max(0.1, cost))

        return float(lat_ms), float(err), float(cost)

    def step(self, cfg: KnobConfig, requests_per_step: int = 8) -> None:
        env = self.environment(self.t)

        # simulate a small bundle of requests (cheap)
        for _ in range(requests_per_step):
            lat, err, cost = self._request_metrics(cfg, env)
            self._lat_samples.append(lat)
            self._err_samples.append(err)
            self._cost_samples.append(cost)

        # hard constraint counting (for rollback triggers)
        p99 = np.percentile(self._lat_samples[-requests_per_step:], 99)
        if p99 > env["sla_p99_ms"]:
            self._viol += 1
        if np.mean(self._cost_samples[-requests_per_step:]) > env["cost_budget"]:
            self._viol += 1

        self.t += 1

    def drain_digest(self) -> Dict[str, float]:
        if len(self._lat_samples) == 0:
            return {"n": 0}

        lat_arr = np.array(self._lat_samples, dtype=np.float64)
        err_arr = np.array(self._err_samples, dtype=np.float64)
        cost_arr = np.array(self._cost_samples, dtype=np.float64)

        digest = {
            "t": float(self.t),
            "n": float(len(lat_arr)),
            "lat_p50_ms": float(np.percentile(lat_arr, 50)),
            "lat_p95_ms": float(np.percentile(lat_arr, 95)),
            "lat_p99_ms": float(np.percentile(lat_arr, 99)),
            "err_rate": float(np.mean(err_arr)),
            "cost": float(np.mean(cost_arr)),
            "violations": float(self._viol),
        }

        # reset accumulators
        self._lat_samples.clear()
        self._err_samples.clear()
        self._cost_samples.clear()
        self._viol = 0
        return digest

    def shadow_rollout(self, cfg: KnobConfig, horizon_steps: int, seed_offset: int) -> Dict[str, float]:
        """
        Cheap evaluation proxy (shadow/replay).
        Deterministic: uses a fixed seed based on current time-step + offset.
        """
        shadow = DataPlane(seed=10_000 + seed_offset + int(self.t))
        shadow.t = self.t
        for _ in range(horizon_steps):
            shadow.step(cfg, requests_per_step=6)
        return shadow.drain_digest()


# -----------------------------
# Tier 1: deterministic executor
# -----------------------------
@dataclass
class Guardrails:
    bounds: Dict[str, Tuple[float, float]]
    max_delta: Dict[str, float]
    min_apply_interval_steps: int = 8
    max_bad_digests_before_snapback: int = 2

    # rollback thresholds
    lat_p99_rollback_ms: float = 165.0
    err_rollback: float = 0.14
    cost_rollback: float = 1.40
    violations_rollback: float = 1.0


class AuditLog:
    def __init__(self, out_path: str):
        self.out_path = out_path
        self._fh = open(out_path, "w", encoding="utf-8")
        self.events = 0

    def emit(self, etype: str, **fields: Any) -> None:
        evt = {"type": etype, "ts": time.time(), **fields}
        self._fh.write(json.dumps(evt) + "\n")
        self._fh.flush()
        self.events += 1

    def close(self):
        try:
            self._fh.close()
        except Exception:
            pass


class Tier1Executor:
    def __init__(self, guardrails: Guardrails, baseline: KnobConfig, audit: AuditLog):
        self.g = guardrails
        self.baseline = baseline
        self.current = baseline
        self.audit = audit

        self._last_apply_t = 0
        self._bad_streak = 0

        # allowlist equals keys in bounds
        self.allow = set(self.g.bounds.keys())

    def _clip(self, k: str, v: float) -> float:
        lo, hi = self.g.bounds[k]
        return float(np.clip(v, lo, hi))

    def validate_and_apply(self, proposal: Dict[str, float], step_t: int) -> KnobConfig:
        # rate limit
        if step_t - self._last_apply_t < self.g.min_apply_interval_steps:
            self.audit.emit("reject_rate_limit", step_t=step_t, proposal=proposal, current=asdict(self.current))
            return self.current

        cur = asdict(self.current)
        applied = dict(cur)

        # enforce allowlist + bounds + max-delta per step
        for k, v in proposal.items():
            if k not in self.allow:
                continue
            v = float(v)
            v = self._clip(k, v)

            delta = v - float(cur[k])
            maxd = float(self.g.max_delta[k])
            if abs(delta) > maxd:
                v = float(cur[k]) + math.copysign(maxd, delta)
                v = self._clip(k, v)

            applied[k] = float(v)

        self.current = KnobConfig(**applied)
        self._last_apply_t = step_t
        self.audit.emit("apply", step_t=step_t, proposal=proposal, applied=applied)
        return self.current

    def observe_and_maybe_rollback(self, digest: Dict[str, float], env: Dict[str, float]) -> KnobConfig:
        # deterministic rollback: compare to hard thresholds and current SLA/budget
        bad = False
        if digest["lat_p99_ms"] > max(self.g.lat_p99_rollback_ms, env["sla_p99_ms"] + 35.0):
            bad = True
        if digest["err_rate"] > self.g.err_rollback:
            bad = True
        if digest["cost"] > max(self.g.cost_rollback, env["cost_budget"] + 0.25):
            bad = True
        if digest["violations"] > self.g.violations_rollback:
            bad = True

        if bad:
            self._bad_streak += 1
            self.audit.emit("bad_digest", digest=digest, env=env, streak=self._bad_streak, current=asdict(self.current))
        else:
            self._bad_streak = 0

        if self._bad_streak >= self.g.max_bad_digests_before_snapback:
            self.audit.emit("snapback", reason="bad_digest_streak", baseline=asdict(self.baseline))
            self.current = self.baseline
            self._bad_streak = 0

        return self.current


# -----------------------------
# Tier 2: SPSA optimizer (multi-knob)
# -----------------------------
class SPSA:
    """
    Multi-dimensional SPSA:
      - 2 evaluations per update (independent of dimension)
      - deterministic with seed

    Minimizes objective J(cfg, env), computed from shadow digest.
    """
    def __init__(self, seed: int, a: float, c: float):
        self.rng = RNG(seed)
        self.a = float(a)
        self.c = float(c)

    def _perturb(self, cfg: KnobConfig) -> Dict[str, float]:
        # delta_i ∈ {+1,-1}
        return {k: (1.0 if self.rng.randint(0, 2) == 1 else -1.0) for k in KNOBS}

    def propose(
        self,
        dp: DataPlane,
        cfg: KnobConfig,
        env: Dict[str, float],
        horizon: int,
        objective_weights: Dict[str, float],
        seed_offset: int,
    ) -> Dict[str, float]:
        base = asdict(cfg)
        delta = self._perturb(cfg)
        c = self.c

        # plus/minus configs
        plus = {k: base[k] + c * delta[k] for k in KNOBS}
        minus = {k: base[k] - c * delta[k] for k in KNOBS}

        cfg_plus = KnobConfig(**plus)
        cfg_minus = KnobConfig(**minus)

        d_plus = dp.shadow_rollout(cfg_plus, horizon_steps=horizon, seed_offset=seed_offset + 1)
        d_minus = dp.shadow_rollout(cfg_minus, horizon_steps=horizon, seed_offset=seed_offset + 2)

        # objective from digest (penalize constraints)
        def J(d: Dict[str, float]) -> float:
            # penalties for violating SLA/budget; lexicographic-ish via large multipliers
            lat_pen = max(0.0, d["lat_p99_ms"] - env["sla_p99_ms"])
            cost_pen = max(0.0, d["cost"] - env["cost_budget"])
            viol_pen = d["violations"]

            return (
                objective_weights["err"] * d["err_rate"]
                + objective_weights["lat"] * (d["lat_p99_ms"] / max(1.0, env["sla_p99_ms"]))
                + objective_weights["cost"] * (d["cost"] / max(1e-6, env["cost_budget"]))
                + objective_weights["lat_pen"] * lat_pen
                + objective_weights["cost_pen"] * cost_pen
                + objective_weights["viol_pen"] * viol_pen
            )

        y_plus = float(J(d_plus))
        y_minus = float(J(d_minus))

        # SPSA gradient estimate and update
        proposal: Dict[str, float] = {}
        for k in KNOBS:
            g_hat = (y_plus - y_minus) / (2.0 * c * delta[k])
            proposal[k] = float(base[k] - self.a * g_hat)

        # attach debug for inspection (optional)
        proposal["_debug"] = {
            "y_plus": y_plus, "y_minus": y_minus,
            "plus": d_plus, "minus": d_minus,
        }
        return proposal


# -----------------------------
# Orchestrator
# -----------------------------
def default_guardrails() -> Guardrails:
    bounds = {
        "pool_size": (1.0, 20.0),
        "cache_ratio": (0.0, 1.0),
        "prefetch": (0.0, 6.0),
        "quant_aggr": (0.0, 1.0),
        "scheduler_aggr": (0.0, 1.0),
        "timeout_slack": (0.0, 1.6),
    }
    max_delta = {
        "pool_size": 1.2,
        "cache_ratio": 0.08,
        "prefetch": 0.5,
        "quant_aggr": 0.08,
        "scheduler_aggr": 0.08,
        "timeout_slack": 0.12,
    }
    return Guardrails(bounds=bounds, max_delta=max_delta)


def run(
    seed: int,
    steps: int,
    digest_every: int,
    shadow_horizon: int,
    out_dir: str
) -> None:
    os.makedirs(out_dir, exist_ok=True)

    dp = DataPlane(seed=seed)
    audit = AuditLog(os.path.join(out_dir, "audit_log.jsonl"))

    # Baseline: intentionally "meh" to show adaptation
    baseline = KnobConfig(
        pool_size=4.5,
        cache_ratio=0.35,
        prefetch=1.0,
        quant_aggr=0.25,
        scheduler_aggr=0.35,
        timeout_slack=0.25,
    )

    g = default_guardrails()
    tier1 = Tier1Executor(g, baseline, audit)

    # SPSA settings: small steps, stable
    spsa = SPSA(seed=seed + 7, a=0.16, c=0.06)

    objective_weights = {
        "err": 1.3,
        "lat": 0.4,
        "cost": 0.3,
        "lat_pen": 0.06,    # strong constraint discouragement
        "cost_pen": 0.10,
        "viol_pen": 0.25,
    }

    history: List[Dict[str, float]] = []
    cfg = baseline

    for _ in range(steps):
        dp.step(cfg, requests_per_step=8)

        if dp.t % digest_every == 0:
            env = dp.environment(dp.t)
            digest = dp.drain_digest()

            # Tier 1 rollback check based on last digest
            cfg = tier1.observe_and_maybe_rollback(digest, env)

            # Tier 2 proposes next knob changes using 2 shadow rollouts
            proposal = spsa.propose(
                dp=dp,
                cfg=cfg,
                env=env,
                horizon=shadow_horizon,
                objective_weights=objective_weights,
                seed_offset=dp.t + 1000,
            )

            # remove debug from proposal before Tier 1 apply (keep it out of hot path)
            debug = proposal.pop("_debug", None)
            cfg = tier1.validate_and_apply(proposal, step_t=dp.t)

            row = {
                **{k: float(getattr(cfg, k)) for k in KNOBS},
                **digest,
                **env,
            }
            history.append(row)

    df = pd.DataFrame(history)
    df.to_csv(os.path.join(out_dir, "history.csv"), index=False)

    # summary
    summary = {
        "steps": steps,
        "digest_every": digest_every,
        "digests": int(len(df)),
        "mean_lat_p99_ms": float(df["lat_p99_ms"].mean()) if len(df) else None,
        "mean_err_rate": float(df["err_rate"].mean()) if len(df) else None,
        "mean_cost": float(df["cost"].mean()) if len(df) else None,
        "total_violations": int(df["violations"].sum()) if len(df) else None,
        "snapbacks": sum(1 for line in open(os.path.join(out_dir, "audit_log.jsonl"), "r", encoding="utf-8")
                         if '"type": "snapback"' in line),
    }
    with open(os.path.join(out_dir, "summary.json"), "w", encoding="utf-8") as f:
        json.dump(summary, f, indent=2)

    # plot
    import matplotlib.pyplot as plt
    fig = plt.figure(figsize=(12, 10))

    ax1 = fig.add_subplot(4, 1, 1)
    ax1.plot(df["t"], df["lat_p99_ms"])
    ax1.plot(df["t"], df["sla_p99_ms"])
    ax1.set_title("Latency p99 vs SLA (constraint-first)")

    ax2 = fig.add_subplot(4, 1, 2)
    ax2.plot(df["t"], df["err_rate"])
    ax2.plot(df["t"], [0.14]*len(df))
    ax2.set_title("Error rate (rollback threshold shown)")

    ax3 = fig.add_subplot(4, 1, 3)
    ax3.plot(df["t"], df["cost"])
    ax3.plot(df["t"], df["cost_budget"])
    ax3.set_title("Cost vs budget")

    ax4 = fig.add_subplot(4, 1, 4)
    ax4.plot(df["t"], df["pool_size"], label="pool_size")
    ax4.plot(df["t"], df["cache_ratio"], label="cache_ratio")
    ax4.plot(df["t"], df["quant_aggr"], label="quant_aggr")
    ax4.plot(df["t"], df["scheduler_aggr"], label="scheduler_aggr")
    ax4.set_title("Knob trajectories (Tier-2 SPSA, Tier-1 clipped)")
    ax4.legend(ncol=4, fontsize=8)

    fig.tight_layout()
    fig.savefig(os.path.join(out_dir, "poc_a_run.png"), dpi=160)

    audit.close()

    print("=== POC-A complete ===")
    print(json.dumps(summary, indent=2))
    print(f"Outputs in: {out_dir}")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--seed", type=int, default=42)
    ap.add_argument("--steps", type=int, default=4000)
    ap.add_argument("--digest-every", type=int, default=25)
    ap.add_argument("--shadow-horizon", type=int, default=30)
    ap.add_argument("--out-dir", type=str, default="out")
    args = ap.parse_args()

    run(
        seed=args.seed,
        steps=args.steps,
        digest_every=args.digest_every,
        shadow_horizon=args.shadow_horizon,
        out_dir=args.out_dir
    )


if __name__ == "__main__":
    main()
