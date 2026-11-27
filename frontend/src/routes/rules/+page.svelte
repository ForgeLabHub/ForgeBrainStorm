<script lang="ts">
  import Icon, { type IconName } from '$lib/Icon.svelte';

  interface ModeRule {
    code: string;
    title: string;
    icon: IconName;
    clock: string;
    lives: string;
    points: string;
    note: string;
    ranked: boolean;
  }

  const modes: ModeRule[] = [
    {
      code: 'classic',
      title: 'Классика',
      icon: 'target',
      clock: 'из настроек комнаты',
      lives: 'нет',
      points: '100 за верный ответ',
      note: 'Базовый режим: все отвечают одновременно, ошибка ничего не стоит.',
      ranked: true
    },
    {
      code: 'ffa',
      title: 'Каждый за себя',
      icon: 'swords',
      clock: 'из настроек комнаты',
      lives: 'нет',
      points: '40…100, зависит от скорости',
      note: 'Очки = 40 + 60 × доля оставшегося времени, шаг 5.',
      ranked: true
    },
    {
      code: 'lives',
      title: 'На выживание',
      icon: 'heart',
      clock: 'из настроек комнаты',
      lives: '3',
      points: '100 за верный ответ',
      note: 'Ошибка отнимает жизнь. На нуле — выбывание, ответы больше не принимаются.',
      ranked: true
    },
    {
      code: 'blitz',
      title: 'Блиц',
      icon: 'bolt',
      clock: '10 секунд всегда',
      lives: 'нет',
      points: '60 → 120, растут с серией',
      note: 'Множитель +25 % за каждый верный ответ подряд, потолок ×2. Ошибка сбрасывает серию.',
      ranked: true
    },
    {
      code: 'team',
      title: 'Команды',
      icon: 'users',
      clock: 'из настроек комнаты',
      lives: 'нет',
      points: 'личные очки, командный зачёт',
      note: 'Игроки делятся на «Синих» и «Красных» по порядку входа; побеждает команда с большей суммой.',
      ranked: true
    },
    {
      code: 'coop',
      title: 'Кооператив',
      icon: 'users',
      clock: 'из настроек комнаты',
      lives: 'нет',
      points: '50, +10 за каждый ответ в серии',
      note: 'Одна общая копилка: все играют за одну сторону, итог — сумма очков.',
      ranked: true
    },
    {
      code: 'sandbox',
      title: 'Тренировка',
      icon: 'seedling',
      clock: 'из настроек комнаты',
      lives: 'нет',
      points: '50 за верный ответ',
      note: 'Партия не попадает в рейтинг, историю и XP — можно пробовать без риска.',
      ranked: false
    },
    {
      code: 'svoyaigra',
      title: 'Своя Игра',
      icon: 'spark',
      clock: '15 секунд всегда',
      lives: 'нет',
      points: '60…100, зависит от скорости',
      note: 'Скоростная вариация: очки = 60 + 40 × доля оставшегося времени, шаг 10.',
      ranked: true
    },
    {
      code: 'sudden',
      title: 'Внезапная смерть',
      icon: 'flame',
      clock: '20 секунд всегда',
      lives: '1',
      points: '120…200, зависит от скорости',
      note: 'Одна ошибка — и выбыл. Верный ответ платит по удвоенной шкале.',
      ranked: true
    },
    {
      code: 'marathon',
      title: 'Марафон',
      icon: 'hourglass',
      clock: '20 секунд всегда',
      lives: 'нет',
      points: '50 → 100, растут с серией',
      note: 'Ровно 30 вопросов. Ошибка сбрасывает серию, но не выбивает из игры.',
      ranked: true
    }
  ];

  const powerUps = [
    { name: 'Двойные очки', text: 'Удваивает очки за следующий верный ответ.' },
    { name: 'Заморозка', text: 'Добавляет 10 секунд к текущему вопросу.' },
    { name: 'Убрать два', text: 'Скрывает два неверных варианта — видите только вы.' }
  ];

  const cheats = [
    { name: 'Подсмотреть', uses: '3 раза', text: 'Показывает верный вариант только вам.' },
    { name: 'Заморозка', uses: '2 раза', text: 'Плюс 10 секунд всем игрокам в комнате.' },
    { name: 'Кража', uses: '1 раз', text: 'Забирает 50 очков у текущего лидера.' },
    { name: 'Двойные очки', uses: '2 раза', text: 'Удваивает очки за следующий верный ответ.' }
  ];
</script>

<svelte:head>
  <title>Правила — BrainStorm</title>
</svelte:head>

