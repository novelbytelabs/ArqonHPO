
import os
import matplotlib.pyplot as plt
import numpy as np
import pandas as pd
from collections import defaultdict
from .schema import *

def generate_plots_with_runs(runs_file, trace_file, output_dir):
    print("Generating plots...")
    if not os.path.exists(output_dir):
        os.makedirs(output_dir)

    try:
        runs_df = pd.read_csv(runs_file)
        trace_df = pd.read_csv(trace_file)
    except Exception as e:
        print(f"Skipping plot generation (pandas error): {e}")
        return
    
    # Filter only OK runs
    valid_runs = runs_df[runs_df[COL_STATUS] == "ok"]
    run_ids = set(valid_runs[COL_RUN_ID])
    trace_df = trace_df[trace_df[COL_RUN_ID].isin(run_ids)]
    
    if trace_df.empty:
        print("No valid trace data to plot.")
        return

    # Merge metadata to trace
    # We need: Workload, Profile, Method, Threshold
    meta_cols = [COL_RUN_ID, COL_WORKLOAD, COL_PROFILE_NAME, COL_METHOD, COL_THRESHOLD]
    merged = trace_df.merge(valid_runs[meta_cols], on=COL_RUN_ID)
    
    # Group by Workload + Profile
    groups = merged.groupby([COL_WORKLOAD, COL_PROFILE_NAME])
    
    for (workload, profile), group in groups:
        # 1. Best vs Time Plot
        try:
            plot_best_vs_time(group, workload, profile, output_dir)
        except Exception as e:
             print(f"Error plotting best_vs_time for {workload}: {e}")
        
        # 2. CDF Time to Threshold
        try:
            # We need hit times.
            # Get hits from group
            hits = group[group[COL_HIT_THRESHOLD] == 1]
            if not hits.empty:
                # First hit per run
                first_hits = hits.sort_values(COL_TIME_MS_ELAPSED).groupby(COL_RUN_ID).first()
                # We need all runs metadata for CDF denominator
                plot_cdf(first_hits, valid_runs, workload, profile, output_dir)
        except Exception as e:
            print(f"Error plotting CDF for {workload}: {e}")

def plot_best_vs_time(group, workload, profile, output_dir):
    plt.figure(figsize=(10, 6))
    plt.title(f"Best vs Time: {workload} ({profile})")
    plt.xlabel("Time (ms)")
    plt.ylabel("Best So Far (Log Scale)" if "rosenbrock" in workload or "sphere" in workload else "Best So Far")
    # Heuristic for log scale: strict positive values usually imply approx to 0
    # But let's check values?
    # Simple logic: use log scale if values > 0 and range is large.
    # For now, default linear unless verified. User requested plots, didn't enforce scale.
    # Actually typically HPO is log regret. But we plot raw value.
    # Let's stick to linear or symlog.
    plt.yscale("symlog") 
    plt.grid(True, which="both", ls="-", alpha=0.2)
    
    methods = group[COL_METHOD].unique()
    colors = {"arqon_full": "blue", "optuna_tpe": "orange", "random": "gray", "arqon_probe_only": "purple"}
    
    for method in methods:
        sub = group[group[COL_METHOD] == method]
        max_time = sub[COL_TIME_MS_ELAPSED].max()
        if pd.isna(max_time): continue
        
        # Create bins
        bins = np.linspace(0, max_time, 100)
        
        # For each run, get best curve
        run_curves = []
        for rid, rdata in sub.groupby(COL_RUN_ID):
            rdata = rdata.sort_values(COL_TIME_MS_ELAPSED)
            t = rdata[COL_TIME_MS_ELAPSED].values
            v = rdata[COL_BEST_SO_FAR].values
            
            # Interpolate (Step function, "previous")
            # numpy.interp is linear. We want previous.
            # Use searchsorted
            idx = np.searchsorted(t, bins, side='right') - 1
            # handle idx=-1 (before first point) -> use first point value 
            idx = np.maximum(idx, 0)
            interp_v = v[idx]
            run_curves.append(interp_v)
            
        run_curves = np.array(run_curves)
        if len(run_curves) == 0: continue
        
        median = np.median(run_curves, axis=0)
        p25 = np.percentile(run_curves, 25, axis=0)
        p75 = np.percentile(run_curves, 75, axis=0)
        
        c = colors.get(method, "black")
        plt.plot(bins, median, label=method, color=c)
        plt.fill_between(bins, p25, p75, color=c, alpha=0.2)
        
    plt.legend()
    plt.savefig(f"{output_dir}/{workload}__{profile}__best_vs_time.png")
    plt.close()

def plot_cdf(first_hits, all_runs, workload, profile, output_dir):
    # first_hits: DataFrame with index=RUN_ID, cols=time_ms_elapsed, etc. (for Hits only)
    # all_runs: runs info to get Total N per method
    
    plt.figure(figsize=(10, 6))
    plt.title(f"CDF Time to Threshold: {workload} ({profile})")
    plt.xlabel("Time (ms)")
    plt.ylabel("Probability of Hit")
    plt.grid(True)
    
    subset_runs = all_runs[(all_runs[COL_WORKLOAD] == workload) & (all_runs[COL_PROFILE_NAME] == profile)]
    methods = subset_runs[COL_METHOD].unique()
    colors = {"arqon_full": "blue", "optuna_tpe": "orange", "random": "gray", "arqon_probe_only": "purple"}

    for method in methods:
        n_total = len(subset_runs[subset_runs[COL_METHOD] == method])
        if n_total == 0: continue
        
        meth_hits = first_hits[first_hits[COL_METHOD] == method]
        hit_times = sorted(meth_hits[COL_TIME_MS_ELAPSED].values)
        
        # CDF
        # Points: (t, k/N)
        # We start at 0,0 ? No, start at first hit.
        y_vals = np.arange(1, len(hit_times) + 1) / n_total
        hit_rate = len(hit_times) / n_total
        label = f"{method} (hit={hit_rate:.2f})"
        
        c = colors.get(method, "black")
        if len(hit_times) > 0:
            # Append max time point to extend line?
            # Or just step plot
            plt.step(hit_times, y_vals, where='post', label=label, color=c)
        else:
            plt.plot([], [], label=label, color=c) 
            
    plt.legend(loc="lower right")
    plt.savefig(f"{output_dir}/{workload}__{profile}__cdf_time_to_threshold.png")
    plt.close()
