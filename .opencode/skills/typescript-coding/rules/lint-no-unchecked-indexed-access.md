# lint-no-unchecked-indexed-access

> Enable `noUncheckedIndexedAccess` in `tsconfig.json`

## Why It Matters

By default, TypeScript types `arr[i]` as `T`, not `T | undefined`, even though indexing past the end of an array or into a missing object key returns `undefined` at runtime. This is one of the most common gaps between what the type system claims and what actually happens — every array index access and every `Record<string, T>` lookup is a potential `undefined` that the compiler pretends can't occur. `noUncheckedIndexedAccess` closes that gap by typing all indexed access as `T | undefined`, forcing an explicit check (or a safe navigation) at every access site, which is exactly where the corresponding runtime bug would otherwise surface as "Cannot read properties of undefined."

## Bad

```jsonc
// tsconfig.json — noUncheckedIndexedAccess not enabled (the default)
{ "compilerOptions": { "strict": true } }
```

```typescript
function getFirstError(errors: string[]) {
  const first = errors[0]; // typed as `string`, but is `undefined` if errors is empty
  return first.toUpperCase(); // compiles fine, throws at runtime on an empty array
}

const scores: Record<string, number> = { alice: 90 };
const bobScore = scores['bob']; // typed as `number`, actually `undefined`
console.log(bobScore.toFixed(2)); // compiles fine, crashes at runtime
```

## Good

```jsonc
// tsconfig.json
{
  "compilerOptions": {
    "strict": true,
    "noUncheckedIndexedAccess": true
  }
}
```

```typescript
function getFirstError(errors: string[]) {
  const first = errors[0]; // now typed as `string | undefined`
  if (!first) {
    throw new Error('no errors to report');
  }
  return first.toUpperCase(); // safe, narrowed
}

const scores: Record<string, number> = { alice: 90 };
const bobScore = scores['bob']; // typed as `number | undefined`
console.log(bobScore?.toFixed(2) ?? 'no score'); // handled explicitly
```

## Interaction With `.at()` and Loops

```typescript
// .at() already returns T | undefined regardless of this flag — it's consistent
const last = errors.at(-1); // string | undefined either way

// for...of avoids indexing entirely, sidestepping the issue
for (const error of errors) {
  console.log(error.toUpperCase()); // `error` is always `string`, never undefined
}
```

This flag has real migration cost on an existing large codebase (every indexed access needs review), but for new projects it should be on from day one.

## See Also

- [lint-strict-tsconfig](lint-strict-tsconfig.md) - Enable `strict: true` and other strictness flags in `tsconfig.json`
- [type-index-signature-safety](type-index-signature-safety.md) - Treat index signature access as possibly `undefined`
- [lint-no-non-null-assertion](lint-no-non-null-assertion.md) - Enable `@typescript-eslint/no-non-null-assertion`
