<script lang="ts">
	import { onMount } from 'svelte';
	let visible = $state(false);

	onMount(() => {
		const timer = setTimeout(() => (visible = true), 100);
		return () => clearTimeout(timer);
	});
</script>

<div class="flex min-h-[80vh] flex-col items-center justify-center px-4 text-center">
	<div class="transition-all duration-700 {visible ? 'translate-y-0 opacity-100' : 'translate-y-8 opacity-0'}">
		<h1 class="text-5xl font-bold tracking-tight text-white sm:text-7xl">
			<span class="bg-gradient-to-r from-blue-400 to-purple-400 bg-clip-text text-transparent">
				BrainStorm
			</span>
		</h1>

		<p class="mt-6 max-w-2xl text-lg text-slate-400 sm:text-xl">
			Интеллектуальная викторина с AI-генерацией вопросов
		</p>

		<div class="mt-10 flex flex-col gap-3 sm:flex-row sm:justify-center">
			<a
				href="/lobby"
				class="rounded-lg bg-blue-500 px-8 py-3 font-medium text-white transition hover:bg-blue-600"
			>
				Начать игру
			</a>
			<a
				href="/auth/register"
				class="rounded-lg border border-white/10 bg-white/5 px-8 py-3 font-medium text-white transition hover:bg-white/10"
			>
				Регистрация
			</a>
		</div>

		<div class="mt-20 grid grid-cols-1 gap-6 sm:grid-cols-3">
			<div class="rounded-xl border border-white/5 bg-white/[0.02] p-6 text-left">
				<div class="mb-3 flex h-10 w-10 items-center justify-center rounded-lg bg-blue-500/10">
					<svg class="h-5 w-5 text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"/>
					</svg>
				</div>
				<h3 class="mb-2 font-semibold text-white">6 режимов</h3>
				<p class="text-sm text-slate-400">Классика, FFA, команды, выживание и другие</p>
			</div>

			<div class="rounded-xl border border-white/5 bg-white/[0.02] p-6 text-left">
				<div class="mb-3 flex h-10 w-10 items-center justify-center rounded-lg bg-purple-500/10">
					<svg class="h-5 w-5 text-purple-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z"/>
					</svg>
				</div>
				<h3 class="mb-2 font-semibold text-white">AI-вопросы</h3>
				<p class="text-sm text-slate-400">GPT-4o-mini генерирует уникальные вопросы</p>
			</div>

			<div class="rounded-xl border border-white/5 bg-white/[0.02] p-6 text-left">
				<div class="mb-3 flex h-10 w-10 items-center justify-center rounded-lg bg-cyan-500/10">
					<svg class="h-5 w-5 text-cyan-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z"/>
					</svg>
				</div>
				<h3 class="mb-2 font-semibold text-white">Лидерборд</h3>
				<p class="text-sm text-slate-400">Соревнуйтесь и зарабатывайте XP</p>
			</div>
		</div>
	</div>
</div>
