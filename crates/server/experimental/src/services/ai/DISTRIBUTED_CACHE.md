# Distributed AI Cache

Распределённый кэш для AI-генерации вопросов и подсказок с использованием Redis.

## Обзор

`DistributedAiCache` предоставляет распределённое кэширование для AI-ответов, позволяя нескольким инстансам сервера совместно использовать один кэш. Это снижает нагрузку на AI API и уменьшает costs.

## Архитектура

```
Instance A                  Redis                  Instance B
    |                          |                         |
    |--> cache.set(key, data) -|-> Store in Redis        |
    |                          |                         |
    |                          |<- cache.get(key) -------|
    |                          |    (cache hit!)         |
    |                          |                         |
    v                          v                         v
 Local Fallback           Redis Cache              Local Fallback
 (DashMap)                (Shared)                 (DashMap)
```

## Ключевые особенности

- **Redis-backed**: Хранит данные в Redis для доступа между инстансами
- **Graceful degradation**: Автоматически переключается на локальный кэш, если Redis недоступен
- **TTL поддержка**: Вопросы кэшируются на 1 час, подсказки на 30 минут
- **Сериализация**: JSON сериализация для хранения сложных структур данных
- **Namespace**: Поддержка префиксов для изоляции данных

## Использование

### Инициализация

```rust
use brainstorm::infrastructure::RedisClient;
use brainstorm::services::ai::distributed_cache::DistributedAiCache;
use std::sync::Arc;

// Создаём Redis клиент
let redis = Arc::new(
    RedisClient::new("redis://127.0.0.1:6379")
        .await
        .expect("Failed to connect to Redis")
);

// Создаём распределённый кэш с префиксом "brainstorm"
let cache = Arc::new(DistributedAiCache::new(redis, "brainstorm"));
```

### Кэширование вопросов

```rust
use brainstorm::services::ai::GenerationResponse;

let response = GenerationResponse {
    questions: vec![/* ... */],
    total_generated: 10,
    cached: false,
};

// Сохраняем в кэш (ключ: topic:difficulty:count:options)
cache.set("math:easy:10:4".to_string(), response).await;

// Получаем из кэша
if let Some(cached) = cache.get("math:easy:10:4").await {
    println!("Cache hit! Got {} questions", cached.questions.len());
}
```

### Кэширование подсказок

```rust
// Сохраняем подсказку
cache.set_hint("hint:2plus2".to_string(), "Think about basic arithmetic".to_string()).await;

// Получаем подсказку
if let Some(hint) = cache.get_hint("hint:2plus2").await {
    println!("Hint: {}", hint);
}
```

### Очистка кэша

```rust
// Очищает локальный кэш (Redis ключи истекают по TTL)
cache.clear().await?;
```

### Статистика

```rust
let stats = cache.stats().await;
println!("Questions cached: {}", stats.questions_cached);
println!("Hints cached: {}", stats.hints_cached);
```

## TTL и expiration

- **Вопросы**: 1 час (3600 секунд)
- **Подсказки**: 30 минут (1800 секунд)

Redis автоматически удаляет истёкшие ключи. Локальный кэш также имеет механизм очистки.

## Graceful degradation

Если Redis недоступен:
1. `cache.get()` возвращает `None` (не блокирует работу)
2. `cache.set()` сохраняет только в локальный кэш
3. Локальный кэш продолжает работать нормально

Это обеспечивает высокую доступность даже при проблемах с Redis.

## Ключи Redis

Формат ключей:
- Вопросы: `{prefix}:question:{key}`
- Подсказки: `{prefix}:hint:{key}`

Пример:
- `brainstorm:question:math:easy:10:4`
- `brainstorm:hint:what_is_2_plus_2`

## Тестирование

### Unit тесты

```bash
cargo test --lib services::ai::distributed_cache
```

### Integration тесты

```bash
# Запустите Redis
docker-compose up -d redis

# Запустите тесты
cargo test --test distributed_cache_integration_test -- --ignored
```

## Мониторинг

Метрики для мониторинга:
- Cache hit rate (через `/api/ai/cache/stats`)
- Redis connection status (через `/readyz`)
- Latency для `get`/`set` операций

## Production considerations

### Redis конфигурация

Рекомендуемые настройки Redis:
```conf
maxmemory 256mb
maxmemory-policy allkeys-lru
```

### Масштабирование

Для production используйте:
- **Redis Sentinel** для high availability
- **Redis Cluster** для горизонтального масштабирования
- **Redis replication** для отказоустойчивости

### Безопасность

- Используйте Redis аутентификацию в production
- Настройте TLS для Redis connections
- Ограничьте доступ к Redis через firewall

## Performance

### Benchmarks

Типичная производительность:
- `get` operation: ~1-2ms (Redis) / <0.1ms (local)
- `set` operation: ~1-3ms (Redis) / <0.1ms (local)
- Serialization: ~0.5-1ms

### Оптимизации

1. **Batch operations**: Группируйте несколько операций
2. **Connection pooling**: Используйте connection pool для Redis
3. **Compression**: Сжимайте большие payloads (опционально)

## Troubleshooting

### Кэш не работает

1. Проверьте подключение к Redis:
   ```bash
   redis-cli ping
   ```

2. Проверьте логи приложения на наличие ошибок Redis

3. Убедитесь, что Redis запущен:
   ```bash
   docker-compose ps
   ```

### Высокая latency

1. Проверьте network latency до Redis
2. Увеличьте `maxmemory` в Redis
3. Рассмотрите использование Redis Cluster

### Memory issues

1. Уменьшите TTL для кэша
2. Настройте `maxmemory-policy` в Redis
3. Добавьте monitoring для memory usage

## Related

- [Redis Client Documentation](../../infrastructure/README.md)
- [AI Router Documentation(../../../../src/services/ai/router.rs)
- [Epic 1: Horizontal Scaling(../../../../docs/experimental/EPIC_1_HORIZONTAL_SCALING.md)
