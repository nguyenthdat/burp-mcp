# mod-separate-concerns

> One module, one responsibility — don't mix data access, business logic, and HTTP handling

## Why It Matters

Modules that mix concerns (e.g., parsing HTTP requests AND querying a database) are hard to test, reuse, and refactor. Splitting code by responsibility — HTTP handling, business logic, data access — creates composable, testable units. This is the foundation of maintainable Node.js applications.

## Bad

```js
// server.js — handles HTTP, business logic, and database all in one file
import express from 'express';
import { query } from './db.js';

const app = express();

app.get('/users/:id', async (req, res) => {
  const [user] = await query('SELECT * FROM users WHERE id = ?', [req.params.id]);
  if (!user) return res.status(404).json({ error: 'Not found' });

  // Business logic inline
  const fullName = `${user.first_name} ${user.last_name}`;
  const isAdult = new Date().getFullYear() - user.birth_year >= 18;

  res.json({ ...user, fullName, isAdult });
});
```

## Good

```js
// user-repository.js — data access layer
import { query } from './db.js';

export async function findById(id) {
  const [user] = await query('SELECT * FROM users WHERE id = ?', [id]);
  return user ?? null;
}

// user-service.js — business logic
export function enrichUser(user) {
  return {
    ...user,
    fullName: `${user.first_name} ${user.last_name}`,
    isAdult: new Date().getFullYear() - user.birth_year >= 18,
  };
}

// user-controller.js — HTTP handling
import { findById } from './user-repository.js';
import { enrichUser } from './user-service.js';

export async function getUser(req, res) {
  const user = await findById(req.params.id);
  if (!user) return res.status(404).json({ error: 'Not found' });
  res.json(enrichUser(user));
}

// server.js — routing only
import express from 'express';
import { getUser } from './user-controller.js';

const app = express();
app.get('/users/:id', getUser);
```

## When Exceptions Apply

For small scripts, CLIs, or prototypes (< 100 lines total), separation is overengineering. Apply when the codebase grows.

## See Also

- [proj-layer-architecture](./proj-layer-architecture.md) - Controller → service → repository layers
- [mod-side-effect-free](./mod-side-effect-free.md) - Side-effect-free modules
