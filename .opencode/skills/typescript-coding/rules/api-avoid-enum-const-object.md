# api-avoid-enum-const-object

> Prefer literal unions or `as const` objects over `enum`

## Why It Matters

TypeScript's `enum` has real, well-documented pitfalls: numeric enums allow any number to be assigned where the enum type is expected (defeating the type safety enums are supposed to provide), enums generate actual runtime JavaScript objects that aren't erased like the rest of TypeScript's type syntax (which conflicts with `isolatedModules`/`verbatimModuleSyntax` builds and adds bundle size), and `const enum` — the workaround for the runtime cost — is unsupported by isolated-module compilers like esbuild and Babel. Literal unions with `as const` objects give you the same ergonomics with none of these issues, and are erased entirely at compile time like the rest of TypeScript's type system.

## Bad

```typescript
enum Status {
  Pending,
  Active,
  Archived,
}

function process(status: Status) {}

process(0); // compiles! numeric enums accept any number
process(Status.Active); // also fine, but the type gives no real protection

// const enum avoids some runtime cost but breaks under isolatedModules /
// single-file transpilation (esbuild, Babel, ts-jest in isolated mode)
const enum Fast { A, B }
```

## Good

```typescript
export const Status = {
  Pending: "pending",
  Active: "active",
  Archived: "archived",
} as const;

export type Status = (typeof Status)[keyof typeof Status]; // "pending" | "active" | "archived"

function process(status: Status) {}

process("pending");     // OK
process("Status.Active"); // compile error — literal union rejects anything not in the set
process(Status.Active); // OK, and IDE autocomplete still works via Status.<Tab>
```

## Comparison

| Concern | `enum` | `as const` object + literal union |
|---|---|---|
| Runtime footprint | Generates a real JS object (or two, for reverse-mapped numeric enums) | Erased entirely unless you reference the const object's values |
| Works with `isolatedModules`/single-file transpilers | `const enum` does not | Yes, always |
| Type safety for numeric-like values | Numeric enums accept any `number` | Only exact literal values accepted |
| IDE autocomplete for known values | Yes, via `Status.Active` | Yes, via `Status.Active` (same ergonomics) |
| Serializes cleanly to JSON | Numeric enums serialize as numbers (often surprising) | Serializes as whatever literal you chose (usually a readable string) |

## Enforcing With ESLint

```jsonc
{
  "rules": {
    "no-restricted-syntax": ["warn", { "selector": "TSEnumDeclaration", "message": "Prefer literal unions or `as const` objects over enum." }]
  }
}
```

## See Also

- [api-interface-vs-type](api-interface-vs-type.md) - Use `interface` for extendable object shapes, `type` for unions/aliases/mapped types
- [imm-as-const-literal](imm-as-const-literal.md) - Using `as const` to infer the narrowest possible literal types
- [type-const-assertion](type-const-assertion.md) - How `as const` changes type inference for objects and arrays
