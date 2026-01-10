# Prometheus Integration

Detailed guide for monitoring ArqonHPO with Prometheus.

---

## Enable Metrics

### CLI

```bash
arqonhpo --metrics-addr 127.0.0.1:9898 run --config config.json --script eval.sh
```

### Dashboard Server

```bash
arqonhpo dashboard --state state.json --metrics-addr 127.0.0.1:9898
```

Metrics available at: `http://127.0.0.1:9898/metrics`

---

## Prometheus Configuration

Add to `prometheus.yml`:

```yaml
scrape_configs:
  - job_name: "arqonhpo"
    static_configs:
      - targets: ["localhost:9898"]
    scrape_interval: 15s
    metrics_path: /metrics
```

---

## Available Metrics

### Counters

```promql
# Total operations
arqonhpo_ask_calls_total
arqonhpo_tell_calls_total
arqonhpo_candidates_emitted_total
arqonhpo_results_ingested_total

# Safety events
arqonhpo_violations_total{type="DeltaTooLarge"}
arqonhpo_violations_total{type="RateLimitExceeded"}
arqonhpo_violations_total{type="DirectionFlipViolation"}
arqonhpo_rollbacks_total
```

### Gauges

```promql
# Current state
arqonhpo_history_len
arqonhpo_budget_remaining
arqonhpo_best_value
arqonhpo_config_generation

# SPSA state
arqonhpo_spsa_iteration
arqonhpo_spsa_learning_rate
arqonhpo_spsa_perturbation_scale

# Safety
arqonhpo_safe_mode_active  # 1 = in safe mode
```

### Histograms

```promql
# Latency distributions (seconds)
arqonhpo_eval_duration_seconds_bucket{le="0.001"}
arqonhpo_eval_duration_seconds_bucket{le="0.01"}
arqonhpo_eval_duration_seconds_bucket{le="0.1"}
arqonhpo_eval_duration_seconds_bucket{le="1"}
arqonhpo_eval_duration_seconds_bucket{le="10"}
arqonhpo_eval_duration_seconds_bucket{le="+Inf"}

arqonhpo_ask_duration_seconds_bucket
arqonhpo_apply_duration_seconds_bucket
```

---

## PromQL Examples

### Throughput

```promql
# Evaluations per second
rate(arqonhpo_tell_calls_total[5m])

# Candidates per second
rate(arqonhpo_candidates_emitted_total[5m])
```

### Latency

```promql
# 50th percentile eval latency
histogram_quantile(0.5, rate(arqonhpo_eval_duration_seconds_bucket[5m]))

# 99th percentile eval latency
histogram_quantile(0.99, rate(arqonhpo_eval_duration_seconds_bucket[5m]))

# Average ask latency
rate(arqonhpo_ask_duration_seconds_sum[5m]) / rate(arqonhpo_ask_duration_seconds_count[5m])
```

### Safety

```promql
# Violation rate
rate(arqonhpo_violations_total[5m])

# Violations by type
sum by (type) (rate(arqonhpo_violations_total[5m]))

# Rollback frequency
increase(arqonhpo_rollbacks_total[1h])

# Safe mode duration
changes(arqonhpo_safe_mode_active[1h])
```

### Progress

```promql
# Budget consumption rate
rate(arqonhpo_history_len[5m])

# Time to completion (estimate)
arqonhpo_budget_remaining / rate(arqonhpo_history_len[5m])
```

---

## Grafana Dashboard

### Import JSON

```json
{
  "title": "ArqonHPO Monitoring",
  "panels": [
    {
      "title": "Throughput",
      "type": "graph",
      "targets": [
        {
          "expr": "rate(arqonhpo_tell_calls_total[5m])",
          "legendFormat": "evals/s"
        }
      ]
    },
    {
      "title": "Best Objective",
      "type": "stat",
      "targets": [{ "expr": "arqonhpo_best_value", "legendFormat": "best" }]
    },
    {
      "title": "Eval Latency p99",
      "type": "graph",
      "targets": [
        {
          "expr": "histogram_quantile(0.99, rate(arqonhpo_eval_duration_seconds_bucket[5m]))",
          "legendFormat": "p99"
        }
      ]
    },
    {
      "title": "Violations",
      "type": "graph",
      "targets": [
        {
          "expr": "sum by (type) (rate(arqonhpo_violations_total[5m]))",
          "legendFormat": "{{type}}"
        }
      ]
    },
    {
      "title": "Budget Progress",
      "type": "gauge",
      "targets": [
        {
          "expr": "1 - (arqonhpo_budget_remaining / (arqonhpo_history_len + arqonhpo_budget_remaining))",
          "legendFormat": "progress"
        }
      ]
    },
    {
      "title": "Safe Mode",
      "type": "stat",
      "targets": [
        { "expr": "arqonhpo_safe_mode_active", "legendFormat": "active" }
      ],
      "valueMappings": [
        { "value": 0, "text": "Normal", "color": "green" },
        { "value": 1, "text": "SAFE MODE", "color": "red" }
      ]
    }
  ]
}
```

---

## Alerting Rules

Add to Prometheus rules:

```yaml
groups:
  - name: arqonhpo
    rules:
      - alert: ArqonHighViolationRate
        expr: rate(arqonhpo_violations_total[5m]) > 0.1
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High violation rate in ArqonHPO"
          description: "{{ $value }} violations/s over 5 minutes"

      - alert: ArqonSafeModeActive
        expr: arqonhpo_safe_mode_active == 1
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "ArqonHPO in safe mode"
          description: "Optimization paused due to safety trigger"

      - alert: ArqonHighEvalLatency
        expr: histogram_quantile(0.99, rate(arqonhpo_eval_duration_seconds_bucket[5m])) > 60
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "High evaluation latency"
          description: "p99 latency {{ $value }}s"

      - alert: ArqonStalled
        expr: rate(arqonhpo_tell_calls_total[30m]) == 0
        for: 30m
        labels:
          severity: warning
        annotations:
          summary: "ArqonHPO appears stalled"
          description: "No evaluations in 30 minutes"
```

---

## Kubernetes ServiceMonitor

For Prometheus Operator:

```yaml
apiVersion: monitoring.coreos.com/v1
kind: ServiceMonitor
metadata:
  name: arqonhpo
  labels:
    app: arqonhpo
spec:
  selector:
    matchLabels:
      app: arqonhpo
  endpoints:
    - port: metrics
      interval: 15s
      path: /metrics
```

---

## Next Steps

- [Observability Overview](../reference/observability.md) — All observability features
- [Dashboard](../reference/dashboard.md) — Web UI
- [Kubernetes](../cookbook/kubernetes.md) — K8s deployment
