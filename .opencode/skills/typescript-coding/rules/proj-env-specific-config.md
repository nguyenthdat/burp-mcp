# proj-env-specific-config

> Keep environment-specific configuration separate from code

## Why It Matters

Hardcoding an API URL, feature flag, or credential directly in source code means every environment change (staging vs. production, a new region) requires a code change and redeploy, and it risks committing secrets to version control. Separating configuration from code — via environment variables, a config service, or environment-specific files loaded at runtime — lets the exact same build artifact run correctly in every environment, which is a prerequisite for reliable CI/CD (build once, promote the same artifact through each stage) and for not leaking secrets into a git history.

## Bad

```typescript
// api-client.ts
const API_BASE_URL = 'https://api-staging.example.com'; // hardcoded, wrong in prod
const STRIPE_KEY = 'sk_live_51H...'; // secret committed to source control

export function getClient() {
  return new ApiClient(API_BASE_URL, STRIPE_KEY);
}
```

## Good

```typescript
// config.ts — reads from environment, validated once at startup
import { z } from 'zod';

const configSchema = z.object({
  API_BASE_URL: z.string().url(),
  STRIPE_SECRET_KEY: z.string().min(1),
});

export const config = configSchema.parse(process.env);

// api-client.ts
import { config } from './config';

export function getClient() {
  return new ApiClient(config.API_BASE_URL, config.STRIPE_SECRET_KEY);
}
```

```bash
# .env.production (not committed; injected by the deploy platform)
API_BASE_URL=https://api.example.com
STRIPE_SECRET_KEY=sk_live_...

# .env.example (committed; documents required variables with placeholder values)
API_BASE_URL=
STRIPE_SECRET_KEY=
```

## Layering Config by Environment

```
config/
  default.json     # shared defaults
  development.json  # overrides for local dev
  production.json   # overrides for prod (no secrets — those stay in env vars)
```

Use a library like `convict` or a simple deep-merge of `default` + `NODE_ENV`-specific file for structured (non-secret) config, while secrets and per-deploy values always come from environment variables or a secrets manager (Vault, AWS Secrets Manager) — never from a file committed to the repo.

## See Also

- [node-env-var-validation](node-env-var-validation.md) - Validate environment variables against a schema at startup
- [proj-lockfile-commit](proj-lockfile-commit.md) - Commit the lockfile for reproducible installs
- [type-zod-schema-inference](type-zod-schema-inference.md) - Derive static types from Zod schemas instead of duplicating them
