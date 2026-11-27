# Производительность

ForgeBrainStorm оптимизирован для высокой производительности и низкой задержки на всех этапах работы.

## Архитектурные решения

### Rust Backend

**Почему Rust:**
- Zero-cost abstractions
- Отсутствие garbage collector
- Предсказуемая производительность
- Безопасность памяти без runtime overhead

**Ключевые оптимизации:**

```rust
// Async/await для неблокирующих операций
async fn generate_questions(settings: &RoomSettings) -> Result<Vec<Question>> {
    let response = client.post(&url)
        .timeout(Duration::from_secs(30))
        .send()
        .await?;
    // ...
}
```

### WebSocket оптимизации

1. **Минимальный размер сообщений**
```rust
#[derive(Serialize)]
struct GameUpdate {
    t: &'static str,  // type - короткое имя поля
    q: Option<Question>,
    p: Vec<PlayerState>,
}
```

2. **Бинарный формат (опционально)**
   - Поддержка MessagePack для уменьшения размера
   - Снижение нагрузки на парсинг JSON

3. **Компрессия**
   - Per-message deflate для WebSocket
   - Снижение трафика на 60-80%

## Задержки

### Целевые показатели

| Операция | Цель | Реальность |
|----------|------|----------|
| Создание комнаты | < 100ms | ~50ms |
| Присоединение | < 50ms | ~30ms |
| Генерация вопроса | < 3s | ~2s |
| Обновление состояния | < 20ms | ~10ms |
| Подсчёт очков | < 5ms | ~2ms |

### Измерение задержек

```rust
use std::time::Instant;

let start = Instant::now();
let result = await process_game_update();
let duration = start.elapsed();

metrics.histogram("game_update_duration", duration);
```

## Масштабируемость

### Горизонтальное масштабирование

**Текущая архитектура:**
- In-memory room storage
- WebSocket connection per room
- Single-node deployment

**Планы масштабирования:**

1. **Redis для room state**
```rust
struct RoomStore {
    redis: RedisClient,
    local_cache: HashMap<String, Room>,
}
```

2. **Distributed WebSocket**
   - Sticky sessions через load balancer
   - Redis pub/sub для межсерверного взаимодействия

3. **Sharding**
   - Rooms по shard_id
   - Каждый shard на отдельном node

### Ограничения

**Текущие лимиты:**
- ~1000 одновременных комнат на node
- ~10,000 подключений WebSocket
- ~100 RPS для генерации вопросов

**Узкие места:**
- AI API calls (внешний сервис)
- Memory per room (~1MB)
- CPU для валидации ответов

## Оптимизации

### 1. Кэширование

**AI responses:**
```rust
struct QuestionCache {
    cache: LruCache<String, Vec<Question>>,
    ttl: Duration,
}
```

**Room metadata:**
```rust
// Кэширование списка комнат
let rooms = cache.get_or_set("rooms_list", || {
    room_manager.list_rooms()
}, Duration::from_secs(5));
```

### 2. Batch operations

**Групповые обновления:**
```rust
// Вместо отправки каждого обновления отдельно
let updates = vec![update1, update2, update3];
websocket.send_batch(updates).await?;
```

### 3. Lazy evaluation

**Отложенная генерация:**
```rust
// Генерируем вопросы только когда игра начинается
if game_state.phase == GamePhase::Playing {
    questions = generate_questions(settings).await?;
}
```

### 4. Connection pooling

**HTTP клиент:**
```rust
let client = reqwest::Client::builder()
    .pool_max_idle_per_host(10)
    .pool_idle_timeout(Duration::from_secs(30))
    .build()?;
```

## Мониторинг

### Метрики

**Prometheus метрики:**
```rust
// Количество активных комнат
metrics.gauge("active_rooms", room_manager.count());

// Время генерации вопроса
metrics.histogram("question_generation_time", duration);

// Количество WebSocket сообщений
metrics.counter("websocket_messages_sent", 1);
```

### Logging

**Structured logging:**
```rust
tracing::info!(
    room_code = %room.code,
    player_count = room.players.len(),
    "Game started"
);
```

**Уровни:**
- ERROR: Критические ошибки
- WARN: Проблемы, но работа продолжается
- INFO: Важные события
- DEBUG: Детальная отладка
- TRACE: Каждое действие

## Профилирование

### CPU profiling

```bash
# Generate flame graph
cargo flamegraph --bin forgebrainstorm

# Analyze hot paths
perf record -g --call-graph dwarf ./target/release/forgebrainstorm
perf report
```

### Memory profiling

```bash
# Track allocations
cargo valgrind --tool=massif ./target/release/forgebrainstorm

# Analyze heap
ms_print massif.out.<pid>
```

### WebSocket profiling

```bash
# Monitor connections
ss -tnp | grep :8080 | wc -l

# Track message rate
tcpdump -i lo -n -e 'port 8080' -c 1000
```

## Тестирование производительности

### Load testing

```bash
# Используем k6
k6 run load-test.js

# Или wrk
wrk -t12 -c400 -d30s http://localhost:8080/api/rooms
```

### Benchmark suite

```rust
#[bench]
fn bench_question_generation(b: &mut Bencher) {
    b.iter(|| {
        generate_questions(&settings)
    });
}

// Запуск
cargo bench
```

## Оптимизации для мобильных

### Frontend

1. **Code splitting**
```svelte
// Lazy load тяжёлых компонентов
const HeavyComponent = await import('./HeavyComponent.svelte');
```

2. **Virtual scrolling**
```svelte
<VirtualList items={rooms} itemHeight={80}>
    {#snippet children(room)}
        <RoomCard {room} />
    {/snippet}
</VirtualList>
```

3. **Debouncing**
```typescript
const debouncedSearch = debounce((query: string) => {
    searchRooms(query);
}, 300);
```

### Backend

1. **Минимальные payloads**
```rust
// Только необходимые поля
struct MinimalRoom {
    code: String,
    mode: String,
    players: u8,
}
```

2. **Compressed responses**
```rust
// gzip compression
.layer(CompressionLayer::new())
```

3. **Efficient serialization**
```rust
// Использовать serde с оптимизациями
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Room {
    // ...
}
```

## Best practices

### Do's

✅ Использовать connection pooling
✅ Кэшировать expensive operations
✅ Батчить database queries
✅ Использовать async/await правильно
✅ Профилировать перед оптимизацией
✅ Мониторить production метрики

### Don'ts

❌ Блокировать async контекст
❌ Создавать новые клиенты на каждый запрос
❌ Игнорировать error handling
❌ Оптимизировать без измерений
❌ Использовать unwrap() в production
❌ Забывать про timeout

## Дополнительные ресурсы

- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Tokio Performance Guide](https://tokio.rs/tokio/topics/profiling)
- [WebSocket Best Practices](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket)
- [Mobile Web Performance](https://web.dev/fast/)
