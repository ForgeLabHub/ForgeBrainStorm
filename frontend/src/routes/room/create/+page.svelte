<script lang="ts">
	import { api } from '$lib/api';
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';
	import { MODES } from '$lib/ui';

	const suggestions = ['Космос', 'История России', 'Кино', 'Музыка', 'Спорт', 'Наука', 'География', 'IT'];
	const difficulties = [
		{ value: 'easy', label: 'Лёгкая' },
		{ value: 'medium', label: 'Средняя' },
		{ value: 'hard', label: 'Сложная' }
	];
	const questionPresets = [5, 10, 15, 20];
	const optionPresets = [3, 4, 6];
	const timerPresets = [15, 30, 60];

	let topic = '';
	let difficulty = 'medium';
	let mode = 'classic';
	let questionCount = 10;
	let numOptions = 4;
	let timerSeconds = 30;
	let creating = false;
	let error = '';

	let providers: string[] = [];
	let selectedProvider = '';

	onMount(async () => {
		try {
			const response = await api.getAiProviders();
			providers = response.providers || [];
			selectedProvider = response.default || providers[0] || '';
		} catch {
			providers = [];
		}
	});

	async function createRoom() {
		if (!topic.trim()) {
			error = 'Напишите тему — по ней AI составит вопросы';
			return;
		}

		creating = true;
		error = '';

		try {
			const response = await api.createRoom({
				topic: topic.trim(),
				difficulty,
				mode,
				question_count: questionCount,
				num_options: numOptions,
				timer_seconds: timerSeconds,
				ai_provider: selectedProvider || undefined
			});

			if (response.ok && response.room) {
				goto(`/room/${response.room.code}`);
			} else {
				error = response.error || 'Не удалось создать комнату';
			}
		} catch (e: any) {
			error = e.message || 'Сервер не отвечает. Попробуйте ещё раз.';
		} finally {
			creating = false;
		}
	}
</script>

