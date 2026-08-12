# async-avoid-async-foreach

> Avoid `Array.prototype.forEach` with an async callback

## Why It Matters

`Array.prototype.forEach` calls its callback for each element but completely ignores the return value, including any promise the callback returns. Passing an `async` function to `forEach` therefore fires off all the promises without ever awaiting them: the `forEach` call itself returns immediately, before any of the async work finishes, and any rejection becomes an unhandled promise rejection instead of a catchable error. This is one of the most common sources of "my async code runs out of order" bugs in TypeScript.

## Bad

```typescript
async function saveAll(records: Record[]): Promise<void> {
  records.forEach(async (record) => {
    await db.save(record);
  });
  console.log("all saved"); // logs immediately, before any save has completed
}
```

## Good

```typescript
async function saveAll(records: Record[]): Promise<void> {
  // Sequential (if order/DB load matters):
  for (const record of records) {
    await db.save(record);
  }
  console.log("all saved"); // logs only after every save finishes
}

async function saveAllConcurrently(records: Record[]): Promise<void> {
  // Concurrent (if records are independent):
  await Promise.all(records.map((record) => db.save(record)));
  console.log("all saved");
}
```

## Why TypeScript Doesn't Catch This For You

```typescript
interface Array<T> {
  forEach(callbackfn: (value: T, index: number, array: T[]) => void, thisArg?: any): void;
}
```

`forEach`'s callback type is `(value, index, array) => void`. An `async` function returns `Promise<void>`, which is structurally assignable to `void` in TypeScript's callback-position bivariance — so the compiler accepts it without complaint even though the promise is silently dropped. `@typescript-eslint/no-misused-promises` catches this pattern at lint time by flagging promise-returning functions passed where a plain `void`-returning callback is expected.

## Enforcing With ESLint

```jsonc
{
  "rules": {
    "@typescript-eslint/no-misused-promises": ["error", { "checksVoidReturn": { "arguments": true } }]
  }
}
```

## See Also

- [async-avoid-sequential-await](async-avoid-sequential-await.md) - Avoid awaiting independent operations sequentially inside loops
- [async-no-floating-promises](async-no-floating-promises.md) - Never leave a promise floating; await, return, or explicitly void it
- [fn-array-methods-over-loops](fn-array-methods-over-loops.md) - Prefer array methods over manual loops for synchronous transforms
