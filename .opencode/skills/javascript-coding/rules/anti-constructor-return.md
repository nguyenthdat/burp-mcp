# anti-constructor-return

> Don't return a value from a constructor — it's either ignored or overrides the instance

## Why It Matters

Constructors implicitly return `this`. Explicitly returning a non-object value is silently ignored. Returning an object overrides `this`, meaning `new MyClass()` returns something that's not an instance of `MyClass`. Both behaviors are confusing and violate the principle of least surprise.

## Bad

```js
class User {
  constructor(data) {
    if (!data) {
      return null;  // Ignored! `new User(null)` still returns a User instance
    }
    this.name = data.name;
  }
}

// Returns a different object — not instanceof User
class MaybeUser {
  constructor(data) {
    if (!data) return { error: 'No data' };
    this.name = data.name;
  }
}

const user = new MaybeUser(null);
user instanceof MaybeUser;  // false — what?
```

## Good

```js
// Use a factory function instead of constructor return
function createUser(data) {
  if (!data) return null;
  return { name: data.name };
}

// Or throw on invalid input
class User {
  constructor(data) {
    if (!data) throw new TypeError('User data is required');
    this.name = data.name;
  }
}

// Use static factory methods for conditional creation
class User {
  constructor(data) {
    this.name = data.name;
  }

  static create(data) {
    if (!data) return null;
    return new User(data);
  }
}
```

## When Exceptions Apply

There are no exceptions. Use factory functions or static methods instead of overriding constructor return behavior.

## See Also

- [fn-composition-over-inheritance](./fn-composition-over-inheritance.md) - Prefer composition
- [err-custom-error-classes](./err-custom-error-classes.md) - Custom Error classes
