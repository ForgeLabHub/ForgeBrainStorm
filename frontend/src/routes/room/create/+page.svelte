<script lang="ts">
	import { api } from '$lib/api';
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';

	let topic = $state('');
	let difficulty = $state('medium');
	let mode = $state('classic');
	let questionCount = $state(10);
	let numOptions = $state(4);
	let timerSeconds = $state(30);
	let creating = $state(false);
	let error = $state('');
	let visible = $state(false);

	let aiProviders: string[] = $state([]);
	let selectedProvider = $state('');

	$: if (typeof window !== 'undefined') {
		setTimeout(() => (visible = true), 100);
	}

	onMount(async () => {
		try {
			const response = await api.getAiProviders();
			aiProviders = response.providers || [];
			selectedProvider = response.default || aiProviders[0] || '';
		} catch (e) {
			console.error('Failed to load AI providers:', e);
		}
	});

	async function createRoom(e: Event) {
		e.preventDefault();
		if (!topic.trim()) {
			error = 'Введите тему викторины';
			return;
		}
		creating = true;
		error = '';
		try {
			const response = await api.createRoom(topic.trim(), mode);
			if (response.ok && response.room) {
				goto(`/room/${response.room.code}`);
			} else {
				error = response.error || 'Не удалось создать комнату';
			}
		} catch (e) {
			error = 'Ошибка соединения с сервером';
		} finally {
			creating = false;
		}
	}

	const difficulties = [
		{ value: 'easy', label: 'Лёгкая', desc: 'Для начинающих', icon: '🌱', color: 'from-green-500 to-emerald-500' },
		{ value: 'medium', label: 'Средняя', desc: 'Для всех', icon: '⚡', color: 'from-blue-500 to-cyan-500' },
		{ value: 'hard', label: 'Сложная', desc: 'Для экспертов', icon: '🔥', color: 'from-red-500 to-orange-500' },
	];

	const modes = [
		{ value: 'classic', label: 'Классика', desc: 'Все отвечают одновременно', icon: '🎯' },
		{ value: 'ffa', label: 'Каждый за себя', desc: 'Кто быстрее ответит', icon: '⚔️' },
		{ value: 'team', label: 'Команды', desc: 'Игра командами', icon: '👥' },
		{ value: 'lives', label: 'На выживание', desc: '3 жизни — кто выживет', icon: '❤️' },
	];
</script>

