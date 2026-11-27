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
	let timerLocked = $derived(mode === 'blitz' || mode === 'svoyaigra');
	let creating = $state(false);
	let error = $state('');
	let visible = $state(false);

	let aiProviders: string[] = $state([]);
	let selectedProvider = $state('');

	onMount(() => {
		const timer = setTimeout(() => (visible = true), 100);
		return () => clearTimeout(timer);
	});

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
			const response = await api.createRoom({
				topic: topic.trim(),
				mode,
				difficulty,
				question_count: questionCount,
				num_options: numOptions,
				timer_seconds: timerSeconds
			});
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
		{ value: 'easy', label: 'Лёгкая', desc: 'Для начинающих', color: 'text-emerald-400', bg: 'bg-emerald-500/10' },
		{ value: 'medium', label: 'Средняя', desc: 'Для всех', color: 'text-blue-400', bg: 'bg-blue-500/10' },
		{ value: 'hard', label: 'Сложная', desc: 'Для экспертов', color: 'text-orange-400', bg: 'bg-orange-500/10' },
	];

	const modes = [
		{ value: 'classic', label: 'Классика', desc: '100 очков за верный ответ', accent: 'border-blue-500/30' },
		{ value: 'ffa', label: 'Каждый за себя', desc: 'Чем быстрее ответ, тем больше очков', accent: 'border-amber-500/30' },
		{ value: 'lives', label: 'На выживание', desc: '3 жизни — ошибка их отнимает', accent: 'border-red-500/30' },
		{ value: 'blitz', label: 'Блиц', desc: '10 секунд, серия умножает очки', accent: 'border-yellow-500/30' },
		{ value: 'team', label: 'Команды', desc: 'Две команды, счёт общий', accent: 'border-purple-500/30' },
		{ value: 'coop', label: 'Кооператив', desc: 'Работайте вместе ради общей цели', accent: 'border-green-500/30' },
		{ value: 'svoyaigra', label: 'Своя Игра', desc: 'Выбирай категорию и ставку очков', accent: 'border-cyan-500/30' },
		{ value: 'sandbox', label: 'Тренировка', desc: 'Без рейтинга и жизней', accent: 'border-slate-500/30' },
	];

	const FIXED_TIMERS: Record<string, number> = { blitz: 10, svoyaigra: 15 };
</script>

<div class="mx-auto max-w-4xl px-4 py-8 sm:px-6">
	<!-- Header -->
	<div class="mb-8 transition-all duration-700 {visible ? 'translate-y-0 opacity-100' : 'translate-y-8 opacity-0'}">
		<div class="flex items-center gap-3">
			<div class="flex h-10 w-10 items-center justify-center rounded-xl bg-gradient-to-br from-blue-500 to-purple-600">
				<svg class="h-5 w-5 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6"></path>
				</svg>
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
								<div class="flex h-10 w-10 items-center justify-center rounded-lg {difficulty === d.value ? d.bg : 'bg-white/5'} transition-all">
									<svg class="h-5 w-5 {difficulty === d.value ? d.color : 'text-slate-400'}" fill="none" stroke="currentColor" viewBox="0 0 24 24">
										{#if d.value === 'easy'}
											<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"></path>
										{:else if d.value === 'medium'}
											<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z"></path>
										{:else}
											<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17.657 18.657A8 8 0 016.343 7.343S7 9 9 10c0-2 .5-5 2.986-7C14 5 16.09 5.777 17.656 7.343A7.975 7.975 0 0120 13a7.975 7.975 0 01-2.343 5.657z"></path>
										{/if}
									</svg>
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
							<label class="group flex items-center gap-3 rounded-xl border p-3 transition-all cursor-pointer {
								mode === m.value
									? `border-white/20 bg-white/[0.05] ${m.accent}`
									: 'border-white/5 bg-white/[0.02] hover:border-white/20 hover:bg-white/[0.04]'
							}">
								<input type="radio" bind:group={mode} value={m.value} class="sr-only" />
								<div class="flex h-10 w-10 items-center justify-center rounded-lg {mode === m.value ? 'bg-white/10' : 'bg-white/5'} transition-all">
									<svg class="h-5 w-5 {mode === m.value ? 'text-white' : 'text-slate-400'}" fill="none" stroke="currentColor" viewBox="0 0 24 24">
										{#if m.value === 'classic'}
											<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"></path>
										{:else if m.value === 'ffa'}
											<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z"></path>
										{:else if m.value === 'lives'}
											<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4.318 6.318a4.5 4.5 0 000 6.364L12 20.364l7.682-7.682a4.5 4.5 0 00-6.364-6.364L12 7.636l-1.318-1.318a4.5 4.5 0 00-6.364 0z"></path>
										{:else if m.value === 'blitz'}
											<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z"></path>
										{:else if m.value === 'team'}
											<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z"></path>
										{:else if m.value === 'coop'}
											<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197M13 7a4 4 0 11-8 0 4 4 0 018 0z"></path>
										{:else if m.value === 'svoyaigra'}
											<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-6 9l2 2 4-4"></path>
										{:else}
											<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"></path>
											<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"></path>
										{/if}
									</svg>
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
							value={timerLocked ? (FIXED_TIMERS[mode] || 30) : timerSeconds}
							oninput={(event) => (timerSeconds = Number(event.currentTarget.value))}
							disabled={timerLocked}
							min="10"
							max="120"
							class="w-full rounded-xl border border-white/10 bg-white/5 px-4 py-2.5 text-xl font-bold text-white focus:border-blue-500/50 focus:outline-none focus:ring-2 focus:ring-blue-500/20 disabled:cursor-not-allowed disabled:text-slate-400"
						/>
						<span class="mt-1 block text-xs text-slate-500">
							{timerLocked ? `Фиксировано: ${FIXED_TIMERS[mode]}с` : '10–120'}
						</span>
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
