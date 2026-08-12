# imm-as-const-literal

> Freeze literal object/array structures with `as const`

## Why It Matters

Without `as const`, TypeScript widens object and array literals to their general shapes: string properties become `string`, arrays become mutable `T[]`, and numeric literals become `number`. This throws away the exact literal information the compiler could otherwise use for narrowing, discriminated unions, and tuple positions. `as const` locks the literal down to its most specific type and marks nested properties `readonly`, giving you compile-time immutability and much stronger type inference for free.

## Bad

```typescript
const config = {
  env: "production",
  retries: 3,
  regions: ["us-east-1", "eu-west-1"],
};
// config.env is typed `string`, not `"production"`
// config.regions is `string[]`, so this compiles but shouldn't be allowed:
config.env = "staging";
config.regions.push("ap-south-1");

function connect(region: "us-east-1" | "eu-west-1") {}
connect(config.regions[0]); // Error: string not assignable to the union
```

## Good

```typescript
const config = {
  env: "production",
  retries: 3,
  regions: ["us-east-1", "eu-west-1"],
} as const;
// config.env: "production"
// config.regions: readonly ["us-east-1", "eu-west-1"]

config.env = "staging"; // Error: cannot assign to readonly property
config.regions.push("ap-south-1"); // Error: push does not exist on readonly tuple

function connect(region: "us-east-1" | "eu-west-1") {}
connect(config.regions[0]); // OK: literal type flows through
```

## Deriving Union Types From as const Arrays

```typescript
const ROLES = ["admin", "editor", "viewer"] as const;
type Role = (typeof ROLES)[number]; // "admin" | "editor" | "viewer"

function assertRole(value: string): asserts value is Role {
  if (!ROLES.includes(value as Role)) {
    throw new Error(`Invalid role: ${value}`);
  }
}
```

This pattern is the standard way to keep a runtime array and its corresponding union type in sync without duplicating the values.

## as const vs Object.freeze

| | `as const` | `Object.freeze` |
|---|---|---|
| Enforcement | Compile-time only | Runtime (shallow) |
| Cost | Zero runtime cost | Small runtime overhead |
| Deep | Deep (nested literals become readonly) | Shallow by default |
| Use when | Data never leaves trusted, type-checked code | Data crosses an untyped boundary (JSON, plugin, `any`) |

## See Also

- [type-const-assertion](type-const-assertion.md) - Use `as const` for literal type narrowing in function signatures
- [imm-object-freeze-runtime](imm-object-freeze-runtime.md) - Use `Object.freeze` when you need a runtime immutability guarantee
- [type-template-literal](type-template-literal.md) - Build precise string types from literal unions
