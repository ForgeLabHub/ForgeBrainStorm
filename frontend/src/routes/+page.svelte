<script lang="ts">
<<<<<<< HEAD
	import { onMount } from 'svelte';
	let visible = $state(false);

	onMount(() => {
		const timer = setTimeout(() => (visible = true), 100);
		return () => clearTimeout(timer);
	});
=======
	let visible = $state(false);

	$: if (typeof window !== 'undefined') {
		setTimeout(() => (visible = true), 100);
	}
>>>>>>> 4808142 (Merge pull request #15 from ForgeLabHub/arena/ca3aa14f-forgebrainstorm)
</script>

<div class="flex min-h-[85vh] flex-col items-center justify-center px-4 text-center relative">
	<!-- Animated background elements -->
	<div class="absolute inset-0 overflow-hidden pointer-events-none">
		<div class="absolute top-1/4 left-1/4 w-64 h-64 bg-blue-500/20 rounded-full blur-3xl animate-pulse"></div>
		<div class="absolute bottom-1/4 right-1/4 w-64 h-64 bg-purple-500/20 rounded-full blur-3xl animate-pulse" style="animation-delay: 1s;"></div>
	</div>

	<div class="relative transition-all duration-1000 {visible ? 'translate-y-0 opacity-100' : 'translate-y-12 opacity-0'}">
		<!-- Badge -->
		<div class="inline-flex items-center gap-2 px-4 py-1.5 rounded-full bg-white/5 border border-white/10 mb-8">
			<div class="h-2 w-2 rounded-full bg-green-400 animate-pulse"></div>
			<span class="text-xs font-medium text-slate-300">Powered by AI</span>
		</div>

		<!-- Main heading -->
		<h1 class="text-6xl font-black tracking-tight sm:text-8xl">
			<span class="gradient-text">BrainStorm</span>
		</h1>

		<p class="mt-8 max-w-2xl text-xl text-slate-400 sm:text-2xl font-light leading-relaxed">
			Интеллектуальная викторина с
			<span class="text-white font-medium">AI-генерацией</span>
			вопросов
		</p>

		<!-- CTA buttons -->
		<div class="mt-12 flex flex-col gap-4 sm:flex-row sm:justify-center">
			<a
				href="/lobby"
				class="btn-primary group relative rounded-xl px-8 py-4 font-semibold text-white overflow-hidden"
			>
				<span class="relative z-10 flex items-center gap-2">
					Начать игру
					<svg class="h-5 w-5 transition-transform group-hover:translate-x-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7l5 5m0 0l-5 5m5-5H6"/>
					</svg>
				</span>
			</a>
			<a
				href="/auth/register"
				class="btn-secondary rounded-xl px-8 py-4 font-semibold text-white"
			>
				Регистрация
			</a>
		</div>

		<!-- Features grid -->
		<div class="mt-24 grid grid-cols-1 gap-6 sm:grid-cols-3">
			<!-- Feature 1 -->
			<div class="glass-card rounded-2xl p-6 text-left group hover:scale-105 transition-all duration-300">
				<div class="mb-4 flex h-12 w-12 items-center justify-center rounded-xl bg-gradient-to-br from-blue-500/20 to-blue-600/20 border border-blue-500/20 group-hover:border-blue-500/40 transition">
					<svg class="h-6 w-6 text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"/>
					</svg>
				</div>
				<h3 class="mb-2 text-lg font-bold text-white">6 режимов игры</h3>
				<p class="text-sm text-slate-400 leading-relaxed">Классика, FFA, команды, выживание и другие</p>
			</div>

			<!-- Feature 2 -->
			<div class="glass-card rounded-2xl p-6 text-left group hover:scale-105 transition-all duration-300">
				<div class="mb-4 flex h-12 w-12 items-center justify-center rounded-xl bg-gradient-to-br from-purple-500/20 to-purple-600/20 border border-purple-500/20 group-hover:border-purple-500/40 transition">
					<svg class="h-6 w-6 text-purple-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z"/>
					</svg>
				</div>
				<h3 class="mb-2 text-lg font-bold text-white">AI-генерация</h3>
				<p class="text-sm text-slate-400 leading-relaxed">GPT-4o-mini создаёт уникальные вопросы</p>
			</div>

			<!-- Feature 3 -->
			<div class="glass-card rounded-2xl p-6 text-left group hover:scale-105 transition-all duration-300">
				<div class="mb-4 flex h-12 w-12 items-center justify-center rounded-xl bg-gradient-to-br from-cyan-500/20 to-cyan-600/20 border border-cyan-500/20 group-hover:border-cyan-500/40 transition">
					<svg class="h-6 w-6 text-cyan-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z"/>
					</svg>
				</div>
				<h3 class="mb-2 text-lg font-bold text-white">Лидерборд</h3>
				<p class="text-sm text-slate-400 leading-relaxed">Соревнуйтесь и зарабатывайте XP</p>
			</div>
		</div>
	</div>
</div>
