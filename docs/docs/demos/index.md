# Solutions Overview

ArqonHPO is designed for systems where latency and safety are non-negotiable.

## By Use Case

### 🤖 [Inference Serving](inference.md)
Dynamic batch sizes, KV-cache eviction policies, and router weights. Optimize throughput under p99 latency constraints.

### 🏭 [Systems & SRE](systems.md)
Database connection pools, JVM garbage collection tuning, and admission control queues. Keep systems stable under load.

### 🚁 [Edge & Robotics](edge.md)
Control loops on constrained hardware. 100ns execution time means you can optimize *inside* a 1kHz control loop.
