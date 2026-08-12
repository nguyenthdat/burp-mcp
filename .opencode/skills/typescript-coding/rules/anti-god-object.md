# anti-god-object

> Don't build "God" objects/functions with too many responsibilities

## Why It Matters

A single class or function that handles validation, business logic, persistence, logging, and formatting all at once becomes the one file everyone has to touch for almost any change, which means merge conflicts concentrate there, a change to one responsibility risks breaking an unrelated one bundled in the same object, and it's nearly impossible to unit test in isolation — testing "just the validation" requires standing up the database and the logger too, because they're all entangled in one class. Splitting responsibilities into focused, independently-testable units (each with a narrow, well-named purpose) makes each piece reason-about-able on its own, and makes the dependency graph between them explicit instead of implicit inside one giant object.

## Bad

```typescript
class OrderManager {
  private db: Database;
  private logger: Logger;

  async processOrder(orderData: unknown) {
    // Validation
    if (typeof orderData !== 'object' || orderData === null) throw new Error('invalid');
    const order = orderData as Order;

    // Business logic
    const total = order.items.reduce((sum, i) => sum + i.price * i.qty, 0);
    const discount = total > 100 ? total * 0.1 : 0;

    // Persistence
    await this.db.query('INSERT INTO orders ...', [order, total, discount]);

    // Notification
    await this.sendEmail(order.customerEmail, `Your order total: ${total - discount}`);

    // Logging
    this.logger.info(`Order processed: ${order.id}`);

    // Formatting for the API response
    return { id: order.id, total: total - discount, status: 'confirmed' };
  }

  private async sendEmail(to: string, body: string) {
    /* ... */
  }
}
```

## Good

```typescript
function validateOrder(input: unknown): Order {
  return orderSchema.parse(input);
}

function calculateOrderTotal(order: Order): OrderTotal {
  const subtotal = order.items.reduce((sum, i) => sum + i.price * i.qty, 0);
  const discount = subtotal > 100 ? subtotal * 0.1 : 0;
  return { subtotal, discount, total: subtotal - discount };
}

class OrderRepository {
  constructor(private db: Database) {}
  async save(order: Order, total: OrderTotal) {
    await this.db.query('INSERT INTO orders ...', [order, total]);
  }
}

class OrderNotifier {
  constructor(private mailer: Mailer) {}
  async notifyConfirmed(order: Order, total: OrderTotal) {
    await this.mailer.send(order.customerEmail, `Your order total: ${total.total}`);
  }
}

// Orchestration stays thin — it composes focused units, doesn't implement them
async function processOrder(
  input: unknown,
  repo: OrderRepository,
  notifier: OrderNotifier,
  logger: Logger,
) {
  const order = validateOrder(input);
  const total = calculateOrderTotal(order);
  await repo.save(order, total);
  await notifier.notifyConfirmed(order, total);
  logger.info(`Order processed: ${order.id}`);
  return { id: order.id, total: total.total, status: 'confirmed' as const };
}
```

## See Also

- [fn-pure-functions](fn-pure-functions.md) - Prefer pure functions with no hidden side effects
- [fn-composition-over-inheritance](fn-composition-over-inheritance.md) - Compose small units instead of building deep inheritance hierarchies
- [api-minimal-surface](api-minimal-surface.md) - Expose the smallest public surface a module needs
