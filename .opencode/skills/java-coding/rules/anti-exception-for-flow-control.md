# anti-exception-for-flow-control

> Don't use exceptions for expected, routine flow control

## Why It Matters

Throwing and catching an exception is dramatically more expensive than a conditional branch, primarily because of stack trace capture, and using it for expected outcomes (like "item not found") obscures the code's real intent behind machinery meant for truly exceptional conditions. It also makes debugging harder: when everything routine also throws, a debugger configured to break on exceptions becomes useless, and genuine bugs get lost in the noise of routine "exceptions."

## Bad

```java
public boolean containsUser(Map<String, User> users, String id) {
  try {
    User user = users.get(id);
    user.getName(); // Deliberately NPEs to detect "not present" - expensive and unclear
    return true;
  } catch (NullPointerException e) {
    return false;
  }
}

public int findIndex(List<String> items, String target) {
  try {
    for (int i = 0; ; i++) {
      if (items.get(i).equals(target)) { // Relies on IndexOutOfBoundsException to end the loop
        return i;
      }
    }
  } catch (IndexOutOfBoundsException e) {
    return -1;
  }
}
```

## Good

```java
public boolean containsUser(Map<String, User> users, String id) {
  return users.containsKey(id); // Expresses intent directly, no exception involved
}

public int findIndex(List<String> items, String target) {
  for (int i = 0; i < items.size(); i++) {
    if (items.get(i).equals(target)) {
      return i;
    }
  }
  return -1;
  // Or simply: return items.indexOf(target);
}
```

## Reserve Exceptions for Genuinely Exceptional Conditions

```java
// Good use of exceptions: an unrecoverable, unexpected failure, not a
// routine "did we find it" check.
public Order loadOrder(String orderId) throws OrderNotFoundException {
  Order order = repository.find(orderId);
  if (order == null) {
    throw new OrderNotFoundException(orderId); // Legitimate: caller can't proceed without it
  }
  return order;
}

// vs. the routine case, which should just be a boolean/Optional check
public boolean orderExists(String orderId) {
  return repository.find(orderId) != null;
}
```

## See Also

- [`err-fail-fast-validation`](err-fail-fast-validation.md) - Exceptions are for validation failures the caller can't ignore, not for normal branching
- [`null-optional-return-type`](null-optional-return-type.md) - Optional expresses "may be absent" without throwing
- [`err-no-control-flow`](err-no-control-flow.md) - The positive rule this anti-pattern violates
