# sec-samesite-cookies

> Set SameSite=Strict or Lax on all cookies, especially session cookies

## Why It Matters

The `SameSite` attribute controls when cookies are sent with cross-site requests. Without it, cookies are sent on all requests, enabling CSRF attacks. `SameSite=Strict` provides maximum protection; `SameSite=Lax` balances security with usability by allowing cookies on top-level navigation GET requests.

## Bad

```js
// No SameSite — vulnerable to CSRF
res.cookie('session', sessionToken, {
  httpOnly: true,
  secure: true,
  maxAge: 3600000,
});

// Express session without SameSite
app.use(session({
  secret: process.env.SESSION_SECRET,
  resave: false,
  saveUninitialized: false,
  cookie: { secure: true, httpOnly: true },
}));
```

## Good

```js
// SameSite=Lax — good default for most apps
res.cookie('session', sessionToken, {
  httpOnly: true,
  secure: true,
  sameSite: 'lax',
  maxAge: 3600000,
});

// SameSite=Strict for high-security cookies
res.cookie('auth_token', token, {
  httpOnly: true,
  secure: true,
  sameSite: 'strict',
  maxAge: 900000,
});

// Express session with SameSite
app.use(session({
  secret: process.env.SESSION_SECRET,
  resave: false,
  saveUninitialized: false,
  cookie: {
    secure: process.env.NODE_ENV === 'production',
    httpOnly: true,
    sameSite: 'lax',
    maxAge: 24 * 60 * 60 * 1000,  // 24 hours
  },
}));
```

## SameSite Values

| Value | Behavior | Use Case |
|-------|----------|----------|
| `Strict` | Never sent on cross-site requests | Auth tokens, admin panels |
| `Lax` | Sent on top-level GET navigations | Session cookies (default) |
| `None` | Always sent (requires `Secure`) | Cross-site embeds, OAuth flows |

## When Exceptions Apply

`SameSite=None` is required for cookies that must be sent cross-site (e.g., third-party authentication, embedded widgets). Must be accompanied by `Secure` (HTTPS only).

## See Also

- [sec-csrf-protection](./sec-csrf-protection.md) - CSRF protection
- [sec-helmet-headers](./sec-helmet-headers.md) - Security headers
