# sec-rate-limit

> Implement rate limiting on endpoints to prevent abuse and DoS attacks

## Why It Matters

Without rate limiting, an attacker can brute-force authentication, exhaust server resources, or scrape data at scale. Rate limiting protects against denial-of-service, credential stuffing, and API abuse. It's a required control in most security compliance frameworks (OWASP, PCI-DSS).

## Bad

```js
// No rate limiting — unlimited requests
app.post('/api/login', async (req, res) => {
  const user = await authenticate(req.body.email, req.body.password);
  // Attacker can try millions of passwords
});

app.get('/api/data', async (req, res) => {
  const data = await fetchExpensiveData();  // Expensive operation, no limits
  res.json(data);
});
```

## Good

```js
import rateLimit from 'express-rate-limit';

// General rate limit for all routes
const generalLimiter = rateLimit({
  windowMs: 15 * 60 * 1000,  // 15 minutes
  max: 100,                   // 100 requests per window
  standardHeaders: true,
  legacyHeaders: false,
  message: { error: 'Too many requests, please try again later' },
});

app.use(generalLimiter);

// Strict rate limit for login
const loginLimiter = rateLimit({
  windowMs: 15 * 60 * 1000,
  max: 5,  // 5 attempts per 15 minutes
  skipSuccessfulRequests: true,  // Only count failed attempts
  message: { error: 'Too many login attempts, please try again later' },
});

app.post('/api/login', loginLimiter, async (req, res) => {
  const user = await authenticate(req.body.email, req.body.password);
  if (!user) return res.status(401).json({ error: 'Invalid credentials' });
  res.json({ token: createToken(user) });
});

// Rate limit by user ID for authenticated endpoints
const apiLimiter = rateLimit({
  windowMs: 60 * 1000,
  max: 60,
  keyGenerator: (req) => req.user?.id ?? req.ip,
});

app.use('/api/', apiLimiter);
```

## When Exceptions Apply

Rate limiting is not needed for internal service-to-service calls within a trusted network. Apply it at API gateways and public-facing endpoints.

## See Also

- [sec-input-size-limits](./sec-input-size-limits.md) - Limit input sizes
- [sec-helmet-headers](./sec-helmet-headers.md) - Security headers
