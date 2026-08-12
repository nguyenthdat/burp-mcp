# modern-switch-expressions

> Use arrow-form switch expressions over statement switches

## Why It Matters

Classic `switch` statements fall through by default, requiring an explicit `break` on every branch, and they can't produce a value directly - forcing an intermediate mutable variable that the compiler cannot prove is always assigned. Arrow-form switch expressions (Java 14+) remove fall-through entirely, let `switch` be used as an expression, and let the compiler verify exhaustiveness, eliminating an entire category of "forgot the break" bugs.

## Bad

```java
int daysInMonth;
switch (month) {
    case 1:
    case 3:
    case 5:
    case 7:
    case 8:
    case 10:
    case 12:
        daysInMonth = 31;
        break;
    case 4:
    case 6:
    case 9:
    case 11:
        daysInMonth = 30;
        break;
    case 2:
        daysInMonth = 28;
        break; // forgetting this break would silently fall into default
    default:
        throw new IllegalArgumentException("Invalid month: " + month);
}
```

## Good

```java
int daysInMonth = switch (month) {
    case 1, 3, 5, 7, 8, 10, 12 -> 31;
    case 4, 6, 9, 11 -> 30;
    case 2 -> 28;
    default -> throw new IllegalArgumentException("Invalid month: " + month);
};
```

## `yield` for Multi-Statement Branches

When a branch needs more than one expression, use a block with `yield` instead of falling back to `case:` labels:

```java
String describe(int score) {
    return switch (score / 10) {
        case 10, 9 -> "Excellent";
        case 8 -> {
            String note = score >= 85 ? "solid" : "good";
            yield "Good (" + note + ")";
        }
        default -> "Needs improvement";
    };
}
```

## Exhaustiveness Over Enums and Sealed Types

Arrow-form switch expressions over an `enum` or a `sealed` hierarchy do not require a `default` branch if every case is covered, and the compiler will flag it as an error if one is missing after a case is added later - the classic-statement equivalent has no such check.

## See Also

- [`type-pattern-matching-switch`](type-pattern-matching-switch.md) - Combining switch expressions with type patterns
- [`modern-guarded-patterns-when`](modern-guarded-patterns-when.md) - Adding `when` conditions to individual arms
- [`modern-record-deconstruction-patterns`](modern-record-deconstruction-patterns.md) - Destructuring record components inside switch arms
