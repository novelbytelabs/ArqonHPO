# Observability

ArqonHPO provides comprehensive observability through structured logs, Prometheus metrics, and real-time dashboards.

---

## Structured Logs

The CLI uses `tracing` and emits structured logs to stderr.

### Enable JSON Output

```bash
arqonhpo --log-format json --log-level info <command>
```

### Log Levels

| Level   | Content                              |
| ------- | ------------------------------------ |
| `error` | Failures only                        |
| `warn`  | Violations, guardrail triggers       |
| `info`  | Ask/tell events, phase changes       |
| `debug` | Strategy decisions, proposal details |
| `trace` | SPSA iterations, config snapshots    |

### Log Fields

| Field        | Description              |
| ------------ | ------------------------ |
| `command`    | Current CLI command      |
| `config`     | Path to config file      |
| `state`      | Path to state file       |
| `artifact`   | Path to artifact file    |
| `phase`      | Current PCR phase        |
| `iteration`  | SPSA iteration count     |
| `generation` | Config generation number |

---

## Prometheus Metrics

Enable metrics endpoint:

```bash
arqonhpo --metrics-addr 127.0.0.1:9898 <command>

# Or with dashboard
arqonhpo dashboard --state state.json --metrics-addr 127.0.0.1:9898
```

Scrape at `http://127.0.0.1:9898/metrics`.

### Counters

| Metric                              | Labels | Description                |
| ----------------------------------- | ------ | -------------------------- |
| `arqonhpo_ask_calls_total`          | —      | Total ask() invocations    |
| `arqonhpo_tell_calls_total`         | —      | Total tell() invocations   |
| `arqonhpo_candidates_emitted_total` | —      | Total candidates generated |
| `arqonhpo_results_ingested_total`   | —      | Total results processed    |
| `arqonhpo_violations_total`         | `type` | Safety violations by type  |
| `arqonhpo_rollbacks_total`          | —      | Rollback operations        |

### Gauges

| Metric                       | Labels | Description                    |
| ---------------------------- | ------ | ------------------------------ |
| `arqonhpo_history_len`       | —      | Current history size           |
| `arqonhpo_budget_remaining`  | —      | Remaining evaluation budget    |
| `arqonhpo_best_value`        | —      | Current best objective value   |
| `arqonhpo_config_generation` | —      | Current config generation      |
| `arqonhpo_spsa_iteration`    | —      | Current SPSA iteration         |
| `arqonhpo_safe_mode_active`  | —      | 1 if in safe mode, 0 otherwise |

### Histograms

| Metric                            | Buckets                  | Description          |
| --------------------------------- | ------------------------ | -------------------- |
| `arqonhpo_eval_duration_seconds`  | 0.001, 0.01, 0.1, 1, 10  | Evaluation latency   |
| `arqonhpo_ask_duration_seconds`   | 0.0001, 0.001, 0.01, 0.1 | Ask latency          |
| `arqonhpo_apply_duration_seconds` | 0.00001, 0.0001, 0.001   | Config apply latency |

### Example Queries (PromQL)

```promql
# Average evaluations per second
rate(arqonhpo_tell_calls_total[5m])

# 99th percentile eval latency
histogram_quantile(0.99, rate(arqonhpo_eval_duration_seconds_bucket[5m]))

# Violation rate by type
rate(arqonhpo_violations_total[5m])

# Config update frequency
rate(arqonhpo_config_generation[1m])

# Safe mode duration
changes(arqonhpo_safe_mode_active[1h])
```

---

## Grafana Dashboard

Import this JSON or build from these panels:

### Recommended Panels

1. **Throughput** — `rate(arqonhpo_tell_calls_total[5m])`
2. **Best Value** — `arqonhpo_best_value`
3. **Eval Latency p99** — `histogram_quantile(0.99, ...)`
4. **Violations** — `rate(arqonhpo_violations_total[5m])` by type
5. **Budget Remaining** — `arqonhpo_budget_remaining`
6. **Safe Mode Status** — `arqonhpo_safe_mode_active`

### Alert Rules

```yaml
groups:
  - name: arqonhpo
    rules:
      - alert: HighViolationRate
        expr: rate(arqonhpo_violations_total[5m]) > 0.1
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High violation rate in ArqonHPO"

      - alert: SafeModeActive
        expr: arqonhpo_safe_mode_active == 1
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "ArqonHPO entered safe mode"
```

---

## Tracing Spans

When `log-level=trace`, spans are emitted:

| Span         | Parent  | Description              |
| ------------ | ------- | ------------------------ |
| `ask`        | —       | Full ask() operation     |
| `probe`      | `ask`   | Probe phase sampling     |
| `classify`   | `ask`   | Landscape classification |
| `refine`     | `ask`   | Strategy execution       |
| `tell`       | —       | Full tell() operation    |
| `apply`      | `tell`  | Config application       |
| `guardrails` | `apply` | Safety checks            |

### OpenTelemetry (Planned)

OTel export planned for v0.4. Track: [Issue #XX](https://github.com/novelbytelabs/ArqonHPO/issues)

---

## TUI Monitoring

Real-time terminal dashboard:

```bash
arqonhpo tui --state state.json --events events.jsonl
```

See [TUI Reference](tui.md) for details.

---

## Web Dashboard

Browser-based monitoring:

```bash
arqonhpo dashboard --state state.json --addr 127.0.0.1:3030
```

See [Dashboard Reference](dashboard.md) for API endpoints.

---

## Audit Events

All safety-relevant events are logged to the audit trail:

| Event             | Trigger                      |
| ----------------- | ---------------------------- |
| `apply_success`   | Config update applied        |
| `apply_rejected`  | Proposal violated guardrails |
| `rollback`        | Reverted to baseline         |
| `safe_mode_enter` | Entered safe mode            |
| `safe_mode_exit`  | Exited safe mode             |
| `baseline_set`    | New baseline established     |

Access via:

- Dashboard API: `GET /api/events`
- CLI export: `arqonhpo export --state state.json`

---

## Next Steps

- [Dashboard](dashboard.md) — Web UI reference
- [Hotpath API](hotpath.md) — Internal telemetry APIs
- [Safety](../concepts/safety.md) — Understanding violations
