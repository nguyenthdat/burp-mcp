# sec-csrf-protection

> Use CSRF tokens for state-changing operations (POST, PUT, DELETE)

## Why It Matters

Cross-Site Request Forgery (CSRF) tricks authenticated users into performing unintended actions. Without CSRF protection, an attacker can forge requests from another site using the victim's session cookies. CSRF tokens ensure that state-changing requests originate from your application, not from a malicious third-party site.

## Bad

```js
// No CSRF protection — vulnerable to forged requests
app.post('/api/transfer', (req, res) => {
  const { to, amount } = req.body;
  // Attacker can forge this request from another site
  transferFunds(req.user.id, to, amount);
  res.json({ success: true });
});
```

## Good

```js
import csrf from 'csurf';
import cookieParser from 'cookie-parser';

const csrfProtection = csrf({ cookie: true });

app.use(cookieParser());
app.use(csrfProtection);

// Include CSRF token in the response for the client
app.get('/api/csrf-token', (req, res) => {
  res.json({ token: req.csrfToken() });
});

// Protected endpoint — requires valid CSRF token
app.post('/api/transfer', csrfProtection, (req, res) => {
  const { to, amount } = req.body;
  transferFunds(req.user.id, to, amount);
  res.json({ success: true });
});
```

## Double Submit Cookie Pattern

```js
function csrfMiddleware(req, res, next) {
  if (['POST', 'PUT', 'DELETE', 'PATCH'].includes(req.method)) {
    const headerToken = req.headers['x-csrf-token'];
    const cookieToken = req.cookies['csrf-token'];

    if (!headerToken || !cookieToken || headerToken !== cookieToken) {
      return res.status(403).json({ error: 'Invalid CSRF token' });
    }
  }
  next();
}
```

## When Exceptions Apply

CSRF protection is unnecessary for:
- APIs that use token-based authentication (Authorization header, not cookies)
- Read-only endpoints (GET, HEAD, OPTIONS)
- Server-to-server communication

## See Also

- [sec-helmet-headers](./sec-helmet-headers.md) - Security headers
- [sec-samesite-cookies](./sec-samesite-cookies.md) - SameSite as secondary defense
