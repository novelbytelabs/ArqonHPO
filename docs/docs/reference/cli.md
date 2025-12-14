# CLI Reference

ArqonHPO provides a command-line interface for batch optimization.

!!! warning "Under Construction"
    The CLI is planned for a future release. For now, use the Python API.

## Planned Usage

```bash
arqonhpo run --config config.json --script ./evaluate.sh
```

## Config File

`config.json`:

```json
{
  "seed": 42,
  "budget": 100,
  "bounds": {
    "x": {"min": -5, "max": 5}
  }
}
```

## Evaluation Script

The CLI will call your script with parameters as environment variables:

```bash
#!/bin/bash
# evaluate.sh
echo "RESULT=$(python my_simulation.py --x=$ARQON_x)"
```
