# type-narrow-guards

> Use user-defined type guards (`is`) to narrow union types safely

## Why It Matters

When a union type can't be narrowed by a simple `typeof` or `instanceof` check (for example, distinguishing two object shapes), TypeScript needs an explicit hint. A function whose return type is `value is Foo` teaches the compiler how to narrow the type at every call site, so the narrowing logic is written once, tested once, and reused everywhere instead of being re-implemented ad hoc (and inconsistently) each time it's needed.

## Bad

```typescript
type Dog = { kind: "dog"; bark(): void };
type Cat = { kind: "cat"; meow(): void };

function makeSound(pet: Dog | Cat) {
  // Repeated, easy-to-get-wrong inline narrowing at every call site
  if ("bark" in pet) {
    (pet as Dog).bark();
  } else {
    (pet as Cat).meow();
  }
}
```

## Good

```typescript
type Dog = { kind: "dog"; bark(): void };
type Cat = { kind: "cat"; meow(): void };

function isDog(pet: Dog | Cat): pet is Dog {
  return pet.kind === "dog";
}

function makeSound(pet: Dog | Cat) {
  if (isDog(pet)) {
    pet.bark(); // narrowed to Dog, no cast needed
  } else {
    pet.meow(); // narrowed to Cat
  }
}
```

## Guarding Arrays and Filters

Type predicates shine when filtering arrays, where TypeScript can't otherwise narrow the element type:

```typescript
function isNonNull<T>(value: T | null | undefined): value is T {
  return value !== null && value !== undefined;
}

const ids: (number | null)[] = [1, null, 2, null, 3];
const clean: number[] = ids.filter(isNonNull); // without the guard, this stays (number | null)[]
```

## Asserting Instead of Narrowing

For invariants you want to enforce rather than branch on, use an `asserts` function instead of duplicating the guard-then-throw pattern:

```typescript
function assertIsDog(pet: Dog | Cat): asserts pet is Dog {
  if (pet.kind !== "dog") {
    throw new Error(`expected dog, got ${pet.kind}`);
  }
}

function petDog(pet: Dog | Cat) {
  assertIsDog(pet);
  pet.bark(); // narrowed for the rest of the function
}
```

## Common Pitfall

A type guard's body is *not* checked against its predicate — a wrong `is` claim will lie to the compiler and reintroduce runtime bugs. Keep the check condition and the asserted type in sync, and cover guards with unit tests.

## See Also

- [type-discriminated-union](type-discriminated-union.md) - Model variants with discriminated unions and a common tag field
- [type-exhaustive-switch](type-exhaustive-switch.md) - Enforce exhaustiveness checks with a never assertion
- [type-avoid-assertion](type-avoid-assertion.md) - Avoid as type assertions; prefer narrowing or validation
