# anti-any-cast-double

> Don't force incorrect types with `as unknown as T` double casts

## Why It Matters

TypeScript normally only allows a type assertion (`as T`) between types that overlap — if you try to assert directly between two unrelated types, the compiler raises an error precisely because it has evidence the cast is unsound. `as unknown as T` bypasses that safety check entirely by routing through `unknown`, which is assignable to and from anything: it silences the compiler's warning without addressing the underlying type mismatch it was correctly flagging. Every occurrence is a place where the type system's guarantees no longer hold, and because it looks like "just another assertion," reviewers can miss that it's actually the compiler's strongest possible objection being forcibly overridden.

## Bad

```typescript
interface User {
  id: string;
  name: string;
}

interface Product {
  sku: string;
  price: number;
}

function getUser(): User {
  const product: Product = { sku: 'abc', price: 9.99 };
  return product as unknown as User; // compiles, but `id`/`name` don't exist at runtime
}

// A common escape when the "real" fix (aligning the types) feels too tedious in the moment
const response = await fetch(url);
const data = (await response.json()) as unknown as ExpectedShape; // no validation at all
```

## Good

```typescript
// Fix the actual mismatch: either the function's return type is wrong,
// or the value needs to be constructed correctly.
function getUser(product: Product): User {
  throw new Error('cannot derive a User from a Product — check the calling code');
}

// For genuinely unknown runtime data, validate instead of asserting
import { z } from 'zod';

const expectedSchema = z.object({ id: z.string(), name: z.string() });

async function getUser(url: string) {
  const response = await fetch(url);
  const data = await response.json();
  return expectedSchema.parse(data); // throws with a clear error if the shape is wrong
}
```

## When a Double Cast Might Be Justified

Extremely rare cases — bridging a third-party library's incorrect or overly-narrow type definitions, where you have independently verified the runtime shape is correct — may warrant a scoped `as unknown as T`, but it should come with a comment explaining exactly why the direct assertion doesn't type-check and what verification backs the claim:

```typescript
// The vendor's type defs are missing `metadata` even though it's always
// present at runtime per their docs (see LIBRARY-1234). Verified via manual testing.
const enriched = vendorResult as unknown as VendorResultWithMetadata;
```

## See Also

- [anti-any-abuse](anti-any-abuse.md) - Don't use `any` to silence type errors
- [type-avoid-assertion](type-avoid-assertion.md) - Avoid type assertions in favor of proper narrowing
- [type-zod-schema-inference](type-zod-schema-inference.md) - Derive static types from Zod schemas instead of duplicating them
