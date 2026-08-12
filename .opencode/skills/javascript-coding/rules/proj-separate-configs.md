# proj-separate-configs

> Separate configuration from code — use config files, environment variables, or a dedicated config module

## Why It Matters

Hardcoding configuration (ports, URLs, thresholds) in source code requires code changes to reconfigure the application for different environments. Configuration should vary between deployments while code stays the same. Separating config makes the application portable, enables configuration-as-code in CI/CD, and follows the 12-factor app methodology.

## Bad

```js
// Hardcoded configuration in code
const API_URL = 'https://api.production.example.com';
const PORT = 3000;
const TIMEOUT = 5000;
const DB_HOST = 'localhost';
```

## Good

```js
// config.js — all configuration in one place
export const config = {
  apiUrl: process.env.API_URL ?? 'http://localhost:4000',
  port: parseInt(process.env.PORT, 10) || 3000,
  timeout: parseInt(process.env.TIMEOUT, 10) || 5000,
  database: {
    host: process.env.DB_HOST ?? 'localhost',
    port: parseInt(process.env.DB_PORT, 10) || 5432,
    name: process.env.DB_NAME ?? 'myapp',
  },
};

// Or load from a config file
import { readFile } from 'node:fs/promises';

const rawConfig = JSON.parse(await readFile('./config.json', 'utf8'));
export const config = Object.freeze(rawConfig);
```

## Configuration Priority

```js
// CLI args > env vars > config file > defaults
function loadConfig() {
  return {
    port: process.argv.includes('--port')
      ? parseInt(process.argv[process.argv.indexOf('--port') + 1], 10)
      : parseInt(process.env.PORT, 10) || 3000,
  };
}
```

## When Exceptions Apply

Truly constant values (mathematical constants, enum-like values, protocol constants) can live in code. Configuration is for values that vary between environments.

## See Also

- [node-env-config](./node-env-config.md) - Environment config loading
- [type-validate-config](./type-validate-config.md) - Validate config at startup
