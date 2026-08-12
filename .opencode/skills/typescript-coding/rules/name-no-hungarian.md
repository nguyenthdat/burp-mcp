# name-no-hungarian

> Avoid Hungarian notation and redundant type suffixes in identifier names

## Why It Matters

Hungarian notation (`strName`, `iCount`, `bIsValid`) and redundant type suffixes (`userObj`, `configData`, `itemsList`, `nameStr`) were a workaround for languages without reliable static typing or IDE tooling — the type had to live in the name because nothing else surfaced it. TypeScript's compiler and editor tooling show the type on hover, in autocomplete, and at every call site, so the prefix/suffix adds nothing but redundant noise, and worse, becomes actively wrong and misleading the moment the underlying type changes but nobody updates the name.

## Bad

```typescript
function getUserObj(userIdStr: string): UserObj {
  const usersList: UserObj[] = fetchUsersArr();
  return usersList.find((userObj) => userObj.idStr === userIdStr)!;
}

let iRetryCount: number = 0;
let bIsLoading: boolean = false;
let arrItems: string[] = [];
let objConfig: Config = loadConfig();
```

## Good

```typescript
function getUser(userId: string): User {
  const users: User[] = fetchUsers();
  return users.find((user) => user.id === userId)!;
}

let retryCount = 0;
let isLoading = false;
let items: string[] = [];
let config: Config = loadConfig();
```

## Names Should Describe Role or Meaning, Not Type

The type is already visible via inference and hover-in-editor; the name's job is to say what the value *means* in context, which a type suffix can't do:

```typescript
// Type-suffix naming tells you nothing about meaning:
const dataObj = response.json();

// Role-based naming tells you what it actually is:
const orderConfirmation = response.json();
```

## Exceptions Where a Suffix Genuinely Disambiguates

- Distinguishing a raw/unvalidated value from its parsed counterpart when both exist in the same scope: `rawInput: string` vs. `parsedInput: FormValues`.
- DOM/element references, where a suffix like `Element` or `Ref` clarifies it's a handle rather than the underlying data: `submitButtonRef`.
- Branded/nominal types where the suffix communicates a specific validated shape rather than raw type info: `userIdBranded` is unnecessary, but naming that documents provenance (e.g., `validatedEmail` vs. `rawEmail`) is about meaning, not Hungarian notation, and is fine.

## See Also

- [name-avoid-abbreviations](name-avoid-abbreviations.md) - Avoid unclear abbreviations in identifiers
- [type-branded-nominal](type-branded-nominal.md) - Use branded/nominal types to distinguish structurally-identical values
- [name-camelCase-vars](name-camelCase-vars.md) - Use `camelCase` for variables and functions
