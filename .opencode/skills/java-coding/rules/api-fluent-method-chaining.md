# api-fluent-method-chaining

> Design fluent, chainable APIs deliberately

## Why It Matters

Fluent chains read like a sentence and eliminate the noise of a temporary variable per step, but the pattern only pays off when each method's return type and side effects are unambiguous — a chain that silently mixes mutation and immutable transformation, or that hides which calls are required versus optional, turns "readable" into "misleading." Deliberate fluent design means every link in the chain either clearly mutates `this` and returns it, or clearly returns a new object, never both.

## Bad

```java
public class QueryBuilder {
    private String table;
    private List<String> conditions = new ArrayList<>();
    private int limit = -1;

    // Inconsistent: some methods mutate and return this, others return
    // a copy - callers can't tell without reading the implementation
    public QueryBuilder from(String table) {
        this.table = table;
        return this;
    }

    public QueryBuilder where(String condition) {
        conditions.add(condition);
        return this;
    }

    public QueryBuilder limit(int n) {
        QueryBuilder copy = new QueryBuilder(); // returns a NEW instance here
        copy.table = this.table;
        copy.conditions = new ArrayList<>(this.conditions);
        copy.limit = n;
        return copy;
    }
}

QueryBuilder qb = new QueryBuilder().from("users").where("active = true");
qb.limit(10); // return value discarded - did this mutate qb or not? Unclear and, here, wrong.
```

## Good

```java
public final class QueryBuilder {
    private String table;
    private final List<String> conditions = new ArrayList<>();
    private int limit = -1;

    // Every method mutates this builder and returns it - the contract is consistent
    public QueryBuilder from(String table) {
        this.table = table;
        return this;
    }

    public QueryBuilder where(String condition) {
        conditions.add(condition);
        return this;
    }

    public QueryBuilder limit(int n) {
        this.limit = n;
        return this;
    }

    public Query build() {
        return new Query(table, List.copyOf(conditions), limit);
    }
}

Query query = new QueryBuilder()
        .from("users")
        .where("active = true")
        .limit(10)
        .build(); // terminal method makes an immutable, final result explicit
```

## Fluent Immutable Chains (No Shared Mutable State)

```java
public record Filter(String field, String operator, String value) {

    // Each method returns a new, independent object - safe to share and reuse
    public Filter withValue(String newValue) {
        return new Filter(field, operator, newValue);
    }
}

Filter base = new Filter("status", "=", "pending");
Filter shipped = base.withValue("shipped"); // base is untouched, no aliasing surprises
```

## See Also

- [`api-builder-complex-construction`](api-builder-complex-construction.md) - The most common home for fluent chains
- [`api-immutable-by-default`](api-immutable-by-default.md) - Choosing between mutate-and-return vs. copy-and-return chains
- [`coll-comparator-composition`](coll-comparator-composition.md) - Fluent chaining applied to `Comparator`
- [`conc-completablefuture-composition`](conc-completablefuture-composition.md) - Fluent chaining applied to asynchronous pipelines
