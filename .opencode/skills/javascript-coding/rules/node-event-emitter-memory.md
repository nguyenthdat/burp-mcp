# node-event-emitter-memory

> Remove event listeners when they're no longer needed to prevent memory leaks

## Why It Matters

Event emitters hold references to listener functions. If listeners aren't removed, the emitter can't be garbage collected, and neither can anything the listener closes over. In long-running processes, this accumulates and causes memory leaks. Use `once()` for one-shot listeners and `removeListener()` in cleanup.

## Bad

```js
import { EventEmitter } from 'node:events';

class DataPoller extends EventEmitter {
  start() {
    this.interval = setInterval(() => {
      this.emit('data', fetchData());
    }, 1000);
  }
}

// Listeners accumulate — never removed
for (let i = 0; i < 1000; i++) {
  const poller = new DataPoller();
  poller.on('data', (data) => {
    console.log(data);  // Listener holds reference to poller — can't GC
  });
  poller.start();
  // poller is never cleaned up — memory leak
}
```

## Good

```js
class DataPoller extends EventEmitter {
  start() {
    this.interval = setInterval(() => {
      this.emit('data', fetchData());
    }, 1000);
  }

  stop() {
    clearInterval(this.interval);
    this.removeAllListeners();  // Clean up
  }
}

// Use once() for one-time events
emitter.once('ready', () => {
  console.log('Ready — this listener auto-removes');
});

// Track and remove listeners
function setup() {
  const handler = (data) => process(data);

  emitter.on('data', handler);

  return () => {
    emitter.removeListener('data', handler);  // Cleanup function
  };
}

const cleanup = setup();
// Later:
cleanup();  // Removes the listener
```

## When Exceptions Apply

Long-lived global emitters (e.g., a process-level event bus) that exist for the application's lifetime don't need listener cleanup. Clean up emitters with shorter lifetimes.

## See Also

- [node-signal-handling](./node-signal-handling.md) - Graceful shutdown
- [err-global-handlers](./err-global-handlers.md) - Global handlers
