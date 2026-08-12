# node-cluster-fork

> Use the cluster module or PM2 for multi-core CPU utilization

## Why It Matters

Node.js runs on a single thread. On a multi-core machine, a single process uses only one core, leaving the others idle. The cluster module forks multiple worker processes (one per core) that share a server port. Process managers like PM2 provide clustering plus process monitoring, auto-restart, and zero-downtime reloads.

## Bad

```js
// Single process — uses only 1 CPU core
import { createServer } from 'node:http';

const server = createServer((req, res) => {
  // CPU-bound work here blocks all other requests
  res.end('Hello');
});

server.listen(3000);
```

## Good

```js
// Cluster — one worker per CPU core
import cluster from 'node:cluster';
import { availableParallelism } from 'node:os';
import { createServer } from 'node:http';

if (cluster.isPrimary) {
  const numCPUs = availableParallelism();

  console.log(`Primary ${process.pid} starting ${numCPUs} workers`);

  for (let i = 0; i < numCPUs; i++) {
    cluster.fork();
  }

  cluster.on('exit', (worker, code, signal) => {
    console.log(`Worker ${worker.process.pid} died. Restarting...`);
    cluster.fork();  // Auto-restart
  });
} else {
  const server = createServer((req, res) => {
    res.end(`Hello from worker ${process.pid}`);
  });

  server.listen(3000);
  console.log(`Worker ${process.pid} started`);
}
```

## PM2 (Production-Grade)

```bash
# Start with max processes
pm2 start server.js -i max

# Zero-downtime reload
pm2 reload server.js

# Monitor
pm2 monit

# Generate startup script
pm2 startup
```

## When Exceptions Apply

Clustering isn't needed for I/O-bound services that are bottlenecked on external resources. For containerized deployments (Kubernetes), let the orchestrator manage replicas instead of using the cluster module.

## See Also

- [async-worker-threads](./async-worker-threads.md) - Worker threads for CPU-bound tasks
- [node-signal-handling](./node-signal-handling.md) - Graceful shutdown
