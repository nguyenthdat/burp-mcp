# name-avoid-abbreviations

> Avoid unclear abbreviations in identifiers

## Why It Matters

Abbreviations save the writer a few keystrokes but cost every future reader time decoding them, and the decoding cost compounds across a codebase's lifetime — a name that's obvious to the author on the day they write it is often ambiguous to a teammate (or the same author six months later) without the surrounding context fresh in mind. Modern editors provide autocomplete, so the "typing cost" argument for abbreviating no longer holds; a full, unambiguous name is effectively free to type and permanently cheaper to read.

## Bad

```typescript
function calcTtlPrc(itms: Item[], usrDisc: number): number {
  let tp = 0;
  for (const i of itms) {
    tp += i.prc * i.qty;
  }
  return tp * (1 - usrDisc);
}

interface UsrCfg {
  nm: string;
  addr: string;
  prefLang: string;
}
```

## Good

```typescript
function calculateTotalPrice(items: Item[], userDiscount: number): number {
  let totalPrice = 0;
  for (const item of items) {
    totalPrice += item.price * item.quantity;
  }
  return totalPrice * (1 - userDiscount);
}

interface UserConfig {
  name: string;
  address: string;
  preferredLanguage: string;
}
```

## Widely-Understood Abbreviations Are Fine

Not all shortening is bad — abbreviations that are more standard and recognizable than their expansion, especially in a specific technical domain, don't need spelling out:

```typescript
const id = user.id;                  // universally understood
const url = new URL(request.url);    // standard acronym
const html = renderTemplate(view);   // standard acronym
function parseJson(input: string) {} // standard acronym
const i = 0, j = 0;                  // conventional loop indices in tight scope
const ctx = createContext();         // extremely common in this exact form (React/Node)
const req: Request, res: Response;   // idiomatic in HTTP handler signatures
```

The test: would a new team member, unfamiliar with this specific codebase but familiar with the domain (web development, TypeScript), immediately recognize the abbreviation without guessing? `ctx`, `id`, `url`, `req`/`res` pass; `usrDisc`, `tp`, `nm` do not.

## A Practical Guideline

Prefer the full word unless the abbreviation is:
1. An industry-standard acronym (`URL`, `HTML`, `ID`, `API`), or
2. Scoped to a very short lifetime/visibility (a loop index, a one-line lambda parameter) where the full name would add nothing.

## See Also

- [name-no-hungarian](name-no-hungarian.md) - Avoid Hungarian notation and redundant type suffixes in identifier names
- [name-verb-noun-functions](name-verb-noun-functions.md) - Name functions with a leading verb describing the action they perform
- [doc-inline-why-not-what](doc-inline-why-not-what.md) - Write comments that explain why, not what the code already says
