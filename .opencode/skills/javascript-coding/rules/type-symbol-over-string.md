# type-symbol-over-string

> Use `Symbol` for truly unique property keys to avoid name collisions

## Why It Matters

String keys on objects can collide — if two libraries both use the property name `"type"` or `"id"` on the same object, they overwrite each other. `Symbol` creates guaranteed-unique keys that cannot collide, making them ideal for metadata, protocol markers, and internal properties that should not be enumerable.

## Bad

```js
// String keys can collide
const serviceA = { type: 'a', data: 'Service A data' };
const serviceB = { type: 'b', data: 'Service B data' };

// If these get mixed on one object...
const mixed = { ...serviceA, ...serviceB };
mixed.type;  // 'b' — overwritten, lost serviceA's type

// Internal property polluting iteration
const obj = { name: 'item', _internalId: 123 };
for (const key in obj) {
  console.log(key);  // 'name', '_internalId' — internal leaks
}
```

## Good

```js
// Symbols — guaranteed unique
const typeKey = Symbol('type');
const serviceA = { [typeKey]: 'a', data: 'Service A data' };
const serviceB = { [typeKey]: 'b', data: 'Service B data' };

// Same key won't collide
const mixed = { ...serviceA, ...serviceB };
mixed[typeKey];  // References same Symbol — whichever was spread last wins

// Different symbols won't collide
const typeKeyA = Symbol('type');
const typeKeyB = Symbol('type');
obj[typeKeyA] = 'a';
obj[typeKeyB] = 'b';  // Different property — no collision

// Symbols are not enumerable
const obj = { name: 'item', [Symbol('id')]: 123 };
console.log(Object.keys(obj));  // ['name']
console.log(Object.getOwnPropertySymbols(obj));  // [Symbol(id)]
```

## Well-Known Symbols

```js
// Use standard symbols for protocol customization
class IterableCollection {
  *[Symbol.iterator]() {
    yield* this.items;
  }

  [Symbol.toPrimitive](hint) {
    return hint === 'number' ? this.items.length : this.toString();
  }
}
```

## When Exceptions Apply

Symbols are not serializable in JSON (they're silently dropped). Don't use them for data that needs to be sent over the network or stored. Use symbols for runtime metadata, protocol markers, and private-like properties.

## See Also

- [type-tagged-unions](./type-tagged-unions.md) - Discriminated unions
- [type-no-magic-strings](./type-no-magic-strings.md) - Constants over magic strings
