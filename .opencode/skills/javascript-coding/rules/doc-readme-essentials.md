# doc-readme-essentials

> Include installation, usage, API overview, and license in every project README

## Why It Matters

The README is the first thing someone sees when they encounter your project. Without clear installation and usage instructions, potential users move on. A good README answers: What is this? How do I install it? How do I use it? What's the API? How do I contribute? What's the license?

## Bad

```markdown
# My Project

A cool project.

## Usage

See the source code.
```

## Good

```markdown
# My Project

A brief description of what this project does and why it exists.

## Installation

```bash
npm install my-project
```

## Quick Start

```js
import { createServer } from 'my-project';

const server = await createServer({ port: 3000 });
console.log('Server running on http://localhost:3000');
```

## API

### `createServer(options)`

Creates a new server instance.

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `options.port` | `number` | `3000` | Port to listen on |
| `options.host` | `string` | `'localhost'` | Host to bind to |

Returns: `Promise<Server>`

## Configuration

| Environment Variable | Default | Description |
|---------------------|---------|-------------|
| `DATABASE_URL` | — | PostgreSQL connection string (required) |
| `PORT` | `3000` | Server port |

## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md).

## License

MIT
```

## When Exceptions Apply

Private/internal projects may need less detail. But even internal tools benefit from install + usage instructions. At minimum, document how to run the project locally.

## See Also

- [doc-changelog](./doc-changelog.md) - Maintain a changelog
- [proj-env-files](./proj-env-files.md) - .env.example pattern
