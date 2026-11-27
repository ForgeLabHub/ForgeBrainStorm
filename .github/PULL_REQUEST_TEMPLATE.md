<!-- Keep PRs to one topic: refactor, feature, fix. Do not merge your own PR. -->

## Что
<одно предложение: что меняется для читателя кода или пользователя>

## Зачем
<ссылка на issue / мотивация; что сломано или чего не хватает сейчас>

## Как
- <ключевые решения и альтернативы, которые отклонены>
- <новые зависимости: зачем, размер, альтернативы, лицензия — или «нет»>

## Тестирование
```bash
cargo xtask ci        # fmt --check + clippy -D warnings + cargo test --workspace
```
<плюс то, что проверено руками: браузер, docker, e2e-скрипт>

## Риски и откат
<feature flag? breaking change для API/протокола? как откатить: revert коммита / выключить флаг>

## Чек-лист
- [ ] тесты (новая логика покрыта)
- [ ] доки обновлены (`docs/`, README)
- [ ] `CHANGELOG.md` → `Unreleased`
- [ ] `cargo xtask ci` зелёный
- [ ] нет новых `unwrap()`/`expect()` в prod-коде
