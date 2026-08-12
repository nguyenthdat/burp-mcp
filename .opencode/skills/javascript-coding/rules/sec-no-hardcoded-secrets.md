# sec-no-hardcoded-secrets

> Never commit secrets to source code — use environment variables or a secrets manager

## Why It Matters

Hardcoded API keys, database passwords, tokens, and certificates in source code are exposed to anyone with repository access. They persist in git history even after removal, and often leak in logs, error messages, and client-side bundles. Use environment variables for development and a secrets manager (AWS Secrets Manager, HashiCorp Vault, Doppler) for production.

## Bad

```js
// Secrets in source code
const API_KEY = 'sk-live-abc123xyz';
const DB_PASSWORD = 'supersecretpassword';
const JWT_SECRET = 'my-jwt-secret-key';

// Hardcoded in config
const config = {
  stripe: {
    secretKey: 'sk_test_abc123',
  },
};

// Inlined in code
await stripe.charges.create({
  api_key: 'sk_live_xyz789',
});
```

## Good

```js
// Load from environment
const config = {
  apiKey: process.env.API_KEY,
  dbPassword: process.env.DB_PASSWORD,
  jwtSecret: process.env.JWT_SECRET,
};

// Validate at startup
if (!config.apiKey) {
  throw new Error('API_KEY environment variable is required');
}

// From a dedicated config module
import { config } from './config.js';

await stripe.charges.create({
  api_key: config.stripe.secretKey,
});
```

## .env File Hygiene

```bash
# .env — NEVER commit this file
API_KEY=sk-live-abc123
DB_PASSWORD=secret

# .env.example — DO commit this (template only)
API_KEY=
DB_PASSWORD=
JWT_SECRET=
```

```gitignore
# .gitignore
.env
.env.local
.env.*.local
*.pem
*.key
credentials.json
```

## When Exceptions Apply

Default values for local development (e.g., `process.env.PORT ?? 3000`) are acceptable. Never default to real secrets.

## See Also

- [proj-env-files](./proj-env-files.md) - Use .env.example not .env in git
- [type-validate-config](./type-validate-config.md) - Validate config at startup
