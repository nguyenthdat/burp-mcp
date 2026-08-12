# proj-layer-architecture

> Follow the controller → service → repository layered architecture for server applications

## Why It Matters

Without layers, HTTP parsing, business logic, and database queries become entangled. Changing the database requires rewriting business logic; testing requires spinning up a real server. Layers isolate concerns: controllers handle HTTP, services contain business rules, repositories manage data access. Each layer can be tested independently with mocked dependencies.

## Bad

```js
// Everything in one route handler — untestable, unmaintainable
app.post('/orders', async (req, res) => {
  const { userId, items } = req.body;

  // Validation inline
  if (!userId) return res.status(400).json({ error: 'Missing userId' });

  // Business logic inline with database
  const [user] = await db.query('SELECT * FROM users WHERE id = ?', [userId]);
  if (!user) return res.status(404).json({ error: 'User not found' });

  let total = 0;
  for (const item of items) {
    const [product] = await db.query('SELECT price FROM products WHERE id = ?', [item.productId]);
    total += product.price * item.quantity;
  }

  await db.query('INSERT INTO orders (userId, total) VALUES (?, ?)', [userId, total]);
  res.json({ orderId: result.insertId, total });
});
```

## Good

```js
// Controller — HTTP only
// controllers/order-controller.js
export async function createOrder(req, res) {
  try {
    const order = await orderService.create(req.body);
    res.status(201).json(order);
  } catch (err) {
    if (err instanceof ValidationError) {
      return res.status(400).json({ error: err.message });
    }
    throw err;
  }
}

// Service — business logic only
// services/order-service.js
export async function create(input) {
  validate(input);
  const user = await userRepo.findById(input.userId);
  if (!user) throw new NotFoundError('User');
  const total = calculateTotal(input.items);
  return orderRepo.create({ userId: user.id, total });
}

// Repository — data access only
// repositories/order-repository.js
export async function create(data) {
  const [result] = await db.query(
    'INSERT INTO orders (userId, total) VALUES (?, ?)',
    [data.userId, data.total],
  );
  return { id: result.insertId, ...data };
}
```

## When Exceptions Apply

For simple CRUD APIs with no business logic, the service layer may be thin enough to skip. For prototypes, start simple and add layers as complexity grows.

## See Also

- [mod-separate-concerns](./mod-separate-concerns.md) - One module, one responsibility
- [proj-src-lib-dir](./proj-src-lib-dir.md) - Directory structure
