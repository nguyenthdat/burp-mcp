# coll-avoid-legacy-classes

> Avoid legacy `Vector`/`Hashtable`/`Stack` classes

## Why It Matters

`Vector`, `Hashtable`, and `Stack` predate the Collections Framework and synchronize every method call, even in single-threaded code, which adds needless lock overhead. `Stack` additionally extends `Vector`, inheriting random-access methods like `insertElementAt` that violate stack discipline. Modern replacements (`ArrayList`, `HashMap`, `ArrayDeque`) are faster, and `java.util.concurrent` classes are the correct choice when thread safety is actually needed.

## Bad

```java
// Vector - synchronizes every add/get even when only one thread ever touches it
Vector<String> names = new Vector<>();
names.add("alice");
names.add("bob");

// Hashtable - synchronized, disallows null keys/values, legacy iteration via Enumeration
Hashtable<String, Integer> inventory = new Hashtable<>();
inventory.put("widget", 42);
Enumeration<String> keys = inventory.keys();
while (keys.hasMoreElements()) {
    String key = keys.nextElement();
    System.out.println(key + " -> " + inventory.get(key));
}

// Stack - extends Vector, exposes non-stack methods, synchronized overhead
Stack<Integer> callStack = new Stack<>();
callStack.push(1);
callStack.push(2);
callStack.insertElementAt(99, 0); // legal but breaks stack semantics entirely
int top = callStack.pop();
```

## Good

```java
// ArrayList - unsynchronized, use explicit synchronization or concurrent
// collections only where actually needed
List<String> names = new ArrayList<>();
names.add("alice");
names.add("bob");

// HashMap for single-threaded use, standard iteration
Map<String, Integer> inventory = new HashMap<>();
inventory.put("widget", 42);
for (var entry : inventory.entrySet()) {
    System.out.println(entry.getKey() + " -> " + entry.getValue());
}

// ArrayDeque as a Stack replacement - push/pop/peek only, no accidental misuse
Deque<Integer> callStack = new ArrayDeque<>();
callStack.push(1);
callStack.push(2);
int top = callStack.pop();
```

## When Thread Safety Is Actually Required

Don't reach for the legacy classes even then - use `java.util.concurrent`:

```java
Map<String, Integer> concurrentInventory = new ConcurrentHashMap<>();
List<String> concurrentNames = Collections.synchronizedList(new ArrayList<>());
Deque<Integer> concurrentStack = new ConcurrentLinkedDeque<>();
```

## See Also

- [`coll-choose-right-collection`](coll-choose-right-collection.md) - Picking the right structure for the access pattern
- [`conc-concurrent-collections`](conc-concurrent-collections.md) - Proper thread-safe collection choices
- [`conc-synchronized-scope`](conc-synchronized-scope.md) - Why blanket synchronization is costly
- [`coll-choose-right-collection`](coll-choose-right-collection.md) - General anti-pattern of reaching for pre-Java-5 APIs instead of choosing deliberately
