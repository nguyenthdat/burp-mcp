# async-generator-streams

> Use async generators to model lazy, pull-based async sequences

## Why It Matters

Eagerly loading an entire dataset into an array before processing it wastes memory and delays the first result until everything is fetched. Async generators (`async function*`) produce values one at a time, on demand, pausing at each `yield` until the consumer asks for the next one via `for await...of`. This gives you composable, lazy pipelines for paginated APIs, file parsing, or infinite sequences, without loading more into memory than the consumer actually needs at any given moment.

## Bad

```typescript
async function fetchAllLogLines(fileId: string): Promise<string[]> {
  // Loads the entire (potentially multi-GB) log file into memory
  // before the caller can process even the first line.
  const allLines: string[] = [];
  let cursor: string | null = null;
  do {
    const page = await fetchLogPage(fileId, cursor);
    allLines.push(...page.lines);
    cursor = page.nextCursor;
  } while (cursor);
  return allLines;
}
```

## Good

```typescript
async function* streamLogLines(fileId: string): AsyncGenerator<string> {
  let cursor: string | null = null;
  do {
    const page = await fetchLogPage(fileId, cursor);
    yield* page.lines;
    cursor = page.nextCursor;
  } while (cursor);
}

async function findFirstError(fileId: string): Promise<string | undefined> {
  for await (const line of streamLogLines(fileId)) {
    if (line.includes("ERROR")) return line; // stops fetching further pages
  }
  return undefined;
}
```

## Composing Async Generators

```typescript
async function* map<T, R>(source: AsyncIterable<T>, fn: (item: T) => R): AsyncGenerator<R> {
  for await (const item of source) {
    yield fn(item);
  }
}

async function* filter<T>(source: AsyncIterable<T>, predicate: (item: T) => boolean): AsyncGenerator<T> {
  for await (const item of source) {
    if (predicate(item)) yield item;
  }
}

// Pipelines stay lazy end-to-end: nothing is fetched until iterated
const errorTimestamps = map(
  filter(streamLogLines(fileId), (line) => line.includes("ERROR")),
  (line) => parseTimestamp(line),
);

for await (const ts of errorTimestamps) {
  console.log(ts);
}
```

## See Also

- [async-for-await-iteration](async-for-await-iteration.md) - Use `for await...of` to consume async iterables
- [node-streams-backpressure](node-streams-backpressure.md) - Respect backpressure when working with Node.js streams
- [perf-avoid-unnecessary-allocation](perf-avoid-unnecessary-allocation.md) - Avoid materializing large collections you don't need in full
