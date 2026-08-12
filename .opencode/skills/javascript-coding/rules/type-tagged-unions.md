# type-tagged-unions

> Use discriminated unions with a `type` field to model mutually exclusive states

## Why It Matters

JavaScript lacks a built-in sum type (like Rust's `enum`). When an object can represent multiple mutually exclusive states, a `type` discriminator field makes it clear which variant is active and what properties are valid. Without a discriminator, code must check for the presence of optional properties, which is fragile and misses edge cases.

## Bad

```js
// Ambiguous shape — which properties are valid depends on context
function processResponse(response) {
  if (response.data) {
    // Is response.success guaranteed when data exists?
    return response.data;
  }
  if (response.error) {
    // Is response.code for the error or something else?
    return { error: response.error, code: response.code };
  }
  // What state is this? No way to know.
}
```

## Good

```js
// Discriminated union with a `type` field
function processResponse(response) {
  switch (response.type) {
    case 'success':
      return response.data;  // TypeScript/reader knows { type: 'success', data: T }
    case 'error':
      return { error: response.message, code: response.code };
    case 'loading':
      return null;  // No data yet
    default:
      throw new Error(`Unknown response type: ${response.type}`);
  }
}

// Usage is self-documenting
const result = await apiCall();
processResponse(result);  // { type: 'success', data: [...] }

// Creating variants
function createSuccess(data) {
  return { type: 'success', data };
}

function createError(message, code) {
  return { type: 'error', message, code };
}

function createLoading() {
  return { type: 'loading' };
}
```

## State Machine Pattern

```js
// Order state machine with discriminated union
const OrderStates = {
  DRAFT: 'draft',
  CONFIRMED: 'confirmed',
  SHIPPED: 'shipped',
  DELIVERED: 'delivered',
  CANCELLED: 'cancelled',
};

function transition(order, toState) {
  const validTransitions = {
    [OrderStates.DRAFT]: [OrderStates.CONFIRMED, OrderStates.CANCELLED],
    [OrderStates.CONFIRMED]: [OrderStates.SHIPPED, OrderStates.CANCELLED],
    [OrderStates.SHIPPED]: [OrderStates.DELIVERED],
    [OrderStates.DELIVERED]: [],
    [OrderStates.CANCELLED]: [],
  };

  if (!validTransitions[order.state]?.includes(toState)) {
    throw new Error(`Cannot transition from ${order.state} to ${toState}`);
  }

  return { ...order, state: toState };
}
```

## When Exceptions Apply

For simple objects with only 2 states, a boolean flag may be sufficient (`{ ok: true, value: ... }` or `{ ok: false, error: ... }`). Use discriminated unions when states grow to 3+.

## See Also

- [type-no-magic-strings](./type-no-magic-strings.md) - Constants over magic strings
- [type-symbol-over-string](./type-symbol-over-string.md) - Symbol for unique keys
