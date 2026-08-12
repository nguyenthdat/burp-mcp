# anti-dynamic-export

> Don't use `export default` conditionally — exports must be statically analyzable

## Why It Matters

Conditional exports (`if (condition) export default`) break static analysis. Bundlers, tree-shakers, and linters rely on the module graph being statically decidable. Conditional exports make the module's public API unpredictable and runtime-dependent. Always export at the top level.

## Bad

```js
// Conditional export — unpredictable API surface
let Service;
if (process.env.USE_V2) {
  Service = V2Service;
} else {
  Service = V1Service;
}

export default Service;

// Export inside a conditional block
if (condition) {
  export function helper() { }  // Syntax error in strict ESM anyway
}
```

## Good

```js
// Always export at the top level
export { V1Service, V2Service };

// Let the consumer decide which to use based on config
// consumer.js
import { V1Service, V2Service } from './services.js';

const Service = process.env.USE_V2 ? V2Service : V1Service;
export default Service;

// Or export a factory
export function createService(version) {
  return version === 'v2' ? new V2Service() : new V1Service();
}
```

## When Exceptions Apply

There are no exceptions. The ESM specification requires exports to be at the top level and statically analyzable.

## See Also

- [mod-named-over-default](./mod-named-over-default.md) - Prefer named exports
- [mod-esm-over-cjs](./mod-esm-over-cjs.md) - ES modules
