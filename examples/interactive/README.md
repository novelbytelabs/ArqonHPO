# Interactive Ask/Tell Example

Start the interactive CLI:

```bash
cargo run -p arqonhpo-cli -- interactive --config examples/interactive/config.json
```

Send JSONL commands:

```json
{"cmd":"ask","batch":2}
```

```json
{"cmd":"tell","results":[{"params":{"x":0.1,"y":-0.2},"value":0.9,"cost":1.0}]}
```
