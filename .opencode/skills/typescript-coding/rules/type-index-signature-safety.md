# type-index-signature-safety

> Enable `noUncheckedIndexedAccess` and guard indexed access results

## Why It Matters

By default, TypeScript types `arr[i]` and `obj[key]` as `T`, never as `T | undefined`, even though indexing out of bounds or with a nonexistent key returns `undefined` at runtime. That mismatch is a frequent source of "cannot read properties of undefined" crashes that the type checker should have caught. `noUncheckedIndexedAccess` makes every indexed access include `undefined` in its type, forcing you to check the result before using it — exactly where the real risk lives.

## Bad

```typescript
// tsconfig without noUncheckedIndexedAccess
function firstWord(sentence: string): string {
  const words = sentence.split(" ");
  return words[0].toUpperCase(); // fine here, but...
}

function nthWord(sentence: string, n: number): string {
  const words = sentence.split(" ");
  return words[n].toUpperCase(); // crashes at runtime if n is out of range
}

const scores: Record<string, number> = { alice: 90 };
const bobScore = scores["bob"]; // typed as `number`, but is actually `undefined`
console.log(bobScore.toFixed(1)); // runtime crash, no compiler warning
```

## Good

```typescript
// tsconfig.json: { "compilerOptions": { "noUncheckedIndexedAccess": true } }
function nthWord(sentence: string, n: number): string {
  const words = sentence.split(" ");
  const word = words[n]; // typed as `string | undefined`
  if (word === undefined) {
    throw new RangeError(`no word at index ${n}`);
  }
  return word.toUpperCase();
}

const scores: Record<string, number> = { alice: 90 };
const bobScore = scores["bob"]; // typed as `number | undefined`
console.log(bobScore?.toFixed(1) ?? "no score"); // safe, handles the missing case
```

## Configuration

```json
{
  "compilerOptions": {
    "strict": true,
    "noUncheckedIndexedAccess": true
  }
}
```

This flag is not part of `strict`, so it must be enabled separately — it's one of the highest-signal, most-often-skipped flags in real-world `tsconfig.json` files.

## Working Around Verified-Safe Access

When you've already validated an index is in range (e.g. right after a `.length` check in a loop), narrow with a non-null assertion sparingly and locally, or prefer `.at()` / array destructuring which communicate intent more clearly than a raw index:

```typescript
const [first, ...rest] = words; // `first` is `string | undefined` too, but destructuring reads clearly

for (let i = 0; i < words.length; i++) {
  const word = words[i]!; // safe: loop bound guarantees i is in range
}
```

## See Also

- [type-strict-null-checks](type-strict-null-checks.md) - Enable strictNullChecks and model absence with undefined/null explicitly
- [lint-no-unchecked-indexed-access](lint-no-unchecked-indexed-access.md) - Lint rule pairing with the noUncheckedIndexedAccess compiler flag
- [lint-strict-tsconfig](lint-strict-tsconfig.md) - Enforce strict compiler options across the project
