# async-top-level-await

> Use top-level `await` only at module entry points

## Why It Matters

Top-level `await` (available in ES modules since ES2022, and in Node.js ESM) pauses the *evaluation of the entire module graph* that depends on that module until the awaited promise settles. Used casually in a widely-imported utility module, it can silently delay application startup, create load-order surprises, or deadlock circular imports that both use top-level await. Confining it to entry-point files (the script Node runs directly, or a bootstrap module) keeps the blocking behavior visible and intentional rather than an implicit side effect buried in a shared dependency.

## Bad

```typescript
// src/lib/config.ts — imported from dozens of places
export const config = await loadConfigFromRemoteServer();
// Every module that imports anything from this file now has its
// evaluation blocked on a network call, even if it only needs a
// synchronous helper from the same file.
```

## Good

```typescript
// src/lib/config.ts — synchronous, lazy
let cached: Config | undefined;

export async function getConfig(): Promise<Config> {
  cached ??= await loadConfigFromRemoteServer();
  return cached;
}

// src/main.ts — the entry point, top-level await is appropriate here
import { getConfig } from "./lib/config.js";

const config = await getConfig();
startServer(config);
```

## When Top-Level Await at the Entry Point Is the Right Tool

```typescript
// scripts/migrate.ts — a standalone script, run directly via `node scripts/migrate.ts`
import { connect } from "./db.js";

const db = await connect(process.env.DATABASE_URL!);
await db.runMigrations();
await db.close();
console.log("Migrations complete");
```

## Circular Import Hazard

```typescript
// a.ts
import { b } from "./b.js";
export const a = await Promise.resolve("a");

// b.ts
import { a } from "./a.js"; // may see `a` as unresolved/TDZ error
export const b = "b";
```

Two modules that both use top-level await and import from each other can deadlock or throw, because ESM must finish evaluating one before the other can proceed. Keep top-level await out of modules involved in import cycles.

## See Also

- [proj-verbatim-module-syntax](proj-verbatim-module-syntax.md) - Configuring TypeScript's ESM/CJS interop correctly
- [node-esm-first](node-esm-first.md) - Prefer ES modules over CommonJS in new Node.js projects
- [proj-module-boundaries](proj-module-boundaries.md) - Keep import graphs acyclic and boundaries explicit
