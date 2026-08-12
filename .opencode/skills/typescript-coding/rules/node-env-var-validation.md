# node-env-var-validation

> Validate environment variables against a schema at startup

## Why It Matters

`process.env` is typed as `{ [key: string]: string | undefined }`, so every access is potentially `undefined` at compile time, yet most code casts it away with `process.env.PORT!` or string concatenation, deferring the failure to runtime — often deep inside a request handler, hours after deploy. Validating all required environment variables once at process startup turns a silent `undefined` (or a typo'd variable name) into a loud, immediate crash with a clear error message, before the process starts accepting traffic. It also gives the rest of the codebase a single, fully-typed `env` object instead of scattered `process.env.X` accesses.

## Bad

```typescript
// Scattered, unchecked access throughout the codebase
const port = Number(process.env.PORT); // NaN if unset, no error
const dbUrl = process.env.DATABASE_URL!; // lies to the compiler
const isProd = process.env.NODE_ENV === 'production';

app.listen(port); // may silently listen on port NaN -> 0
```

## Good

```typescript
import { z } from 'zod';

const envSchema = z.object({
  NODE_ENV: z.enum(['development', 'test', 'production']).default('development'),
  PORT: z.coerce.number().int().positive().default(3000),
  DATABASE_URL: z.string().url(),
  LOG_LEVEL: z.enum(['debug', 'info', 'warn', 'error']).default('info'),
});

export type Env = z.infer<typeof envSchema>;

function loadEnv(): Env {
  const result = envSchema.safeParse(process.env);
  if (!result.success) {
    console.error('Invalid environment configuration:', result.error.flatten().fieldErrors);
    process.exit(1); // fail fast, before the server starts accepting traffic
  }
  return result.data;
}

export const env = loadEnv();

// Elsewhere: fully typed, no `!`, no NaN surprises
app.listen(env.PORT);
```

## Setup

```bash
npm install zod
```

Call `loadEnv()` once at the top of your entrypoint (`src/server.ts`), before wiring up routes, database connections, or background jobs — so a missing variable prevents the process from doing any work rather than failing mid-request. For frontend/bundled code (Vite, Next.js), validate at build time as well, since `process.env` access is often statically replaced.

## See Also

- [type-zod-schema-inference](type-zod-schema-inference.md) - Derive static types from Zod schemas instead of duplicating them
- [proj-env-specific-config](proj-env-specific-config.md) - Keep environment-specific configuration separate from code
- [node-process-exit-avoid](node-process-exit-avoid.md) - Avoid `process.exit()` in library code; let the caller control the process
- [err-boundary-validation](err-boundary-validation.md) - Validate external input at the boundary of your system
