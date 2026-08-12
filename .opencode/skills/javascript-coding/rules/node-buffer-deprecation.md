# node-buffer-deprecation

> Use `Buffer.from()`, `Buffer.alloc()`, and `Buffer.allocUnsafe()` — never `new Buffer()`

## Why It Matters

The `new Buffer()` constructor was deprecated in Node.js 6 and removed in Node.js 10 due to security and usability issues. `new Buffer(n)` creates an uninitialized buffer that may contain sensitive data from previous allocations. `new Buffer(string)` has inconsistent behavior. The static factory methods (`Buffer.from`, `Buffer.alloc`, `Buffer.allocUnsafe`) are explicit about intent.

## Bad

```js
// Deprecated — inconsistent behavior, security risk
const buf1 = new Buffer(10);         // Uninitialized memory
const buf2 = new Buffer('hello');    // Deprecated
const buf3 = new Buffer([1, 2, 3]);  // Deprecated
```

## Good

```js
// Use static factory methods
const buf1 = Buffer.alloc(10);              // Zero-filled (safe)
const buf2 = Buffer.allocUnsafe(10);        // Uninitialized (fast, but fill immediately)
const buf3 = Buffer.from('hello');          // From string
const buf4 = Buffer.from([1, 2, 3]);        // From array
const buf5 = Buffer.from(buf4);             // Copy of another buffer

// allocUnsafe + fill for performance-critical code
const buf = Buffer.allocUnsafe(4096);
buf.fill(0);  // Fill immediately
```

## Buffer vs TypedArrays

```js
// Buffer is a subclass of Uint8Array
const buf = Buffer.from([0x48, 0x65, 0x6c, 0x6c, 0x6f]);
buf.toString();  // 'Hello'

// Use TextEncoder/TextDecoder for string ↔ bytes
const encoder = new TextEncoder();
const bytes = encoder.encode('Hello');

const decoder = new TextDecoder();
const text = decoder.decode(bytes);
```

## When Exceptions Apply

`allocUnsafe` is safe when you immediately overwrite the entire buffer with new data (e.g., reading from a file into the buffer). If any portion might remain uninitialized, use `alloc`.

## See Also

- [sec-no-hardcoded-secrets](./sec-no-hardcoded-secrets.md) - Never expose secrets
- [node-stream-over-buffer](./node-stream-over-buffer.md) - Stream vs buffer
