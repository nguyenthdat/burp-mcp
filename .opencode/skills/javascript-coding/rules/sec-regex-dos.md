# sec-regex-dos

> Avoid catastrophic backtracking in regular expressions with untrusted input

## Why It Matters

Certain regular expression patterns can exhibit exponential backtracking on carefully crafted input, causing the event loop to block for seconds or minutes (ReDoS — Regular Expression Denial of Service). This is a denial-of-service vector that can bring down a Node.js server with a single malicious request. Regexes with nested quantifiers, alternation, and backreferences are especially dangerous.

## Bad

```js
// Vulnerable to ReDoS — nested quantifiers
const emailRegex = /^([a-zA-Z0-9])(([\-.]|[_]+)?([a-zA-Z0-9]+))*(@){1}[a-z0-9]+[.]{1}(([a-z]{2,3})|([a-z]{2,3}[.]{1}[a-z]{2,3}))$/;

// Vulnerable — (a+)+ causes exponential backtracking on "aaaaaaaaaaaaaaaaaaaa!"
const pattern = /^(a+)+$/;

// Vulnerable — alternation inside repetition
const regex = /(\w+\s?)*:/;

// Using untrusted input with a complex regex
app.post('/validate', (req, res) => {
  const valid = emailRegex.test(req.body.email);  // Can hang the server
  res.json({ valid });
});
```

## Good

```js
// Use simple, linear regex patterns
const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;

// Or use the built-in URL/email validation
function isValidEmail(email) {
  try {
    // Basic structure check + length limits
    if (email.length > 254) return false;
    const [local, domain] = email.split('@');
    return local.length > 0 && domain.includes('.');
  } catch {
    return false;
  }
}

// Set a timeout for regex execution
import { setTimeout } from 'node:timers/promises';

async function safeRegexTest(regex, input, timeoutMs = 100) {
  const result = await Promise.race([
    Promise.resolve(regex.test(input)),
    setTimeout(timeoutMs).then(() => {
      throw new Error('Regex execution timed out');
    }),
  ]);
  return result;
}

// Use a ReDoS-safe library
import { isEmail } from 'validator';
if (!isEmail(req.body.email)) {
  return res.status(400).json({ error: 'Invalid email' });
}
```

## Red Flags for ReDoS

```js
// These patterns are ReDoS-prone:
/(a+)+/          // Nested quantifier
/(a|aa)+/        // Overlapping alternation
/(.*a){n}/       // Repetition of a capturing group with .*
/(?:a|b)*a/      // Alternation with overlapping suffix
```

## When Exceptions Apply

Regexes used only on trusted input (e.g., compile-time parsing) or with fixed-length patterns are safe. Always validate untrusted input with simple, linear regexes or purpose-built libraries.

## See Also

- [sec-input-size-limits](./sec-input-size-limits.md) - Limit input sizes
- [perf-prepare-regex](./perf-prepare-regex.md) - Compile regexes outside loops
