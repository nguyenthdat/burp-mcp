# proj-middleware-stack

> Compose middleware in a dedicated file rather than scattering `app.use()` across the codebase

## Why It Matters

Middleware order is critical — a logging middleware must come before auth, which must come before route handlers. When `app.use()` calls are scattered across multiple files, the order becomes implicit and easy to break. A dedicated middleware setup file makes the pipeline explicit, auditable, and testable.

## Bad

```js
// server.js — middleware added here
app.use(express.json());
app.use(cors());

// routes/users.js — more middleware added elsewhere
app.use('/users', authenticate);
app.use('/users', rateLimiter);

// services/order.js — even more!
app.use('/orders', validateOrder);
```

## Good

```js
// middleware/index.js — all middleware in one place
import express from 'express';
import cors from 'cors';
import helmet from 'helmet';
import { authenticate } from './auth.js';
import { rateLimiter } from './rate-limiter.js';
import { requestLogger } from './logger.js';
import { errorHandler } from './error-handler.js';

export function setupMiddleware(app) {
  // Security
  app.use(helmet());
  app.use(cors());

  // Parsing
  app.use(express.json());
  app.use(express.urlencoded({ extended: true }));

  // Logging
  app.use(requestLogger);

  // Auth (skip for public routes)
  app.use('/api', authenticate);

  // Rate limiting
  app.use('/api', rateLimiter);

  // Error handling (last!)
  app.use(errorHandler);
}

// server.js — clean entry point
import express from 'express';
import { setupMiddleware } from './middleware/index.js';
import { setupRoutes } from './routes/index.js';

const app = express();
setupMiddleware(app);
setupRoutes(app);
app.listen(3000);
```

## When Exceptions Apply

Route-specific middleware (e.g., a validation middleware for a single endpoint) can be applied inline on the route. Global middleware belongs in the middleware setup file.

## See Also

- [proj-layer-architecture](./proj-layer-architecture.md) - Layer architecture
- [name-middleware-suffix](./name-middleware-suffix.md) - Middleware naming
