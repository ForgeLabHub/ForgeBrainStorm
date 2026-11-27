# Мобильная оптимизация

ForgeBrainStorm оптимизирован для работы на всех мобильных устройствах с минимальной задержкой и максимальным комфортом.

## Ключевые оптимизации

### 1. Устранение задержки клика

```css
a, button {
    touch-action: manipulation;
}
```

Убирает 300ms задержку на мобильных устройствах.

### 2. Отключение подсветки при тапе

```css
* {
    -webkit-tap-highlight-color: transparent;
    -webkit-touch-callout: none;
}
```

Убирает стандартную подсветку элементов при нажатии.

### 3. Плавная прокрутка на iOS

```css
html {
    -webkit-overflow-scrolling: touch;
}
```

Включает аппаратное ускорение прокрутки на iOS.

### 4. Оптимизация анимаций

#### Отключение тяжёлых эффектов на мобильных

```css
@media (max-width: 640px) {
    .glass-card:hover {
        transform: none;
    }
}
```

Убирает hover-эффекты с transform на мобильных для экономии батареи.

#### Упрощение backdrop-filter

```css
@media (max-width: 640px) {
    .glass-card {
        backdrop-filter: blur(8px);
        -webkit-backdrop-filter: blur(8px);
    }
}
```

Снижает blur с 12px до 8px на мобильных для лучшей производительности.

### 5. Адаптивная типографика

```css
@media (max-width: 640px) {
    html {
        font-size: 14px;
    }
}
```

Уменьшает базовый размер шрифта на мобильных для лучшего отображения.

### 6. Поддержка reduced motion

```css
@media (prefers-reduced-motion: reduce) {
    *,
    *::before,
    *::after {
        animation-duration: 0.01ms !important;
        animation-iteration-count: 1 !important;
        transition-duration: 0.01ms !important;
    }
}
```

Уважает настройки пользователя и отключает анимации для людей с вестибулярными расстройствами.

## Адаптивный дизайн

### Breakpoints

```css
/* Mobile first */
sm: 640px   /* Большие телефоны */
md: 768px   /* Планшеты */
lg: 1024px  /* Маленькие ноутбуки */
xl: 1280px  /* Десктопы */
2xl: 1536px /* Большие экраны */
```

### Примеры адаптации

#### Кнопки

```svelte
<button class="w-full sm:w-auto px-6 py-3">
    Создать комнату
</button>
```

На мобильных кнопки занимают всю ширину, на больших экранах - обычный размер по содержимому.

#### Отступы

```svelte
<div class="p-4 sm:p-6 md:p-8">
    Контент
</div>
```

Уменьшенные отступы на мобильных, увеличенные на больших экранах.

#### Сетки

```svelte
<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
    Карточки
</div>
```

Одна колонка на мобильных, две на планшетах, три на десктопах.

## Touch-friendly интерфейс

### Минимальные размеры элементов

Все интерактивные элементы имеют минимальный размер 44x44px (рекомендация Apple HIG).

```svelte
<button class="min-h-[44px] min-w-[44px]">
    Кнопка
</button>
```

### Достаточные отступы между элементами

```svelte
<div class="flex gap-3 sm:gap-4">
    Элементы
</div>
```

Минимум 12px между кликабельными элементами на мобильных.

### Swipe gestures

Поддержка swipe для навигации:

```typescript
let touchStartX = 0;
let touchEndX = 0;

function handleTouchStart(e: TouchEvent) {
    touchStartX = e.touches[0].clientX;
}

function handleTouchEnd(e: TouchEvent) {
    touchEndX = e.changedTouches[0].clientX;
    const diff = touchEndX - touchStartX;
    if (Math.abs(diff) > 50) {
        if (diff > 0) {
            // Swipe right
        } else {
            // Swipe left
        }
    }
}
```

## Производительность

### Lazy loading

Изображения и тяжёлые компоненты загружаются только когда видны:

```svelte
{#if isVisible}
    <HeavyComponent />
{/if}
```

### Виртуализация списков

Для длинных списков используется виртуализация:

```svelte
<VirtualList items={rooms} itemHeight={80}>
    {#snippet children(room)}
        <RoomCard {room} />
    {/snippet}
</VirtualList>
```

### Оптимизация WebSocket

- Автоматическое переподключение
- Heartbeat для проверки соединения
- Буферизация сообщений при потере связи
- Дебаунсинг частых обновлений

### Кэширование

```typescript
const cache = new Map();

async function getCachedData(key: string, fetcher: () => Promise<any>) {
    if (cache.has(key)) {
        return cache.get(key);
    }
    const data = await fetcher();
    cache.set(key, data);
    return data;
}
```

## Battery optimization

### Отключение анимаций при низком заряде

```typescript
if ('getBattery' in navigator) {
    const battery = await navigator.getBattery();
    if (battery.level < 0.2 && !battery.charging) {
        document.documentElement.classList.add('low-battery');
    }
}
```

```css
.low-battery * {
    animation: none !important;
    transition: none !important;
}
```

### Pause on visibility change

```typescript
document.addEventListener('visibilitychange', () => {
    if (document.hidden) {
        // Pause animations and timers
    } else {
        // Resume
    }
});
```

## Тестирование

### Chrome DevTools

1. Device Mode (Ctrl+Shift+M)
2. Network throttling (Slow 3G)
3. CPU throttling (4x slowdown)
4. Lighthouse audit

### Реальные устройства

Тестирование на:
- iPhone (Safari)
- Android (Chrome)
- iPad (Safari)
- Android tablets

### Метрики

Целевые показатели:
- First Contentful Paint: < 1.5s
- Largest Contentful Paint: < 2.5s
- Time to Interactive: < 3.5s
- Cumulative Layout Shift: < 0.1
- Frames per second: 60fps

## Чеклист мобильной оптимизации

- [ ] Все кнопки минимум 44x44px
- [ ] Текст читаем без зума
- [ ] Нет горизонтальной прокрутки
- [ ] Формы удобно заполнять
- [ ] Изображения оптимизированы
- [ ] Анимации плавные (60fps)
- [ ] Touch targets не перекрываются
- [ ] Поддержка landscape orientation
- [ ] Работает offline (PWA)
- [ ] Нет 300ms delay
- [ ] Reduced motion respected
- [ ] Battery-friendly

## Дополнительные ресурсы

- [Web Vitals](https://web.dev/vitals/)
- [Mobile First Design](https://developers.google.com/web/fundamentals/design-and-ux/responsive/)
- [Touch Targets](https://developers.google.com/web/fundamentals/design-and-ux/input/touch)
- [Performance Optimization](https://web.dev/fast/)
