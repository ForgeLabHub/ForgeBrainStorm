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

	onMount(async () => {
		try {
			const response = await api.getAiProviders();
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
		{ value: 'easy', label: 'Лёгкая', desc: 'Для начинающих', color: 'from-green-500/20 to-green-600/20 border-green-500/30 text-green-400' },
		{ value: 'medium', label: 'Средняя', desc: 'Для всех', color: 'from-yellow-500/20 to-yellow-600/20 border-yellow-500/30 text-yellow-400' },
		{ value: 'hard', label: 'Сложная', desc: 'Для экспертов', color: 'from-red-500/20 to-red-600/20 border-red-500/30 text-red-400' },
	];

	const modes = [
		{ value: 'classic', label: 'Классика', desc: 'Все отвечают одновременно', icon: 'M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z' },
		{ value: 'ffa', label: 'Каждый за себя', desc: 'Кто быстрее ответит', icon: 'M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z' },
		{ value: 'team', label: 'Команды', desc: 'Игра командами', icon: 'M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197M13 7a4 4 0 11-8 0 4 4 0 018 0z' },
		{ value: 'lives', label: 'На выживание', desc: '3 жизни', icon: 'M4.318 6.318a4.5 4.5 0 000 6.364L12 20.364l7.682-7.682a4.5 4.5 0 00-6.364-6.364L12 7.636l-1.318-1.318a4.5 4.5 0 00-6.364 0z' },
		{ value: 'coop', label: 'Кооператив', desc: 'Работайте вместе', icon: 'M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z' },
		{ value: 'svoyaigra', label: 'Своя Игра', desc: 'Выбирай вопросы', icon: 'M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-3 7h3m-3 4h3m-6-4h.01M9 16h.01' },
	];
</script>

<div class="mx-auto max-w-3xl animate-fade-in">
	<!-- Header -->
	<div class="mb-8">
		<h1 class="text-3xl font-bold text-white">Создать комнату</h1>
		<p class="mt-2 text-slate-400">Настройте параметры викторины</p>
	</div>

	<form onsubmit={createRoom} class="space-y-6">
		<!-- Topic -->
		<div class="glass-card rounded-2xl p-6">
			<label class="block">
				<span class="mb-3 block text-sm font-semibold text-slate-300 uppercase tracking-wide">Тема викторины</span>
				<input
					type="text"
					bind:value={topic}
					placeholder="Например: История, Космос, Литература"
					maxlength="60"
					class="input-modern w-full rounded-xl px-4 py-3 text-white placeholder-slate-600"
				/>
				<span class="mt-2 block text-xs text-slate-500 text-right">{topic.length}/60</span>
			</label>
		</div>

		<!-- Difficulty & Mode -->
		<div class="grid grid-cols-1 gap-6 sm:grid-cols-2">
			<!-- Difficulty -->
			<div class="glass-card rounded-2xl p-6">
				<h3 class="mb-4 text-sm font-semibold text-slate-300 uppercase tracking-wide">Сложность</h3>
				<div class="space-y-2">
					{#each difficulties as d}
						<label class="flex cursor-pointer items-center gap-3 rounded-xl p-4 transition-all hover:scale-[1.02] {
							difficulty === d.value
								? `bg-gradient-to-r border ${d.color} shadow-lg`
								: 'bg-white/[0.02] border border-white/5 hover:border-white/20'
						}">
							<input type="radio" bind:group={difficulty} value={d.value} class="sr-only" />
							<div class="min-w-0 flex-1">
								<div class="font-semibold text-white">{d.label}</div>
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

			<!-- Mode -->
			<div class="glass-card rounded-2xl p-6">
				<h3 class="mb-4 text-sm font-semibold text-slate-300 uppercase tracking-wide">Режим игры</h3>
				<div class="space-y-2">
					{#each modes as m}
						<label class="flex cursor-pointer items-center gap-3 rounded-xl p-4 transition-all hover:scale-[1.02] {
							mode === m.value
								? 'bg-gradient-to-r from-blue-500/20 to-purple-500/20 border border-blue-500/30 shadow-lg'
								: 'bg-white/[0.02] border border-white/5 hover:border-white/20'
						}">
							<input type="radio" bind:group={mode} value={m.value} class="sr-only" />
							<div class="flex h-10 w-10 items-center justify-center rounded-lg bg-white/5 border border-white/10">
								<svg class="h-5 w-5 text-slate-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
									<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d={m.icon}/>
								</svg>
							</div>
							<div class="min-w-0 flex-1">
								<div class="font-semibold text-white">{m.label}</div>
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

		<!-- Parameters -->
		<div class="glass-card rounded-2xl p-6">
			<h3 class="mb-4 text-sm font-semibold text-slate-300 uppercase tracking-wide">Параметры</h3>
			<div class="grid grid-cols-3 gap-4">
				<div>
					<label class="block">
						<span class="mb-2 block text-xs font-medium text-slate-400">Вопросов</span>
						<input
							type="number"
							bind:value={questionCount}
							min="5"
							max="30"
							class="input-modern w-full rounded-xl px-3 py-2.5 text-lg font-bold text-white text-center"
						/>
					</label>
				</div>

				<div>
					<label class="block">
						<span class="mb-2 block text-xs font-medium text-slate-400">Вариантов</span>
						<input
							type="number"
							bind:value={numOptions}
							min="2"
							max="6"
							class="input-modern w-full rounded-xl px-3 py-2.5 text-lg font-bold text-white text-center"
						/>
					</label>
				</div>

				<div>
					<label class="block">
						<span class="mb-2 block text-xs font-medium text-slate-400">Время (сек)</span>
						<input
							type="number"
							bind:value={timerSeconds}
							min="10"
							max="120"
							class="input-modern w-full rounded-xl px-3 py-2.5 text-lg font-bold text-white text-center"
						/>
					</label>
				</div>
			</div>
		</div>

		{#if error}
			<div class="flex items-center gap-2 rounded-xl bg-red-500/10 border border-red-500/20 px-4 py-3 text-sm text-red-400">
				<svg class="h-5 w-5 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>
				</svg>
				{error}
			</div>
		{/if}

		<button
			type="submit"
			disabled={creating || !topic.trim()}
			class="btn-primary w-full rounded-xl py-4 font-semibold text-white disabled:opacity-50"
		>
			{#if creating}
				<span class="flex items-center justify-center gap-2">
					<svg class="h-5 w-5 animate-spin" fill="none" viewBox="0 0 24 24">
						<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
						<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"></path>
					</svg>
					Создание...
				</span>
			{:else}
				<span class="flex items-center justify-center gap-2">
					Создать комнату
					<svg class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7l5 5m0 0l-5 5m5-5H6"/>
					</svg>
				</span>
			{/if}
		</button>
	</form>
</div>
