# anti-throw-non-error

> Don't throw non-Error objects — always throw an instance of Error or its subclasses

## Why It Matters

Throwing a string, number, or plain object produces no stack trace, making debugging nearly impossible. Error-reporting tools can't extract `error.message` or `error.stack` from non-Error throws. The `cause` property (ES2022) only works on Error instances. Always throw an Error.

## Bad

```js
// String — no stack trace
throw 'Something went wrong';

// Number — no stack trace
throw 404;

// Plain object — no .stack
throw { message: 'Not found', code: 404 };

// null/undefined
throw null;
```

## Good

```js
// Error instance — stack trace captured
throw new Error('Something went wrong');

// Custom error classes with metadata
throw new NotFoundError('User not found');

// Built-in error types when appropriate
throw new TypeError('value must be a string');
throw new RangeError('index out of range');

// With cause chaining
try {
  await fetch(url);
} catch (err) {
  throw new Error(`Failed to fetch ${url}`, { cause: err });
}
```

## ESLint Enforcement

```js
// eslint.config.mjs
{
  rules: {
    'no-throw-literal': 'error',
  },
}
```

## When Exceptions Apply

There are no exceptions. Always throw Error instances. ESLint's `no-throw-literal` catches this automatically.

## See Also

- [err-no-throw-string](./err-no-throw-string.md) - Always throw Error instances
- [err-custom-error-classes](./err-custom-error-classes.md) - Custom error classes
