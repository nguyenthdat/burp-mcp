# Swift Bindings

Use these commands and code patterns for generating and consuming UniFFI
Swift bindings in iOS, macOS, or SwiftPM projects.

## Generate Swift bindings

From UDL:

```bash
cargo run --bin uniffi-bindgen generate src/<name>.udl --language swift
```

From compiled library:

```bash
cargo build --release
cargo run --bin uniffi-bindgen generate \
  --library target/release/lib<name>.dylib \
  --language swift \
  --out-dir bindings/swift
```

In Xcode build phase:

```bash
$HOME/.cargo/bin/cargo run -p uniffi-bindgen -- generate \
  "$INPUT_FILE_PATH" \
  --language swift \
  --out-dir "$DERIVED_FILE_DIR"
```

## Swift consumer code

```swift
// Import the generated module (if compiled as a module)
import <name>

// Use the generated API
let todoList = TodoList()
todoList.addEntry(text: "Buy groceries")
todoList.addEntry(text: "Write UniFFI bindings")

let items = todoList.getItems()
for item in items {
    print("\(item.text) - done: \(item.done)")
}

// Call a top-level function
let greeting = hello(name: "Swift")
print(greeting) // "Hello, Swift!"
```

### Async / await

```swift
Task {
    let todos = try await todoList.loadFromServer(url: "https://api.example.com/todos")
    for todo in todos {
        print(todo.text)
    }
}
```

Async Rust functions are exposed as native Swift `async throws` functions.

### Error handling

```swift
do {
    let todo = try findTodo(id: 0)
} catch let error as MyError {
    switch error {
    case .notFound(let resource):
        print("Not found: \(resource)")
    case .permissionDenied(let reason):
        print("Denied: \(reason)")
    case .timeout:
        print("Timed out")
    }
}
```

Each `#[derive(uniffi::Error)]` variant becomes a case on the generated error
enum in Swift, conforming to `Error`.

### Enums with data

```swift
switch event {
case .none:
    print("No event")
case .text(let value):
    print("Text: \(value)")
case .clicked(let x, let y):
    print("Clicked at \(x), \(y)")
case .multiTouch(let points):
    print("Touches: \(points.count)")
}
```

## Compile Swift module

Combine the cdylib and generated Swift into a module:

```bash
swiftc \
    -module-name <name> \
    -emit-library -o lib<name>.dylib \
    -emit-module -emit-module-path ./ \
    -parse-as-library \
    -L ./target/debug/ \
    -l<name> \
    -Xcc -fmodule-map-file=<name>FFI.modulemap \
    <name>.swift
```

This produces `lib<name>.dylib`, `<name>.swiftmodule`, and supporting files
that can be imported in Swift projects.

## Xcode project integration

1. Add the `.udl` file to the project.
2. Add a **Run Script** build phase:

```bash
$HOME/.cargo/bin/cargo run -p uniffi-bindgen -- generate \
    "$INPUT_FILE_PATH" \
    --language swift \
    --out-dir "$DERIVED_FILE_DIR"
```

3. Add `$DERIVED_FILE_DIR` to Xcode's header search paths.
4. Link the compiled Rust `cdylib` in **Build Phases > Link Binary With
   Libraries**.

## iOS xcframework

For iOS distribution, compile the Rust library for all target architectures
and wrap in an xcframework:

```bash
# Build for iOS device (arm64)
cargo build --release --target aarch64-apple-ios

# Build for iOS simulator (arm64)
cargo build --release --target aarch64-apple-ios-sim

# Create xcframework
xcodebuild -create-xcframework \
    -library target/aarch64-apple-ios/release/lib<name>.a \
    -library target/aarch64-apple-ios-sim/release/lib<name>.a \
    -output <name>.xcframework
```

Add the `.xcframework` to Xcode's **Frameworks, Libraries, and Embedded
Content**.

## uniffi.toml for Swift

```toml
[bindings.swift]
cdylib_name = "my_library"
```

`cdylib_name` must match the library filename that Swift loads.

## Common type mappings

| Rust | Swift |
|------|-------|
| `Vec<T>` | `[T]` |
| `HashMap<K,V>` | `Dictionary<K,V>` |
| `Option<T>` | `T?` |
| `String` | `String` |
| `Vec<u8>` | `Data` |
| `()` | `Void` / `()` |
| `SystemTime` | `Date` |
| `Duration` | `TimeInterval` |
| `u8`–`u64` | `UInt8`–`UInt64` |
| `i8`–`i64` | `Int8`–`Int64` |
| `f32`, `f64` | `Float`, `Double` |

## Best practices

- Generate bindings in Xcode build phases to keep them in sync with Rust.
- Commit generated Swift files for projects without CI that runs UniFFI.
- Use `--library` mode for multi-crate projects.
- For iOS, build `aarch64-apple-ios` and `aarch64-apple-ios-sim` targets and
  package as xcframework.
- Profile async performance — the foreign executor bridge adds overhead.
- The Swift module compilation step is optional for simple projects; add the
  generated `.swift` file directly to Xcode for small codebases.
