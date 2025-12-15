
import math
import statistics
from collections import defaultdict
import csv
import os

try:
    import pandas as pd
    HAS_PANDAS = True
except ImportError:
    HAS_PANDAS = False
    
from .schema import *

def aggregate_results(runs_file, trace_file, output_file):
    """
     aggregations:
     - Group by (suite, profile, method, workload, shift_policy)
     - Compute hit rates, median times, etc.
    """
    if HAS_PANDAS:
        _aggregate_pandas(runs_file, trace_file, output_file)
    else:
        _aggregate_pure_python(runs_file, trace_file, output_file)

def _aggregate_pandas(runs_file, trace_file, output_file):
    print("Aggregating with Pandas...")
    runs_df = pd.read_csv(runs_file)
    trace_df = pd.read_csv(trace_file)
    
    # Merge on run_id
    # We need metadata from runs_df attached to trace
    # But for summary, we group by metadata
    
    # Filter completed runs
    valid_runs = runs_df[runs_df[COL_STATUS] == "ok"]
    run_ids = set(valid_runs[COL_RUN_ID])
    
    # Filter trace
    trace_df = trace_df[trace_df[COL_RUN_ID].isin(run_ids)]
    
    # Calculate per-run metrics
    # 1. Best at budget (min of last best_so_far)
    run_best_budget = trace_df.groupby(COL_RUN_ID)[COL_BEST_SO_FAR].min()
    
    # 2. Best at horizon
    # Get rows where eval_idx <= horizon
    # We need horizon from valid_runs map
    horizon_map = valid_runs.set_index(COL_RUN_ID)[COL_HORIZON].to_dict()
    
    def get_best_at_horizon(grp):
        rid = grp.name
        h = horizon_map.get(rid, 100)
        sub = grp[grp[COL_EVAL_IDX] <= h]
        if sub.empty: return float('inf')
        return sub[COL_BEST_SO_FAR].min()

    run_best_horizon = trace_df.groupby(COL_RUN_ID).apply(get_best_at_horizon)
    
    # 3. Hit stats
    # Filter rows where hit_by_horizon == 1
    hits = trace_df[trace_df[COL_HIT_BY_HORIZON] == 1]
    # For each run, get first hit
    first_hits = hits.sort_values(COL_EVAL_IDX).groupby(COL_RUN_ID).first()
    
    # Combine into run_metrics DF
    run_metrics = pd.DataFrame({
        COL_RUN_ID: valid_runs[COL_RUN_ID],
        "best_budget": valid_runs[COL_RUN_ID].map(run_best_budget),
        "best_horizon": valid_runs[COL_RUN_ID].map(run_best_horizon),
        "is_hit": valid_runs[COL_RUN_ID].isin(first_hits.index),
        "evals_to_hit": valid_runs[COL_RUN_ID].map(first_hits[COL_EVAL_IDX]),
        "time_to_hit": valid_runs[COL_RUN_ID].map(first_hits[COL_TIME_MS_ELAPSED])
    })
    
    # Merge metadata columns from runs_df
    meta_cols = [COL_SUITE_NAME, COL_PROFILE_NAME, COL_METHOD, COL_WORKLOAD, COL_SHIFT_POLICY, COL_THRESHOLD, COL_HORIZON]
    run_combined = run_metrics.merge(valid_runs[[COL_RUN_ID] + meta_cols], on=COL_RUN_ID)
    
    # Group and Aggregate
    grouped = run_combined.groupby(meta_cols)
    
    summary_rows = []
    for name, group in grouped:
        n_runs = len(group)
        n_hits = group["is_hit"].sum()
        hit_rate = n_hits / n_runs if n_runs > 0 else 0.0
        
        # Hit stats (only for hits)
        hit_subset = group[group["is_hit"]]
        if not hit_subset.empty:
            med_evals = hit_subset["evals_to_hit"].median()
            p90_evals = hit_subset["evals_to_hit"].quantile(0.9)
            med_time = hit_subset["time_to_hit"].median()
            p90_time = hit_subset["time_to_hit"].quantile(0.9)
        else:
            med_evals = p90_evals = med_time = p90_time = float('nan')
            
        # Best stats (all runs)
        med_budget = group["best_budget"].median()
        p25_budget = group["best_budget"].quantile(0.25)
        p75_budget = group["best_budget"].quantile(0.75)
        med_horizon = group["best_horizon"].median()
        
        row = dict(zip(meta_cols, name))
        row.update({
            COL_N_RUNS: n_runs,
            COL_HIT_RATE_BY_HORIZON: hit_rate,
            COL_MEDIAN_EVALS_TO_HIT: med_evals,
            COL_P90_EVALS_TO_HIT: p90_evals,
            COL_MEDIAN_TIME_MS_TO_HIT: med_time,
            COL_P90_TIME_MS_TO_HIT: p90_time,
            COL_MEDIAN_BEST_AT_BUDGET: med_budget,
            COL_P25_BEST_AT_BUDGET: p25_budget,
            COL_P75_BEST_AT_BUDGET: p75_budget,
            COL_MEDIAN_BEST_AT_HORIZON: med_horizon
        })
        summary_rows.append(row)
        
    # Write
    summary_df = pd.DataFrame(summary_rows)
    # Ensure all columns present
    for c in SUMMARY_COLUMNS:
        if c not in summary_df.columns:
            summary_df[c] = float('nan')
            
    summary_df[SUMMARY_COLUMNS].to_csv(output_file, index=False)
    print(f"Summary written to {output_file}")


