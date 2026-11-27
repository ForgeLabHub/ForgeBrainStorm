<script lang="ts">
  import { page } from '$app/stores';

  const title = $derived($page.status === 404 ? 'Страница не найдена' : 'Что-то сломалось');
  const hint = $derived(
    $page.status === 404
      ? 'Проверьте адрес или вернитесь к комнатам — возможно, код комнаты устарел.'
      : $page.error?.message || 'Попробуйте обновить страницу через минуту.'
  );
</script>

<svelte:head>
  <title>{$page.status} — BrainStorm</title>
</svelte:head>

<div class="mx-auto flex min-h-[60vh] max-w-xl flex-col items-center justify-center px-4 text-center">
  <p class="text-6xl font-black text-white/10">{$page.status}</p>
  <h1 class="mt-2 text-2xl font-bold text-white">{title}</h1>
  <p class="mt-2 text-slate-400">{hint}</p>
  <div class="mt-6 flex flex-wrap justify-center gap-3">
    <a
      href="/lobby"
      class="rounded-xl bg-gradient-to-r from-blue-500 to-purple-600 px-5 py-2.5 font-semibold text-white shadow-lg transition hover:shadow-blue-500/20"
    >
      К комнатам
    </a>
    <a
      href="/"
      class="rounded-xl border border-white/10 bg-white/5 px-5 py-2.5 font-semibold text-slate-200 transition hover:bg-white/10"
    >
      На главную
    </a>
  </div>
</div>
