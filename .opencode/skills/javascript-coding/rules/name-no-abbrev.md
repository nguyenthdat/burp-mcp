# name-no-abbrev

> Avoid abbreviations except for universally recognized ones (`req`, `res`, `err`, `db`, `id`)

## Why It Matters

Abbreviations save a few keystrokes at the cost of readability. `getUsrPrf()` must be mentally expanded to `getUserProfile()`. New team members, search tools, and your future self all benefit from explicit names. The few universally recognized abbreviations (`req`, `res`, `err`, `db`, `id`) are acceptable because they're domain vocabulary, not shortcuts.

## Bad

```js
// Cryptic abbreviations
function getUsrPrf(id) { /* ... */ }
const usrCnt = await getUserCount();
const addrLn1 = user.address?.line1;
const btn = document.querySelector('#submitBtn');

function calcDscnt(prc, pct) { /* ... */ }
```

## Good

```js
// Full words — instantly understandable
function getUserProfile(id) { /* ... */ }
const userCount = await getUserCount();
const addressLine1 = user.address?.line1;
const submitButton = document.querySelector('#submitBtn');

function calculateDiscount(price, percent) { /* ... */ }
```

## Universally Accepted Abbreviations

```js
// These are fine — they're domain vocabulary, not shortcuts
const req = { /* request */ };
const res = { /* response */ };

let err = null;
try { /* ... */ } catch (error) { err = error; }

const db = await connectToDatabase();
const userId = req.params.id;
const apiUrl = 'https://api.example.com';
const htmlOutput = render(template, data);
const urlParams = new URLSearchParams(query);
const io = new Server(server);
```

## Abbreviation Decision Table

| Abbreviation | Full Word | Acceptable? | Reason |
|-------------|-----------|-------------|--------|
| `req` / `res` | request / response | Yes | Universal in HTTP code |
| `err` | error | Yes | Universal in catch blocks |
| `db` | database | Yes | Domain vocabulary |
| `id` | identifier | Yes | Universal convention |
| `url` / `uri` | URL / URI | Yes | Acronym, not abbreviation |
| `idx` | index | No | Just use `index` |
| `ctx` | context | Borderline | Common in middleware but prefer `context` |
| `cfg` | config | No | Just use `config` |
| `usr` | user | No | Just use `user` |
| `btn` | button | No | Just use `button` |

## See Also

- [name-camelCase](./name-camelCase.md) - camelCase convention
- [name-verb-function](./name-verb-function.md) - Verb-first function names
