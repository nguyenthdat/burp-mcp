# coll-comparator-composition

> Compose `Comparator`s with `comparing`/`thenComparing`

## Why It Matters

Hand-written `compareTo`/`Comparator` implementations with manual field-by-field `if` chains are verbose and error-prone, especially around tie-breaking and reversed order. `Comparator.comparing`, `thenComparing`, and `reversed` compose small, readable pieces into a single expression that reads in the same order the sort actually applies.

## Bad

```java
// Manual multi-field comparison logic, easy to get the tie-break wrong
List<Employee> employees = loadEmployees();
employees.sort((a, b) -> {
    int deptCompare = a.department().compareTo(b.department());
    if (deptCompare != 0) {
        return deptCompare;
    }
    int salaryCompare = Double.compare(b.salary(), a.salary()); // descending, easy to invert by mistake
    if (salaryCompare != 0) {
        return salaryCompare;
    }
    return a.name().compareTo(b.name());
});
```

## Good

```java
// Reads as "by department, then by salary descending, then by name"
List<Employee> employees = loadEmployees();
employees.sort(
    Comparator.comparing(Employee::department)
        .thenComparing(Comparator.comparingDouble(Employee::salary).reversed())
        .thenComparing(Employee::name));
```

## Common Composition Patterns

```java
// Null-safe comparison - nulls sort last regardless of the underlying comparator
Comparator<Employee> byManagerName = Comparator.comparing(
    Employee::managerName, Comparator.nullsLast(Comparator.naturalOrder()));

// Reusable comparator constant
Comparator<Order> BY_DATE_DESC = Comparator.comparing(Order::placedAt).reversed();

// Reverse an entire composed chain at once
Comparator<Employee> newestFirst = Comparator.comparing(Employee::hireDate).reversed();

// Sorting a stream with a composed comparator
List<Employee> sorted = employees.stream()
    .sorted(Comparator.comparing(Employee::department)
        .thenComparing(Employee::name))
    .toList();
```

## Primitive-Specialized Comparators

Use `comparingInt`/`comparingLong`/`comparingDouble` instead of `comparing` with a boxed key extractor to avoid boxing overhead in the comparator itself:

```java
// Boxes each Integer key during every comparison
Comparator<Employee> byAgeBoxed = Comparator.comparing(Employee::age);

// Stays primitive throughout the comparison
Comparator<Employee> byAge = Comparator.comparingInt(Employee::age);
```

## See Also

- [`api-equals-hashcode-contract`](api-equals-hashcode-contract.md) - Related contract concerns for object identity
- [`coll-primitive-streams-hot-path`](coll-primitive-streams-hot-path.md) - Avoiding boxing, same principle applied to comparators
- [`coll-choose-right-collection`](coll-choose-right-collection.md) - `TreeSet`/`TreeMap` accept a `Comparator` for custom ordering
- [`type-generic-method-inference`](type-generic-method-inference.md) - Type inference behavior relevant to comparator chains
