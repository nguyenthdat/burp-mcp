# test-vitest-jest-setup

> Follow standard Vitest/Jest project conventions for config and structure

## Why It Matters

Inconsistent test configuration (mixed globals, ad-hoc `ts-node` transforms, per-file `tsconfig` overrides) causes tests that pass locally but fail in CI, or that silently skip type-checking. Following the standard Vitest/Jest layout — a single config file, a shared setup file, and predictable file-naming — means any engineer can run `npm test` in any package of the repo and get the same result, and tooling (coverage, watch mode, IDE test runners) works out of the box.

## Bad

```typescript
// Ad-hoc, per-file test runner setup scattered across the repo
// some-file.test.ts
import "reflect-metadata"; // needed but not declared anywhere central
// @ts-nocheck
const assert = require("assert");

assert.equal(1 + 1, 2); // no test runner, no reporter, no CI integration
```

## Good

```typescript
// vitest.config.ts
import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    globals: false, // prefer explicit imports over injected globals
    environment: "node",
    setupFiles: ["./test/setup.ts"],
    coverage: {
      provider: "v8",
      reporter: ["text", "html"],
      thresholds: { lines: 80, functions: 80 },
    },
  },
});
```

```typescript
// src/cart.test.ts
import { describe, expect, it } from "vitest";
import { Cart } from "./cart";

describe("Cart.total", () => {
  it("should sum item prices", () => {
    const cart = new Cart();
    cart.addItem({ id: "sku-1", price: 100 });
    expect(cart.total()).toBe(100);
  });
});
```

## Configuration

- Colocate `*.test.ts` (or `*.spec.ts`) next to the source file it covers, or mirror the source tree under `test/` — pick one convention repo-wide (see `proj-colocate-tests`).
- Keep one `vitest.config.ts` (or `jest.config.ts`) per package in a monorepo, extending a shared base config rather than duplicating options.
- Prefer explicit imports (`import { describe, it, expect } from "vitest"`) over `globals: true` so editors and type-checkers resolve test APIs without extra ambient types.
- Run tests in CI with `--reporter=junit` (or Jest's `--ci --reporters=default --reporters=jest-junit`) so failures surface in PR annotations.

## See Also

- [proj-colocate-tests](proj-colocate-tests.md) - keep test files next to the source they cover
- [test-coverage-meaningful](test-coverage-meaningful.md) - Target meaningful coverage of behavior, not a 100% coverage vanity metric
- [lint-ci-lint-gate](lint-ci-lint-gate.md) - gate merges on lint and type-check passing in CI
