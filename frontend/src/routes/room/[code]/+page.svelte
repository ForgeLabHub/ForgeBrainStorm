<script lang="ts">
	import { page } from '$app/stores';
	import { api, ws } from '$lib/api';
	import { onMount, onDestroy } from 'svelte';

	const roomCode = $derived(($page.params.code ?? '').toUpperCase());

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

<div class="mx-auto max-w-4xl px-4 sm:px-0 animate-fade-in">
	{#if gamePhase === 'loading'}
		<div class="py-20 text-center">
			<svg class="mx-auto h-10 w-10 animate-spin text-blue-500" fill="none" viewBox="0 0 24 24">
				<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
				<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"></path>
			</svg>
		</div>
	{:else}
		<!-- Header -->
		<div class="mb-8">
			<div class="flex items-center gap-3 flex-wrap">
				<button
					onclick={copyCode}
					class="group relative rounded-xl bg-gradient-to-r from-blue-500/20 to-purple-500/20 border border-blue-500/30 px-4 py-2 font-mono text-sm font-bold text-white hover:border-blue-500/50 transition-all shadow-lg"
				>
					<span class="relative z-10 flex items-center gap-2">
						{roomCode}
						{#if copiedCode}
							<svg class="h-4 w-4 text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"/>
							</svg>
						{:else}
							<svg class="h-4 w-4 text-slate-400 group-hover:text-white transition" fill="none" stroke="currentColor" viewBox="0 0 24 24">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 5H6a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2v-1M8 5a2 2 0 002 2h2a2 2 0 002-2M8 5a2 2 0 012-2h2a2 2 0 012 2m0 0h2a2 2 0 012 2v3m2 4H10m0 0l3-3m-3 3l3 3"/>
							</svg>
						{/if}
					</span>
				</button>
				{#if mode}
					<span class="px-3 py-1 rounded-lg bg-white/5 border border-white/10 text-sm font-medium text-slate-300">
						{getModeLabel(mode)}
					</span>
				{/if}
			</div>
			<h1 class="mt-3 text-2xl font-bold text-white sm:text-3xl">{topic || 'Без темы'}</h1>
		</div>

		{#if error}
			<div class="mb-6 flex items-center gap-2 rounded-xl bg-red-500/10 border border-red-500/20 px-4 py-3 text-sm text-red-400">
				<svg class="h-5 w-5 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>
				</svg>
				{error}
			</div>
		{/if}

		{#if gamePhase === 'waiting'}
			<!-- Waiting phase -->
			<div class="glass-card rounded-2xl p-8 text-center">
				<div class="mb-6">
					<div class="mx-auto mb-4 flex h-16 w-16 items-center justify-center rounded-2xl bg-gradient-to-br from-blue-500/20 to-purple-500/20 border border-blue-500/30">
						<svg class="h-8 w-8 text-blue-400 animate-pulse" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z"/>
						</svg>
					</div>
					<h2 class="text-xl font-bold text-white">Ожидание игроков</h2>
					<p class="mt-2 text-sm text-slate-400">
						Поделитесь кодом комнаты:
						<button onclick={copyCode} class="font-mono text-blue-400 hover:text-blue-300 font-bold transition">{roomCode}</button>
					</p>
				</div>

				{#if players.length > 0}
					<div class="mb-6 flex flex-wrap justify-center gap-2">
						{#each players as player, i}
							<div class="flex items-center gap-2 rounded-xl bg-white/5 border border-white/10 px-3 py-2 animate-fade-in" style="animation-delay: {i * 50}ms">
								<div class="h-6 w-6 rounded-full bg-gradient-to-br from-blue-500 to-purple-600 flex items-center justify-center text-xs font-bold text-white shadow-lg">
									{player.name.charAt(0).toUpperCase()}
								</div>
								<span class="text-sm font-medium text-white">{player.name}</span>
								{#if player.is_host}
									<span class="rounded-lg bg-gradient-to-r from-yellow-500/20 to-orange-500/20 border border-yellow-500/30 px-2 py-0.5 text-[10px] font-bold text-yellow-400">HOST</span>
								{/if}
							</div>
						{/each}
					</div>
				{/if}

				{#if isHost && players.length > 0}
					<button
						onclick={startGame}
						class="btn-primary rounded-xl px-8 py-3 font-semibold text-white"
					>
						<span class="flex items-center gap-2">
							Начать игру
							<svg class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z"/>
							</svg>
						</span>
					</button>
				{:else if !isHost}
					<p class="text-sm text-slate-500">Ожидание хоста...</p>
				{:else}
					<p class="text-sm text-slate-500">Нужен хотя бы 1 игрок</p>
				{/if}
			</div>

		{:else if gamePhase === 'playing' && question}
			<!-- Playing phase -->
			<div class="space-y-5">
				<!-- Timer -->
				<div class="glass-card rounded-2xl p-5">
					<div class="flex items-center justify-between mb-3">
						<span class="text-xs font-semibold text-slate-400 uppercase tracking-wide">Время</span>
						<span class="text-2xl font-black font-mono {getTimerColor()}">{timeLeft}с</span>
					</div>
					<div class="h-2 rounded-full bg-white/5 overflow-hidden">
						<div class="h-full rounded-full bg-gradient-to-r {getTimerBarColor()} transition-all duration-1000 shadow-lg" style="width: {getTimerPercent()}%"></div>
					</div>
				</div>

				<!-- Question -->
				<div class="glass-card rounded-2xl p-6 sm:p-8">
					{#if question.category}
						<span class="inline-block mb-3 rounded-lg bg-gradient-to-r from-purple-500/20 to-pink-500/20 border border-purple-500/30 px-3 py-1 text-xs font-semibold text-purple-400">
							{question.category}
						</span>
					{/if}
					<h2 class="text-lg sm:text-xl font-bold text-white leading-relaxed">{question.question}</h2>
				</div>

				<!-- Options -->
				<div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
					{#each question.options as option, i}
						{@const isSelected = selectedAnswer === i}
						<button
							onclick={() => submitAnswer(i)}
							disabled={selectedAnswer !== null}
							class="group relative glass-card rounded-xl p-5 text-left transition-all hover:scale-[1.02] {
								isSelected
									? 'bg-gradient-to-r from-blue-500/20 to-blue-600/20 border-blue-500/50 shadow-lg shadow-blue-500/20'
									: selectedAnswer !== null
										? 'opacity-50'
										: ''
							} disabled:cursor-default"
						>
							<div class="flex items-center gap-4">
								<div class="flex h-10 w-10 items-center justify-center rounded-xl font-bold text-sm transition-all {
									isSelected
										? 'bg-gradient-to-br from-blue-500 to-blue-600 text-white shadow-lg shadow-blue-500/50'
										: 'bg-white/5 border border-white/10 text-slate-400 group-hover:border-white/20'
								}">
									{String.fromCharCode(65 + i)}
								</div>
								<span class="font-semibold {isSelected ? 'text-blue-300' : 'text-white'}">{option}</span>
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
			<!-- Result phase -->
			<div class="space-y-5">
				<!-- Result -->
				<div class="glass-card rounded-2xl p-8 text-center">
					<div class="mb-4">
						{#if lastResult?.correct}
							<div class="mx-auto flex h-20 w-20 items-center justify-center rounded-2xl bg-gradient-to-br from-green-500/20 to-emerald-500/20 border border-green-500/30 shadow-lg shadow-green-500/20">
								<svg class="h-10 w-10 text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
									<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"/>
								</svg>
							</div>
						{:else}
							<div class="mx-auto flex h-20 w-20 items-center justify-center rounded-2xl bg-gradient-to-br from-red-500/20 to-rose-500/20 border border-red-500/30 shadow-lg shadow-red-500/20">
								<svg class="h-10 w-10 text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
									<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z"/>
								</svg>
							</div>
						{/if}
					</div>
					<h2 class="text-2xl font-bold {lastResult?.correct ? 'text-green-400' : 'text-red-400'}">
						{lastResult?.correct ? 'Правильно!' : 'Неправильно'}
					</h2>
					{#if lastResult}
						<p class="mt-3 text-sm text-slate-400">
							Правильный ответ: <span class="font-bold text-white">"{String.fromCharCode(65 + lastResult.correctIndex)}"</span>
						</p>
						{#if lastResult.explanation}
							<div class="mt-4 rounded-xl bg-white/5 border border-white/10 p-4 text-left">
								<p class="text-sm text-slate-300 leading-relaxed">{lastResult.explanation}</p>
							</div>
						{/if}
					{/if}
				</div>

				<!-- Leaderboard -->
				<div class="glass-card rounded-2xl p-6">
					<h3 class="mb-4 text-sm font-semibold text-slate-300 uppercase tracking-wide">Результаты</h3>
					<div class="space-y-2">
						{#each players.sort((a, b) => b.score - a.score) as player, i}
							<div class="flex items-center justify-between rounded-xl bg-white/[0.02] border border-white/5 p-3 animate-fade-in" style="animation-delay: {i * 50}ms">
								<div class="flex items-center gap-3">
									<span class="flex h-8 w-8 items-center justify-center rounded-lg text-sm font-bold {
										i === 0 ? 'bg-gradient-to-br from-yellow-500/20 to-orange-500/20 border border-yellow-500/30 text-yellow-400' :
										i === 1 ? 'bg-gradient-to-br from-slate-400/20 to-slate-500/20 border border-slate-400/30 text-slate-300' :
										i === 2 ? 'bg-gradient-to-br from-orange-500/20 to-amber-500/20 border border-orange-500/30 text-orange-400' :
										'bg-white/5 border border-white/10 text-slate-500'
									}">{i + 1}</span>
									<span class="font-semibold text-white">{player.name}</span>
								</div>
								<span class="text-lg font-black text-white">{player.score}</span>
							</div>
						{/each}
					</div>
				</div>
			</div>

		{:else if gamePhase === 'finished'}
			<!-- Finished phase -->
			<div class="glass-card rounded-2xl p-8 text-center">
				<div class="mb-6">
					<div class="mx-auto flex h-20 w-20 items-center justify-center rounded-2xl bg-gradient-to-br from-yellow-500/20 to-orange-500/20 border border-yellow-500/30 shadow-lg shadow-yellow-500/20">
						<svg class="h-10 w-10 text-yellow-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 3v4M3 5h4M6 17v4m-2-2h4m5-16l2.286 6.857L21 12l-5.714 2.143L13 21l-2.286-6.857L5 12l5.714-2.143L13 3z"/>
						</svg>
					</div>
				</div>
				<h2 class="text-3xl font-bold text-white">Игра окончена</h2>
				{#if players.length > 0}
					<p class="mt-3 text-slate-400">
						Победитель: <span class="font-bold text-yellow-400">{players[0]?.name}</span>
					</p>
				{/if}

				<div class="mt-8 space-y-3">
					{#each players.sort((a, b) => b.score - a.score).slice(0, 5) as player, i}
						<div class="flex items-center justify-between rounded-xl bg-white/5 border border-white/10 p-4 {i === 0 ? 'ring-2 ring-yellow-500/30 shadow-lg shadow-yellow-500/10' : ''} animate-fade-in" style="animation-delay: {i * 100}ms">
							<div class="flex items-center gap-4">
								<span class="text-lg font-black text-slate-500">#{i + 1}</span>
								<span class="font-semibold text-white">{player.name}</span>
							</div>
							<span class="text-xl font-black text-white">{player.score}</span>
						</div>
					{/each}
				</div>

				<div class="mt-8 flex flex-col gap-3 sm:flex-row sm:justify-center">
					<a href="/lobby" class="btn-secondary rounded-xl px-6 py-3 font-semibold text-white">
						В лобби
					</a>
					<button onclick={startGame} class="btn-primary rounded-xl px-6 py-3 font-semibold text-white">
						Играть снова
					</button>
				</div>
			</div>
		{/if}
	{/if}
</div>
