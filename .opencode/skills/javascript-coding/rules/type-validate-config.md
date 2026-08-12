# type-validate-config

> Validate environment variables and configuration at application startup

## Why It Matters

Applications that access invalid or missing configuration crash with cryptic errors deep in the code, sometimes after running for hours. Validating configuration at startup fails fast with clear error messages, preventing runtime surprises. This is especially critical for environment variables (12-factor app pattern) which have no compile-time checks.

## Bad

```js
// No validation — fails at runtime with cryptic errors
import express from 'express';

const app = express();
const port = process.env.PORT;  // undefined → NaN
const dbUrl = process.env.DATABASE_URL;

app.listen(port, () => {
  console.log(`Server running on port ${port}`);  // "Server running on port undefined"
});

const db = connect(dbUrl);  // Error: cannot connect to undefined
```

## Good

```js
import { z } from 'zod';

const EnvSchema = z.object({
  PORT: z.coerce.number().int().positive().default(3000),
  DATABASE_URL: z.string().url(),
  REDIS_URL: z.string().url().optional(),
  JWT_SECRET: z.string().min(32),
  NODE_ENV: z.enum(['development', 'production', 'test']).default('development'),
  LOG_LEVEL: z.enum(['debug', 'info', 'warn', 'error']).default('info'),
});

const env = EnvSchema.parse(process.env);

// env is fully typed and validated
const app = express();
app.listen(env.PORT, () => {
  console.log(`Server running on port ${env.PORT}`);
});

const db = connect(env.DATABASE_URL);
```

## Manual Validation (Without Libraries)

```js
function validateEnv() {
  const required = ['DATABASE_URL', 'JWT_SECRET'];
  const missing = required.filter(key => !process.env[key]);

  if (missing.length > 0) {
    throw new Error(`Missing required environment variables: ${missing.join(', ')}`);
  }

  return {
    port: parseInt(process.env.PORT, 10) || 3000,
    databaseUrl: process.env.DATABASE_URL,
    jwtSecret: process.env.JWT_SECRET,
    nodeEnv: process.env.NODE_ENV || 'development',
  };
}

export const config = validateEnv();
```

## When Exceptions Apply

For quick scripts and local development tools, skip validation. For any service that runs in production, validate configuration at startup.

## See Also

- [node-env-config](./node-env-config.md) - Load env from a single module
- [type-zod-validation](./type-zod-validation.md) - Schema validation with zod
