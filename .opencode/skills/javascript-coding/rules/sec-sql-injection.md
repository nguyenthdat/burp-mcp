# sec-sql-injection

> Use parameterized queries — never concatenate user input into SQL strings

## Why It Matters

SQL injection is one of the most common and dangerous web vulnerabilities. Concatenating user input into SQL strings allows attackers to modify queries, bypass authentication, exfiltrate data, or destroy tables. Parameterized queries (prepared statements) separate SQL structure from data, making injection impossible.

## Bad

```js
// String interpolation — vulnerable to SQL injection
const username = req.body.username;  // "admin' --"
const user = await db.query(
  `SELECT * FROM users WHERE username = '${username}'`
);

// Template literals with user input
await db.query(`INSERT INTO posts (title) VALUES ('${req.body.title}')`);

// Dynamic column names from user input
const sort = req.query.sort;  // "id; DROP TABLE users; --"
await db.query(`SELECT * FROM posts ORDER BY ${sort}`);
```

## Good

```js
// Parameterized queries — safe from injection
const [user] = await db.query(
  'SELECT * FROM users WHERE username = ?',
  [req.body.username],
);

await db.query(
  'INSERT INTO posts (title, body) VALUES (?, ?)',
  [req.body.title, req.body.body],
);

// Dynamic sort column — whitelist validation
const ALLOWED_SORT_COLUMNS = ['id', 'title', 'created_at'];

function sortPosts(req, res) {
  const sort = req.query.sort;
  if (!ALLOWED_SORT_COLUMNS.includes(sort)) {
    return res.status(400).json({ error: 'Invalid sort column' });
  }

  const [posts] = await db.query(
    `SELECT * FROM posts ORDER BY ${db.escapeId(sort)}`,  // Escape identifier
  );
  res.json(posts);
}

// With an ORM (Prisma, Drizzle, Knex)
import { sql } from 'drizzle-orm';

const posts = await db
  .select()
  .from(postsTable)
  .where(sql`title = ${req.body.title}`);
```

## When Exceptions Apply

Parameterized queries cover all data values. For dynamic identifiers (table names, column names), use whitelisting with identifier escaping — never trust user input directly.

## See Also

- [sec-input-sanitize](./sec-input-sanitize.md) - Sanitize all user input
- [sec-avoid-os-command](./sec-avoid-os-command.md) - Command injection prevention