<div class="mx-auto w-full max-w-2xl">
	<div class="mb-6">
		<a href="/lobby" class="mb-3 inline-flex items-center gap-1.5 text-sm text-dim hover:text-chalk">
			<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
				<path d="M14.5 5.5 8 12l6.5 6.5" />
			</svg>
			Комнаты
		</a>
		<h1 class="font-display text-[26px] tracking-tight text-chalk sm:text-3xl">Новая комната</h1>
		<p class="mt-1 text-sm text-dim">Настройте игру — изменить код потом нельзя, но перезапустить можно.</p>
	</div>

	<form on:submit|preventDefault={createRoom} class="space-y-4">
		<section class="card card-pad">
			<label for="topic" class="label">Тема вопросов</label>
			<input
				id="topic"
				type="text"
				bind:value={topic}
				maxlength="60"
				placeholder="Например: космос и планеты"
				class="field"
				autocomplete="off"
			/>
			<div class="mt-3 flex flex-wrap gap-1.5">
				{#each suggestions as suggestion}
					<button
						type="button"
						on:click={() => (topic = suggestion)}
						class="chip min-h-[36px] px-3 text-[13px] transition-colors hover:border-amber/50 hover:text-chalk"
					>
						{suggestion}
					</button>
				{/each}
			</div>
			{#if providers.length === 0}
				<p class="mt-3 text-xs leading-relaxed text-dim">
					AI-ключ не настроен, поэтому вопросы возьмутся из встроенного набора по теме. Играть это не мешает.
				</p>
			{/if}
		</section>

		<section class="card card-pad">
			<span class="label">Сложность</span>
			<div class="grid grid-cols-3 gap-1.5" role="group" aria-label="Сложность">
				{#each difficulties as item}
					<button
						type="button"
						on:click={() => (difficulty = item.value)}
						aria-pressed={difficulty === item.value}
						class="min-h-[44px] rounded-[12px] border px-2 text-sm font-medium transition-colors {difficulty ===
						item.value
							? 'border-amber bg-ink-800 text-amber'
							: 'border-line text-dim hover:text-chalk'}"
					>
						{item.label}
					</button>
				{/each}
			</div>
		</section>

		<div class="grid gap-4 sm:grid-cols-2">
			<section class="card card-pad">
				<span class="label">Сколько вопросов</span>
				<div class="grid grid-cols-4 gap-1.5" role="group" aria-label="Количество вопросов">
					{#each questionPresets as count}
						<button
							type="button"
							on:click={() => (questionCount = count)}
							aria-pressed={questionCount === count}
							class="min-h-[44px] rounded-[12px] border text-sm font-medium transition-colors {questionCount === count
								? 'border-amber bg-ink-800 text-amber'
								: 'border-line text-dim hover:text-chalk'}"
						>
							{count}
						</button>
					{/each}
				</div>
			</section>

			<section class="card card-pad">
				<span class="label">Время на вопрос</span>
				<div class="grid grid-cols-3 gap-1.5" role="group" aria-label="Время на вопрос">
					{#each timerPresets as seconds}
						<button
							type="button"
							on:click={() => (timerSeconds = seconds)}
							aria-pressed={timerSeconds === seconds}
							class="min-h-[44px] rounded-[12px] border text-sm font-medium transition-colors {timerSeconds === seconds
								? 'border-amber bg-ink-800 text-amber'
								: 'border-line text-dim hover:text-chalk'}"
						>
							{seconds} с
						</button>
					{/each}
				</div>
			</section>
		</div>

		<section class="card card-pad">
			<span class="label">Вариантов ответа</span>
			<div class="grid grid-cols-3 gap-1.5" role="group" aria-label="Количество вариантов ответа">
				{#each optionPresets as count}
					<button
						type="button"
						on:click={() => (numOptions = count)}
						aria-pressed={numOptions === count}
						class="min-h-[44px] rounded-[12px] border text-sm font-medium transition-colors {numOptions === count
							? 'border-amber bg-ink-800 text-amber'
							: 'border-line text-dim hover:text-chalk'}"
					>
						{count}
					</button>
				{/each}
			</div>
		</section>

		<section class="card card-pad">
			<span class="label">Режим</span>
			<div class="space-y-1.5">
				{#each Object.entries(MODES) as [value, meta]}
					<button
						type="button"
						on:click={() => (mode = value)}
						aria-pressed={mode === value}
						class="flex w-full items-center gap-3 rounded-[12px] border px-3.5 py-3 text-left transition-colors {mode ===
						value
							? 'border-amber bg-ink-800'
							: 'border-line hover:bg-ink-800'}"
					>
						<span
							class="grid h-5 w-5 shrink-0 place-items-center rounded-full border {mode === value
								? 'border-amber'
								: 'border-line'}"
						>
							{#if mode === value}
								<span class="h-2.5 w-2.5 rounded-full bg-amber"></span>
							{/if}
						</span>
						<span class="min-w-0">
							<span class="block text-sm font-medium text-chalk">{meta.label}</span>
							<span class="block text-xs text-dim">{meta.hint}</span>
						</span>
					</button>
				{/each}
			</div>
		</section>

		{#if providers.length > 0}
			<section class="card card-pad">
				<label for="provider" class="label">Провайдер вопросов</label>
				<select id="provider" bind:value={selectedProvider} class="field appearance-none">
					{#each providers as provider}
						<option value={provider}>{provider}</option>
					{/each}
				</select>
			</section>
		{/if}

		{#if error}
			<p class="rounded-[12px] border border-coral/30 bg-coral/10 px-4 py-3 text-sm text-coral" role="alert">
				{error}
			</p>
		{/if}

		<div class="pt-1">
			<button type="submit" disabled={creating || !topic.trim()} class="btn btn-primary w-full">
				{#if creating}
					<span class="h-4 w-4 animate-spin rounded-full border-2 border-ink-950/40 border-t-ink-950"></span>
					Создаём комнату…
				{:else}
					Создать комнату
				{/if}
			</button>
			<p class="mt-2 text-center text-xs text-dim">
				После создания вы получите код — отправьте его друзьям.
			</p>
		</div>
	</form>
</div>
