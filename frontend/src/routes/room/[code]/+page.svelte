<script lang="ts">
	import { page } from '$app/stores';
	import { api, ws } from '$lib/api';
	import { onMount, onDestroy } from 'svelte';

<<<<<<< HEAD
	const roomCode = $derived(($page.params.code ?? '').toUpperCase());
=======
	$: roomCode = $page.params.code.toUpperCase();
>>>>>>> 4808142 (Merge pull request #15 from ForgeLabHub/arena/ca3aa14f-forgebrainstorm)

	type Player = {
		name: string;
		score: number;
		is_host: boolean;
		username?: string;
	};

	type Question = {
		question: string;
		options: string[];
		category?: string;
		time_limit: number;
	};

	let players: Player[] = $state([]);
	let question: Question | null = $state(null);
	let selectedAnswer: number | null = $state(null);
	let timeLeft = $state(0);
	let gamePhase: 'loading' | 'waiting' | 'playing' | 'result' | 'finished' = $state('loading');
	let isHost = $state(false);
	let topic = $state('');
	let mode = $state('');
	let error = $state('');
	let timerInterval: ReturnType<typeof setInterval> | null = null;
	let lastResult: { correct: boolean; correctIndex: number; explanation: string } | null = $state(null);
	let copiedCode = $state(false);

	onMount(() => {
		ws.connect();
		ws.on('connected', () => {
			ws.send('join_room', { code: roomCode });
		});
		ws.on('room_joined', (data) => {
			isHost = data.is_host;
			topic = data.topic || '';
			mode = data.mode || 'classic';
			gamePhase = 'waiting';
			if (data.players) {
				players = data.players.map((p: any) => ({
					name: p.display_name || p.username,
					score: p.score || 0,
					is_host: p.is_host || false,
					username: p.username,
				}));
			}
		});
		ws.on('players_updated', (data) => {
			if (data?.players) {
				players = data.players.map((p: any) => ({
					name: p.display_name || p.username,
					score: p.score || 0,
					is_host: p.is_host || false,
					username: p.username,
				}));
			}
		});
		ws.on('player_joined', (data) => {
			if (data?.player) {
				players = [...players, {
					name: data.player.display_name || data.player.username,
					score: data.player.score || 0,
					is_host: data.player.is_host || false,
					username: data.player.username,
				}];
			}
		});
		ws.on('player_left', (data) => {
			players = players.filter(p => p.username !== data?.username);
		});
		ws.on('question_emitted', (data) => {
			question = { ...data.question, time_limit: data.time_limit || 30 };
			timeLeft = data.time_limit || 30;
			selectedAnswer = null;
			gamePhase = 'playing';
			startTimer();
		});
		ws.on('timer_update', (data) => {
			timeLeft = data.seconds_remaining || 0;
		});
		ws.on('question_result', (data) => {
			gamePhase = 'result';
			stopTimer();
			lastResult = data;
		});
		ws.on('game_over', (data) => {
			gamePhase = 'finished';
			stopTimer();
			if (data?.final_leaderboard) {
				players = data.final_leaderboard.map((p: any) => ({
					name: p.username,
					score: p.score,
					is_host: false,
					username: p.username,
				}));
			}
		});
		ws.on('game_started', () => {
			gamePhase = 'playing';
		});
		ws.on('error', (data) => {
			error = data.message || 'Ошибка';
		});
	});

	onDestroy(() => {
		stopTimer();
	});

	function startTimer() {
		stopTimer();
		timerInterval = setInterval(() => {
			timeLeft = Math.max(0, timeLeft - 1);
			if (timeLeft === 0) stopTimer();
		}, 1000);
	}

	function stopTimer() {
		if (timerInterval) {
			clearInterval(timerInterval);
			timerInterval = null;
		}
	}

	function submitAnswer(index: number) {
		if (selectedAnswer !== null || gamePhase !== 'playing') return;
		selectedAnswer = index;
		ws.send('submit_answer', { answer_index: index });
	}

	function startGame() {
		ws.send('start_game', {});
	}

	async function copyCode() {
		try {
			await navigator.clipboard.writeText(roomCode);
			copiedCode = true;
			setTimeout(() => copiedCode = false, 2000);
		} catch (err) {
			console.error('Failed to copy:', err);
		}
	}

	function getTimerColor(): string {
		if (timeLeft > 15) return 'text-green-400';
		if (timeLeft > 5) return 'text-yellow-400';
		return 'text-red-400';
	}

	function getTimerPercent(): number {
		if (!question) return 0;
		return (timeLeft / question.time_limit) * 100;
	}

	function getTimerBarColor(): string {
		if (timeLeft > 15) return 'from-green-500 to-emerald-500';
		if (timeLeft > 5) return 'from-yellow-500 to-amber-500';
		return 'from-red-500 to-rose-500';
	}

	function getModeLabel(m: string): string {
		const labels: Record<string, string> = {
			classic: 'Классика',
			ffa: 'FFA',
			team: 'Команды',
			lives: 'Выживание',
			coop: 'Кооператив',
			svoyaigra: 'Своя Игра',
		};
		return labels[m] || m;
	}
</script>

<div class="mx-auto max-w-4xl px-4 sm:px-0">
	{#if gamePhase === 'loading'}
		<div class="py-20 text-center">
			<svg class="mx-auto h-8 w-8 animate-spin text-blue-500" fill="none" viewBox="0 0 24 24">
				<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
				<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"></path>
			</svg>
		</div>
	{:else}
		<!-- Header -->
		<div class="mb-6">
			<div class="flex items-center gap-3">
				<button
					onclick={copyCode}
					class="rounded border border-white/10 bg-white/5 px-2.5 py-1 font-mono text-sm text-slate-400 hover:border-white/20 transition"
				>
					{roomCode}
					{#if copiedCode}
						<span class="ml-2 text-green-400">Скопировано</span>
					{/if}
				</button>
				{#if mode}
					<span class="text-sm text-slate-500">{getModeLabel(mode)}</span>
				{/if}
			</div>
			<h1 class="mt-2 text-xl font-bold text-white sm:text-2xl">{topic || 'Без темы'}</h1>
		</div>

		{#if error}
			<div class="mb-4 rounded-lg border border-red-500/20 bg-red-500/10 p-3 text-sm text-red-400">
				{error}
			</div>
		{/if}

		{#if gamePhase === 'waiting'}
			<div class="rounded-xl border border-white/5 bg-white/[0.02] p-6 sm:p-8 text-center">
				<h2 class="mb-2 text-xl font-bold text-white">Ожидание игроков</h2>
				<p class="mb-6 text-sm text-slate-400">
					Поделитесь кодом:
					<button onclick={copyCode} class="font-mono text-blue-400 hover:text-blue-300">{roomCode}</button>
				</p>

				{#if players.length > 0}
					<div class="mb-6 flex flex-wrap justify-center gap-2">
						{#each players as player}
							<div class="flex items-center gap-2 rounded-lg bg-white/5 border border-white/10 px-3 py-1.5">
								<div class="h-5 w-5 rounded-full bg-gradient-to-br from-blue-500 to-purple-600 flex items-center justify-center text-[10px] font-bold text-white">
									{player.name.charAt(0).toUpperCase()}
								</div>
								<span class="text-sm text-white">{player.name}</span>
								{#if player.is_host}
									<span class="rounded bg-yellow-500/20 px-1 py-0.5 text-[9px] font-bold text-yellow-400">HOST</span>
								{/if}
							</div>
						{/each}
					</div>
				{/if}

				{#if isHost && players.length > 0}
					<button
						onclick={startGame}
						class="rounded-lg bg-green-500 px-8 py-3 font-medium text-white hover:bg-green-600 transition"
					>
						Начать игру
					</button>
				{:else if !isHost}
					<p class="text-sm text-slate-500">Ожидание хоста...</p>
				{:else}
					<p class="text-sm text-slate-500">Нужен хотя бы 1 игрок</p>
				{/if}
			</div>

		{:else if gamePhase === 'playing' && question}
			<div class="space-y-4">
				<!-- Timer -->
				<div class="rounded-lg border border-white/5 bg-white/[0.02] p-3">
					<div class="flex items-center justify-between mb-2">
						<span class="text-xs text-slate-500">Время</span>
						<span class="text-lg font-bold font-mono {getTimerColor()}">{timeLeft}с</span>
					</div>
					<div class="h-1.5 rounded-full bg-white/5 overflow-hidden">
						<div class="h-full rounded-full bg-gradient-to-r {getTimerBarColor()} transition-all duration-1000" style="width: {getTimerPercent()}%"></div>
					</div>
				</div>

				<!-- Question -->
				<div class="rounded-xl border border-white/5 bg-white/[0.02] p-5 sm:p-6">
					{#if question.category}
						<span class="inline-block mb-2 rounded bg-purple-500/10 px-2 py-0.5 text-xs font-medium text-purple-400">
							{question.category}
						</span>
					{/if}
					<h2 class="text-base sm:text-lg font-bold text-white leading-relaxed">{question.question}</h2>
				</div>

				<!-- Options -->
				<div class="grid grid-cols-1 gap-2 sm:grid-cols-2">
					{#each question.options as option, i}
						{@const isSelected = selectedAnswer === i}
						<button
							onclick={() => submitAnswer(i)}
							disabled={selectedAnswer !== null}
							class="group relative rounded-lg border p-4 text-left transition {
								isSelected
									? 'border-blue-500 bg-blue-500/10'
									: selectedAnswer !== null
										? 'border-white/5 bg-white/[0.02] opacity-50'
										: 'border-white/10 bg-white/[0.02] hover:border-white/20 hover:bg-white/[0.04]'
							} disabled:cursor-default"
						>
							<div class="flex items-center gap-3">
								<div class="flex h-8 w-8 items-center justify-center rounded font-bold text-sm {
									isSelected
										? 'bg-blue-500 text-white'
										: 'bg-white/5 text-slate-400'
								} transition-colors">
									{String.fromCharCode(65 + i)}
								</div>
								<span class="font-medium {isSelected ? 'text-blue-300' : 'text-white'}">{option}</span>
							</div>
						</button>
					{/each}
				</div>

				{#if selectedAnswer !== null}
					<div class="flex items-center justify-center gap-2 text-sm text-slate-400">
						<svg class="h-4 w-4 animate-pulse" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"></path>
						</svg>
						Ожидание результата...
					</div>
				{/if}
			</div>

		{:else if gamePhase === 'result'}
			<div class="space-y-4">
				<!-- Result -->
				<div class="rounded-xl border border-white/5 bg-white/[0.02] p-6 text-center">
					<div class="mb-3">
						{#if lastResult?.correct}
							<svg class="mx-auto h-12 w-12 text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"/>
							</svg>
						{:else}
							<svg class="mx-auto h-12 w-12 text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z"/>
							</svg>
						{/if}
					</div>
					<h2 class="text-xl font-bold {lastResult?.correct ? 'text-green-400' : 'text-red-400'}">
						{lastResult?.correct ? 'Правильно' : 'Неправильно'}
					</h2>
					{#if lastResult}
						<p class="mt-2 text-sm text-slate-400">
							Ответ: <span class="font-semibold text-white">{String.fromCharCode(65 + lastResult.correctIndex)}</span>
						</p>
						{#if lastResult.explanation}
							<div class="mt-4 rounded-lg bg-white/5 border border-white/10 p-3 text-left">
								<p class="text-sm text-slate-300">{lastResult.explanation}</p>
							</div>
						{/if}
					{/if}
				</div>

				<!-- Leaderboard -->
				<div class="rounded-xl border border-white/5 bg-white/[0.02] p-4">
					<h3 class="mb-3 text-sm font-medium text-slate-300">Результаты</h3>
					<div class="space-y-1.5">
						{#each players.sort((a, b) => b.score - a.score) as player, i}
							<div class="flex items-center justify-between rounded-lg bg-white/[0.02] border border-white/5 p-2.5">
								<div class="flex items-center gap-2">
									<span class="flex h-6 w-6 items-center justify-center rounded text-xs font-bold {
										i === 0 ? 'bg-yellow-500/20 text-yellow-400' :
										i === 1 ? 'bg-slate-400/20 text-slate-300' :
										i === 2 ? 'bg-orange-500/20 text-orange-400' :
										'bg-white/5 text-slate-500'
									}">{i + 1}</span>
									<span class="text-sm font-medium text-white">{player.name}</span>
								</div>
								<span class="text-sm font-bold text-white">{player.score}</span>
							</div>
						{/each}
					</div>
				</div>
			</div>

		{:else if gamePhase === 'finished'}
			<div class="rounded-xl border border-white/5 bg-white/[0.02] p-6 sm:p-8 text-center">
				<svg class="mx-auto mb-4 h-16 w-16 text-yellow-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 3v4M3 5h4M6 17v4m-2-2h4m5-16l2.286 6.857L21 12l-5.714 2.143L13 21l-2.286-6.857L5 12l5.714-2.143L13 3z"/>
				</svg>
				<h2 class="mb-2 text-2xl font-bold text-white">Игра окончена</h2>
				{#if players.length > 0}
					<p class="text-slate-400">
						Победитель: <span class="font-semibold text-yellow-400">{players[0]?.name}</span>
					</p>
				{/if}

				<div class="mt-6 space-y-2">
					{#each players.sort((a, b) => b.score - a.score).slice(0, 5) as player, i}
						<div class="flex items-center justify-between rounded-lg bg-white/5 border border-white/10 p-3 {i === 0 ? 'ring-1 ring-yellow-500/30' : ''}">
							<div class="flex items-center gap-3">
								<span class="text-sm font-bold text-slate-500">#{i + 1}</span>
								<span class="font-medium text-white">{player.name}</span>
							</div>
							<span class="text-lg font-bold text-white">{player.score}</span>
						</div>
					{/each}
				</div>

				<div class="mt-6 flex flex-col gap-3 sm:flex-row sm:justify-center">
					<a href="/lobby" class="rounded-lg border border-white/10 bg-white/5 px-6 py-2.5 font-medium text-white hover:bg-white/10 transition">
						В лобби
					</a>
					<button onclick={startGame} class="rounded-lg bg-blue-500 px-6 py-2.5 font-medium text-white hover:bg-blue-600 transition">
						Играть снова
					</button>
				</div>
			</div>
		{/if}
	{/if}
</div>
