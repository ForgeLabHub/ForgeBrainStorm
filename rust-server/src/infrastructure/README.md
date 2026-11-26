# Infrastructure Module

This module contains clients for external infrastructure services.

## Redis Client

The `RedisClient` provides a wrapper around Redis operations with:

- **Connection management** via `ConnectionManager` for automatic reconnection
- **Graceful degradation** — if Redis is unavailable, the application continues in local-only mode
- **Pub/Sub support** for cross-instance message broadcasting
- **Atomic operations** like `SETNX` for distributed locking

### Usage

```rust
use brainstorm::infrastructure::RedisClient;

// Initialize client
let client = RedisClient::new("redis://127.0.0.1:6379").await?;

// Check connection
if client.is_connected().await {
    // Set with TTL
    client.set("key", "value", Some(3600)).await?;
    
    // Get value
    let value = client.get("key").await?;
    
    // Publish message
    client.publish("channel", "message").await?;
}
```

### Configuration

Redis URL is configured via:
- `config/default.toml`: `redis_url = "redis://127.0.0.1:6379"`
- Environment variable: `REDIS_URL=redis://...`

If `redis_url` is empty or Redis is unavailable, the application runs in local-only mode (no distributed caching or pub/sub).

### Health Check

The `/readyz` endpoint reports Redis status:
```json
{
  "status": "ready",
  "db": "ok",
  "redis": "ok"  // or "local_only"
}
```

### Testing

Integration tests require a running Redis instance:

```bash
# Start Redis via docker-compose
docker-compose up -d redis

# Run integration tests
cargo test --test redis_integration_test -- --ignored
```

## Future: Pub/Sub Integration

The next step is to integrate Redis Pub/Sub into `room_manager.rs` for horizontal scaling:

1. When a player joins a room, subscribe to `room:{code}` channel
2. Broadcast room events via Redis Pub/Sub
3. All instances receive the event and forward to their connected clients
4. This allows running multiple server instances behind a load balancer

See `EPIC_1_REDIS_PUBSUB.md` for the detailed implementation plan.
