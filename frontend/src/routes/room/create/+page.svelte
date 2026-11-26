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
		{ value: 'easy', label: 'Лёгкая', desc: 'Для начинающих' },
		{ value: 'medium', label: 'Средняя', desc: 'Для всех' },
		{ value: 'hard', label: 'Сложная', desc: 'Для экспертов' },
	];

	const modes = [
		{ value: 'classic', label: 'Классика', desc: 'Все отвечают одновременно' },
		{ value: 'ffa', label: 'Каждый за себя', desc: 'Кто быстрее ответит' },
		{ value: 'team', label: 'Команды', desc: 'Игра командами' },
		{ value: 'lives', label: 'На выживание', desc: '3 жизни' },
	];
</script>

<div class="mx-auto max-w-2xl">
	<div class="mb-8">
		<h1 class="text-2xl font-bold text-white">Создать комнату</h1>
		<p class="mt-2 text-sm text-slate-400">Настройте параметры викторины</p>
	</div>

	<form onsubmit={createRoom} class="space-y-6">
		<!-- Topic -->
		<div class="rounded-xl border border-white/5 bg-white/[0.02] p-4">
			<label class="block">
				<span class="mb-2 block text-sm font-medium text-slate-300">Тема викторины</span>
				<input
					type="text"
					bind:value={topic}
					placeholder="Например: История, Космос, Литература"
					maxlength="60"
					class="w-full rounded-lg border border-white/10 bg-white/5 px-4 py-2.5 text-white placeholder-slate-500 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
				/>
				<span class="mt-1 block text-xs text-slate-500">{topic.length}/60</span>
			</label>
		</div>

		<!-- Difficulty & Mode -->
		<div class="grid grid-cols-1 gap-6 sm:grid-cols-2">
			<div class="rounded-xl border border-white/5 bg-white/[0.02] p-4">
				<h3 class="mb-3 text-sm font-medium text-slate-300">Сложность</h3>
				<div class="space-y-2">
					{#each difficulties as d}
						<label class="flex cursor-pointer items-center gap-3 rounded-lg border p-3 transition {
							difficulty === d.value
								? 'border-white/20 bg-white/[0.05]'
								: 'border-white/5 bg-white/[0.02] hover:border-white/20'
						}">
							<input type="radio" bind:group={difficulty} value={d.value} class="sr-only" />
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

			<div class="rounded-xl border border-white/5 bg-white/[0.02] p-4">
				<h3 class="mb-3 text-sm font-medium text-slate-300">Режим игры</h3>
				<div class="space-y-2">
					{#each modes as m}
						<label class="flex cursor-pointer items-center gap-3 rounded-lg border p-3 transition {
							mode === m.value
								? 'border-white/20 bg-white/[0.05]'
								: 'border-white/5 bg-white/[0.02] hover:border-white/20'
						}">
							<input type="radio" bind:group={mode} value={m.value} class="sr-only" />
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

		<!-- Parameters -->
		<div class="grid grid-cols-3 gap-3">
			<div class="rounded-xl border border-white/5 bg-white/[0.02] p-4">
				<label class="block">
					<span class="mb-2 block text-xs font-medium uppercase tracking-wide text-slate-400">Вопросов</span>
					<input
						type="number"
						bind:value={questionCount}
						min="5"
						max="30"
						class="w-full rounded-lg border border-white/10 bg-white/5 px-3 py-2 text-lg font-bold text-white focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
					/>
				</label>
			</div>

			<div class="rounded-xl border border-white/5 bg-white/[0.02] p-4">
				<label class="block">
					<span class="mb-2 block text-xs font-medium uppercase tracking-wide text-slate-400">Вариантов</span>
					<input
						type="number"
						bind:value={numOptions}
						min="2"
						max="6"
						class="w-full rounded-lg border border-white/10 bg-white/5 px-3 py-2 text-lg font-bold text-white focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
					/>
				</label>
			</div>

			<div class="rounded-xl border border-white/5 bg-white/[0.02] p-4">
				<label class="block">
					<span class="mb-2 block text-xs font-medium uppercase tracking-wide text-slate-400">Время (сек)</span>
					<input
						type="number"
						bind:value={timerSeconds}
						min="10"
						max="120"
						class="w-full rounded-lg border border-white/10 bg-white/5 px-3 py-2 text-lg font-bold text-white focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
					/>
				</label>
			</div>
		</div>

		{#if error}
			<div class="rounded-lg border border-red-500/20 bg-red-500/10 p-3 text-sm text-red-400">
				{error}
			</div>
		{/if}

		<button
			type="submit"
			disabled={creating || !topic.trim()}
			class="w-full rounded-lg bg-blue-500 py-3 font-medium text-white transition hover:bg-blue-600 disabled:cursor-not-allowed disabled:opacity-50"
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
				Создать комнату
			{/if}
		</button>
	</form>
</div>
