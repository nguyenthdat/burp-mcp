# name-private-underscore-avoid

> Use `private`/`#` for privacy instead of a leading-underscore convention

## Why It Matters

A leading underscore (`_privateField`) is only a naming convention — it's still a fully public, fully accessible property at both compile time and runtime, so nothing stops external code from reading or writing it. TypeScript's `private` modifier and JavaScript's native `#` private fields are enforced: `private` is checked by the compiler (and stripped safely since it doesn't exist at runtime), while `#` fields are enforced by the JavaScript runtime itself, meaning even code that bypasses TypeScript entirely (plain `.js`, dynamic property access, `any`-typed code) cannot reach them.

## Bad

```typescript
class BankAccount {
  _balance: number;
  _transactionLog: string[];

  constructor(initialBalance: number) {
    this._balance = initialBalance;
    this._transactionLog = [];
  }

  deposit(amount: number) {
    this._balance += amount;
  }
}

const account = new BankAccount(100);
account._balance = 1_000_000; // "private" in name only — this compiles and runs fine
```

## Good

```typescript
class BankAccount {
  #balance: number;
  #transactionLog: string[];

  constructor(initialBalance: number) {
    this.#balance = initialBalance;
    this.#transactionLog = [];
  }

  deposit(amount: number) {
    this.#balance += amount;
    this.#transactionLog.push(`deposit:${amount}`);
  }

  get balance(): number {
    return this.#balance;
  }
}

const account = new BankAccount(100);
account.balance = 1_000_000; // Error: no setter exists
// account.#balance is not even syntactically reachable from outside the class
```

## `private` (TypeScript) vs `#` (JavaScript Native)

| | `private` keyword | `#` field |
|---|---|---|
| Enforced by | TypeScript compiler only | JavaScript runtime |
| Visible via `Object.keys`/JSON | Yes (it's a normal property under the hood) | No |
| Reachable from compiled `.js` / `any`-typed code | Yes — the check disappears at runtime | No — genuinely inaccessible |
| Works in older transpilation targets | Always | Requires ES2022+ target (widely supported now) |

For genuinely sensitive internal state (security-relevant fields, invariants that must never be bypassed), prefer `#`. For ordinary internal implementation detail where only compile-time discipline is needed and you want it visible to serialization/debugging, TypeScript's `private` is sufficient and slightly more ergonomic with decorators and some reflection-based libraries.

## See Also

- [name-no-hungarian](name-no-hungarian.md) - Avoid Hungarian notation and redundant type suffixes in identifier names
- [api-minimal-surface](api-minimal-surface.md) - Expose the smallest public API surface that satisfies consumers
- [imm-readonly-class-fields](imm-readonly-class-fields.md) - Mark class fields `readonly` when they are set once in the constructor
