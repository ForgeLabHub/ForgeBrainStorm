# Экспериментальные модули (не в сборке)

Здесь лежит код из веток `arena/*`, который был влит в `main` но **не компилировался
и не был подключён** к серверу. Он сохранён, чтобы работа не пропала, и вынесен из
`src/`, поэтому `cargo check`/`cargo test` его не видят: объявления `mod` на эти
файлы нет.

| Файл | Что это | Почему не в сборке |
|---|---|---|
| `src/infrastructure/` | Redis pub/sub, клиент Redis | требует `redis` в проде, нет тестов на живой шине |
| `src/routes/rag.rs` | HTTP-обёртка над RAG | модуль `services::ai::rag` не компилируется |
| `src/services/ai/rag.rs` | retrieval-augmented generation | незаконченная реализация, падает на типах |
| `src/services/ai/dynamic_difficulty.rs` | подстройка сложности (DDA) | нет источника статистики по игроку |
| `src/services/ai/distributed_cache.rs` | общий кэш вопросов в Redis | зависит от `infrastructure`, не проверен |
| `src/services/ai/mock.rs` | мок-провайдер для тестов | не подключён к `router` |
| `src/ws/game_loop.rs`, `room_manager.rs`, `protocol.rs` | альтернативный игровой цикл | дублирует рабочий `ws/mod.rs`, 68 ошибок компиляции |
| `tests/*.rs` | интеграционные тесты к перечисленному | собираются только вместе с ним |

## Как вернуть модуль в строй

1. Взять файл из `experimental/`, положить в `src/` и объявить `mod` в родителе.
2. `cargo check --all-targets` до нуля ошибок, тесты — рядом с кодом.
3. Описать поведение в `docs/` и добавить в `docs/verification/matrix.md`, чем оно
   проверено.
4. Только после этого переносить соответствующий файл из `experimental/tests/`.

Правило простое: код, который не собирается, не живёт в `src/`. Полная история —
в git (`git log -- rust-server/src/ws/room_manager.rs`).
