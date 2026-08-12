# type-readonly-arrays

> Accept `readonly T[]` for parameters that shouldn't be mutated

## Why It Matters

A function parameter typed `T[]` grants the function permission to call `.push`, `.sort`, `.splice`, and every other mutating array method on data that belongs to the caller, even if the function never intends to modify it. That silent permission is a common source of action-at-a-distance bugs — a "read-only" helper that quietly reorders or empties a shared array. Typing the parameter as `readonly T[]` makes the no-mutation contract enforceable by the compiler instead of relying on a docstring or convention.

## Bad

```typescript
function findMax(numbers: number[]): number {
  // Nothing stops an implementation (or a future edit) from mutating the input
  numbers.sort((a, b) => b - a); // sorts the CALLER's array in place!
  return numbers[0];
}

const scores = [10, 50, 20];
const max = findMax(scores);
console.log(scores); // [50, 20, 10] — caller's array was silently reordered
```

## Good

```typescript
function findMax(numbers: readonly number[]): number {
  numbers.sort((a, b) => b - a); // Error: sort does not exist on readonly number[]
  return Math.max(...numbers); // non-mutating alternative
}

const scores = [10, 50, 20];
const max = findMax(scores);
console.log(scores); // [10, 50, 20] — unchanged, contract enforced by the type
```

## Mutating Methods Blocked by `readonly T[]`

| Method | Blocked? |
|---|---|
| `push`, `pop`, `shift`, `unshift` | Yes |
| `sort`, `reverse`, `splice`, `fill`, `copyWithin` | Yes |
| `map`, `filter`, `slice`, `reduce`, `forEach`, `at`, indexing | No (all non-mutating, still allowed) |

## Returning Readonly Too

The same reasoning applies to return types: if a function hands back internal state, return `readonly T[]` (or a frozen array) so callers don't assume they can mutate it without consequence.

```typescript
class Inventory {
  #items: Item[] = [];

  list(): readonly Item[] {
    return this.#items; // callers can read, but not push/splice this array
  }
}
```

## Interop Note

`readonly T[]` is a compile-time-only guarantee — it does not call `Object.freeze` at runtime. A caller can still bypass it with a type assertion (`arr as number[]`), so it protects against accidental mutation from well-typed code, not from adversarial code.

## See Also

- [type-const-assertion](type-const-assertion.md) - Use as const to infer literal, readonly types
- [imm-avoid-array-mutation](imm-avoid-array-mutation.md) - Avoid mutating arrays in place
- [imm-avoid-param-mutation](imm-avoid-param-mutation.md) - Avoid mutating function parameters
- [api-readonly-public-types](api-readonly-public-types.md) - Expose readonly types on public APIs
