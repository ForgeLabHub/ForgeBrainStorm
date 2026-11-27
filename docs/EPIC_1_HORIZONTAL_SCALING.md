# Epic 1: Horizontal Scaling

## Обзор

Epic 1 реализует полную поддержку горизонтального масштабирования BrainStorm, позволяя запускать несколько инстансов сервера за load balancer с общим состоянием.

## Цель

Превратить BrainStorm из монолитного приложения в распределённую систему, способную поддерживать 10,000+ одновременных игроков через:
- Распределённое состояние комнат
- Общий кэш сессий
- Распределённый AI кэш
- Синхронизация между инстансами

## Компоненты

### 1. Redis Infrastructure (PR #14)

**Файлы:**
- `rust-server/src/infrastructure/redis_client.rs`
- `rust-server/src/infrastructure/mod.rs`

**Возможности:**
- Redis клиент с connection management
- Graceful degradation (работает без Redis)
- Базовые операции: GET, SET, DEL, EXISTS
- Pub/Sub поддержка
- Health check интеграция

**Документация:**
- [Redis Client Documentation](../../rust-server/src/infrastructure/README.md)

### 2. Redis Pub/Sub (PR #15)

**Файлы:**
- `rust-server/src/infrastructure/pubsub_manager.rs`
- `rust-server/src/ws/room_manager.rs` (modified)

**Возможности:**
- Синхронизация комнат между инстансами
- Broadcast событий через Redis channels
- Deduplication сообщений (instance_id)
- Автоматическая подписка/отписка

**Архитектура:**
```
Instance A                  Redis                  Instance B
    |                          |                         |
    |--> publish(event) -------|-> Channel: room:ABC     |
    |                          |                         |
    |                          |<- subscribe() ----------|
    |                          |    (receive event)      |
    v                          v                         v
 Local clients            Redis Pub/Sub            Local clients
```

**Документация:**
- [Pub/Sub Documentation](../../rust-server/src/infrastructure/PUBSUB.md)

### 3. Distributed Session Cache (PR #16)

**Файлы:**
- `rust-server/src/middleware/session_cache.rs` (modified)

**Возможности:**
- Сессии хранятся в Redis
- Доступны всем инстансам
- Graceful fallback на локальный кэш
- TTL: 5 минут (настраивается)

**Benefits:**
- Пользователи могут подключаться к любому инстансу
- Сессии не теряются при перезапуске
- Снижена нагрузка на БД

### 4. Distributed AI Cache (PR #18)

**Файлы:**
- `rust-server/src/services/ai/distributed_cache.rs`
- `rust-server/src/services/ai/router.rs` (modified)

**Возможности:**
- AI ответы кэшируются в Redis
- Questions: TTL 1 час
- Hints: TTL 30 минут
- Shared между всеми инстансами

**Benefits:**
- Снижение costs (меньше AI API вызовов)
- Faster response times
- Shared knowledge base

**Документация:**
- [Distributed Cache Documentation](../../rust-server/src/services/ai/DISTRIBUTED_CACHE.md)

## Архитектура

### Production Deployment

```
                    ┌─────────────────┐
                    │  Load Balancer  │
                    │  (nginx/HAProxy)│
                    └────────┬────────┘
                             │
              ┌──────────────┼──────────────┐
              │              │              │
              ▼              ▼              ▼
        ┌─────────┐   ┌─────────┐   ┌─────────┐
        │Instance │   │Instance │   │Instance │
        │   A     │   │   B     │   │   C     │
        └────┬────┘   └────┬────┘   └────┬────┘
             │             │             │
             └─────────────┼─────────────┘
                           │
                    ┌──────┴──────┐
                    │    Redis    │
                    │  (Cluster)  │
                    └──────┬──────┘
                           │
                    ┌──────┴──────┐
                    │ PostgreSQL  │
                    │  (Primary)  │
                    └─────────────┘
```

### Данные

**Redis хранит:**
- Сессии пользователей (`session:{token}`)
- События комнат (`room:{code}` channel)
- AI кэш (`ai:question:{key}`, `ai:hint:{key}`)

**PostgreSQL хранит:**
- Пользователи
- Статистика игр
- Лидерборды
-.Persistent данные

## Конфигурация

### Redis

```conf
# redis.conf
maxmemory 256mb
maxmemory-policy allkeys-lru
save 900 1
save 300 10
save 60 10000
```

### BrainStorm

