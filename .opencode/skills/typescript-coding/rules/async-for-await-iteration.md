# async-for-await-iteration

> Use `for await...of` to consume async iterables

## Why It Matters

Async iterables — streams, paginated API results, database cursors — produce values over time rather than all at once, and manually driving their iterator protocol (`.next()`, checking `.done`, handling backpressure) is verbose and error-prone. `for await...of` handles all of that for you: it calls `.next()`, awaits each result, unwraps the value, stops on `done`, and — critically — calls the iterator's `return()` method to release resources if the loop exits early via `break`, `return`, or a thrown error.

## Bad

```typescript
async function printAll(stream: AsyncIterable<string>) {
  const iterator = stream[Symbol.asyncIterator]();
  while (true) {
    const { value, done } = await iterator.next();
    if (done) break;
    console.log(value);
  }
  // If an error above prevented reaching `done`, or `break` were used,
  // iterator.return() is never called and underlying resources may leak.
}
```

## Good

```typescript
async function printAll(stream: AsyncIterable<string>) {
  for await (const line of stream) {
    console.log(line);
  }
  // return() is called automatically on break/throw/normal completion
}
```

## Writing Your Own Async Iterable (Paginated API Example)

```typescript
async function* paginate<T>(fetchPage: (cursor: string | null) => Promise<Page<T>>): AsyncGenerator<T> {
  let cursor: string | null = null;
  do {
    const page = await fetchPage(cursor);
    yield* page.items;
    cursor = page.nextCursor;
  } while (cursor !== null);
}

async function processAllUsers() {
  for await (const user of paginate(fetchUserPage)) {
    await processUser(user);
    if (user.id === "stop-here") break; // triggers generator cleanup via return()
  }
}
```

## Consuming Node.js Streams

```typescript
import { createReadStream } from "node:fs";
import { createInterface } from "node:readline";

async function countLines(path: string): Promise<number> {
  const rl = createInterface({ input: createReadStream(path) });
  let count = 0;
  for await (const _line of rl) {
    count++;
  }
  return count;
}
```

## See Also

- [async-generator-streams](async-generator-streams.md) - Use async generators to model lazy, pull-based async sequences
- [node-streams-backpressure](node-streams-backpressure.md) - Respect backpressure when working with Node.js streams
- [async-avoid-async-foreach](async-avoid-async-foreach.md) - Avoid `Array.prototype.forEach` with an async callback
