# name-camelCase

> Use camelCase for variables, functions, and methods

## Why It Matters

camelCase is the JavaScript ecosystem standard. Nearly every library, framework, and built-in API uses it. Consistent casing reduces cognitive friction when reading code and prevents the bugs that come from assuming one convention and finding another. ESLint and Prettier enforce this automatically.

## Bad

```js
// Inconsistent casing
const user_name = 'Alice';
const UserName = 'Alice';     // PascalCase — reserved for classes
const username = 'Alice';     // All lowercase — hard to read multi-word names

function get_user(id) { /* ... */ }
function GetUser(id) { /* ... */ }
function processHTTPRequest() { }
```

## Good

```js
// camelCase — standard for variables, functions, methods
const userName = 'Alice';
const firstName = 'Bob';
const httpStatusCode = 200;

function getUser(id) { /* ... */ }
function processHttpRequest(req) { /* ... */ }
function isValidEmail(email) { /* ... */ }

class UserService {
  async findById(id) { /* ... */ }
  async createUser(data) { /* ... */ }
}
```

## Acronym Handling

```js
// Treat acronyms as words — only capitalize the first letter
const httpClient = new HttpClient();      // NOT: HTTPClient
const userId = 'abc';                     // NOT: userID
const parseXml = (str) => { /* ... */ };  // NOT: parseXML
const apiKey = process.env.API_KEY;       // NOT: APIKey

// Two-letter acronyms stay uppercase
const io = new IO();  // But as a variable: IO
const ui = new UI();

// But: in PascalCase classes, all words capitalized
class HttpClient { }
class XMLParser { }  // Acronym stays uppercase in PascalCase
```

## When Exceptions Apply

Follow the convention of the framework you're using. React event handlers use `onClick`, Express uses `app.use`. Consistency within the ecosystem matters more than strict rules.

## See Also

- [name-PascalCase](./name-PascalCase.md) - PascalCase for classes
- [name-UPPER_SNAKE](./name-UPPER_SNAKE.md) - Constants
- [name-verb-function](./name-verb-function.md) - Verb-first function names
