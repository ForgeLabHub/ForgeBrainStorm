# Экспериментальная документация

Описания функций, которые пришли из веток `arena/*` и пока **не реализованы** в
рабочем коде. Документы сохранены как проектные заметки: то, что здесь написано,
нельзя считать описанием текущего поведения.

| Документ | Тема | Статус |
|---|---|---|
| [features/retrieval_augmented_generation.md](features/retrieval_augmented_generation.md) | RAG для генерации вопросов | прототип в `rust-server/experimental/src/services/ai/rag.rs` |
| [features/dynamic_difficulty_adjustment.md](features/dynamic_difficulty_adjustment.md) | DDA | прототип, не подключён |
| [features/game_modes.md](features/game_modes.md) | coop / svoyaigra / adaptive | заменено [../game-modes.md](../game-modes.md) |
| [features/mobile_optimization.md](features/mobile_optimization.md) | мобильная адаптация | часть вошла в `frontend/` |
| [performance.md](performance.md) | производительность | заметки, замеров нет |
| [EPIC_1_HORIZONTAL_SCALING.md](EPIC_1_HORIZONTAL_SCALING.md) | горизонтальное масштабирование | план |
| [WEBSOCKET_GAMEPLAY_REPORT.md](WEBSOCKET_GAMEPLAY_REPORT.md) | отчёт о игровом цикле | исторический |

Актуальные режимы игры — [../game-modes.md](../game-modes.md), устройство
бэкенда — [../rust-backend.md](../rust-backend.md).
