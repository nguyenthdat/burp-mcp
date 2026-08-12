# api-avoid-telescoping-constructors

> Avoid telescoping constructors; use a builder

## Why It Matters

The telescoping constructor pattern — a constructor for every combination of required and optional parameters — grows combinatorially as fields are added, and callers end up passing values for parameters they don't care about just to reach the overload with the one they want. Beyond the sheer number of overloads, parameters of the same type (several `int`s or `boolean`s in a row) become impossible to tell apart at the call site, turning every invocation into a game of counting positions against the Javadoc.

## Bad

```java
public class NutritionFacts {
    private final int servingSize;
    private final int servings;
    private final int calories;
    private final int fat;
    private final int sodium;
    private final int carbohydrate;

    public NutritionFacts(int servingSize, int servings) {
        this(servingSize, servings, 0);
    }

    public NutritionFacts(int servingSize, int servings, int calories) {
        this(servingSize, servings, calories, 0);
    }

    public NutritionFacts(int servingSize, int servings, int calories, int fat) {
        this(servingSize, servings, calories, fat, 0);
    }

    public NutritionFacts(int servingSize, int servings, int calories, int fat, int sodium) {
        this(servingSize, servings, calories, fat, sodium, 0);
    }

    public NutritionFacts(int servingSize, int servings, int calories, int fat,
                           int sodium, int carbohydrate) {
        this.servingSize = servingSize;
        this.servings = servings;
        this.calories = calories;
        this.fat = fat;
        this.sodium = sodium;
        this.carbohydrate = carbohydrate;
    }
}

// Which int is sodium and which is carbohydrate? Have to check the declaration.
NutritionFacts cocaCola = new NutritionFacts(240, 8, 100, 0, 35, 27);
```

## Good

```java
public final class NutritionFacts {
    private final int servingSize;
    private final int servings;
    private final int calories;
    private final int fat;
    private final int sodium;
    private final int carbohydrate;

    private NutritionFacts(Builder b) {
        this.servingSize = b.servingSize;
        this.servings = b.servings;
        this.calories = b.calories;
        this.fat = b.fat;
        this.sodium = b.sodium;
        this.carbohydrate = b.carbohydrate;
    }

    public static final class Builder {
        // required
        private final int servingSize;
        private final int servings;
        // optional - sensible defaults
        private int calories = 0;
        private int fat = 0;
        private int sodium = 0;
        private int carbohydrate = 0;

        public Builder(int servingSize, int servings) {
            this.servingSize = servingSize;
            this.servings = servings;
        }

        public Builder calories(int val) { calories = val; return this; }
        public Builder fat(int val) { fat = val; return this; }
        public Builder sodium(int val) { sodium = val; return this; }
        public Builder carbohydrate(int val) { carbohydrate = val; return this; }

        public NutritionFacts build() { return new NutritionFacts(this); }
    }
}

// Each value is named at the call site - no counting positions
NutritionFacts cocaCola = new NutritionFacts.Builder(240, 8)
        .calories(100)
        .sodium(35)
        .carbohydrate(27)
        .build();
```

## A Lighter Alternative: Named Parameters via a Record

```java
// When every field is required and there is no combinatorial overload problem,
// a record with a single canonical constructor is simpler than a builder
public record NutritionFacts(
        int servingSize, int servings, int calories, int fat, int sodium, int carbohydrate) {}
```

## See Also

- [`api-builder-complex-construction`](api-builder-complex-construction.md) - The pattern that replaces telescoping constructors
- [`api-static-factory-over-constructor`](api-static-factory-over-constructor.md) - A lighter-weight alternative when overloads are still manageable
- [`api-record-data-carrier`](api-record-data-carrier.md) - Records as an alternative when all fields are required
- [`api-fluent-method-chaining`](api-fluent-method-chaining.md) - Designing the builder's chained setters
