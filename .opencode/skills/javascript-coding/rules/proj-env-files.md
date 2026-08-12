# proj-env-files

> Use `.env.example` as a template committed to git — never commit `.env` with real values

## Why It Matters

`.env` files contain secrets and environment-specific configuration. Committing them exposes secrets and causes conflicts between developers. `.env.example` provides a template with placeholder values, documenting what environment variables are needed without exposing any real values.

## Bad

```bash
# .env — committed to git, public secrets!
DATABASE_URL=postgres://user:realpassword@db.example.com:5432/prod
JWT_SECRET=my-super-secret-key-123
AWS_ACCESS_KEY_ID=AKIA...
```

## Good

```bash
# .env.example — committed, no secrets
DATABASE_URL=postgres://user:password@localhost:5432/dbname
JWT_SECRET=your-secret-key-here
AWS_ACCESS_KEY_ID=
AWS_SECRET_ACCESS_KEY=
PORT=3000
LOG_LEVEL=info
```

```gitignore
# .gitignore
.env
.env.local
.env.*.local
*.pem
credentials.json
```

## Documentation

```markdown
## Configuration

Copy `.env.example` to `.env` and fill in your values:

```bash
cp .env.example .env
```

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `DATABASE_URL` | Yes | — | PostgreSQL connection string |
| `JWT_SECRET` | Yes | — | Secret for signing JWTs |
| `PORT` | No | `3000` | Server port |

## When Exceptions Apply

All projects, including internal tools, should follow this pattern. There are no exceptions to keeping secrets out of git.

## See Also

- [sec-no-hardcoded-secrets](./sec-no-hardcoded-secrets.md) - Never commit secrets
- [node-env-config](./node-env-config.md) - Config loading pattern