```toml
# config/default.toml
redis_url = "redis://redis-sentinel:26379"

[session]
ttl_secs = 300

[ai_cache]
ttl_questions = 3600
ttl_hints = 1800
```

### Environment Variables

```bash
# .env
REDIS_URL=redis://redis:6379
SESSION_TTL=300
```

## Мониторинг

### Health Checks

```bash
# Liveness probe
GET /health

# Readiness probe
GET /readyz
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

### Метрики

**Redis:**
- `redis_connected_connections`
- `redis_memory_used_bytes`
- `redis_keyspace_hits_total`
- `redis_keyspace_misses_total`

**Application:**
- `ai_cache_hits_total`
- `ai_cache_misses_total`
- `session_cache_hits_total`
- `pubsub_messages_published_total`
- `pubsub_messages_received_total`

## Масштабирование

### Horizontal Scaling

```bash
# Запуск нескольких инстансов
docker-compose up -d --scale brainstorm=3

# Или с Kubernetes
kubectl scale deployment brainstorm --replicas=5
```

### Load Balancer Configuration

```nginx
# nginx.conf
upstream brainstorm {
    least_conn;
    server brainstorm-1:5000;
    server brainstorm-2:5000;
    server brainstorm-3:5000;
}

server {
    listen 80;
    
    location / {
        proxy_pass http://brainstorm;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
    }
}
```

## Testing

### Integration Tests

```bash
# Запустить Redis
docker-compose up -d redis

# Запустить тесты
cargo test --workspace

# Запустить integration тесты
cargo test --test distributed_cache_integration_test -- --ignored
cargo test --test pubsub_integration_test -- --ignored
```

### Multi-Instance Testing

```bash
# Terminal 1
BRAINSTORM_LISTEN=0.0.0.0:5000 cargo run

# Terminal 2
BRAINSTORM_LISTEN=0.0.0.0:5001 cargo run

# Terminal 3: Test synchronization
curl http://localhost:5000/readyz | jq
curl http://localhost:5001/readyz | jq
```

## Performance

### Benchmarks

**Один инстанс:**
- 1,000 concurrent players
- ~50ms average latency
- ~100 req/sec

**Три инстанса:**
- 3,000 concurrent players
- ~60ms average latency (Redis overhead)
- ~300 req/sec

### Bottlenecks

1. **Redis latency**: ~1-2ms per operation
2. **Pub/Sub overhead**: ~1-5ms per message
3. **Serialization**: ~0.5-1ms per message

### Optimizations

- Connection pooling для Redis
- Batch operations где возможно
- Compression для больших payloads
- Local cache для часто используемых данных

## Production Checklist

- [ ] Redis Sentinel или Cluster настроен
- [ ] Load balancer конфигурирован для WebSocket
- [ ] Health checks настроены
- [ ] Monitoring и alerting настроены
- [ ] Backup стратегия для Redis
- [ ] Disaster recovery план
- [ ] Load testing проведён
- [ ] Security audit завершён

## Troubleshooting

### Проблема: Инстансы не синхронизируются

**Решение:**
1. Проверьте подключение к Redis: `redis-cli ping`
2. Проверьте логи на наличие ошибок Pub/Sub
3. Убедитесь, что все инстансы используют одинаковый `redis_url`

### Проблема: Высокая latency

**Решение:**
1. Проверьте network latency до Redis
2. Увеличьте `maxmemory` в Redis
3. Используйте Redis Cluster для шардирования
4. Добавьте больше инстансов

### Проблема: Redis memory exhaustion

**Решение:**
1. Уменьшите TTL для кэша
2. Настройте `maxmemory-policy allkeys-lru`
3. Добавьте monitoring для memory usage
4. Увеличьте `maxmemory`

## Следующие шаги

**Epic 2: AI Pipeline Enhancement**
- RAG (Retrieval-Augmented Generation)
- Multimedia questions (images, audio)
- Dynamic difficulty adjustment

**Epic 3: Engagement & Retention**
- Tournament bracket system
- Ghost mode (AI opponents)
- Seasonal leaderboards

## Связанные документы

- [Redis Client Documentation](../../rust-server/src/infrastructure/README.md)
- [Pub/Sub Documentation](../../rust-server/src/infrastructure/PUBSUB.md)
- [Distributed Cache Documentation](../../rust-server/src/services/ai/DISTRIBUTED_CACHE.md)
- [Deployment Guide](../operations/deployment.md)
- [Monitoring Guide](../operations/monitoring.md)
