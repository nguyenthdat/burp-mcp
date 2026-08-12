# Kotlin Bindings

Use these commands and code patterns for generating and consuming UniFFI
Kotlin bindings in Android or JVM projects.

## Generate Kotlin bindings

From UDL:

```bash
cargo run --bin uniffi-bindgen generate src/<name>.udl --language kotlin
```

From compiled library:

```bash
cargo build --release
cargo run --bin uniffi-bindgen generate \
  --library target/release/lib<name>.so \
  --language kotlin \
  --out-dir bindings/kotlin
```

Both produce `<namespace>.kt` — the generated Kotlin file to add to your
project.

## Kotlin consumer code

```kotlin
// Load the native library
System.loadLibrary("<name>")

// Use the generated API
val todoList = TodoList()
todoList.addItem("Buy groceries")
todoList.addItem("Write UniFFI bindings")

val items: List<TodoEntry> = todoList.getItems()
for (item in items) {
    println("${item.text} - done: ${item.done}")
}

// Call a top-level function
val greeting = hello("Kotlin")
println(greeting) // "Hello, Kotlin!"
```

### Async / coroutines

```kotlin
import kotlinx.coroutines.runBlocking

runBlocking {
    val todos: List<TodoEntry> = todoList.loadFromServer("https://api.example.com/todos")
    todos.forEach { println(it.text) }
}
```

Async Rust functions return `Deferred<T>` in Kotlin. `await()` suspends the
coroutine. Requires adding to `build.gradle`:

```groovy
dependencies {
    implementation "org.jetbrains.kotlinx:kotlinx-coroutines-core:1.6.4"
}
```

### Error handling

```kotlin
try {
    val todo = findTodo(0)
} catch (e: MyError.NotFound) {
    println("Not found: ${e.resource}")
} catch (e: MyError) {
    println("Other error: ${e.message}")
}
```

Each `#[derive(uniffi::Error)]` variant becomes a sealed subclass of the error
type in Kotlin, with typed fields.

### Enums with data

```kotlin
when (event) {
    is Event.None -> println("No event")
    is Event.Text -> println("Text: ${event.v1}")
    is Event.Clicked -> println("Clicked at ${event.x}, ${event.y}")
    is Event.MultiTouch -> println("Touches: ${event.v1.size}")
}
```

Enum variant names with associated data use `v1`, `v2`, etc. for tuple fields
and named accessors for struct-like fields (`x`, `y`).

## Gradle integration

Add to `build.gradle` to auto-generate bindings during build:

```groovy
android.libraryVariants.all { variant ->
    def t = tasks.register("generate${variant.name.capitalize()}UniFFIBindings", Exec) {
        workingDir "${project.projectDir}"
        commandLine 'uniffi-bindgen', 'generate',
            '<PATH TO .udl FILE>',
            '--language', 'kotlin',
            '--out-dir', "${buildDir}/generated/source/uniffi/${variant.name}/java"
    }
    variant.javaCompileProvider.get().dependsOn(t)
    def sourceSet = variant.sourceSets.find { it.name == variant.name }
    sourceSet.java.srcDir new File(buildDir, "generated/source/uniffi/${variant.name}/java")
    idea.module.generatedSourceDirs += file("${buildDir}/generated/source/uniffi/${variant.name}/java/uniffi")
}
```

## uniffi.toml for Kotlin

```toml
[bindings.kotlin]
package_name = "com.example.mylibrary"
cdylib_name = "my_library"

[bindings.kotlin.external_packages]
url = "java.net.URL"
uuid = "java.util.UUID"
```

- `package_name`: the Kotlin package for generated code.
- `cdylib_name`: the library filename for `System.loadLibrary()`.
- `external_packages`: map external crate names to Kotlin fully-qualified
  types.

## Library loading

UniFFI-generated Kotlin code calls `System.loadLibrary(cdylib_name)`. The
cdylib must be:
- In `jniLibs/<abi>/` for Android.
- On `java.library.path` for desktop JVM.

For Android, place the compiled `.so` files in:
```
app/src/main/jniLibs/
├── arm64-v8a/libmy_library.so
├── armeabi-v7a/libmy_library.so
└── x86_64/libmy_library.so
```

## Common type mappings

| Rust | Kotlin |
|------|--------|
| `Vec<T>` | `List<T>` |
| `HashMap<K,V>` | `Map<K,V>` |
| `Option<T>` | `T?` |
| `String` | `String` |
| `Vec<u8>` | `ByteArray` |
| `()` | `Unit` |
| `SystemTime` | `java.time.Instant` |
| `Duration` | `java.time.Duration` |
| `u8`–`u64` | `UByte`–`ULong` |
| `i8`–`i64` | `Byte`–`Long` |

## Best practices

- Generate bindings in CI to ensure they stay in sync with the Rust source.
- Commit the generated `.kt` file for auditability; regenerate on every Rust
  change.
- Use `--library` mode for multi-crate projects to avoid UDL duplication.
- Run `System.loadLibrary` once at app startup; the generated code does this
  automatically from `uniffi.toml` config.
- Profile async performance — UniFFI async has overhead from the foreign
  executor bridge.
