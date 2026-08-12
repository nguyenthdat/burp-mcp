# async-no-async-constructor

> Avoid `async` constructors; use a static async factory method instead

## Why It Matters

JavaScript constructors must return an instance synchronously — they cannot be declared `async` and cannot return a `Promise` (the language simply ignores a returned value that isn't derived from `this`). Trying to "fake" async initialization in a constructor — firing off a promise and stashing it on the instance — leaves the object in a partially-initialized state that callers can accidentally use before it's ready. A static async factory method makes the requirement to wait for initialization explicit and enforced by the type system.

## Bad

```typescript
class DatabaseConnection {
  private client?: Client;

  constructor(url: string) {
    // Fire-and-forget: constructor returns before this resolves
    this.connect(url);
  }

  private async connect(url: string) {
    this.client = await createClient(url);
  }

  query(sql: string) {
    // client may be undefined here — no way for TypeScript to know
    return this.client!.query(sql);
  }
}

const db = new DatabaseConnection(url);
db.query("SELECT 1"); // may throw or hang depending on timing
```

## Good

```typescript
class DatabaseConnection {
  private constructor(private readonly client: Client) {}

  static async connect(url: string): Promise<DatabaseConnection> {
    const client = await createClient(url);
    return new DatabaseConnection(client);
  }

  query(sql: string) {
    // client is guaranteed initialized by the time this type exists
    return this.client.query(sql);
  }
}

const db = await DatabaseConnection.connect(url);
db.query("SELECT 1"); // always safe
```

## Why a Private Constructor Helps

Making the constructor `private` (or `protected`) prevents callers from bypassing the factory and constructing a half-initialized instance directly. Combined with marking dependent fields `readonly` and non-optional, the type system now guarantees that any value of type `DatabaseConnection` is fully connected — there's no `undefined` state to check for at every call site.

## Alternative: Async Init Method for Long-Lived Singletons

```typescript
class ConfigStore {
  private static instance: Promise<ConfigStore> | undefined;

  static get(): Promise<ConfigStore> {
    this.instance ??= ConfigStore.load();
    return this.instance;
  }

  private static async load(): Promise<ConfigStore> {
    const data = await fetchRemoteConfig();
    return new ConfigStore(data);
  }

  private constructor(private readonly data: Config) {}
}
```

## See Also

- [async-no-async-constructor](async-no-async-constructor.md) - (this rule)
- [api-builder-pattern](api-builder-pattern.md) - Use a builder/fluent API for objects with many optional construction parameters
- [err-result-pattern](err-result-pattern.md) - Model fallible construction with a Result type instead of throwing
