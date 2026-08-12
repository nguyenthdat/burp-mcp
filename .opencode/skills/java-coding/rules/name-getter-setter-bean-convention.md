# name-getter-setter-bean-convention

> Follow JavaBean getter/setter naming for accessors

## Why It Matters

Countless frameworks (Jackson, JPA/Hibernate, JSP EL, Spring data binding, Bean Validation) use reflection-based introspection that specifically looks for `getX`/`setX`/`isX` method name patterns to discover properties; deviating from that convention silently breaks serialization, form binding, or ORM mapping without a compile error. Consistent bean naming also lets IDEs auto-generate accessors and lets other developers predict a property's method names without reading the class.

## Bad

```java
public class Customer {

    private String name;
    private int age;
    private boolean active;

    public String name() {         // not a valid bean getter - Jackson won't find it
        return name;
    }

    public void changeName(String name) {  // not "setName" - binding frameworks miss it
        this.name = name;
    }

    public int fetchAge() {        // non-standard prefix
        return age;
    }

    public boolean getActive() {   // should be "isActive" for a boolean
        return active;
    }
}
```

## Good

```java
public class Customer {

    private String name;
    private int age;
    private boolean active;

    public String getName() {
        return name;
    }

    public void setName(String name) {
        this.name = name;
    }

    public int getAge() {
        return age;
    }

    public void setAge(int age) {
        this.age = age;
    }

    public boolean isActive() {
        return active;
    }

    public void setActive(boolean active) {
        this.active = active;
    }
}
```

## Records and Immutable Types Are Exempt

Records intentionally break the JavaBean getter convention: the accessor is named exactly after the component, with no `get`/`is` prefix. This is by design, not an oversight, so do not "fix" it to match bean style, and be aware some bean-introspection-based frameworks need a Jackson module (`jackson-module-parameter-names` or record support in modern Jackson) to work with records correctly.

```java
public record Customer(String name, int age, boolean active) {
    // Accessors are name(), age(), active() - not getName()/isActive()
}
```

For builder-style fluent APIs that are not JavaBeans (see `api-builder-complex-construction`), it is also acceptable to drop `get`/`set` entirely in favor of the bare property name, as long as the class is not expected to be introspected as a bean.

## See Also

- [`name-boolean-is-has-can`](name-boolean-is-has-can.md) - Name boolean accessors isX/hasX/canX
- [`name-methods-camel`](name-methods-camel.md) - Use camelCase for methods and fields
- [`api-record-data-carrier`](api-record-data-carrier.md) - Use records for simple data carriers
- [`api-builder-complex-construction`](api-builder-complex-construction.md) - Use builders for complex construction
