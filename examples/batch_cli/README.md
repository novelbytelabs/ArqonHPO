# Batch CLI Example

```bash
cargo run -p arqonhpo-cli -- run --config examples/batch_cli/config.json --script examples/batch_cli/evaluate.sh --state state.json
```

The command prints JSON results to stdout and persists solver state to `state.json`.
