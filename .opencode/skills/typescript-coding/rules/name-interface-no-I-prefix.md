# name-interface-no-I-prefix

> Don't prefix interfaces with `I`

## Why It Matters

The `I` prefix (`IUser`, `IRepository`) comes from languages like C# and older Java conventions where it distinguished an interface from a class at a glance. In TypeScript, `interface` and `type` are structurally interchangeable — a class can implement an interface, a plain object literal can satisfy it, and callers rarely need to know or care whether a given type was declared with `interface` or `type`. Prefixing every interface with `I` adds visual noise to every usage site for a distinction TypeScript itself doesn't treat as meaningfully different, and it creates awkward renames the moment an interface's implementation detail changes (a type that starts as an `interface` and later becomes a `type` alias, or vice versa, forcing every call site to rename).

## Bad

```typescript
interface IUser {
  id: string;
  name: string;
}

interface IRepository<T> {
  findById(id: string): Promise<T | null>;
  save(entity: T): Promise<void>;
}

class UserRepository implements IRepository<IUser> {
  async findById(id: string): Promise<IUser | null> {
    // ...
    return null;
  }
  async save(entity: IUser): Promise<void> {}
}
```

## Good

```typescript
interface User {
  id: string;
  name: string;
}

interface Repository<T> {
  findById(id: string): Promise<T | null>;
  save(entity: T): Promise<void>;
}

class UserRepository implements Repository<User> {
  async findById(id: string): Promise<User | null> {
    // ...
    return null;
  }
  async save(entity: User): Promise<void> {}
}
```

## Naming Collisions Between an Interface and Its Implementation

When a single concrete class implements a single interface and both need distinct names, prefer naming the *implementation* more specifically rather than decorating the interface:

```typescript
interface Logger {
  log(message: string): void;
}

class ConsoleLogger implements Logger { /* ... */ }
class FileLogger implements Logger { /* ... */ }
// Not: interface ILogger, class Logger implements ILogger
```

This reads naturally at every call site (`function process(logger: Logger)`) instead of forcing every consumer to think in terms of an `I`-prefixed abstraction layer.

## See Also

- [name-PascalCase-types](name-PascalCase-types.md) - Use `PascalCase` for types, interfaces, classes, and enums
- [api-interface-vs-type](api-interface-vs-type.md) - Choose between `interface` and `type` based on concrete tradeoffs, not habit
- [name-no-hungarian](name-no-hungarian.md) - Avoid Hungarian notation and redundant type suffixes in identifier names
