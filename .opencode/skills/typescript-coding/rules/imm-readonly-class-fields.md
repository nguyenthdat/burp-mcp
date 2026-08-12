# imm-readonly-class-fields

> Mark class fields `readonly` when they are set once in the constructor

## Why It Matters

A class field that is only ever assigned in the constructor but declared without `readonly` invites accidental reassignment somewhere in a method, months later, by someone who didn't realize the field was meant to be fixed for the object's lifetime. `readonly` turns that assumption into a compiler-enforced contract: any out-of-constructor write becomes a compile error, and readers scanning the class immediately know which fields represent identity/configuration versus which fields represent mutable state.

## Bad

```typescript
class HttpClient {
  baseUrl: string;
  timeoutMs: number;
  private requestCount: number;

  constructor(baseUrl: string, timeoutMs: number) {
    this.baseUrl = baseUrl;
    this.timeoutMs = timeoutMs;
    this.requestCount = 0;
  }

  async get(path: string) {
    this.requestCount++;
    // ...
  }

  // Nothing stops a future method (or a bug) from doing this:
  resetForTests() {
    this.baseUrl = ""; // should never happen, but nothing prevents it
  }
}
```

## Good

```typescript
class HttpClient {
  readonly baseUrl: string;
  readonly timeoutMs: number;
  private requestCount: number;

  constructor(baseUrl: string, timeoutMs: number) {
    this.baseUrl = baseUrl;
    this.timeoutMs = timeoutMs;
    this.requestCount = 0;
  }

  async get(path: string) {
    this.requestCount++; // fine — not readonly, expected to change
    // ...
  }

  resetForTests() {
    this.baseUrl = ""; // compile error: cannot assign to readonly property
  }
}
```

## Parameter Properties Combine Declaration And Assignment

TypeScript's constructor parameter properties let you declare, mark `readonly`, and assign a field in one line, eliminating the boilerplate of Bad's constructor body entirely:

```typescript
class HttpClient {
  private requestCount = 0;

  constructor(
    readonly baseUrl: string,
    readonly timeoutMs: number,
  ) {}

  async get(path: string) {
    this.requestCount++;
  }
}
```

## readonly Is Shallow

Like `Object.freeze`, `readonly` on a field only prevents reassigning the field itself — it does not make an object or array field's contents immutable:

```typescript
class Cart {
  readonly items: string[] = [];
}

const cart = new Cart();
cart.items = []; // Error: readonly
cart.items.push("shirt"); // OK — the array's contents are still mutable
```

Type the field as `readonly string[]` (or `ReadonlyArray<string>`) if the contents must also stay fixed from outside the class.

## See Also

- [imm-avoid-param-mutation](imm-avoid-param-mutation.md) - Never mutate a function's input parameters
- [type-readonly-arrays](type-readonly-arrays.md) - Prefer `ReadonlyArray<T>` for array parameters you don't intend to mutate
- [api-readonly-public-types](api-readonly-public-types.md) - Expose readonly types on public APIs to prevent external mutation
