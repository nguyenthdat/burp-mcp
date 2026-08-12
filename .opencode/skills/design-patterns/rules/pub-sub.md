# pub-sub

> Broadcast events from producers to decoupled subscribers, typically across process or service boundaries, without either side knowing the other's identity.

## Intent & Pressure

Reach for Pub/Sub when producers and consumers must be decoupled across processes or services (a service publishes "order placed" and several unrelated services react independently — billing, shipping, analytics), and the set of subscribers can change without the publisher being redeployed. The pressure is the distributed generalization of [observer](observer.md): in-process callback lists aren't enough once consumers live in different services, need durability, or need independent scaling.

Do not reach for a message broker when everything runs in one process — that's plain [observer](observer.md). Do not reach for Pub/Sub as a substitute for a direct synchronous call when the caller genuinely needs an immediate, ordered response — it trades that away for decoupling and (usually) eventual delivery.

## Native-Construct Alternative

Within one process, an in-process event emitter/observer list is Pub/Sub without the broker. Escalate to a real message broker (Kafka, SQS/SNS, NATS, RabbitMQ, Google Pub/Sub) specifically when consumers span processes/services or need durable, replayable delivery.

## Language Implementations

### Rust

```rust
#[derive(Clone)]
struct OrderCreated { order_id: OrderId }

async fn publish(producer: &KafkaProducer, event: &OrderCreated) -> Result<(), PublishError> {
    producer.send("orders.created", serde_json::to_vec(event)?).await
}

async fn consume(consumer: &mut KafkaConsumer) -> Result<(), ConsumeError> {
    while let Some(message) = consumer.next().await {
        let event: OrderCreated = serde_json::from_slice(&message.payload)?;
        handle_order_created(event).await?;
        consumer.commit(&message).await?;
    }
    Ok(())
}
```

### TypeScript

```typescript
interface OrderCreated { orderId: string; }

async function publish(sns: SNSClient, event: OrderCreated): Promise<void> {
  await sns.send(new PublishCommand({
    TopicArn: ORDERS_TOPIC,
    Message: JSON.stringify(event),
  }));
}

// consumer (e.g., an SQS-backed Lambda or worker)
async function handleMessage(message: SQSMessage): Promise<void> {
  const event: OrderCreated = JSON.parse(message.Body);
  await handleOrderCreated(event);
}
```

### Python

```python
@dataclass
class OrderCreated:
    order_id: str

def publish(client: PubSubClient, event: OrderCreated) -> None:
    client.publish("orders.created", json.dumps(asdict(event)).encode())

def handle_message(message: bytes) -> None:
    event = OrderCreated(**json.loads(message))
    handle_order_created(event)
```

### Go

```go
type OrderCreated struct{ OrderID string `json:"order_id"` }

func Publish(ctx context.Context, producer *kafka.Producer, event OrderCreated) error {
    payload, err := json.Marshal(event)
    if err != nil {
        return err
    }
    return producer.Produce(ctx, "orders.created", payload)
}

func Consume(ctx context.Context, consumer *kafka.Consumer, handle func(OrderCreated) error) error {
    for msg := range consumer.Messages(ctx) {
        var event OrderCreated
        if err := json.Unmarshal(msg.Value, &event); err != nil {
            continue // dead-letter, don't crash the consumer loop
        }
        if err := handle(event); err != nil {
            return err
        }
        consumer.Commit(ctx, msg)
    }
    return nil
}
```

### C#

```csharp
public sealed record OrderCreated(string OrderId);

public async Task PublishAsync(ServiceBusSender sender, OrderCreated evt)
{
    var message = new ServiceBusMessage(JsonSerializer.SerializeToUtf8Bytes(evt));
    await sender.SendMessageAsync(message);
}

public async Task HandleMessageAsync(ServiceBusReceivedMessage message)
{
    var evt = message.Body.ToObjectFromJson<OrderCreated>();
    await HandleOrderCreatedAsync(evt);
}
```

### Kotlin

```kotlin
data class OrderCreated(val orderId: String)

suspend fun publish(producer: KafkaProducer<String, ByteArray>, event: OrderCreated) {
    producer.send(ProducerRecord("orders.created", Json.encodeToString(event).toByteArray()))
}

suspend fun consume(consumer: KafkaConsumer<String, ByteArray>, handle: suspend (OrderCreated) -> Unit) {
    consumer.poll(Duration.ofSeconds(1)).forEach { record ->
        val event = Json.decodeFromString<OrderCreated>(String(record.value()))
        handle(event)
    }
}
```

### C

```c
typedef struct { char order_id[64]; } order_created_t;

int publish_order_created(mq_producer_t *producer, const order_created_t *event) {
    char payload[128];
    snprintf(payload, sizeof(payload), "{\"order_id\":\"%s\"}", event->order_id);
    return mq_producer_send(producer, "orders.created", payload, strlen(payload));
}

int handle_message(const char *payload, size_t len) {
    order_created_t event;
    if (parse_order_created(payload, len, &event) != 0) return -1;
    return handle_order_created(&event);
}
```

### C++

```cpp
struct OrderCreated { std::string orderId; };

void publish(RdKafka::Producer &producer, const OrderCreated &event) {
    auto payload = toJson(event);
    producer.produce("orders.created", RdKafka::Producer::RK_MSG_COPY,
                      payload.data(), payload.size(), nullptr, 0, 0, nullptr);
}

void handleMessage(const RdKafka::Message &message) {
    auto event = fromJson<OrderCreated>(message.payload(), message.len());
    handleOrderCreated(event);
}
```

### Swift

```swift
struct OrderCreated: Codable { let orderId: String }

func publish(_ client: SNSClient, event: OrderCreated) async throws {
    let payload = try JSONEncoder().encode(event)
    try await client.publish(topic: ordersTopic, message: payload)
}

func handleMessage(_ payload: Data) async throws {
    let event = try JSONDecoder().decode(OrderCreated.self, from: payload)
    try await handleOrderCreated(event)
}
```

## Pitfalls

- Assuming exactly-once delivery from a broker that only guarantees at-least-once — consumers must be idempotent (dedupe by event ID).
- No schema versioning on published event payloads, breaking consumers when the publisher changes the shape without coordination.
- Treating message ordering as guaranteed across partitions/shards when the broker only guarantees per-partition/per-key ordering.
- No dead-letter handling for malformed or repeatedly-failing messages, causing a poison message to block the whole consumer.
- Using Pub/Sub where a direct synchronous call was actually required (the caller needs an immediate, ordered, correlated response) — that's the wrong tool for that need.

## See Also

- [observer](observer.md) — the in-process version of the same decoupling idea.
- [event-sourcing](event-sourcing.md) — often the source of the events being published.
- [circuit-breaker](circuit-breaker.md) — protecting a publisher/consumer from a failing broker or downstream dependency.
