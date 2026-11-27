# Redis Pub/Sub Integration for Horizontal Scaling

This document describes the Redis Pub/Sub integration that enables horizontal scaling of the BrainStorm Rust server.

## Overview

The Pub/Sub layer allows multiple server instances to synchronize room state. When a player performs an action in a room (join, leave, submit answer, etc.), the event is:

1. Sent to the local broadcast channel (for WebSocket clients connected to this instance)
2. Published to Redis channel `room:{code}` (for other instances)

Other instances subscribed to that room receive the event and forward it to their local WebSocket clients.

## Architecture

```
Instance A                    Redis                     Instance B
┌─────────────┐               ┌─────────┐               ┌─────────────┐
│ RoomManager │─publish()────▶│ Room:ABC│─subscribe()─▶│ PubSubMgr   │
│             │               │ channel │               │             │
│ broadcast   │◀─subscribe()──│         │◀─publish()────│ broadcast   │
│ (clients)   │               └─────────┘               │ (clients)   │
└─────────────┘                                          └─────────────┘
```

## Deduplication

Each published message includes the source `instance_id` (generated at startup). When a message is received from Redis, the manager checks if it originated from this instance. If so, it's ignored (already sent to local clients). Otherwise, it's forwarded to the local broadcast channel.

## Components

### PubSubManager

Located in `rust-server/src/infrastructure/pubsub_manager.rs`.

**Key methods:**

```rust
// Publish a room event to Redis
pub async fn publish(&self, room_code: &str, message: ServerMessage) -> Result<()>

// Subscribe to a room's Redis channel
pub async fn subscribe<F>(
    &self,
    room_code: &str,
    broadcast_tx: broadcast::Sender<ServerMessage>,
    on_message: F,
) -> Result<()>
where
    F: Fn(ServerMessage) + Send + Sync + 'static

// Unsubscribe from a room's Redis channel
pub async fn unsubscribe(&self, room_code: &str)
```

### RoomManager Integration

The `RoomManager` in `rust-server/src/ws/room_manager.rs` has been updated to:

1. **Subscribe on room creation**: When a room is created, the manager subscribes to `room:{code}` channel
2. **Publish on state changes**: After sending to local broadcast, the manager publishes to Redis
3. **Unsubscribe on room removal**: When a room is removed (empty), the manager unsubscribes

**Modified methods:**
- `create_room` — subscribes to Redis channel
- `join_room` — publishes PlayerJoined and PlayersUpdated to Redis
- `leave_room` — publishes PlayerLeft and PlayersUpdated to Redis; unsubscribes if room is empty
- `start_game` — publishes GameStarted and QuestionEmitted to Redis
- `submit_answer` — publishes QuestionResult to Redis
- `send_chat` — publishes ChatMessage to Redis

### ServerMessage Serialization

`ServerMessage` and all nested types now implement `Deserialize` in addition to `Serialize`, enabling JSON serialization for Redis Pub/Sub.

**Updated types:**
- `ServerMessage` (ws/protocol.rs)
- `PlayerInfo` (ws/protocol.rs)
- `QuestionData` (ws/protocol.rs)
- `PlayerScore` (ws/protocol.rs)

## Configuration

Redis URL is configured via:
- `config/default.toml`: `redis_url = "redis://127.0.0.1:6379"`
- Environment variable: `REDIS_URL=redis://...`

If Redis is unavailable, the application runs in local-only mode (no Pub/Sub).

## Health Check

The `/readyz` endpoint now reports Pub/Sub status:

```json
{
  "status": "ready",
  "db": "ok",
  "redis": "ok",
  "pubsub": {
    "instance_id": "550e8400-e29b-41d4-a716-446655440000",
    "subscriptions": 5
  }
}
```

## Testing

### Unit Tests

```bash
cargo test --lib infrastructure::pubsub_manager
```

### Integration Tests

Integration tests require a running Redis instance:

```bash
# Start Redis via docker-compose
docker-compose up -d redis

# Run integration tests
cargo test --test pubsub_integration_test -- --ignored
```

### Manual Testing (Multi-Instance)

To test horizontal scaling locally:

```bash
# Start Redis and PostgreSQL
docker-compose up -d

# Terminal 1: Instance A (port 5000)
cd rust-server
RUST_LOG=info BRAINSTORM_LISTEN=0.0.0.0:5000 cargo run

# Terminal 2: Instance B (port 5001)
cd rust-server
RUST_LOG=info BRAINSTORM_LISTEN=0.0.0.0:5001 cargo run

# Terminal 3: Check health
curl http://localhost:5000/readyz | jq
curl http://localhost:5001/readyz | jq
```

Both instances should have the same `instance_id` in their Pub/Sub config, but different values (each generates its own at startup).

## Deployment

For production deployment with multiple instances:

1. **Load balancer**: Configure sticky sessions or use a WebSocket-aware load balancer
2. **Redis**: Use Redis Sentinel or Redis Cluster for high availability
3. **Monitoring**: Monitor Pub/Sub latency and subscription count via `/readyz` endpoint

### Example Kubernetes Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: brainstorm-server
spec:
  replicas: 3
  template:
    spec:
      containers:
      - name: server
        image: brainstorm-server:latest
        env:
        - name: REDIS_URL
          value: "redis://redis-sentinel:26379"
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: brainstorm-secrets
              key: database-url
```

## Limitations

1. **Message ordering**: Redis Pub/Sub does not guarantee message ordering across instances. If ordering is critical, consider using Redis Streams.
2. **Message persistence**: Pub/Sub messages are not persisted. If an instance disconnects, it misses messages. For critical state, use Redis Lists or Streams.
3. **Latency**: Pub/Sub adds ~1-5ms latency per message. For most use cases this is negligible, but may affect real-time competitive gameplay.

## Future Improvements

1. **Redis Streams**: Replace Pub/Sub with Streams for message persistence and replay
2. **Sharding**: Shard rooms across multiple Redis instances for better scalability
3. **Compression**: Compress large messages (e.g., question data) before publishing
4. **Metrics**: Export Pub/Sub metrics via Prometheus (messages/sec, latency, errors)

## Related

- [Redis Client Documentation](./README.md)
- [Horizontal scaling roadmap](../../../ROADMAP.md)
- [Epic 1: Horizontal Scaling](../../../docs/EPIC_1_HORIZONTAL_SCALING.md)
