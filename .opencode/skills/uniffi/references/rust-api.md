# Rust API

Use these patterns to define the Rust API that UniFFI exposes to Kotlin and
Swift. Prefer proc-macros for new code; use UDL only when needed.

## Functions

```rust
#[uniffi::export]
fn add(a: u32, b: u32) -> u32 {
    a + b
}
```

Standalone functions become top-level functions in Kotlin/Swift.

## Objects

```rust
#[derive(uniffi::Object)]
pub struct TodoList {
    items: Vec<TodoEntry>,
}

#[uniffi::export]
impl TodoList {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self { items: vec![] }
    }

    #[uniffi::constructor]
    pub fn with_items(initial: Vec<TodoEntry>) -> Self {
        Self { items: initial }
    }

    pub fn add_item(&mut self, text: String) {
        self.items.push(TodoEntry { text, done: false });
    }

    pub fn get_items(&self) -> Vec<TodoEntry> {
        self.items.clone()
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }
}
```

- `#[derive(uniffi::Object)]` marks the struct as a UniFFI object.
- `#[uniffi::constructor]` generates a factory method. Multiple constructors
  are supported.
- `&self` methods are read-only; `&mut self` requires mutable access.
- Kotlin/Swift consumers get a class with typed methods.

## Renaming

```rust
#[derive(uniffi::Object)]
#[uniffi(name = "TodoList")]
pub struct TodoListInternal { /* ... */ }

#[uniffi::export(name = "TodoList")]
impl TodoListInternal {
    #[uniffi::constructor(name = "createEmpty")]
    pub fn new() -> Self { /* ... */ }

    #[uniffi::method(name = "addEntry")]
    pub fn add_item(&mut self, text: String) { /* ... */ }
}
```

## Records (data classes)

```rust
#[derive(uniffi::Record)]
pub struct TodoEntry {
    pub text: String,
    pub done: bool,
    pub due_date: Option<u64>,
}
```

Records become data classes in Kotlin (`data class`) and structs in Swift.
All fields must be public and UniFFI-compatible types.

## Enums

### Simple enum

```rust
#[derive(uniffi::Enum)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
}
```

### Enum with associated data

```rust
#[derive(uniffi::Enum)]
pub enum Event {
    None,
    Text(String),
    Clicked { x: i64, y: i64 },
    MultiTouch(Vec<(i64, i64)>),
}
```

Kotlin: sealed class. Swift: enum with associated values.

## Errors

```rust
#[derive(uniffi::Error, Debug)]
pub enum MyError {
    NotFound { resource: String },
    PermissionDenied(String),
    Timeout,
}
```

Use as `Result<T, MyError>` in exported functions. Kotlin: exception class.
Swift: `Error` protocol conformance.

```rust
#[uniffi::export]
fn find_todo(id: u32) -> Result<TodoEntry, MyError> {
    if id == 0 {
        Err(MyError::NotFound { resource: format!("todo {}", id) })
    } else {
        Ok(TodoEntry { text: "found".into(), done: false, due_date: None })
    }
}
```

## Async functions

```rust
#[uniffi::export]
async fn fetch_todos() -> Vec<TodoEntry> {
    // ... async work
    vec![]
}

#[uniffi::export]
impl TodoList {
    pub async fn load_from_server(&mut self, url: String) -> Result<(), MyError> {
        // ... async network call
        Ok(())
    }
}
```

- Kotlin: returns `Deferred<T>` (requires `kotlinx-coroutines-core`).
- Swift: returns `async` function (native `async/await`).

## UDL syntax (legacy)

For reference when maintaining UDL-based projects:

```webidl
namespace todolist {
    TodoEntry? get_last_entry();
};

dictionary TodoEntry {
    boolean done;
    u64? due_date;
    string text;
};

enum LogLevel {
    "Debug",
    "Info",
    "Warning",
    "Error",
};

[Error]
enum MyError {
    "NotFound",
    "PermissionDenied",
    "Timeout",
};

interface TodoList {
    constructor();
    [Throws=MyError]
    void add_item(string text);
    sequence<TodoEntry> get_items();
    void clear();
};
```

## Custom types

```rust
#[uniffi::export(custom_type = "Guid")]
fn process_id(id: Guid) -> bool {
    // Guid is a custom type with a type converter
}
```

Requires implementing `UniffiCustomTypeConverter` on the Rust side and
defining the foreign type mapping. See the custom types guide for full
details.

## External types

```rust
#[uniffi::export]
fn use_external_url(input: Url) -> String {
    input.to_string()
}
```

```toml
# uniffi.toml
[bindings.kotlin.external_packages]
url = "java.net.URL"
```

External types let you reference types from other UniFFI-ed crates without
duplicating definitions.

## Built-in types reference

| Rust | UDL | Kotlin | Swift |
|------|-----|--------|-------|
| `bool` | `boolean` | `Boolean` | `Bool` |
| `u8`–`u64` | `u8`–`u64` | `UByte`–`ULong` | `UInt8`–`UInt64` |
| `i8`–`i64` | `i8`–`i64` | `Byte`–`Long` | `Int8`–`Int64` |
| `f32` | `float` | `Float` | `Float` |
| `f64` | `double` | `Double` | `Double` |
| `String` | `string` | `String` | `String` |
| `Vec<T>` | `sequence<T>` | `List<T>` | `[T]` |
| `HashMap<K,V>` | `record<K,V>` | `Map<K,V>` | `Dictionary<K,V>` |
| `Option<T>` | `T?` | `T?` | `T?` |
| `Vec<u8>` | `bytes` | `ByteArray` | `Data` |
| `()` | `void` | `Unit` | `Void` / `()` |
| `SystemTime` | `timestamp` | `java.time.Instant` | `Date` |
| `Duration` | `duration` | `java.time.Duration` | `TimeInterval` |
