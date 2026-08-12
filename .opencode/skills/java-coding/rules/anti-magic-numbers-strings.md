# anti-magic-numbers-strings

> Don't scatter unexplained magic numbers/strings

## Why It Matters

A bare `86400` or `"ADMIN"` tells the reader nothing about why that value was chosen or where else it needs to change if the requirement changes, and when the same magic value is copy-pasted across a dozen call sites, updating it correctly everywhere becomes a game of grep-and-pray. Named constants document intent once and guarantee every usage stays in sync when the value changes.

## Bad

```java
public boolean isSessionExpired(Instant lastActivity) {
  return Duration.between(lastActivity, Instant.now()).getSeconds() > 86400;
  // Why 86400? Is it a day? Does every caller agree on that?
}

public void authorize(User user) {
  if (user.getRole().equals("ADMIN")) { // Typo-prone string literal, repeated elsewhere
    grantFullAccess();
  }
}

public double calculateShipping(double weight) {
  if (weight > 20.0) {
    return weight * 4.5; // What is 20.0? What is 4.5? A rate per kg? Undocumented.
  }
  return 9.99;
}
```

## Good

```java
private static final Duration SESSION_TIMEOUT = Duration.ofDays(1);

public boolean isSessionExpired(Instant lastActivity) {
  return Duration.between(lastActivity, Instant.now()).compareTo(SESSION_TIMEOUT) > 0;
}

public enum Role { ADMIN, EDITOR, VIEWER }

public void authorize(User user) {
  if (user.getRole() == Role.ADMIN) { // Compiler-checked, no typos possible
    grantFullAccess();
  }
}

private static final double HEAVY_PACKAGE_THRESHOLD_KG = 20.0;
private static final double HEAVY_PACKAGE_RATE_PER_KG = 4.5;
private static final double STANDARD_SHIPPING_RATE = 9.99;

public double calculateShipping(double weightKg) {
  if (weightKg > HEAVY_PACKAGE_THRESHOLD_KG) {
    return weightKg * HEAVY_PACKAGE_RATE_PER_KG;
  }
  return STANDARD_SHIPPING_RATE;
}
```

## Small, Self-Evident Literals Are Fine

```java
// 0, 1, -1, and 2 in obviously self-explanatory contexts don't need names -
// naming them can add noise rather than clarity.
int firstElement = list.get(0);
boolean isEven = number % 2 == 0;
```

## See Also

- [`type-enum-over-int-constants`](type-enum-over-int-constants.md) - Prefer enums to string/int constants for closed sets of values
- [`name-constants-screaming-snake`](name-constants-screaming-snake.md) - Descriptive naming principles that extend to constants
- [`lint-checkstyle-google-style`](lint-checkstyle-google-style.md) - Checkstyle's `MagicNumberCheck` can flag these automatically
