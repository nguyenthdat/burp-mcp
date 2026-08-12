# err-boundary-validation

> Validate untrusted input at system boundaries with a schema library

## Why It Matters

Data crossing a trust boundary — an HTTP request body, a message queue payload, a CLI argument, a third-party API response — has a type only in your imagination until it's actually checked; TypeScript's type annotations are erased at runtime and provide zero protection against malformed input. Validating with a schema library at the boundary converts "hope the shape is right" into an enforced guarantee, and it fails fast with a precise error at the point where bad data entered, rather than several call frames deeper where the failure is much harder to trace back to its source.

## Bad

```typescript
interface CreateOrderRequest {
  userId: string;
  items: { sku: string; quantity: number }[];
}

app.post("/orders", async (req, res) => {
  // req.body is `any` — this cast is just a hopeful label, not a check
  const body = req.body as CreateOrderRequest;
  const order = await createOrder(body.userId, body.items);
  res.json(order);
});
// A malformed body (missing items, quantity as a string, extra fields) sails through
// and crashes deep inside createOrder with a confusing stack trace.
```

## Good

```typescript
import { z } from "zod";

const CreateOrderRequestSchema = z.object({
  userId: z.string().uuid(),
  items: z
    .array(
      z.object({
        sku: z.string().min(1),
        quantity: z.number().int().positive(),
      }),
    )
    .min(1),
});

app.post("/orders", async (req, res) => {
  const parsed = CreateOrderRequestSchema.safeParse(req.body);
  if (!parsed.success) {
    return res.status(400).json({ error: parsed.error.flatten() });
  }
  const order = await createOrder(parsed.data.userId, parsed.data.items);
  res.json(order);
});
```

## Boundaries That Need Validation

| Boundary | What to validate |
|---|---|
| HTTP request body/query/params | Every field the handler reads |
| Environment variables | Presence, type, and format at startup (fail fast, not mid-request) |
| Message queue / event payloads | Full schema, since producers and consumers can drift versions |
| Responses from third-party APIs | At least the fields you depend on — external APIs change without notice |
| CLI arguments | Types and required flags |
| `JSON.parse` results | Always `unknown` until validated |

## Fail Fast at Startup for Config

```typescript
const EnvSchema = z.object({
  PORT: z.coerce.number().int().positive(),
  DATABASE_URL: z.string().url(),
});

const env = EnvSchema.parse(process.env); // crashes immediately on boot if misconfigured, not mid-request
```

## See Also

- [type-zod-schema-inference](type-zod-schema-inference.md) - Derive static types from a runtime schema instead of maintaining both by hand
- [node-env-var-validation](node-env-var-validation.md) - Validate environment variables at process startup
- [err-custom-error-class](err-custom-error-class.md) - Extend Error with custom subclasses that carry structured context
