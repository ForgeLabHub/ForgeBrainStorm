<script lang="ts">
  import '../app.css';
  import { api } from '$lib/api';
  import { onMount } from 'svelte';
  import { afterNavigate } from '$app/navigation';

  let { children } = $props();

  let isLoggedIn = $state(false);
  let username = $state('');
  let userLevel = $state(1);
  let userXp = $state(0);
  let userCoins = $state(0);
  let mobileMenuOpen = $state(false);

  async function loadUser() {
    try {
      const response = await api.me();
      if (response.user) {
        isLoggedIn = true;
        username = response.user.username;
        userLevel = response.user.level || 1;
        userXp = response.user.xp || 0;
        userCoins = response.user.coins || 0;
        return;
      }
    } catch {
      // Anonymous visitors are expected here; nothing to report.
    }
    isLoggedIn = false;
  }

  async function logout() {
    mobileMenuOpen = false;
    try {
      await api.logout();
    } finally {
      isLoggedIn = false;
      username = '';
      window.location.href = '/';
    }
  }

  onMount(loadUser);
  afterNavigate(loadUser);

  const closeMobileMenu = () => (mobileMenuOpen = false);
</script>

<div class="min-h-screen bg-[#0a0b1a] text-slate-50">
  <div class="pointer-events-none fixed inset-0 overflow-hidden">
    <div class="absolute -right-40 -top-40 h-96 w-96 animate-pulse rounded-full bg-purple-600/10 blur-3xl"></div>
    <div
      class="absolute -left-40 top-1/2 h-80 w-80 animate-pulse rounded-full bg-blue-600/10 blur-3xl"
      style="animation-delay: 1s"
    ></div>
    <div
      class="absolute -bottom-20 right-1/3 h-72 w-72 animate-pulse rounded-full bg-cyan-500/10 blur-3xl"
      style="animation-delay: 2s"
    ></div>
  </div>

  <header class="sticky top-0 z-50 border-b border-white/5 bg-[#0a0b1a]/80 backdrop-blur-xl">
    <nav class="mx-auto flex max-w-7xl items-center justify-between px-4 py-3 sm:px-6">
      <a href="/" class="group flex items-center gap-3" onclick={closeMobileMenu}>
        <div class="relative">
          <div
            class="absolute inset-0 rounded-xl bg-gradient-to-r from-blue-500 to-purple-500 opacity-50 blur-md transition-opacity group-hover:opacity-80"
          ></div>
          <div
            class="relative flex h-9 w-9 items-center justify-center rounded-xl bg-gradient-to-br from-blue-500 to-purple-600 text-lg sm:h-10 sm:w-10 sm:text-xl"
          >
            🧠
          </div>
        </div>
        <span
          class="hidden bg-gradient-to-r from-blue-400 to-purple-400 bg-clip-text text-xl font-bold text-transparent sm:block"
        >
          BrainStorm
        </span>
      </a>

      <div class="hidden items-center gap-2 md:flex">
        <a
          href="/lobby"
          class="rounded-xl px-4 py-2 text-sm font-medium text-slate-300 transition-all hover:bg-white/5 hover:text-white"
        >
          Комнаты
        </a>
        <a
          href="/leaderboard"
          class="rounded-xl px-4 py-2 text-sm font-medium text-slate-300 transition-all hover:bg-white/5 hover:text-white"
        >
          Рейтинг
        </a>
        <a
          href="/room/create"
          class="rounded-xl px-4 py-2 text-sm font-medium text-slate-300 transition-all hover:bg-white/5 hover:text-white"
        >
          Создать
        </a>

        {#if isLoggedIn}
          <a
            href="/profile"
            class="group flex items-center gap-3 rounded-xl border border-white/10 bg-white/5 px-3 py-2 transition-all hover:border-white/20 hover:bg-white/10"
          >
            <div
              class="flex h-8 w-8 items-center justify-center rounded-lg bg-gradient-to-br from-blue-500 to-purple-600 text-sm font-bold text-white"
            >
              {username.charAt(0).toUpperCase()}
            </div>
            <div class="flex flex-col">
              <span class="text-sm font-medium leading-tight text-white">
                {username}
              </span>
                  <span
                    class="ml-1 rounded bg-rose-500/20 px-1.5 py-0.5 text-[10px] font-semibold uppercase text-rose-300"
                    >читер</span
                  >
                {/if}
              </span>
              <span class="text-xs leading-tight text-slate-400">Ур. {userLevel} · 🪙 {userCoins}</span>
            </div>
          </a>
        {:else}
          <a href="/auth/login" class="group relative">
            <div
              class="absolute inset-0 rounded-xl bg-gradient-to-r from-blue-500 to-purple-600 opacity-50 blur-md transition-opacity group-hover:opacity-80"
            ></div>
            <div
              class="relative rounded-xl bg-gradient-to-r from-blue-500 to-purple-600 px-5 py-2 text-sm font-semibold text-white shadow-lg"
            >
              Войти
            </div>
          </a>
        {/if}
      </div>

      <div class="flex items-center gap-3 md:hidden">
        {#if isLoggedIn}
          <a
            href="/profile"
            class="flex h-9 w-9 items-center justify-center rounded-lg bg-gradient-to-br from-blue-500 to-purple-600 text-sm font-bold text-white"
          >
            {username.charAt(0).toUpperCase()}
          </a>
        {:else}
          <a
            href="/auth/login"
            class="rounded-xl bg-gradient-to-r from-blue-500 to-purple-600 px-4 py-2 text-sm font-semibold text-white"
          >
            Войти
          </a>
        {/if}

        <button
          onclick={() => (mobileMenuOpen = !mobileMenuOpen)}
          aria-label="Меню"
          aria-expanded={mobileMenuOpen}
          class="flex h-9 w-9 items-center justify-center rounded-xl border border-white/10 bg-white/5 text-white"
        >
          {#if mobileMenuOpen}
            <svg class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          {:else}
            <svg class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16" />
            </svg>
          {/if}
        </button>
      </div>
    </nav>

    {#if mobileMenuOpen}
      <div class="border-t border-white/5 bg-[#0a0b1a]/95 backdrop-blur-xl md:hidden">
        <div class="mx-auto max-w-7xl space-y-1 px-4 py-4">
          <a
            href="/lobby"
            onclick={closeMobileMenu}
            class="block rounded-xl px-4 py-3 text-slate-300 transition-all hover:bg-white/5"
          >
            Комнаты
          </a>
          <a
            href="/leaderboard"
            onclick={closeMobileMenu}
            class="block rounded-xl px-4 py-3 text-slate-300 transition-all hover:bg-white/5"
          >
            Рейтинг
          </a>
          <a
            href="/room/create"
            onclick={closeMobileMenu}
            class="block rounded-xl px-4 py-3 text-slate-300 transition-all hover:bg-white/5"
          >
            Создать комнату
          </a>

          {#if isLoggedIn}
            <div class="mt-2 border-t border-white/5 pt-3">
              <div class="flex items-center gap-3 px-2 py-2">
                <div
                  class="flex h-9 w-9 items-center justify-center rounded-lg bg-gradient-to-br from-blue-500 to-purple-600 text-sm font-bold text-white"
                >
                  {username.charAt(0).toUpperCase()}
                </div>
                <div>
                  <div class="text-sm font-medium text-white">{username}</div>
                  <div class="text-xs text-slate-400">Ур. {userLevel} · 🪙 {userCoins} · XP {userXp}</div>
                </div>
              </div>
              <a
                href="/profile"
                onclick={closeMobileMenu}
                class="block rounded-xl px-4 py-3 text-slate-300 transition-all hover:bg-white/5"
              >
                Профиль
              </a>
              <button
                onclick={logout}
                class="mt-1 block w-full rounded-xl px-4 py-3 text-left text-rose-300 transition-all hover:bg-rose-500/10"
              >
                Выйти
              </button>
            </div>
          {/if}
        </div>
      </div>
    {/if}
  </header>

  <main class="relative z-10 mx-auto max-w-7xl px-4 py-6 sm:px-6 sm:py-8">
    {@render children()}
  </main>

  <footer class="relative z-10 border-t border-white/5 bg-[#0a0b1a]/80 px-4 py-6 backdrop-blur-xl sm:px-6">
    <div class="mx-auto flex max-w-7xl flex-col items-center justify-between gap-2 sm:flex-row">
      <p class="text-sm text-slate-500">BrainStorm — викторина для компании</p>
      <p class="text-sm text-slate-500">Играйте с друзьями по коду комнаты</p>
    </div>
  </footer>
</div>
