# api-equals-hashcode-contract

> Honor the `equals`/`hashCode` contract together

## Why It Matters

The contract is simple but unforgiving: if two objects are equal according to `equals`, they must produce the same `hashCode`. Overriding one without the other silently breaks every `HashMap`, `HashSet`, and `HashCollection` lookup — entries become "lost" because they hash to one bucket but compare equal to entries expected in another, producing bugs that only appear under specific data distributions and are painful to reproduce.

## Bad

```java
public class Employee {
    private final String id;
    private final String name;

    public Employee(String id, String name) {
        this.id = id;
        this.name = name;
    }

    @Override
    public boolean equals(Object o) {
        if (this == o) return true;
        if (!(o instanceof Employee other)) return false;
        return id.equals(other.id); // equality based on id only
    }
    // hashCode NOT overridden - still uses Object's identity hash!
}

var set = new HashSet<Employee>();
Employee e1 = new Employee("123", "Alice");
set.add(e1);

Employee e2 = new Employee("123", "Alice"); // equals() says this equals e1
System.out.println(e1.equals(e2));  // true
System.out.println(set.contains(e2)); // false! different hash bucket - lookup fails
```

## Good

```java
public final class Employee {
    private final String id;
    private final String name;

    public Employee(String id, String name) {
        this.id = Objects.requireNonNull(id);
        this.name = Objects.requireNonNull(name);
    }

    @Override
    public boolean equals(Object o) {
        if (this == o) return true;
        if (!(o instanceof Employee other)) return false;
        return id.equals(other.id); // use exactly the same fields as hashCode
    }

    @Override
    public int hashCode() {
        return Objects.hash(id); // same field(s) that determine equality
    }
}

var set = new HashSet<Employee>();
Employee e1 = new Employee("123", "Alice");
set.add(e1);

Employee e2 = new Employee("123", "Alice");
System.out.println(set.contains(e2)); // true - contract upheld
```

## Records Get This For Free

```java
public record EmployeeId(String value) {}

// The compiler generates equals/hashCode from all components consistently -
// no manual contract to violate
var ids = new HashSet<EmployeeId>();
ids.add(new EmployeeId("123"));
System.out.println(ids.contains(new EmployeeId("123"))); // true
```

Note that a record's generated `equals`/`hashCode` include every component; if you need a subset of fields (as in the `Employee` example above, keyed only on `id`), you must write both methods explicitly and keep them in sync manually, or model the key as its own record/type.

## See Also

- [`api-record-data-carrier`](api-record-data-carrier.md) - Records generate a consistent contract automatically
- [`api-tostring-diagnostics`](api-tostring-diagnostics.md) - The third method usually overridden alongside these two
- [`coll-choose-right-collection`](coll-choose-right-collection.md) - Hash-based collections that depend on this contract
- [`null-requireNonNull-guard`](null-requireNonNull-guard.md) - Guarding fields used in equality checks against null
