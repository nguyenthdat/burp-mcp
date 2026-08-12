# node-structured-logging

> Use structured, leveled logging instead of `console.log`

## Why It Matters

`console.log` produces unstructured text with no severity level, no timestamp discipline, and no machine-parseable shape, so once logs reach a production aggregator (Datadog, CloudWatch, ELK) they can't be filtered by level, correlated by request ID, or queried by field without brittle regex parsing. It's also synchronous to stdout in a TTY, and every call site independently decides how to format objects and errors, producing inconsistent output across a codebase. A structured logger emits JSON with consistent fields (`level`, `time`, `msg`, plus arbitrary context), lets you set a minimum level per environment, and integrates with tracing systems to attach request-scoped context automatically.

## Bad

```typescript
function handleOrder(order: Order) {
  console.log('Processing order', order.id);
  try {
    charge(order);
    console.log('Order charged successfully');
  } catch (err) {
    console.log('Error charging order:', err); // no level, no stack, no context
  }
}
```

## Good

```typescript
import pino from 'pino';

const logger = pino({
  level: process.env.LOG_LEVEL ?? 'info',
  formatters: {
    level: (label) => ({ level: label }),
  },
});

function handleOrder(order: Order) {
  const log = logger.child({ orderId: order.id }); // request-scoped context

  log.info('processing order');
  try {
    charge(order);
    log.info('order charged successfully');
  } catch (err) {
    log.error({ err }, 'failed to charge order'); // structured, includes stack
    throw err;
  }
}
```

```json
{"level":"error","time":1721000000000,"orderId":"ord_123","err":{"type":"Error","message":"card declined","stack":"..."},"msg":"failed to charge order"}
```

## Setup

```bash
npm install pino
npm install --save-dev pino-pretty  # human-readable output in local dev only
```

```typescript
const logger = pino(
  process.env.NODE_ENV === 'development'
    ? { transport: { target: 'pino-pretty' } }
    : {}
);
```

## Level Guide

| Level | Use for |
|---|---|
| `fatal` | Process is about to crash / exit |
| `error` | An operation failed and was not recovered |
| `warn` | Unexpected but handled condition |
| `info` | Significant application events (request served, job completed) |
| `debug` | Detailed diagnostic information for local development |
| `trace` | Very fine-grained, usually disabled in production |

## See Also

- [err-cause-chaining](err-cause-chaining.md) - Preserve the original error as `cause` when wrapping errors
- [doc-inline-why-not-what](doc-inline-why-not-what.md) - Write comments that explain why, not what
- [node-graceful-shutdown](node-graceful-shutdown.md) - Handle `SIGTERM`/`SIGINT` for graceful shutdown
