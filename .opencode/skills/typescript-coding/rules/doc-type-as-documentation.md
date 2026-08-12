# doc-type-as-documentation

> Let precise types replace comments that only describe a shape

## Why It Matters

A comment describing an object's shape ("id is a string, status is one of active/inactive/banned") duplicates information the type checker could enforce, and unlike a type, a comment can't catch a caller passing the wrong value — it silently drifts out of sync the moment the shape changes. Encoding the shape as a precise type (a union of literals, a branded type, a discriminated union) turns "documentation" into a compiler-checked guarantee: misuse is a red squiggly line in the editor, not a bug discovered in production.

## Bad

```typescript
/**
 * user object:
 * - id: string
 * - status: "active", "inactive", or "banned" (as a string)
 * - role: "admin" or "member"
 */
function canEditPost(user: { id: string; status: string; role: string }, post: Post): boolean {
  return user.status === "active" && (user.role === "admin" || post.authorId === user.id);
}
```

## Good

```typescript
type UserStatus = "active" | "inactive" | "banned";
type UserRole = "admin" | "member";

interface User {
  id: string;
  status: UserStatus;
  role: UserRole;
}

function canEditPost(user: User, post: Post): boolean {
  return user.status === "active" && (user.role === "admin" || post.authorId === user.id);
}
```

Now passing `status: "suspended"` or `role: "moderator"` is a compile error, not a bug found by a user report — the type is the documentation, and it's enforced.

## When A Comment Is Still Needed

Types can't express *why* a shape looks the way it does, so keep a short comment for non-obvious modeling decisions even after the type is precise:

```typescript
interface Invoice {
  // Stored in cents to avoid floating-point rounding errors in totals.
  amountCents: number;
  status: "draft" | "sent" | "paid" | "void";
}
```

## Guidelines

- Replace "one of X, Y, Z" comments with a union of string literal types.
- Replace "either A or B depending on kind" comments with a discriminated union (see `type-discriminated-union`).
- Reserve comments for the *reason* behind a type's shape (units, invariants, external constraints), not the shape itself.
- Run `tsc --noEmit` and enable `noUnusedLocals`/`strict` so unused precise types don't silently rot back into loosely typed comments over time.

## See Also

- [type-discriminated-union](type-discriminated-union.md) - model "either A or B" data as a discriminated union instead of a comment
- [doc-inline-why-not-what](doc-inline-why-not-what.md) - Write comments that explain why, not what the code already says
- [type-branded-nominal](type-branded-nominal.md) - use branded types to encode invariants types alone can't express
