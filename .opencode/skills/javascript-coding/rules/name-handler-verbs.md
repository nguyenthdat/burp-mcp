# name-handler-verbs

> Prefix event handlers with `on` or `handle` to distinguish them from regular functions

## Why It Matters

Event handlers serve a specific role — they respond to events. Prefixing with `on` or `handle` makes it clear that a function is an event response, not a regular action-initiating function. It also prevents naming conflicts: `click()` is an action, `onClick()` is a handler.

## Bad

```js
// Ambiguous — action or handler?
function click() { submit(); }
button.addEventListener('click', click);

function data(response) { process(response); }
fetch('/api').then(data);

// Mixed conventions
function submitForm() { }        // Is this the handler or the action?
function formSubmitHandler() { } // Suffix is inconsistent
```

## Good

```js
// on/handle prefix — clearly a handler
function onClick(event) {
  event.preventDefault();
  submitForm();
}

function handleData(response) {
  process(response);
}

function handleError(err) {
  console.error('Request failed:', err);
}

button.addEventListener('click', onClick);
fetch('/api').then(handleData).catch(handleError);
```

## Conventions by Context

```js
// DOM event handlers — on + event name
function onClick() { }
function onKeyDown() { }
function onSubmit(event) { event.preventDefault(); }

// HTTP request handlers — handle + resource
function handleGetUser(req, res) { }
function handleCreateUser(req, res) { }

// Express middleware — descriptive name
function authenticateUser(req, res, next) { }
function validateBody(req, res, next) { }

// Promise callbacks
function handleSuccess(data) { }
function handleFailure(err) { }
```

## When Exceptions Apply

In Express and similar frameworks, route handlers are often named by their resource and action: `getUser`, `createOrder`. The `handle` prefix is more common in vanilla Node.js and frontend code.

## See Also

- [name-verb-function](./name-verb-function.md) - Verb-first function names
- [name-callback-descriptive](./name-callback-descriptive.md) - Descriptive callback names
