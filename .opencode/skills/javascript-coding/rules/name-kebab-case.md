# name-kebab-case

> Use kebab-case for file and directory names

## Why It Matters

File systems are case-insensitive on Windows and macOS but case-sensitive on Linux. kebab-case avoids case-sensitivity issues entirely while being more readable than all-lowercase. It's the standard in the Node.js ecosystem (npm package names, most open-source projects) and prevents `import` mismatches between development and CI.

## Bad

```js
// Inconsistent file naming
userService.js
UserService.js    // Case-sensitive OS issue
user_service.js   // snake_case — non-standard in JS
user-service.js   // Mix of conventions within the project

// Import mismatches
import { UserService } from './UserService.js';  // Fails on Linux
import { UserService } from './userService.js';
```

## Good

```js
// kebab-case — consistent, safe across OS
user-service.js
order-controller.js
email-sender.js

// Directories too
/services/
/controllers/
/middleware/
/data-access/

// Index files inside directories
/services/index.js  // Or: /services.js

// Test files
user-service.test.js
user-service.spec.js
```

## Exceptions

```js
// Special files — use their standard names
README.md
CHANGELOG.md
CONTRIBUTING.md
LICENSE
.eslintrc.js
.prettierrc
package.json
Dockerfile
docker-compose.yml

// React/Vue components — PascalCase (framework convention)
UserProfile.jsx
OrderList.vue

// Class files (when one class per file) — can match the class name
UserService.js
HttpClient.js
```

## When Exceptions Apply

React components use PascalCase filenames. Configuration files use their standard dotfile names. Follow the convention of the framework you're using.

## See Also

- [name-camelCase](./name-camelCase.md) - camelCase for variables
- [name-PascalCase](./name-PascalCase.md) - PascalCase for classes
