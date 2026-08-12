# lint-no-console-prod

> Warn on `console.log()` in production code — use a structured logger instead

## Why It Matters

`console.log()` writes to stdout with no timestamp, severity level, or structured metadata. In production, this produces unsearchable, unaggregatable output that pollutes logs. A structured logger (pino, winston) adds timestamps, levels, and JSON formatting, enabling log aggregation, filtering, and alerting.

## Bad

```js
// Scattered console logs — unsearchable, no levels
console.log('Server started');
console.log('User logged in:', user.email);  // Privacy leak?
console.log('Error:', err);  // Not stderr, no stack trace
console.log('Response time:', ms, 'ms');  // Not structured
```

## Good

```js
// Structured logger — searchable, leveled, formatted
import pino from 'pino';

const logger = pino({
  level: process.env.LOG_LEVEL || 'info',
  transport: process.env.NODE_ENV === 'development'
    ? { target: 'pino-pretty' }
    : undefined,
});

logger.info({ port: 3000 }, 'Server started');
logger.info({ userId: user.id }, 'User logged in');
logger.error({ err, requestId }, 'Request failed');
logger.debug({ ms }, 'Response time');

// ESLint rule to catch console.log
// eslint.config.mjs
{
  rules: {
    'no-console': ['warn', { allow: ['warn', 'error'] }],
  },
}
```

## When Exceptions Apply

`console.warn()` and `console.error()` are acceptable for CLI tools and development scripts. For servers and libraries, use a proper logger.

## See Also

- [lint-eslint-setup](./lint-eslint-setup.md) - ESLint configuration
- [err-global-handlers](./err-global-handlers.md) - Error logging in handlers
