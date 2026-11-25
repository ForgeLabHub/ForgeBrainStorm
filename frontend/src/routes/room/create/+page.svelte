<script lang="ts">
	import { api } from '$lib/api';
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';

	let topic = '';
	let difficulty = 'medium';
	let mode = 'classic';
	let questionCount = 10;
	let numOptions = 4;
	let timerSeconds = 30;
	let creating = false;
	let error = '';

	let aiProviders: string[] = [];
	let selectedProvider = '';

	onMount(async () => {
		try {
			const response = await api.getAiProviders();
			aiProviders = response.providers || [];
			selectedProvider = response.default || aiProviders[0] || '';
		} catch (e) {
			console.error('Failed to load AI providers:', e);
		}
	});

	async function createRoom() {
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
			console.error('Failed to create room:', e);
			error = 'Ошибка соединения с сервером';
		} finally {
			creating = false;
		}
	}

	// Helper: returns conditional class string based on selection.
	function cls(selected: boolean): string {
		return selected ? 'border-blue-500 bg-blue-500/10' : '';
	}
</script>

<div class="mx-auto max-w-3xl px-6 py-8">
	<div class="mb-8">
		<h1 class="mb-2 text-3xl font-bold text-white">Создать комнату</h1>
		<p class="text-slate-400">Настройте параметры викторины</p>
	</div>

	<form on:submit|preventDefault={createRoom} class="space-y-6">
		<!-- Тема -->
		<div class="rounded-xl border border-slate-700 bg-slate-800 p-6">
			<label class="mb-3 block">
				<span class="mb-2 block text-sm font-medium text-white">Тема викторины</span>
				<input
					type="text"
					bind:value={topic}
					placeholder="Например: История России, Космос, Литература..."
					class="w-full rounded-lg border border-slate-600 bg-slate-700 px-4 py-3 text-white placeholder-slate-400 focus:border-blue-500 focus:outline-none"
					maxlength="60"
				/>
				<span class="mt-1 block text-xs text-slate-400">{topic.length}/60 символов</span>
			</label>
		</div>

		<!-- Сложность и режим -->
		<div class="grid grid-cols-1 gap-6 md:grid-cols-2">
			<div class="rounded-xl border border-slate-700 bg-slate-800 p-6">
				<span class="mb-3 block text-sm font-medium text-white">Сложность</span>
				<div class="space-y-2">
					<label class="flex cursor-pointer items-center rounded-lg border p-3 transition hover:border-blue-500 {cls(difficulty === 'easy')}" class:border-blue-500={difficulty === 'easy'}>
						<input type="radio" bind:group={difficulty} value="easy" class="mr-3" />
						<div>
							<div class="font-medium text-white">Легкая</div>
							<div class="text-xs text-slate-400">Для начинающих</div>
						</div>
					</label>
					<label class="flex cursor-pointer items-center rounded-lg border p-3 transition hover:border-blue-500 {cls(difficulty === 'medium')}" class:border-blue-500={difficulty === 'medium'}>
						<input type="radio" bind:group={difficulty} value="medium" class="mr-3" />
						<div>
							<div class="font-medium text-white">Средняя</div>
							<div class="text-xs text-slate-400">Для всех</div>
						</div>
					</label>
					<label class="flex cursor-pointer items-center rounded-lg border p-3 transition hover:border-blue-500 {cls(difficulty === 'hard')}" class:border-blue-500={difficulty === 'hard'}>
						<input type="radio" bind:group={difficulty} value="hard" class="mr-3" />
						<div>
							<div class="font-medium text-white">Сложная</div>
							<div class="text-xs text-slate-400">Для экспертов</div>
						</div>
					</label>
				</div>
			</div>

			<div class="rounded-xl border border-slate-700 bg-slate-800 p-6">
				<span class="mb-3 block text-sm font-medium text-white">Режим игры</span>
				<div class="space-y-2">
					<label class="flex cursor-pointer items-center rounded-lg border p-3 transition hover:border-blue-500 {cls(mode === 'classic')}" class:border-blue-500={mode === 'classic'}>
						<input type="radio" bind:group={mode} value="classic" class="mr-3" />
						<div>
							<div class="font-medium text-white">Классический</div>
							<div class="text-xs text-slate-400">Все отвечают одновременно</div>
						</div>
					</label>
					<label class="flex cursor-pointer items-center rounded-lg border p-3 transition hover:border-blue-500 {cls(mode === 'ffa')}" class:border-blue-500={mode === 'ffa'}>
						<input type="radio" bind:group={mode} value="ffa" class="mr-3" />
						<div>
							<div class="font-medium text-white">Каждый за себя</div>
							<div class="text-xs text-slate-400">Кто первый ответит</div>
						</div>
					</label>
					<label class="flex cursor-pointer items-center rounded-lg border p-3 transition hover:border-blue-500 {cls(mode === 'team')}" class:border-blue-500={mode === 'team'}>
						<input type="radio" bind:group={mode} value="team" class="mr-3" />
						<div>
							<div class="font-medium text-white">Командный</div>
							<div class="text-xs text-slate-400">Игра командами</div>
						</div>
					</label>
					<label class="flex cursor-pointer items-center rounded-lg border p-3 transition hover:border-blue-500 {cls(mode === 'lives')}" class:border-blue-500={mode === 'lives'}>
						<input type="radio" bind:group={mode} value="lives" class="mr-3" />
						<div>
							<div class="font-medium text-white">На выживание</div>
							<div class="text-xs text-slate-400">3 жизни, кто выживет</div>
						</div>
					</label>
				</div>
			</div>
		</div>

		<!-- Параметры -->
		<div class="grid grid-cols-1 gap-6 md:grid-cols-3">
			<div class="rounded-xl border border-slate-700 bg-slate-800 p-6">
				<label class="block">
					<span class="mb-2 block text-sm font-medium text-white">Количество вопросов</span>
					<input
						type="number"
						bind:value={questionCount}
						min="5"
						max="30"
						class="w-full rounded-lg border border-slate-600 bg-slate-700 px-4 py-2 text-white focus:border-blue-500 focus:outline-none"
					/>
				</label>
			</div>

			<div class="rounded-xl border border-slate-700 bg-slate-800 p-6">
				<label class="block">
					<span class="mb-2 block text-sm font-medium text-white">Вариантов ответа</span>
					<input
						type="number"
						bind:value={numOptions}
						min="2"
						max="6"
						class="w-full rounded-lg border border-slate-600 bg-slate-700 px-4 py-2 text-white focus:border-blue-500 focus:outline-none"
					/>
				</label>
			</div>

			<div class="rounded-xl border border-slate-700 bg-slate-800 p-6">
				<label class="block">
					<span class="mb-2 block text-sm font-medium text-white">Время на вопрос (сек)</span>
					<input
						type="number"
						bind:value={timerSeconds}
						min="10"
						max="120"
						class="w-full rounded-lg border border-slate-600 bg-slate-700 px-4 py-2 text-white focus:border-blue-500 focus:outline-none"
					/>
				</label>
			</div>
		</div>

		<!-- AI Provider -->
		{#if aiProviders.length > 0}
			<div class="rounded-xl border border-slate-700 bg-slate-800 p-6">
				<label class="block">
					<span class="mb-2 block text-sm font-medium text-white">AI-провайдер</span>
					<select
						bind:value={selectedProvider}
						class="w-full rounded-lg border border-slate-600 bg-slate-700 px-4 py-2 text-white focus:border-blue-500 focus:outline-none"
					>
						{#each aiProviders as provider}
							<option value={provider}>{provider}</option>
						{/each}
					</select>
					<span class="mt-1 block text-xs text-slate-400">Можно изменить в настройках AI</span>
				</label>
			</div>
		{/if}

		<!-- Error -->
		{#if error}
			<div class="rounded-lg bg-red-500/10 p-4 text-red-400">
				{error}
			</div>
		{/if}

		<!-- Submit -->
		<button
			type="submit"
			disabled={creating || !topic.trim()}
			class="w-full rounded-lg bg-blue-600 px-6 py-4 text-lg font-semibold text-white transition hover:bg-blue-700 disabled:cursor-not-allowed disabled:opacity-50"
		>
			{#if creating}
				<span class="flex items-center justify-center gap-2">
					<div class="h-5 w-5 animate-spin rounded-full border-2 border-white border-t-transparent"></div>
					Создание...
				</span>
			{:else}
				Создать комнату
			{/if}
		</button>
	</form>
</div>
