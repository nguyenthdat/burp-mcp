# fn-structured-clone

> Use `structuredClone()` over `JSON.parse(JSON.stringify())` for deep cloning

## Why It Matters

`JSON.parse(JSON.stringify())` is a common but deeply flawed deep-clone technique: it loses `Date`, `Map`, `Set`, `RegExp`, `undefined`, functions, circular references, and more. `structuredClone()` (available since Node.js 17) handles all these types correctly and supports transferable objects. It's the correct built-in deep cloning method.

## Bad

```js
// JSON round-trip — loses types and fails on circular refs
const original = {
  date: new Date(),
  items: new Set([1, 2, 3]),
  fn: () => 'hello',
  regex: /test/gi,
};

const clone = JSON.parse(JSON.stringify(original));
// clone.date → string "2024-01-01T00:00:00.000Z" (not Date)
// clone.items → {} (empty object, Set destroyed)
// clone.fn → undefined (lost)
// clone.regex → {} (empty object)
```

## Good

```js
const original = {
  date: new Date(),
  items: new Set([1, 2, 3]),
  regex: /test/gi,
  nested: new Map([['key', 'value']]),
  buffer: new ArrayBuffer(8),
};

const clone = structuredClone(original);
// clone.date → Date object ✓
// clone.items → Set {1, 2, 3} ✓
// clone.regex → /test/gi ✓
// clone.nested → Map ✓
// clone.buffer → ArrayBuffer ✓

// Circular references work
const obj = { name: 'circle' };
obj.self = obj;
const cloned = structuredClone(obj);  // Works! ✓
```

## Transferable Objects

```js
// Transfer ownership — original buffer becomes detached (zero-cost for large data)
const buffer = new ArrayBuffer(1024 * 1024);  // 1MB
const clone = structuredClone({ data: buffer }, { transfer: [buffer] });
// buffer.byteLength === 0  (transferred)
// clone.data.byteLength === 1048576
```

## When Exceptions Apply

`structuredClone()` can't clone functions, DOM nodes, or objects with Symbol keys. For simple data with only JSON-compatible types (string, number, boolean, null, array, plain object), `JSON.parse(JSON.stringify())` works but `structuredClone()` is still preferred.

## See Also

- [fn-immutability](./fn-immutability.md) - Immutable data patterns
- [sec-prototype-pollution](./sec-prototype-pollution.md) - Safe cloning vs prototype pollution
