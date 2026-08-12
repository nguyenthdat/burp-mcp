# fn-optional-chaining

> Use optional chaining (`?.`) instead of manual nested null checks

## Why It Matters

Manually checking each level of a nested, possibly-absent structure (`if (a && a.b && a.b.c)`) is verbose, easy to get wrong (missing one level silently lets a `TypeError` through), and buries the actual access behind boilerplate. Optional chaining short-circuits to `undefined` the moment it hits a nullish value anywhere in the chain, expressing "safely read this deep path, however deep it goes" in a single readable expression, and it works uniformly across property access, method calls, and array indexing.

## Bad

```typescript
interface ApiResponse {
  data?: {
    user?: {
      profile?: {
        avatarUrl?: string;
      };
      getDisplayName?: () => string;
    };
  };
}

function getAvatarUrl(response: ApiResponse): string | undefined {
  if (
    response.data &&
    response.data.user &&
    response.data.user.profile &&
    response.data.user.profile.avatarUrl
  ) {
    return response.data.user.profile.avatarUrl;
  }
  return undefined;
}

function getDisplayName(response: ApiResponse): string | undefined {
  if (response.data && response.data.user && response.data.user.getDisplayName) {
    return response.data.user.getDisplayName();
  }
  return undefined;
}
```

## Good

```typescript
function getAvatarUrl(response: ApiResponse): string | undefined {
  return response.data?.user?.profile?.avatarUrl;
}

function getDisplayName(response: ApiResponse): string | undefined {
  return response.data?.user?.getDisplayName?.();
}
```

## Chaining Forms

```typescript
obj?.prop;          // safe property access
obj?.[key];         // safe computed/index access
obj?.method?.();     // safe method call (also guards against method being absent)
arr?.[0]?.name;      // safe array indexing then property access

// Short-circuits the ENTIRE remaining chain on the first nullish value:
a?.b.c.d; // if a is nullish, the whole expression is undefined — b/c/d never evaluated
```

## Combine With Nullish Coalescing For A Default

```typescript
const avatarUrl = response.data?.user?.profile?.avatarUrl ?? "/default-avatar.png";
```

## Caveat: Don't Overuse It To Silence Type Errors

`?.` on a path the compiler says is *never* nullable (because your types already guarantee it) usually signals a type modeling problem, not a place that needs `?.`. Reserve it for paths that are genuinely optional in the data model — sprinkling `?.` everywhere to make TypeScript stop complaining hides real bugs where a value should never have been missing.

## See Also

- [fn-nullish-coalescing](fn-nullish-coalescing.md) - Use `??` instead of `||` when only `null`/`undefined` should trigger the default
- [type-strict-null-checks](type-strict-null-checks.md) - Enable `strictNullChecks` so `null`/`undefined` are tracked in the type system
- [lint-no-non-null-assertion](lint-no-non-null-assertion.md) - Avoid `!` non-null assertions; prove nullability with real checks instead
