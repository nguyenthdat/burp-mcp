# sec-helmet-headers

> Set security headers using Helmet or manual header configuration

## Why It Matters

HTTP security headers protect against common web vulnerabilities: XSS (Content-Security-Policy), clickjacking (X-Frame-Options), MIME sniffing (X-Content-Type-Options), and more. Without these headers, your application is exposed to well-known browser-based attacks. Helmet is the standard middleware for Express and can be adapted for any Node.js HTTP server.

## Bad

```js
// No security headers — vulnerable to XSS, clickjacking, MIME sniffing
import express from 'express';

const app = express();
app.listen(3000);
```

## Good

```js
import express from 'express';
import helmet from 'helmet';

const app = express();

app.use(helmet());
// Sets: CSP, X-Frame-Options, X-Content-Type-Options,
//       Strict-Transport-Security, X-XSS-Protection, Referrer-Policy,
//       Permissions-Policy, and more
```

## Manual Configuration for Fine-Tuning

```js
app.use(helmet({
  contentSecurityPolicy: {
    directives: {
      defaultSrc: ["'self'"],
      scriptSrc: ["'self'", "'unsafe-inline'"],
      styleSrc: ["'self'", 'https://fonts.googleapis.com'],
      imgSrc: ["'self'", 'data:', 'https://cdn.example.com'],
      connectSrc: ["'self'", 'https://api.example.com'],
    },
  },
  crossOriginEmbedderPolicy: false,  // If loading cross-origin resources
  hsts: {
    maxAge: 31536000,
    includeSubDomains: true,
    preload: true,
  },
}));
```

## Non-Express Servers

```js
import { createServer } from 'node:http';

const server = createServer((req, res) => {
  res.setHeader('X-Content-Type-Options', 'nosniff');
  res.setHeader('X-Frame-Options', 'DENY');
  res.setHeader('X-XSS-Protection', '1; mode=block');
  res.setHeader('Strict-Transport-Security', 'max-age=31536000; includeSubDomains');
  res.setHeader('Referrer-Policy', 'strict-origin-when-cross-origin');

  // Handle request
});
```

## When Exceptions Apply

Skip `contentSecurityPolicy` if your frontend is served separately from your API. Skip HSTS if your site should be accessible over HTTP (development only).

## See Also

- [sec-csrf-protection](./sec-csrf-protection.md) - CSRF tokens
- [sec-samesite-cookies](./sec-samesite-cookies.md) - SameSite cookie attributes
