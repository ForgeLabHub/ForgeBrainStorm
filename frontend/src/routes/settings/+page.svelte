<script lang="ts">
  import { api, type User } from '$lib/api';
  import { DIFFICULTIES, MODES, initial } from '$lib/ui';
  import {
    DEFAULT_ROOM,
    clearRoomDefaults,
    loadRoomDefaults,
    saveRoomDefaults,
    type RoomDefaults
  } from '$lib/settings';
  import { onMount } from 'svelte';

  let user: User | null = $state(null);
  let loading = $state(true);
  let defaults = $state<RoomDefaults>({ ...DEFAULT_ROOM });
  let saved = $state(false);

  onMount(async () => {
    defaults = loadRoomDefaults();
    try {
      const me = await api.me();
      user = me.user ?? null;
    } catch {
      user = null;
    } finally {
      loading = false;
    }
  });

  function roleName(role?: string): string {
    if (role === 'admin') return 'Администратор';
    if (role === 'cheater') return 'Читер';
    return 'Игрок';
  }

  function save() {
    saveRoomDefaults(defaults);
    saved = true;
    setTimeout(() => (saved = false), 2500);
  }

  function reset() {
    clearRoomDefaults();
    defaults = { ...DEFAULT_ROOM };
    saved = true;
    setTimeout(() => (saved = false), 2500);
  }

  async function logout() {
    try {
      await api.logout();
    } finally {
      window.location.href = '/';
    }
  }
</script>

<svelte:head>
  <title>Настройки — BrainStorm</title>
</svelte:head>

