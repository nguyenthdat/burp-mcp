# node-process-exit-avoid

> Avoid `process.exit()` in library code; let the caller control the process

## Why It Matters

`process.exit()` terminates the entire Node process immediately, skipping pending I/O, unflushed logs, and any `finally` blocks or cleanup handlers registered elsewhere in the application. When a library calls `process.exit()` on an internal error, it takes down the host application without giving it any chance to catch the error, log it in its own format, close its own resources, or decide that the error was actually recoverable. Libraries should throw or reject; only the application's top-level entrypoint — which knows what "done" and "fatal" mean for that specific process — should ever call `process.exit()`.

## Bad

```typescript
// validate-config.ts — a library function
export function loadConfig(path: string) {
  if (!fs.existsSync(path)) {
    console.error(`Config file not found: ${path}`);
    process.exit(1); // kills the host application without warning
  }
  return JSON.parse(fs.readFileSync(path, 'utf8'));
}
```

## Good

```typescript
// validate-config.ts — a library function
export class ConfigNotFoundError extends Error {
  constructor(public readonly path: string) {
    super(`Config file not found: ${path}`);
    this.name = 'ConfigNotFoundError';
  }
}

export function loadConfig(path: string) {
  if (!fs.existsSync(path)) {
    throw new ConfigNotFoundError(path);
  }
  return JSON.parse(fs.readFileSync(path, 'utf8'));
}

// main.ts — the application entrypoint decides what "fatal" means
try {
  const config = loadConfig('./config.json');
  startServer(config);
} catch (err) {
  if (err instanceof ConfigNotFoundError) {
    console.error(err.message);
    process.exit(1); // only the entrypoint exits the process
  }
  throw err;
}
```

## When `process.exit()` Is Acceptable

- In the application's own top-level entrypoint file (`main.ts`, `cli.ts`), after all cleanup has run.
- In a CLI tool's final code path, to set a specific exit code for the shell.
- Inside a signal handler performing a deliberate, already-drained shutdown (see `node-graceful-shutdown`).

Never call it from a shared library, a middleware, or any function whose caller might reasonably want to catch and recover from the failure.

## See Also

- [node-graceful-shutdown](node-graceful-shutdown.md) - Handle `SIGTERM`/`SIGINT` for graceful shutdown
- [err-custom-error-class](err-custom-error-class.md) - Model domain errors with custom Error subclasses
- [err-never-swallow](err-never-swallow.md) - Never swallow errors silently
