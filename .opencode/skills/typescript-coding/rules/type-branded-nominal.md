# type-branded-nominal

> Use branded/nominal types to distinguish primitives with the same runtime type

## Why It Matters

TypeScript's structural type system treats any two `string` or `number` aliases as interchangeable, so `UserId` and `OrderId` (both really just `string`) can be swapped by mistake with zero compiler warning. That kind of mix-up compiles cleanly and only surfaces as a data-integrity bug in production — a user looking up the wrong order, a payment applied to the wrong account. Branding attaches a unique, uninhabited marker property to the type so the compiler rejects accidental cross-assignment, while the runtime representation stays a plain primitive.

## Bad

```typescript
type UserId = string;
type OrderId = string;

function getOrder(orderId: OrderId): Order {
  /* ... */
}

function getUser(userId: UserId): User {
  /* ... */
}

const userId: UserId = "user_123";
getOrder(userId); // compiles! UserId and OrderId are both just `string`
```

## Good

```typescript
type Brand<T, B extends string> = T & { readonly __brand: B };

type UserId = Brand<string, "UserId">;
type OrderId = Brand<string, "OrderId">;

function asUserId(id: string): UserId {
  return id as UserId;
}

function asOrderId(id: string): OrderId {
  return id as OrderId;
}

function getOrder(orderId: OrderId): Order {
  /* ... */
}

function getUser(userId: UserId): User {
  /* ... */
}

const userId = asUserId("user_123");
getOrder(userId); // Error: Type 'UserId' is not assignable to type 'OrderId'
getUser(userId); // OK
```

## Validating Brands at Construction

Pair branding with a smart constructor so the brand also implies "this value passed validation," not just "this value has a tag":

```typescript
type Email = Brand<string, "Email">;

function parseEmail(raw: string): Email {
  if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(raw)) {
    throw new Error(`invalid email: ${raw}`);
  }
  return raw as Email;
}

function sendWelcome(email: Email) {
  /* email is guaranteed well-formed here */
}
```

## Common Use Cases

| Scenario | Brand example |
|---|---|
| IDs across entities | `UserId`, `OrderId`, `ProductId` |
| Units of measure | `Meters`, `Seconds`, `Cents` (avoid mixing dollars and cents) |
| Validated strings | `Email`, `Uuid`, `NonEmptyString` |
| Sanitized/trusted input | `SanitizedHtml` vs raw `string` |

## See Also

- [type-avoid-assertion](type-avoid-assertion.md) - Avoid as type assertions; prefer narrowing or validation
- [type-zod-schema-inference](type-zod-schema-inference.md) - Derive static types from a runtime schema instead of maintaining both by hand
- [err-boundary-validation](err-boundary-validation.md) - Validate untrusted input at system boundaries with a schema library
