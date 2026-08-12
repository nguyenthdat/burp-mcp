# api-module-boundary-types

> Define explicit DTOs at module/service boundaries, separate from internal domain models

## Why It Matters

Exposing your internal domain model directly at a module, HTTP, or service boundary couples every consumer to your implementation details: renaming an internal field, changing a database column type, or restructuring an aggregate now breaks every external caller simultaneously. A dedicated DTO (data transfer object) at the boundary decouples "what the outside world sees" from "how we model this internally," so the two can evolve independently — you can refactor internals freely as long as the boundary mapping still produces the same DTO shape.

## Bad

```typescript
// Internal domain model, used directly as the HTTP response shape
interface UserAggregate {
  id: string;
  email: string;
  passwordHash: string;   // never should have left the service
  internalFlags: number;  // implementation detail
  createdAt: Date;
}

app.get("/users/:id", async (req, res) => {
  const user = await userRepository.findById(req.params.id);
  res.json(user); // leaks passwordHash and internalFlags to every client
});
```

## Good

```typescript
// Internal domain model — free to change without notice
interface UserAggregate {
  id: string;
  email: string;
  passwordHash: string;
  internalFlags: number;
  createdAt: Date;
}

// Explicit boundary type — this is the actual public contract
interface UserResponseDto {
  id: string;
  email: string;
  createdAt: string; // ISO string over the wire, not a Date instance
}

function toUserResponseDto(user: UserAggregate): UserResponseDto {
  return {
    id: user.id,
    email: user.email,
    createdAt: user.createdAt.toISOString(),
  };
}

app.get("/users/:id", async (req, res) => {
  const user = await userRepository.findById(req.params.id);
  res.json(toUserResponseDto(user));
});
```

## Boundaries Where This Applies

- HTTP request/response bodies (the DTO, not the ORM entity, is the contract).
- Messages published to a queue/event bus (schema evolution requires this separation).
- Public npm package exports (see api-versioned-public-api).
- Cross-team internal service calls, even within the same monorepo.

## Validating DTOs at the Boundary

```typescript
import { z } from "zod";

const CreateUserRequestSchema = z.object({
  email: z.string().email(),
  password: z.string().min(12),
});

type CreateUserRequest = z.infer<typeof CreateUserRequestSchema>;

app.post("/users", async (req, res) => {
  const parsed = CreateUserRequestSchema.safeParse(req.body);
  if (!parsed.success) {
    return res.status(400).json({ errors: parsed.error.issues });
  }
  await createUser(parsed.data); // parsed.data is typed as CreateUserRequest
});
```

## See Also

- [type-zod-schema-inference](type-zod-schema-inference.md) - Deriving static types from runtime validation schemas
- [err-boundary-validation](err-boundary-validation.md) - Validate untrusted input at the boundary, trust it internally afterward
- [api-versioned-public-api](api-versioned-public-api.md) - Version public package APIs deliberately and follow semver for breaking changes
