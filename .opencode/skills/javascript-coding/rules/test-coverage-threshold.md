# test-coverage-threshold

> Set minimum code coverage thresholds in CI and fail the build when they drop

## Why It Matters

Coverage thresholds prevent the codebase from gradually losing test coverage. Without enforcement, new code often ships without tests. A threshold creates a floor that the team must maintain. Start with a modest threshold (70-80%) and raise it as the codebase matures.

## Bad

```jsonc
// No coverage enforcement — coverage can drop to 0
{
  "scripts": {
    "test": "node --test"
  }
}
```

## Good

```jsonc
// node --test with coverage thresholds (Node.js 22+)
{
  "scripts": {
    "test": "node --test --experimental-test-coverage",
    "test:ci": "node --test --experimental-test-coverage --test-coverage-lines=80 --test-coverage-branches=70 --test-coverage-functions=80"
  }
}
```

```bash
# CI configuration
node --test \
  --experimental-test-coverage \
  --test-coverage-lines=80 \
  --test-coverage-branches=70 \
  --test-coverage-functions=80
```

## Using c8 for Coverage (Node < 22)

```jsonc
{
  "scripts": {
    "test:coverage": "c8 --lines 80 --branches 70 --functions 80 node --test"
  }
}
```

## Coverage Configuration

```jsonc
// .c8rc.json
{
  "lines": 80,
  "branches": 70,
  "functions": 80,
  "statements": 80,
  "include": ["src/**/*.js"],
  "exclude": [
    "src/**/*.test.js",
    "src/**/*.spec.js",
    "tests/**"
  ],
  "reporter": ["text", "lcov", "html"]
}
```

## When Exceptions Apply

Don't enforce coverage on prototype or throwaway code. For critical paths (payment processing, auth), consider higher thresholds (90%+). For UI code, lower thresholds may be acceptable.

## See Also

- [test-node-test-runner](./test-node-test-runner.md) - Node.js built-in test runner
- [test-integration-separate](./test-integration-separate.md) - Separate unit and integration tests