<div class="mx-auto max-w-3xl px-4 py-8 sm:px-6">
  <header class="mb-6">
    <p class="text-xs font-semibold uppercase tracking-widest text-slate-500">Аккаунт</p>
    <h1 class="mt-1 text-3xl font-black tracking-tight text-white">Настройки</h1>
  </header>

  <section class="mb-6 rounded-2xl border border-white/5 bg-white/[0.02] p-5 backdrop-blur-sm sm:p-6">
    {#if loading}
      <div class="flex items-center gap-4">
        <div class="h-14 w-14 animate-pulse rounded-xl bg-white/5"></div>
        <div class="space-y-2">
          <div class="h-5 w-40 animate-pulse rounded bg-white/5"></div>
          <div class="h-4 w-24 animate-pulse rounded bg-white/5"></div>
        </div>
      </div>
    {:else if user}
      <div class="flex flex-wrap items-center gap-4">
        <div
          class="flex h-14 w-14 items-center justify-center rounded-xl bg-gradient-to-br from-blue-500 to-purple-600 text-xl font-bold text-white"
        >
          {initial(user.username)}
        </div>
        <div class="min-w-0">
          <div class="flex flex-wrap items-center gap-2">
            <span class="truncate text-lg font-bold text-white">{user.username}</span>
            <span
              class="rounded-md border border-purple-500/30 bg-purple-500/10 px-2 py-0.5 text-[10px] font-bold uppercase tracking-wider text-purple-300"
            >
              {roleName(user.role)}
            </span>
          </div>
          <p class="text-sm text-slate-400">
            Уровень {user.level} · {user.xp} XP · {user.coins} монет
          </p>
        </div>
        <div class="ml-auto flex gap-2">
          <a
            href="/profile"
            class="rounded-xl border border-white/10 bg-white/5 px-4 py-2 text-sm font-semibold text-white transition hover:bg-white/10"
          >
            Профиль
          </a>
          <button
            type="button"
            onclick={logout}
            class="rounded-xl border border-red-500/20 bg-red-500/10 px-4 py-2 text-sm font-semibold text-red-300 transition hover:bg-red-500/20"
          >
            Выйти
          </button>
        </div>
      </div>
    {:else}
      <div class="flex flex-wrap items-center justify-between gap-4">
        <p class="text-slate-400">Вы не вошли — настройки комнаты всё равно сохранятся в браузере.</p>
        <a
          href="/auth/login"
          class="rounded-xl bg-gradient-to-r from-blue-500 to-purple-600 px-4 py-2 text-sm font-semibold text-white"
        >
          Войти
        </a>
      </div>
    {/if}
  </section>

  <section class="rounded-2xl border border-white/5 bg-white/[0.02] p-5 backdrop-blur-sm sm:p-6">
    <h2 class="text-lg font-bold text-white">Параметры новой комнаты</h2>
    <p class="mb-5 mt-1 text-sm text-slate-400">
      Подставляются в форму создания комнаты, чтобы не выбирать одно и то же каждый раз.
    </p>

    <div class="grid gap-4 sm:grid-cols-2">
      <label class="block">
        <span class="mb-1.5 block text-xs font-semibold uppercase tracking-wider text-slate-500">Режим</span>
        <select
          class="w-full rounded-xl border border-white/10 bg-[#12132a] px-3 py-2.5 text-white outline-none focus:border-purple-500/50"
          bind:value={defaults.mode}
        >
          {#each Object.entries(MODES) as [value, mode] (value)}
            <option {value}>{mode.label}</option>
          {/each}
        </select>
      </label>

      <label class="block">
        <span class="mb-1.5 block text-xs font-semibold uppercase tracking-wider text-slate-500">Сложность</span>
        <select
          class="w-full rounded-xl border border-white/10 bg-[#12132a] px-3 py-2.5 text-white outline-none focus:border-purple-500/50"
          bind:value={defaults.difficulty}
        >
          {#each Object.entries(DIFFICULTIES) as [value, label] (value)}
            <option {value}>{label}</option>
          {/each}
        </select>
      </label>

      <label class="block">
        <span class="mb-1.5 block text-xs font-semibold uppercase tracking-wider text-slate-500">Вопросов</span>
        <input
          type="number"
          min="5"
          max="30"
          class="w-full rounded-xl border border-white/10 bg-[#12132a] px-3 py-2.5 text-white outline-none focus:border-purple-500/50"
          bind:value={defaults.questions}
        />
      </label>

      <label class="block">
        <span class="mb-1.5 block text-xs font-semibold uppercase tracking-wider text-slate-500">Вариантов ответа</span>
        <input
          type="number"
          min="2"
          max="6"
          class="w-full rounded-xl border border-white/10 bg-[#12132a] px-3 py-2.5 text-white outline-none focus:border-purple-500/50"
          bind:value={defaults.options}
        />
      </label>

      <label class="block sm:col-span-2">
        <span class="mb-1.5 block text-xs font-semibold uppercase tracking-wider text-slate-500">Секунд на вопрос</span>
        <input
          type="number"
          min="10"
          max="60"
          step="5"
          class="w-full rounded-xl border border-white/10 bg-[#12132a] px-3 py-2.5 text-white outline-none focus:border-purple-500/50"
          bind:value={defaults.timer}
        />
        <span class="mt-1.5 block text-xs text-slate-500">
          Блиц, «Своя игра», «Внезапная смерть» и «Марафон» задают таймер сами.
        </span>
      </label>
    </div>

    <div class="mt-5 flex flex-wrap items-center gap-3">
      <button
        type="button"
        onclick={save}
        class="rounded-xl bg-gradient-to-r from-blue-500 to-purple-600 px-5 py-2.5 font-semibold text-white shadow-lg transition hover:shadow-blue-500/20"
      >
        Сохранить
      </button>
      <button
        type="button"
        onclick={reset}
        class="rounded-xl border border-white/10 bg-white/5 px-5 py-2.5 font-semibold text-slate-200 transition hover:bg-white/10"
      >
        Сбросить
      </button>
      {#if saved}
        <span class="text-sm font-medium text-green-400">Сохранено</span>
      {/if}
    </div>
  </section>

  <section class="mt-6 grid gap-3 sm:grid-cols-2">
    <a
      href="/rules"
      class="rounded-2xl border border-white/5 bg-white/[0.02] p-5 transition hover:border-purple-500/30"
    >
      <div class="font-semibold text-white">Правила и режимы</div>
      <div class="text-sm text-slate-400">Как считаются очки, жизни и бонусы</div>
    </a>
    <div class="rounded-2xl border border-white/5 bg-white/[0.02] p-5">
      <div class="font-semibold text-white">Генератор вопросов</div>
      <div class="text-sm text-slate-400">
        Провайдера ИИ выбирает администратор в панели управления; без ключа вопросы берутся из
        офлайн-банка.
      </div>
    </div>
  </section>
</div>
