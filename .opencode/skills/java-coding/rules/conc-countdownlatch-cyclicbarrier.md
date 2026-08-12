# conc-countdownlatch-cyclicbarrier

> Use `CountDownLatch`/`CyclicBarrier` for coordination

## Why It Matters

Coordinating "wait until N things have happened" with hand-rolled flags, `wait()`/`notify()`, or busy-polling loops is error-prone and hard to read. `CountDownLatch` (one-time, wait for N events) and `CyclicBarrier` (reusable, wait for N parties to all arrive before any proceeds) are purpose-built, well-tested primitives for exactly these coordination patterns.

## Bad

```java
// Busy-polling a shared counter to know when all workers have finished startup.
private volatile int workersReady = 0;

void startWorkers(int count) {
    for (int i = 0; i < count; i++) {
        new Thread(() -> {
            initialize();
            workersReady++; // not even atomic -- a lost update is possible
        }).start();
    }

    while (workersReady < count) { // busy-wait: burns CPU, no wakeup signal
        try {
            Thread.sleep(10);
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
            return;
        }
    }
    System.out.println("All workers ready");
}
```

## Good

```java
void startWorkers(int count) throws InterruptedException {
    CountDownLatch readyLatch = new CountDownLatch(count);

    for (int i = 0; i < count; i++) {
        Thread.ofVirtual().start(() -> {
            initialize();
            readyLatch.countDown(); // atomic, and wakes waiters immediately
        });
    }

    readyLatch.await(); // blocks efficiently until count reaches zero
    System.out.println("All workers ready");
}
```

## `CyclicBarrier`: Repeated Rendezvous Points

Use `CyclicBarrier` when a fixed set of parties must repeatedly wait for each other at successive phases of work (e.g., a simulation that advances in lockstep rounds):

```java
int parties = 4;
CyclicBarrier barrier = new CyclicBarrier(parties, () -> {
    System.out.println("All parties reached this round's barrier");
});

Runnable roundWorker = () -> {
    for (int round = 0; round < 10; round++) {
        computeRound(round);
        try {
            barrier.await(); // waits for the other 3 parties, then all proceed together
        } catch (InterruptedException | BrokenBarrierException e) {
            Thread.currentThread().interrupt();
            return;
        }
    }
};

for (int i = 0; i < parties; i++) {
    Thread.ofVirtual().start(roundWorker);
}
```

## Latch vs Barrier

A `CountDownLatch` is single-use: once it reaches zero, it stays open forever and cannot be reset. A `CyclicBarrier` resets automatically after each set of parties arrives, making it suitable for repeated synchronization points; it also supports a barrier action that runs once per cycle when the last party arrives.

## See Also

- [`conc-structured-concurrency`](conc-structured-concurrency.md) - Higher-level alternative for fork/join-style coordination
- [`conc-executorservice-shutdown`](conc-executorservice-shutdown.md) - Always shut down an ExecutorService properly
- [`conc-atomic-over-lock`](conc-atomic-over-lock.md) - Use atomic classes for simple counters instead of locks
