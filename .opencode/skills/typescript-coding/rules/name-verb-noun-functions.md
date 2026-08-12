# name-verb-noun-functions

> Name functions with a leading verb describing the action they perform

## Why It Matters

A function's name is the primary interface a caller reads before deciding whether (and how) to call it — a noun-only name (`user`, `total`, `validation`) leaves the reader guessing whether calling it computes something, mutates something, fetches remotely, or just returns a stored value. Leading with a verb (`getUser`, `calculateTotal`, `validateForm`) tells the reader what *action* happens immediately, and different verbs (`get` vs `fetch` vs `find` vs `create`) further signal cost and behavior (a cheap in-memory lookup versus a network call versus something that might return nothing).

## Bad

```typescript
function user(id: string): User { /* looks up and returns a user */ }
function total(items: Item[]): number { /* sums item prices */ }
function validation(form: FormData): boolean { /* validates a form */ }
function email(address: string): boolean { /* checks if an email is valid */ }

// At the call site, none of these read as actions:
const u = user(id);
const isValid = validation(form);
```

## Good

```typescript
function getUser(id: string): User { /* ... */ }
function calculateTotal(items: Item[]): number { /* ... */ }
function validateForm(form: FormData): boolean { /* ... */ }
function isValidEmail(address: string): boolean { /* ... */ }

const u = getUser(id);
const isValid = validateForm(form);
```

## Verb Choice Signals Behavior And Cost

| Verb | Implies |
|---|---|
| `get` | Cheap, synchronous, always returns a value (in-memory/already-available data) |
| `fetch` | Asynchronous, likely network/disk I/O, returns a `Promise` |
| `find` | May return nothing (`undefined`/`null`); a search, not a guaranteed lookup |
| `create` | Allocates/constructs a new instance; may have side effects (writes to a store) |
| `calculate`/`compute` | Pure derivation from inputs, no I/O |
| `validate`/`assert` | Checks a condition; `validate` typically returns a boolean/result, `assert` throws |
| `ensure` | Idempotently guarantees a state, creating it if absent |

Using `fetchUser` instead of `getUser` for a network call, or `findUser` instead of `getUser` when the user might not exist, communicates important behavioral differences without needing to read the implementation.

## Boolean-Returning Functions Are A Special Case

Functions that return `boolean` follow the predicate-prefix convention (`is`/`has`/`can`/`should`) rather than a generic action verb — see `name-boolean-prefix` for that specific case.

## See Also

- [name-boolean-prefix](name-boolean-prefix.md) - Prefix booleans with `is`/`has`/`can`/`should`
- [name-avoid-abbreviations](name-avoid-abbreviations.md) - Avoid unclear abbreviations in identifiers
- [name-async-suffix-when-ambiguous](name-async-suffix-when-ambiguous.md) - Suffix an async function's name when a sync counterpart exists with the same base name
