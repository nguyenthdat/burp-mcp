# node-process-exit

> Use meaningful process exit codes: 0 for success, 1 for error, custom codes for specific failures

## Why It Matters

Process exit codes communicate success or failure to the operating system, CI pipelines, and process managers. A process that crashes without an exit code or always returns 0 masks failures. Explicit exit codes enable orchestration tools to decide whether to restart, alert, or continue.

## Bad

```js
// No exit code — defaults to 0 (success) even on failure
process.on('uncaughtException', (err) => {
  console.error(err);
  process.exit();  // Exit code 0 — CI thinks it succeeded
});

// Indiscriminate exit
if (error) {
  process.exit(1);  // Same code for all errors
}
```

## Good

```js
// Explicit exit codes
const EXIT_CODES = {
  SUCCESS: 0,
  GENERAL_ERROR: 1,
  CONFIG_ERROR: 2,
  DATABASE_ERROR: 3,
};

process.on('uncaughtException', (err) => {
  console.error('Fatal:', err);
  process.exit(EXIT_CODES.GENERAL_ERROR);
});

async function main() {
  try {
    const config = loadConfig();
  } catch (err) {
    console.error('Configuration error:', err.message);
    process.exit(EXIT_CODES.CONFIG_ERROR);
  }

  try {
    await connectToDatabase();
  } catch (err) {
    console.error('Database connection failed:', err.message);
    process.exit(EXIT_CODES.DATABASE_ERROR);
  }

  process.exit(EXIT_CODES.SUCCESS);
}
```

## When Exceptions Apply

For simple CLI tools and scripts, `process.exit(0)` and `process.exit(1)` are sufficient. Introduce custom exit codes when the process is managed by orchestration tools.

## See Also

- [err-global-handlers](./err-global-handlers.md) - Global error handlers
- [node-signal-handling](./node-signal-handling.md) - Graceful shutdown on signals
