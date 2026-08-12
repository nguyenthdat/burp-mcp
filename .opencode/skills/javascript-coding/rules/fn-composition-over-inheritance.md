# fn-composition-over-inheritance

> Prefer function composition and factory functions over class inheritance hierarchies

## Why It Matters

Class inheritance creates rigid hierarchies that are hard to refactor and lead to the "gorilla-banana problem" (you want a banana but get the whole jungle). Function composition — building objects by combining independent functions — is more flexible, testable, and avoids deep prototype chains. JavaScript's prototypal nature makes composition especially natural.

## Bad

```js
// Deep inheritance — fragile, hard to change
class Animal {
  eat() { /* ... */ }
}

class Mammal extends Animal {
  breathe() { /* ... */ }
}

class Dog extends Mammal {
  bark() { /* ... */ }
}

class GuardDog extends Dog {
  guard() { /* ... */ }
  // Now we need a HuntingDog that also barks but doesn't guard...
  // Hierarchy breaks down
}
```

## Good

```js
// Composition — mix and match behaviors
const canEat = () => ({
  eat(food) { console.log(`Eating ${food}`); },
});

const canBark = () => ({
  bark() { console.log('Woof!'); },
});

const canGuard = () => ({
  guard() { console.log('Guarding...'); },
});

function createDog(name) {
  return {
    name,
    ...canEat(),
    ...canBark(),
  };
}

function createGuardDog(name) {
  return {
    ...createDog(name),
    ...canGuard(),
  };
}

const rex = createGuardDog('Rex');
rex.eat('bone');
rex.bark();
rex.guard();
```

## Factory Functions

```js
// Cleaner than classes for most use cases
function createUser({ name, email, role = 'user' }) {
  return {
    name,
    email,
    role,
    isAdmin() { return this.role === 'admin'; },
    toJSON() { return { name, email, role }; },
  };
}
```

## When Exceptions Apply

Use classes when:
- You need `instanceof` checks on a type hierarchy
- You're working with a framework that expects classes (custom elements, React class components — legacy)
- Performance requires shared methods on the prototype (though function composition is usually fine)
- The domain model genuinely benefits from an "is-a" hierarchy

## See Also

- [fn-immutability](./fn-immutability.md) - Immutable patterns
- [fn-pure-functions](./fn-pure-functions.md) - Pure function design