<div class="mx-auto max-w-4xl px-4 py-8 sm:px-6">
	<!-- Header -->
	<div class="mb-8 transition-all duration-700 {visible ? 'translate-y-0 opacity-100' : 'translate-y-8 opacity-0'}">
		<div class="flex items-center gap-3">
			<div class="flex h-10 w-10 items-center justify-center rounded-xl bg-gradient-to-br from-blue-500 to-purple-600 text-xl">
				🎮
			</div>
			<div>
				<h1 class="text-2xl font-bold text-white sm:text-3xl">Создать комнату</h1>
				<p class="text-sm text-slate-400">Настройте параметры викторины</p>
			</div>
		</div>
	</div>

	<form onsubmit={createRoom} class="space-y-6">
		<!-- Topic -->
		<div class="relative overflow-hidden rounded-2xl border border-white/5 bg-white/[0.02] p-5 backdrop-blur-sm transition-all duration-700 delay-100 sm:p-6 {visible ? 'translate-y-0 opacity-100' : 'translate-y-8 opacity-0'}">
			<div class="absolute inset-0 bg-gradient-to-br from-blue-500/5 to-transparent"></div>
			<div class="relative z-10">
				<label class="block">
					<span class="mb-2 flex items-center justify-between text-xs font-medium uppercase tracking-wider text-slate-400">
						<span>Тема викторины</span>
						<span class="{topic.length > 55 ? 'text-orange-400' : 'text-slate-500'} normal-case tracking-normal">
							{topic.length}/60
						</span>
					</span>
					<input
						type="text"
						bind:value={topic}
						placeholder="Например: История России, Космос, Литература..."
						maxlength="60"
						class="w-full rounded-xl border border-white/10 bg-white/5 px-4 py-3 text-lg text-white placeholder-slate-500 backdrop-blur-sm transition focus:border-blue-500/50 focus:bg-white/10 focus:outline-none focus:ring-2 focus:ring-blue-500/20"
					/>
				</label>
			</div>
		</div>

		<!-- Difficulty & Mode -->
		<div class="grid grid-cols-1 gap-6 md:grid-cols-2 transition-all duration-700 delay-200 {visible ? 'translate-y-0 opacity-100' : 'translate-y-8 opacity-0'}">
			<!-- Difficulty -->
			<div class="relative overflow-hidden rounded-2xl border border-white/5 bg-white/[0.02] p-5 backdrop-blur-sm sm:p-6">
				<div class="absolute inset-0 bg-gradient-to-br from-green-500/3 to-transparent"></div>
				<div class="relative z-10">
					<h3 class="mb-4 flex items-center gap-2 text-sm font-medium uppercase tracking-wider text-slate-400">
						<svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z"></path>
						</svg>
						Сложность
					</h3>
					<div class="space-y-2">
						{#each difficulties as d}
							<label class="group flex cursor-pointer items-center gap-3 rounded-xl border p-3 transition-all {
								difficulty === d.value
									? 'border-white/20 bg-white/[0.05]'
									: 'border-white/5 bg-white/[0.02] hover:border-white/20 hover:bg-white/[0.04]'
							}">
								<input type="radio" bind:group={difficulty} value={d.value} class="sr-only" />
								<div class="flex h-10 w-10 items-center justify-center rounded-lg {difficulty === d.value ? `bg-gradient-to-br ${d.color}` : 'bg-white/5'} text-lg transition-transform group-hover:scale-110">
									{d.icon}
								</div>
								<div class="min-w-0 flex-1">
									<div class="font-medium text-white">{d.label}</div>
									<div class="text-xs text-slate-400">{d.desc}</div>
								</div>
								{#if difficulty === d.value}
									<svg class="h-5 w-5 flex-shrink-0 text-green-400" fill="currentColor" viewBox="0 0 20 20">
										<path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd" />
									</svg>
								{/if}
							</label>
						{/each}
					</div>
				</div>
			</div>

			<!-- Mode -->
			<div class="relative overflow-hidden rounded-2xl border border-white/5 bg-white/[0.02] p-5 backdrop-blur-sm sm:p-6">
				<div class="absolute inset-0 bg-gradient-to-br from-purple-500/3 to-transparent"></div>
				<div class="relative z-10">
					<h3 class="mb-4 flex items-center gap-2 text-sm font-medium uppercase tracking-wider text-slate-400">
						<svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z"></path>
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>
						</svg>
						Режим игры
					</h3>
					<div class="space-y-2">
						{#each modes as m}
							<label class="group flex cursor-pointer items-center gap-3 rounded-xl border p-3 transition-all {
								mode === m.value
									? 'border-white/20 bg-white/[0.05]'
									: 'border-white/5 bg-white/[0.02] hover:border-white/20 hover:bg-white/[0.04]'
							}">
								<input type="radio" bind:group={mode} value={m.value} class="sr-only" />
								<div class="flex h-10 w-10 items-center justify-center rounded-lg {mode === m.value ? 'bg-gradient-to-br from-purple-500 to-pink-500' : 'bg-white/5'} text-lg transition-transform group-hover:scale-110">
									{m.icon}
								</div>
								<div class="min-w-0 flex-1">
									<div class="font-medium text-white">{m.label}</div>
									<div class="text-xs text-slate-400">{m.desc}</div>
								</div>
								{#if mode === m.value}
									<svg class="h-5 w-5 flex-shrink-0 text-green-400" fill="currentColor" viewBox="0 0 20 20">
										<path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd" />
									</svg>
								{/if}
							</label>
						{/each}
					</div>
				</div>
			</div>
		</div>

		<!-- Parameters -->
		<div class="grid grid-cols-1 gap-4 sm:grid-cols-3 transition-all duration-700 delay-300 {visible ? 'translate-y-0 opacity-100' : 'translate-y-8 opacity-0'}">
			<div class="relative overflow-hidden rounded-2xl border border-white/5 bg-white/[0.02] p-4 backdrop-blur-sm sm:p-5">
				<div class="absolute inset-0 bg-gradient-to-br from-blue-500/5 to-transparent"></div>
				<div class="relative z-10">
					<label class="block">
						<span class="mb-2 block text-xs font-medium uppercase tracking-wider text-slate-400">Вопросов</span>
						<input
							type="number"
							bind:value={questionCount}
							min="5"
							max="30"
							class="w-full rounded-xl border border-white/10 bg-white/5 px-4 py-2.5 text-xl font-bold text-white focus:border-blue-500/50 focus:outline-none focus:ring-2 focus:ring-blue-500/20"
						/>
						<span class="mt-1 block text-xs text-slate-500">5–30</span>
					</label>
				</div>
			</div>

			<div class="relative overflow-hidden rounded-2xl border border-white/5 bg-white/[0.02] p-4 backdrop-blur-sm sm:p-5">
				<div class="absolute inset-0 bg-gradient-to-br from-purple-500/5 to-transparent"></div>
				<div class="relative z-10">
					<label class="block">
						<span class="mb-2 block text-xs font-medium uppercase tracking-wider text-slate-400">Вариантов</span>
						<input
							type="number"
							bind:value={numOptions}
							min="2"
							max="6"
							class="w-full rounded-xl border border-white/10 bg-white/5 px-4 py-2.5 text-xl font-bold text-white focus:border-blue-500/50 focus:outline-none focus:ring-2 focus:ring-blue-500/20"
						/>
						<span class="mt-1 block text-xs text-slate-500">2–6</span>
					</label>
				</div>
			</div>

			<div class="relative overflow-hidden rounded-2xl border border-white/5 bg-white/[0.02] p-4 backdrop-blur-sm sm:p-5">
				<div class="absolute inset-0 bg-gradient-to-br from-green-500/5 to-transparent"></div>
				<div class="relative z-10">
					<label class="block">
						<span class="mb-2 block text-xs font-medium uppercase tracking-wider text-slate-400">Время (сек)</span>
						<input
							type="number"
							bind:value={timerSeconds}
							min="10"
							max="120"
							class="w-full rounded-xl border border-white/10 bg-white/5 px-4 py-2.5 text-xl font-bold text-white focus:border-blue-500/50 focus:outline-none focus:ring-2 focus:ring-blue-500/20"
						/>
						<span class="mt-1 block text-xs text-slate-500">10–120</span>
					</label>
				</div>
			</div>
		</div>

		<!-- Error -->
		{#if error}
			<div class="flex items-center gap-2 rounded-xl border border-red-500/20 bg-red-500/10 p-4 text-sm text-red-400">
				<svg class="h-5 w-5 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>
				</svg>
				{error}
			</div>
		{/if}

		<!-- Submit -->
		<button
			type="submit"
			disabled={creating || !topic.trim()}
			class="group relative w-full overflow-hidden rounded-2xl py-4 text-lg font-semibold text-white shadow-lg transition-all disabled:cursor-not-allowed disabled:opacity-50"
		>
			<div class="absolute inset-0 bg-gradient-to-r from-blue-500 via-purple-500 to-cyan-500 transition-transform group-hover:scale-105"></div>
			<div class="absolute inset-0 bg-gradient-to-r from-blue-500 via-purple-500 to-cyan-500 opacity-0 blur-xl transition-opacity group-hover:opacity-70"></div>
			<div class="relative flex items-center justify-center gap-3">
				{#if creating}
					<svg class="h-5 w-5 animate-spin" fill="none" viewBox="0 0 24 24">
						<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
						<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"></path>
					</svg>
					<span>Создание комнаты...</span>
				{:else}
					<svg class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z"></path>
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>
					</svg>
					<span>Создать комнату</span>
					<svg class="h-5 w-5 transition-transform group-hover:translate-x-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7l5 5m0 0l-5 5m5-5H6"></path>
					</svg>
				{/if}
			</div>
		</button>
	</form>
</div>
