# type-typeof-guards

> Use `typeof` and `instanceof` checks at public API boundaries

## Why It Matters

JavaScript's dynamic typing means any type of value can be passed to a function. Without guards, invalid inputs crash deep in the call stack with cryptic errors. `typeof` validates primitives, `instanceof` validates custom types, and combined they form a first line of defense that fails fast with clear error messages.

## Bad

```js
// No guards — cryptic error on invalid input
function calculateDiscount(price, percent) {
  return price - (price * percent / 100);
}

calculateDiscount('100', '20');  // NaN instead of error
calculateDiscount(null, 10);     // TypeError deep in the function
```

## Good

```js
// Type guards at the boundary
function calculateDiscount(price, percent) {
  if (typeof price !== 'number' || Number.isNaN(price)) {
    throw new TypeError('price must be a valid number');
  }
  if (typeof percent !== 'number' || Number.isNaN(percent)) {
    throw new TypeError('percent must be a valid number');
  }
  if (percent < 0 || percent > 100) {
    throw new RangeError('percent must be between 0 and 100');
  }
  return price - (price * percent / 100);
}
```

## Custom Type Guards with instanceof

```js
class ApiError extends Error {
  constructor(message, statusCode) {
    super(message);
    this.statusCode = statusCode;
  }
}

async function fetchWithGuard(url) {
  const response = await fetch(url);
  if (!response.ok) {
    throw new ApiError(`Request failed: ${response.status}`, response.status);
  }
  return response.json();
}

// Caller uses instanceof
try {
  const data = await fetchWithGuard('/api/data');
} catch (err) {
  if (err instanceof ApiError) {
    console.error('API error:', err.statusCode);
  } else if (err instanceof TypeError) {
    console.error('Network error');
  }
}
```

## Type Guard Functions

```js
function isString(value) {
  return typeof value === 'string';
}

function isObject(value) {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function hasProperty(obj, prop) {
  return isObject(obj) && prop in obj;
}
```

## When Exceptions Apply

Internal helper functions called with predictable inputs can skip guards. Type guards are most important at module boundaries, API handlers, and library exports.

## See Also

- [err-type-check-inputs](./err-type-check-inputs.md) - Validate types at boundaries
- [type-avoid-implicit-coercion](./type-avoid-implicit-coercion.md) - Use strict equality
