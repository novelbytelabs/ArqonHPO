# ArqonHPO

**HPO for the AI Era.**

> Modern HPO/NAS tools assume human-time feedback loops: run an experiment, wait, analyze, repeat. But production AI systems live in non-stationary environments—traffic shifts, drift happens, hardware throttles, and constraints change. This work shows that if the optimization loop is fast enough to run at microsecond latency, tuning becomes a control primitive: safe, bounded, auditable adaptation that runs continuously in the runtime, while offline discovery expands the approved search space.

## Documentation

- [Quickstart](docs/quickstart/)
- [Cookbook](docs/cookbook/)
- [API Reference](docs/reference/python/)
- [Architecture POC Walkthrough](https://github.com/novelbytelabs/ArqonHPO/blob/main/docs/poc/)

## Getting Started

```bash
pip install arqonhpo
```

See [README.md](../README.md) for full installation and usage instructions.
