import pandas as pd
import matplotlib.pyplot as plt
import json

def plot_history():
    print("Loading history.csv...")
    try:
        df = pd.read_csv("history.csv", names=["step", "objective", "diffusion", "noise", "variant", "engine_us", "total_us"])
    except FileNotFoundError:
        print("history.csv not found")
        return

    fig, axes = plt.subplots(3, 1, figsize=(10, 12), sharex=True)
    
    # Plot 1: Objective & Variant
    ax1 = axes[0]
    ax1.plot(df["step"], df["objective"], label="Objective (Stability)", color="blue")
    ax1.set_ylabel("Quality")
    ax1.legend(loc="upper left")
    ax1.grid(True)
    
    ax1b = ax1.twinx()
    ax1b.plot(df["step"], df["variant"], label="Selected Variant", color="orange", alpha=0.5, drawstyle="steps-post")
    ax1b.set_ylabel("Variant ID")
    ax1b.set_yticks([0, 1])
    ax1b.legend(loc="upper right")
    
    # Plot 2: Tuned Parameters
    ax2 = axes[1]
    ax2.plot(df["step"], df["diffusion"], label="Diffusion Rate", color="green")
    ax2.plot(df["step"], df["noise"], label="Noise Level", color="red")
    ax2.set_ylabel("Parameter Value")
    ax2.legend()
    ax2.grid(True)
    
    # Plot 3: Tier-2 Latency
    ax3 = axes[2]
    ax3.plot(df["step"], df["engine_us"], label="Adaptive Engine (µs)", color="purple")
    ax3.plot(df["step"], df["total_us"], label="Total Tick (µs)", color="gray", alpha=0.3)
    ax3.axhline(y=1000, color="r", linestyle="--", label="Budget (1ms)")
    ax3.set_ylabel("Latency (µs)")
    ax3.set_yscale("log")
    ax3.legend()
    ax3.grid(True)
    
    plt.xlabel("Step")
    plt.suptitle("Arqon Verification Simulation Results")
    plt.tight_layout()
    plt.savefig("sim_results.png")
    print("Saved sim_results.png")

def analyze_audit_log():
    print("\nAudit Log Analysis:")
    try:
        with open("audit_log.jsonl") as f:
            logs = [json.loads(line) for line in f]
    except FileNotFoundError:
        print("audit_log.jsonl not found")
        return

    total = len(logs)
    rejected = sum(1 for l in logs if l.get("status") == "rejected")
    applied = sum(1 for l in logs if l.get("status") == "applied")
    snapbacks = sum(1 for l in logs if l.get("action") == "snapback")
    
    print(f"Total Actions: {total}")
    print(f"Applied: {applied}")
    print(f"Rejected: {rejected}")
    print(f"Snapbacks: {snapbacks}")
    
    if rejected > 0:
        print("\nSample Rejections:")
        for log in [l for l in logs if l.get("status") == "rejected"][:3]:
            print(f"- Step {log['step']}: {log.get('violation', 'Unknown')}")

if __name__ == "__main__":
    plot_history()
    analyze_audit_log()
