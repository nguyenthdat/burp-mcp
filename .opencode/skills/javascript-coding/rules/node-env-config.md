# node-env-config

> Load environment variables from a single configuration module — not scattered across files

## Why It Matters

Environment variables accessed directly (`process.env.X`) throughout the codebase create hidden dependencies: you can't know what env vars a file needs without reading it. A single config module centralizes all env var access, provides defaults, validates types at startup, and makes the application's configuration surface auditable.

## Bad

```js
// Env vars scattered across many files — hidden dependencies
// server.js
const port = process.env.PORT || 3000;

// database.js
const dbUrl = process.env.DATABASE_URL;

// email.js
const apiKey = process.env.SENDGRID_API_KEY;

// cache.js
const redisUrl = process.env.REDIS_URL || 'redis://localhost:6379';
```

## Good

```js
// config.js — single source of truth for all env vars
function loadConfig() {
  const required = ['DATABASE_URL', 'JWT_SECRET'];
  for (const key of required) {
    if (!process.env[key]) {
      throw new Error(`Missing required env var: ${key}`);
    }
  }

  return {
    port: parseInt(process.env.PORT, 10) || 3000,
    databaseUrl: process.env.DATABASE_URL,
    jwtSecret: process.env.JWT_SECRET,
    sendgridApiKey: process.env.SENDGRID_API_KEY,
    redisUrl: process.env.REDIS_URL || 'redis://localhost:6379',
    logLevel: process.env.LOG_LEVEL || 'info',
    nodeEnv: process.env.NODE_ENV || 'development',
  };
}

export const config = loadConfig();

// Consumer
import { config } from './config.js';
app.listen(config.port);
const db = connect(config.databaseUrl);
```

## When Exceptions Apply

For libraries (code consumed by others), don't read env vars directly — accept configuration as function arguments. Environment variables belong to applications, not libraries.

## See Also

- [type-validate-config](./type-validate-config.md) - Validate config at startup
- [proj-env-files](./proj-env-files.md) - .env.example pattern
