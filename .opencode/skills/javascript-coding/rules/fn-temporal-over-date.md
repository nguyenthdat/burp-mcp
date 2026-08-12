# fn-temporal-over-date

> Use the Temporal API (ES2024 / Stage 3) over the legacy Date object

## Why It Matters

JavaScript's `Date` object was ported from Java in 1995 and has fundamental flaws: mutable, month is 0-indexed, timezone handling is inconsistent, parsing is locale-dependent, and arithmetic requires manual millisecond math. The Temporal API provides immutable, timezone-aware, and ergonomic date/time types. It's the future of datetime handling in JavaScript.

## Bad

```js
// Date — mutable, confusing API
const now = new Date();
now.setMonth(now.getMonth() + 1);  // Mutates original

const date = new Date(2024, 0, 1);  // January? Or day 0?
date.getMonth();  // 0 — wait, what?

const parsed = new Date('2024-01-01');  // Interpreted as UTC in some browsers, local in others

const nextWeek = new Date(Date.now() + 7 * 24 * 60 * 60 * 1000);  // DST bugs!
```

## Good

```js
// Temporal — immutable, clear, timezone-aware
import { Temporal } from '@js-temporal/polyfill';  // Until native support

const now = Temporal.Now.plainDateISO();
const nextMonth = now.add({ months: 1 });  // Returns new instance, original unchanged

const date = Temporal.PlainDate.from({ year: 2024, month: 1, day: 1 });
date.month;  // 1 — January!

const parsed = Temporal.PlainDate.from('2024-01-01');  // Unambiguous

const nextWeek = Temporal.Now.plainDateISO().add({ days: 7 });  // Correct across DST

// Timezone-aware operations
const zonedDateTime = Temporal.Now.zonedDateTimeISO('America/New_York');
const tokyoTime = zonedDateTime.withTimeZone('Asia/Tokyo');
```

## Temporal Types

```js
// PlainDate — date without time or timezone
const date = Temporal.PlainDate.from('2024-01-15');

// PlainTime — time without date or timezone
const time = Temporal.PlainTime.from('14:30:00');

// PlainDateTime — date and time without timezone
const dt = Temporal.PlainDateTime.from('2024-01-15T14:30:00');

// ZonedDateTime — date, time, and timezone
const zdt = Temporal.ZonedDateTime.from('2024-01-15T14:30:00-05:00[America/New_York]');

// Instant — absolute point in time (nanosecond precision)
const instant = Temporal.Instant.from('2024-01-15T19:30:00Z');

// Duration — difference between two points in time
const duration = Temporal.Duration.from({ hours: 3, minutes: 15 });
```

## When Exceptions Apply

Temporal is Stage 3 and requires a polyfill (`@js-temporal/polyfill`) in Node.js 22 and earlier. For simple timestamp storage (e.g., `Date.now()`), the legacy API is fine. Use Temporal for date arithmetic, timezone operations, and user-facing date formatting.

## See Also

- [fn-immutability](./fn-immutability.md) - Immutable data patterns
- [type-bigint-precision](./type-bigint-precision.md) - BigInt for large numbers