def _aggregate_pure_python(runs_file, trace_file, output_file):
    print("Aggregating with Pure Python (Pandas missing)...")
    
    # 1. Read Runs
    runs = {}
    with open(runs_file, 'r') as f:
        reader = csv.DictReader(f)
        for row in reader:
            if row[COL_STATUS] == 'ok':
                runs[row[COL_RUN_ID]] = row

    # 2. Read Traces & Process per run
    run_stats = defaultdict(lambda: {
        "best_budget": float('inf'), 
        "best_horizon": float('inf'),
        "hit": False, 
        "evals_to_hit": None, 
        "time_to_hit": None
    })
    
    with open(trace_file, 'r') as f:
        reader = csv.DictReader(f)
        for row in reader:
            rid = row[COL_RUN_ID]
            if rid not in runs: continue
            
            val = float(row[COL_BEST_SO_FAR])
            idx = int(row[COL_EVAL_IDX])
            time_ms = float(row[COL_TIME_MS_ELAPSED])
            hit_h = int(row[COL_HIT_BY_HORIZON])
            
            stats = run_stats[rid]
            horizon = int(runs[rid][COL_HORIZON])
            
            # Update bests
            if val < stats["best_budget"]:
                stats["best_budget"] = val
            if idx <= horizon and val < stats["best_horizon"]:
                stats["best_horizon"] = val
                
            # Check hit
            if hit_h == 1 and not stats["hit"]:
                stats["hit"] = True
                stats["evals_to_hit"] = idx
                stats["time_to_hit"] = time_ms

    # 3. Group
    groups = defaultdict(list)
    for rid, rdata in runs.items():
        key = (
            rdata[COL_SUITE_NAME], rdata[COL_PROFILE_NAME], rdata[COL_METHOD], 
            rdata[COL_WORKLOAD], rdata[COL_SHIFT_POLICY], rdata[COL_THRESHOLD], rdata[COL_HORIZON]
        )
        groups[key].append(run_stats[rid])
        
    # 4. Compute Metrics
    summary_rows = []
    for key, stats_list in groups.items():
        (suite, prof, meth, work, pol, thresh, horizon) = key
        
        n_runs = len(stats_list)
        hits = [s for s in stats_list if s["hit"]]
        n_hits = len(hits)
        
        row = {
            COL_SUITE_NAME: suite, COL_PROFILE_NAME: prof, COL_METHOD: meth,
            COL_WORKLOAD: work, COL_SHIFT_POLICY: pol, COL_THRESHOLD: thresh, COL_HORIZON: horizon,
            COL_N_RUNS: n_runs,
            COL_HIT_RATE_BY_HORIZON: n_hits / n_runs if n_runs > 0 else 0
        }
        
        # Hits metrics
        if hits:
            evals = sorted([s["evals_to_hit"] for s in hits])
            times = sorted([s["time_to_hit"] for s in hits])
            row[COL_MEDIAN_EVALS_TO_HIT] = statistics.median(evals)
            
            # P90
            idx90 = int(0.9 * len(evals))
            # simple index selection for p90
            idx90 = min(idx90, len(evals)-1)
            row[COL_P90_EVALS_TO_HIT] = evals[idx90]
            
            row[COL_MEDIAN_TIME_MS_TO_HIT] = statistics.median(times)
            row[COL_P90_TIME_MS_TO_HIT] = times[idx90] # Approx
        else:
             row[COL_MEDIAN_EVALS_TO_HIT] = ""
             row[COL_P90_EVALS_TO_HIT] = ""
             row[COL_MEDIAN_TIME_MS_TO_HIT] = ""
             row[COL_P90_TIME_MS_TO_HIT] = ""

        # Best metrics
        best_b = sorted([s["best_budget"] for s in stats_list])
        best_h = sorted([s["best_horizon"] for s in stats_list])
        
        if best_b:
            row[COL_MEDIAN_BEST_AT_BUDGET] = statistics.median(best_b)
            # p25, p75
            n = len(best_b)
            row[COL_P25_BEST_AT_BUDGET] = best_b[int(0.25*n)]
            row[COL_P75_BEST_AT_BUDGET] = best_b[int(0.75*n)] # Approx
        
        if best_h:
            row[COL_MEDIAN_BEST_AT_HORIZON] = statistics.median(best_h)
            
        summary_rows.append(row)
        
    # Write
    with open(output_file, 'w') as f:
        writer = csv.DictWriter(f, fieldnames=SUMMARY_COLUMNS)
        writer.writeheader()
        for r in summary_rows:
            # fill missing
            for c in SUMMARY_COLUMNS:
                if c not in r: r[c] = ""
            writer.writerow(r)
    print(f"Summary written to {output_file}")
