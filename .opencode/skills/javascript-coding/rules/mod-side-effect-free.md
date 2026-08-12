# mod-side-effect-free

> Keep modules side-effect-free — reserve side effects for entry points

## Why It Matters

Modules with side effects (code that runs on import, like connecting to a database or mutating globals) make code unpredictable and hard to test. They couple the module's behavior to its import time, preventing tree-shaking and causing order-dependent bugs. A module should only export values; side effects belong in application entry points.

## Bad

```js
// database.js — side effect on import
import { MongoClient } from 'mongodb';

const client = new MongoClient(process.env.MONGO_URL);
await client.connect();  // Top-level await blocks all consumers

export const db = client.db();

// logger.js — mutates global state
import winston from 'winston';

winston.add(new winston.transports.File({ filename: 'app.log' }));

export const logger = winston;
```

## Good

```js
// database.js — exports a factory, no side effects
import { MongoClient } from 'mongodb';

let client = null;

export async function connectDB(url) {
  if (!client) {
    client = new MongoClient(url);
    await client.connect();
  }
  return client.db();
}

export async function closeDB() {
  await client?.close();
  client = null;
}

// logger.js — exports a factory
import winston from 'winston';

export function createLogger(options) {
  return winston.createLogger({
    transports: [new winston.transports.File(options)],
  });
}
```

## Entry Point (Where Side Effects Belong)

```js
// server.js — entry point, side effects are acceptable here
import { connectDB } from './database.js';
import { createLogger } from './logger.js';

const db = await connectDB(process.env.MONGO_URL);
const logger = createLogger({ filename: 'app.log' });

// Now start the server with initialized dependencies
```

## When Exceptions Apply

Side effects are acceptable in:
- Application entry points (`server.js`, `cli.js`, `index.js`)
- Polyfill modules (must run at import time)
- CSS/SCSS imports in bundlers
- Framework setup files (e.g., `jest.setup.js`)

## See Also

- [mod-circular-deps](./mod-circular-deps.md) - Avoid circular dependencies
- [mod-separate-concerns](./mod-separate-concerns.md) - One module, one responsibility
