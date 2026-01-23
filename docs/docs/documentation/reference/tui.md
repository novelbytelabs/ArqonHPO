# Terminal User Interface (TUI)

Optimization happens fast. Sometimes, you need to see it unfold without leaving your terminal.

The ArqonHPO TUI is a zero-latency, high-visibility monitor for your optimization runs. Whether you're debugging a local script or monitoring a job over SSH, the TUI gives you an instant pulse on your solver's progress using a modern, responsive interface.

![ArqonHPO TUI Interface](../../assets/images/tui_screenshot.png)

## Why Use the TUI?

- **Zero Overhead**: runs directly in your terminal; no browser required.
- **Instant Feedback**: Watch the solver explore, converge, and react in real-time.
- **Remote Ready**: Perfect for monitoring headless servers or SSH sessions where web ports aren't forwarded.

---

## The Interface Explained

The TUI is divided into three logical panels, designed to give you the complete picture at a glance:

### 1. Summary Phase (Top)

_The Pulse._
This panel shows the high-level health of your run.

- **Run ID**: The unique identifier for this optimization session (useful for tracking logs).
- **Budget**: How many evaluations are left before the solver stops.
- **History**: The total number of points evaluated so far.

### 2. Recent Evaluations (Middle)

_The Action._
This is where the work happens. It lists the most recent parameter combinations the solver has tried, along with their results.

- **Value**: The objective function result (lower is better for minimization).
- **Params**: The hyperparameters chosen for that evaluation (e.g., `learning_rate`, `batch_size`).
- **Color Coding**:
  - <span style="color: #00ff00">Green</span> values indicate a **new best** finding.
  - Standard white/gray values indicate exploration or non-optimal points.

### 3. Events Stream (Bottom)

_The Narrative._
While evaluations show _what_ happened, events tell you _why_.

- **Convergence Warnings**: When the solver detects it's stuck in a local minima.
- **Phase Shifts**: When the algorithm switches strategies (e.g., from "Probe" to "Refine").
- **Errors**: Immediate feedback on script failures or guardrail violations.

---

## Starting the TUI

To launch the TUI, point it at your solver's state file:

```bash
arqonhpo tui --state state.json
```

If you also want to see the live event stream (highly recommended for debugging), include the events file:

```bash
arqonhpo tui --state state.json --events events.jsonl
```

| Flag           | Description                                                               |
| -------------- | ------------------------------------------------------------------------- |
| `--state`      | **Required.** Path to the JSON state file updated by `arqonhpo ask/tell`. |
| `--events`     | **Optional.** Path to the JSONL events log for rich narrative feedback.   |
| `--refresh-ms` | Update frequency (default: 500ms). Increase for slow SSH connections.     |

---

## Controls & Keybindings

Navigate the interface without touching your mouse.

| Key          | Action            | Context                                          |
| ------------ | ----------------- | ------------------------------------------------ |
| `q` or `Esc` | **Quit**          | Exit the TUI immediately.                        |
| `r`          | **Force Refresh** | Manually reload the state and events files.      |
| `p`          | **Pause**         | Freeze auto-refresh to inspect a specific value. |
| `↑` / `↓`    | **Scroll**        | Scroll through history or logs when paused.      |

---

## configuration & Performance

### Refresh Rate

By default, the TUI polls for changes every **500 milliseconds**.

- **Local Development**: Lower it to `100`ms for a smoother, "matrix-like" feel.

  ```bash
  arqonhpo tui --state state.json --refresh-ms 100
  ```

- **High-Latency SSH**: Raise it to `2000`ms (2 seconds) to reduce bandwidth usage.
  ```bash
  arqonhpo tui --state state.json --refresh-ms 2000
  ```

### Data Sources

The TUI is **stateless**. It simply visualizes the files on disk. This means:

1. You can stop/start the TUI without affecting the running optimization.
2. You can run multiple TUI instances watching the same file (e.g., one on a big screen, one on your laptop).

### Monitoring Multiple Experiments

Since the TUI is bound to a single state file, it displays **one run per window**.

To monitor multiple concurrent experiments:

1. Ensure each experiment writes to a unique state file (e.g., `run_A.json`, `run_B.json`).
2. Open separate terminal tabs or use a multiplexer like `tmux`.
3. Launch a dedicated TUI instance for each run:

   ```bash
   # Terminal 1
   arqonhpo tui --state run_A.json

   # Terminal 2
   arqonhpo tui --state run_B.json
   ```

---

## Next Steps

- **[CLI Reference](cli.md)**: Learn how to generate the state files the TUI consumes.
- **[Dashboard](dashboard.md)**: Prefer a browser? Check out the web-based dashboard for charts and graphs.

---
