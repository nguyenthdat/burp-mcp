# sec-no-eval

> Never use eval() or the Function() constructor with untrusted input

## Why It Matters

`eval()` and `new Function()` execute arbitrary JavaScript code from a string. If any part of that string comes from user input, an attacker can inject malicious code (Remote Code Execution). Both constructors also prevent JavaScript engine optimizations and make your code impossible to analyze statically.

## Bad

```js
// User-controlled code execution — critical RCE vulnerability
const expression = req.query.expression;
const result = eval(expression);

// Dynamic function from user input — same risk
const fn = new Function('x', `return ${req.body.formula}`);
const result = fn(42);

// setTimeout with string (also eval)
setTimeout('deleteAllRecords()', 1000);
```

## Good

```js
// Use a safe math expression parser
import { evaluate } from 'mathjs';

const result = evaluate(req.query.expression);

// Use JSON for data, not code
const config = JSON.parse(req.body);

// Use object property access instead of dynamic evaluation
const handlers = {
  'createUser': createUserHandler,
  'deleteUser': deleteUserHandler,
};

const handler = handlers[req.body.action];
if (handler) {
  await handler(req.body.params);
}

// setTimeout with function reference (not a string)
setTimeout(() => deleteAllRecords(), 1000);
```

## When Exceptions Apply

`eval()` is acceptable only for:
- Build tools and code generators processing trusted source code
- REPL environments where the user is the developer themselves
- Template engines that compile templates at build time, not runtime

Never use eval with user input in a production server.

## See Also

- [sec-input-sanitize](./sec-input-sanitize.md) - Sanitize all user input
- [sec-avoid-os-command](./sec-avoid-os-command.md) - Avoid shell command injection
