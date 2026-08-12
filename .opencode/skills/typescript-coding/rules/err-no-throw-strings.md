# err-no-throw-strings

> Always throw `Error` instances, never strings or plain objects

## Why It Matters

JavaScript lets you `throw` any value at all — a string, a number, a plain object — but only `Error` instances automatically capture a stack trace at the point of construction. Throwing a bare string means that by the time it's caught (possibly several async hops away), there's no way to tell where it originated; you're left debugging with just a message and no call-stack breadcrumb. It also breaks `instanceof Error` checks that error-handling middleware, logging libraries, and test frameworks rely on.

## Bad

```typescript
function withdraw(balance: number, amount: number): number {
  if (amount > balance) {
    throw "insufficient funds"; // no stack trace, fails instanceof Error checks
  }
  return balance - amount;
}

try {
  withdraw(100, 200);
} catch (err) {
  console.log(err.stack); // undefined — strings have no .stack property
}
```

## Good

```typescript
class InsufficientFundsError extends Error {
  constructor(public readonly balance: number, public readonly amount: number) {
    super(`insufficient funds: balance ${balance}, requested ${amount}`);
    this.name = "InsufficientFundsError";
  }
}

function withdraw(balance: number, amount: number): number {
  if (amount > balance) {
    throw new InsufficientFundsError(balance, amount);
  }
  return balance - amount;
}

try {
  withdraw(100, 200);
} catch (err) {
  if (err instanceof InsufficientFundsError) {
    console.log(err.stack); // full stack trace captured at the throw site
    console.log(err.balance, err.amount); // structured data too
  }
}
```

## Why This Matters More in Async Code

In an async call chain, a thrown string loses even the minimal context a synchronous stack might otherwise offer at the catch site, because the catch often runs on a different tick:

```typescript
async function step1() {
  throw "step1 failed"; // by the time this is caught, there's no trace of where in step1 it happened
}
```

## Configuration

```json
{
  "rules": {
    "no-throw-literal": "error",
    "@typescript-eslint/only-throw-error": "error"
  }
}
```

`@typescript-eslint/only-throw-error` (the modern successor to `no-throw-literal` for TypeScript) also flags throwing values typed as `any`, since those can hide a non-Error value passing through unchecked.

## See Also

- [err-custom-error-class](err-custom-error-class.md) - Extend Error with custom subclasses that carry structured context
- [err-typed-catch-unknown](err-typed-catch-unknown.md) - Type the catch binding as unknown and narrow before use
- [lint-typescript-eslint-recommended](lint-typescript-eslint-recommended.md) - Baseline recommended typescript-eslint rule set
