# Integrations

ArqonHPO integrates with your existing infrastructure.

---

## Monitoring & Observability

| Integration | Status | Description |
|-------------|--------|-------------|
| **Prometheus** | ✅ Built-in | Metrics via `--metrics-addr` |
| **Grafana** | ✅ Compatible | Use Prometheus data source |
| **OpenTelemetry** | 🔜 Planned | v0.4 roadmap |

→ [Observability Guide](../reference/observability.md)

---

## ML Frameworks

| Integration | Status | Description |
|-------------|--------|-------------|
| **Ray Tune** | ✅ Custom Searcher | [Ray Guide](../cookbook/ray.md) |
| **MLflow** | 🔜 Planned | Tracking plugin |
| **Weights & Biases** | 🔜 Planned | Callback |

---

## Web Frameworks

| Integration | Status | Description |
|-------------|--------|-------------|
| **FastAPI** | ✅ Example | [FastAPI Guide](../cookbook/fastapi.md) |
| **Flask** | ✅ Compatible | Similar to FastAPI |
| **Django** | ✅ Compatible | Use management commands |

---

## Infrastructure

| Integration | Status | Description |
|-------------|--------|-------------|
| **Kubernetes** | ✅ Patterns | [K8s Guide](../cookbook/kubernetes.md) |
| **Docker** | ✅ Example | See K8s guide |
| **Helm** | 🔜 Planned | Official chart in v0.4 |

---

## Databases

ArqonHPO stores state in JSON files by default. For persistent storage:

| Integration | Status | Description |
|-------------|--------|-------------|
| **Redis** | ✅ Use `--state` | Store state in Redis via wrapper |
| **PostgreSQL** | ✅ Use artifacts | Import/export via SQL |
| **S3** | ✅ Use artifacts | Store artifacts in S3 |

**Example: S3 State Storage**

```python
import boto3
import json
from arqonhpo import ArqonSolver

s3 = boto3.client('s3')

def load_state(bucket, key):
    resp = s3.get_object(Bucket=bucket, Key=key)
    return resp['Body'].read().decode()

def save_state(bucket, key, state):
    s3.put_object(Bucket=bucket, Key=key, Body=state)

# Load existing or create new
try:
    state = load_state('my-bucket', 'arqon/state.json')
    solver = ArqonSolver.from_state(state)
except:
    solver = ArqonSolver(json.dumps(config))

# ... optimization loop ...

# Save state
save_state('my-bucket', 'arqon/state.json', solver.export())
```

---

## Message Queues

| Integration | Status | Description |
|-------------|--------|-------------|
| **RabbitMQ** | ✅ JSONL | Use interactive mode |
| **Kafka** | ✅ JSONL | Use interactive mode |
| **Redis Streams** | ✅ JSONL | Use interactive mode |

**Example: RabbitMQ**

```python
import pika
import json
import subprocess

connection = pika.BlockingConnection()
channel = connection.channel()

channel.queue_declare(queue='arqon_ask')
channel.queue_declare(queue='arqon_results')

# Start interactive mode
proc = subprocess.Popen(
    ['arqonhpo', 'interactive', '--config', 'config.json'],
    stdin=subprocess.PIPE,
    stdout=subprocess.PIPE
)

def on_result(ch, method, props, body):
    result = json.loads(body)
    proc.stdin.write(json.dumps({"cmd": "tell", "results": [result]}).encode() + b'\n')
    proc.stdin.flush()
    
    # Ask for next
    proc.stdin.write(json.dumps({"cmd": "ask", "batch": 1}).encode() + b'\n')
    proc.stdin.flush()
    response = proc.stdout.readline()
    
    channel.basic_publish(exchange='', routing_key='arqon_ask', body=response)

channel.basic_consume(queue='arqon_results', on_message_callback=on_result)
channel.start_consuming()
```

---

## CI/CD

| Integration | Status | Description |
|-------------|--------|-------------|
| **GitHub Actions** | ✅ CLI | Run optimization in workflows |
| **GitLab CI** | ✅ CLI | Same as GitHub |
| **ArqonShip** | ✅ Built-in | Self-healing CI |

→ [ArqonShip Docs](../../arqonship/index.md)

---

## Next Steps

- [Kubernetes](../cookbook/kubernetes.md) — Production deployment
- [FastAPI](../cookbook/fastapi.md) — REST API service
- [Ray Tune](../cookbook/ray.md) — Distributed optimization