<div class="mx-auto max-w-5xl px-4 py-8 sm:px-6">
  <header class="mb-8">
    <p class="text-xs font-semibold uppercase tracking-widest text-slate-500">Как играть</p>
    <h1 class="mt-1 text-3xl font-black tracking-tight text-white">Правила и режимы</h1>
    <p class="mt-2 max-w-2xl text-slate-400">
      Десять режимов, три усиления в руке у каждого игрока и отдельный набор способностей у роли
      «читер». Очки считает сервер — по времени ответа, серии и правилам режима.
    </p>
  </header>

  <section class="mb-10">
    <h2 class="mb-3 text-lg font-bold text-white">Общие правила</h2>
    <ul class="grid gap-2 text-sm text-slate-300 sm:grid-cols-2">
      <li class="rounded-xl border border-white/5 bg-white/[0.02] px-4 py-3">
        Хост создаёт комнату: тема, сложность, режим, 5–30 вопросов, 2–6 вариантов, 10–120 секунд
        на вопрос.
      </li>
      <li class="rounded-xl border border-white/5 bg-white/[0.02] px-4 py-3">
        Вопросы генерирует ИИ; если провайдер недоступен, вопросы берутся из офлайн-банка.
      </li>
      <li class="rounded-xl border border-white/5 bg-white/[0.02] px-4 py-3">
        Все видят один и тот же вопрос одновременно. Ответ принимается один раз и не меняется.
      </li>
      <li class="rounded-xl border border-white/5 bg-white/[0.02] px-4 py-3">
        После вопроса — пауза 5 секунд с правильным ответом и пояснением.
      </li>
    </ul>
  </section>

  <section class="mb-10">
    <h2 class="mb-3 text-lg font-bold text-white">Режимы</h2>
    <div class="grid gap-3 md:grid-cols-2">
      {#each modes as mode (mode.code)}
        <article class="rounded-2xl border border-white/5 bg-white/[0.02] p-5">
          <div class="mb-2 flex items-center gap-3">
            <span
              class="flex h-9 w-9 items-center justify-center rounded-xl bg-purple-500/10 text-purple-300"
            >
              <Icon name={mode.icon} />
            </span>
            <h3 class="font-bold text-white">{mode.title}</h3>
            {#if !mode.ranked}
              <span
                class="rounded-md border border-slate-500/30 bg-slate-500/10 px-2 py-0.5 text-[10px] font-bold uppercase tracking-wider text-slate-400"
              >
                без рейтинга
              </span>
            {/if}
          </div>
          <p class="mb-3 text-sm text-slate-400">{mode.note}</p>
          <dl class="grid grid-cols-3 gap-2 text-xs">
            <div>
              <dt class="text-slate-500">Таймер</dt>
              <dd class="text-slate-200">{mode.clock}</dd>
            </div>
            <div>
              <dt class="text-slate-500">Жизни</dt>
              <dd class="text-slate-200">{mode.lives}</dd>
            </div>
            <div>
              <dt class="text-slate-500">Очки</dt>
              <dd class="text-slate-200">{mode.points}</dd>
            </div>
          </dl>
        </article>
      {/each}
    </div>
  </section>

  <div class="grid gap-6 md:grid-cols-2">
    <section>
      <h2 class="mb-3 text-lg font-bold text-white">Усиления в игре</h2>
      <p class="mb-3 text-sm text-slate-400">Каждый игрок получает эти три карты на старте партии.</p>
      <ul class="space-y-2">
        {#each powerUps as power (power.name)}
          <li class="rounded-xl border border-white/5 bg-white/[0.02] px-4 py-3">
            <div class="font-semibold text-white">{power.name}</div>
            <div class="text-sm text-slate-400">{power.text}</div>
          </li>
        {/each}
      </ul>
    </section>

    <section>
      <h2 class="mb-3 text-lg font-bold text-white">Способности читтера</h2>
      <p class="mb-3 text-sm text-slate-400">
        Открываются аккаунту с ролью «читер». Бюджет на партию выдаёт сервер, поэтому клиент не
        может дать себе лишних использований.
      </p>
      <ul class="space-y-2">
        {#each cheats as cheat (cheat.name)}
          <li class="rounded-xl border border-orange-500/15 bg-orange-500/[0.04] px-4 py-3">
            <div class="flex items-center justify-between gap-3">
              <span class="font-semibold text-white">{cheat.name}</span>
              <span class="text-xs font-semibold uppercase tracking-wider text-orange-300">
                {cheat.uses}
              </span>
            </div>
            <div class="text-sm text-slate-400">{cheat.text}</div>
          </li>
        {/each}
      </ul>
    </section>
  </div>

  <div class="mt-10 flex flex-wrap gap-3">
    <a
      href="/room/create"
      class="rounded-xl bg-gradient-to-r from-blue-500 to-purple-600 px-5 py-2.5 font-semibold text-white shadow-lg transition hover:shadow-blue-500/20"
    >
      Создать комнату
    </a>
    <a
      href="/settings"
      class="rounded-xl border border-white/10 bg-white/5 px-5 py-2.5 font-semibold text-slate-200 transition hover:bg-white/10"
    >
      Настройки
    </a>
  </div>
</div>
