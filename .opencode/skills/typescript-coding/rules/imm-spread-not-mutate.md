# imm-spread-not-mutate

> Create updated copies with spread/rest instead of mutating in place

## Why It Matters

Mutating an object or array that other code holds a reference to causes action-at-a-distance bugs: a component re-renders unexpectedly (or doesn't, because a framework's shallow-equality check sees the same reference), a cache entry silently changes underneath its owner, or a function's caller sees its input altered after the call. Spreading into a new object/array makes updates explicit and produces a new reference, which plays correctly with React state, memoization, and equality checks.

## Bad

```typescript
interface User {
  id: string;
  name: string;
  roles: string[];
}

function promoteToAdmin(user: User): User {
  user.roles.push("admin"); // mutates the caller's array
  return user;
}

function updateName(user: User, name: string): User {
  user.name = name; // mutates the caller's object
  return user;
}

const original = { id: "1", name: "Ada", roles: ["user"] };
const updated = updateName(original, "Grace");
console.log(original.name); // "Grace" — surprise, the original changed too
```

## Good

```typescript
interface User {
  id: string;
  name: string;
  roles: string[];
}

function promoteToAdmin(user: User): User {
  return { ...user, roles: [...user.roles, "admin"] };
}

function updateName(user: User, name: string): User {
  return { ...user, name };
}

const original = { id: "1", name: "Ada", roles: ["user"] };
const updated = updateName(original, "Grace");
console.log(original.name); // "Ada" — untouched
console.log(updated.name);  // "Grace"
```

## Common Update Patterns

```typescript
// Update one field in an array of objects by id
const nextUsers = users.map((u) => (u.id === targetId ? { ...u, active: false } : u));

// Remove a key
const { password, ...safeUser } = user;

// Merge partial updates
const merged = { ...defaults, ...overrides };

// Insert into an array immutably
const withNewItem = [...items.slice(0, index), newItem, ...items.slice(index)];

// Remove an array item immutably
const withoutItem = items.filter((item) => item.id !== targetId);
```

## Caveat: Spread Is Shallow

`{ ...obj }` only copies top-level properties. Nested objects/arrays are still shared references, so mutating `copy.nested.value` also mutates `original.nested.value`. For deeply nested updates, spread each level explicitly or reach for a structural-sharing helper like Immer.

## See Also

- [imm-avoid-array-mutation](imm-avoid-array-mutation.md) - Avoid mutating array methods on shared/shared-reference arrays
- [imm-structural-sharing](imm-structural-sharing.md) - Use structural sharing so immutable updates don't copy untouched subtrees
- [imm-avoid-param-mutation](imm-avoid-param-mutation.md) - Never mutate a function's input parameters
