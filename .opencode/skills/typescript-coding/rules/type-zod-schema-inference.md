# type-zod-schema-inference

> Derive static types from a runtime schema instead of maintaining both by hand

## Why It Matters

Hand-writing an `interface` alongside a separate hand-written validation function means two sources of truth that must be kept in sync manually; the compiler has no way to notice when the validator and the type disagree. A runtime schema library (zod, valibot, arktype) lets you define the shape once and derive both the compile-time type (via `z.infer`) and the runtime validator from it, so they can never drift apart, and untrusted data is checked at the exact point it enters your system.

## Bad

```typescript
interface CreateUserRequest {
  name: string;
  email: string;
  age: number;
}

function isValidCreateUserRequest(body: unknown): body is CreateUserRequest {
  // Hand-written, easy to forget a field or a rule, and easy to let drift from the interface
  return (
    typeof body === "object" &&
    body !== null &&
    "name" in body &&
    typeof (body as any).name === "string"
    // ...email and age validation quietly never got added
  );
}
```

## Good

```typescript
import { z } from "zod";

const CreateUserRequestSchema = z.object({
  name: z.string().min(1),
  email: z.string().email(),
  age: z.number().int().min(0).max(150),
});

// Type is derived — cannot drift from the runtime validator
type CreateUserRequest = z.infer<typeof CreateUserRequestSchema>;

async function handleCreateUser(req: Request) {
  const raw: unknown = await req.json();
  const result = CreateUserRequestSchema.safeParse(raw);
  if (!result.success) {
    throw new ValidationError(result.error.issues);
  }
  const body: CreateUserRequest = result.data; // fully typed, fully validated
  return createUser(body);
}
```

## Parsing vs Safe-Parsing

| Method | Behavior |
|---|---|
| `schema.parse(data)` | Returns validated data or throws a `ZodError` |
| `schema.safeParse(data)` | Returns `{ success: true; data } \| { success: false; error }`, never throws |
| `schema.parseAsync` / `safeParseAsync` | Same, for schemas with async refinements |

Prefer `safeParse` at boundaries where you want to convert failures into your own error type or an HTTP response, and `parse` inside code that's already wrapped in a try/catch expecting to propagate the error.

## Composing and Reusing Schemas

```typescript
const AddressSchema = z.object({
  street: z.string(),
  city: z.string(),
  zip: z.string().regex(/^\d{5}$/),
});

const UserSchema = z.object({
  id: z.string().uuid(),
  address: AddressSchema,
});

// Extend, pick, and omit mirror TypeScript's own utility types
const UserPreviewSchema = UserSchema.pick({ id: true });
type UserPreview = z.infer<typeof UserPreviewSchema>;
```

## See Also

- [err-boundary-validation](err-boundary-validation.md) - Validate untrusted input at system boundaries with a schema library
- [type-unknown-over-any](type-unknown-over-any.md) - Use unknown instead of any for values of uncertain type
- [type-branded-nominal](type-branded-nominal.md) - Use branded/nominal types to distinguish primitives with the same runtime type
