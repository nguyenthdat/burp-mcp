# anti-stringly-typed-data

> Don't represent structured data as ad hoc strings

## Why It Matters

Encoding structured information into a delimited string (`"user:42:admin"`, `"2026-07-20T00:00:00"` built with template literals instead of a `Date`, a comma-joined list passed as one parameter) throws away the type system's ability to validate the structure at the boundary — the compiler sees `string`, so a caller can pass any string at all, and the real validation (does it have the right number of parts? are they in the right order?) happens ad hoc, if at all, deep inside a parsing function. It also makes every consumer re-implement the same fragile split/parse logic, and a change to the string's format becomes an untyped, unenforced contract instead of a type change the compiler can verify at every call site.

## Bad

```typescript
function formatUserRef(id: number, role: string) {
  return `user:${id}:${role}`; // structured data flattened into a string
}

function parseUserRef(ref: string) {
  const [, idStr, role] = ref.split(':'); // fragile, no validation
  return { id: Number(idStr), role };
}

function scheduleJob(cronLike: string) {
  // "daily,09:00,retry=3" — an ad hoc mini-DSL with no compiler support at all
  const [freq, time, retryPart] = cronLike.split(',');
  const retries = Number(retryPart.split('=')[1]);
  // ...
}
```

## Good

```typescript
interface UserRef {
  id: number;
  role: 'admin' | 'member' | 'guest';
}

function formatUserRef(ref: UserRef): string {
  return `user:${ref.id}:${ref.role}`; // string is now a serialization detail, not the model
}

interface JobSchedule {
  frequency: 'daily' | 'weekly';
  time: string; // still a string, but scoped and documented, not a multi-field DSL
  retries: number;
}

function scheduleJob(schedule: JobSchedule) {
  // Every field is typed, validated, and autocompletable at the call site
}
```

## Real-World Signals You've Hit This Anti-Pattern

- A function's single `string` parameter gets `.split(...)` on the first line of its body.
- Comments explain a string's "format" (`// format: "id,name,role"`) instead of the type system enforcing it.
- Multiple call sites each re-implement slightly different parsing/validation for the same string shape.

Prefer a real object type (or a branded string type for genuinely single-token identifiers, see `type-branded-nominal`) and serialize only at the actual I/O boundary (network, disk, URL) where a string is unavoidable.

## See Also

- [type-branded-nominal](type-branded-nominal.md) - Use branded types for nominal typing over structurally-identical primitives
- [type-discriminated-union](type-discriminated-union.md) - Model variant data with discriminated unions, not flags or strings
- [err-boundary-validation](err-boundary-validation.md) - Validate external input at the boundary of your system
