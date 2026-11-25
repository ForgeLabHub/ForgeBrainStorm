<script lang="ts">
	import { page } from '$app/stores';
	import { api, ws } from '$lib/api';
	import { onMount, onDestroy } from 'svelte';

	$: roomCode = $page.params.code.toUpperCase();

	type Player = {
		name: string;
		score: number;
		is_host: boolean;
		is_ready: boolean;
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
	let gamePhase: 'waiting' | 'playing' | 'result' | 'finished' = $state('waiting');
	let isHost = $state(false);
	let topic = $state('');
	let mode = $state('');
	let error = $state('');
	let timerInterval: ReturnType<typeof setInterval> | null = null;
	let lastResult: { correct: boolean; correctIndex: number; explanation: string; player_answers?: any[] } | null = $state(null);
	let finalLeaderboard: any[] | null = $state(null);

	onMount(() => {
		ws.connect();
		ws.on('connected', () => {
			ws.send('join_room', { code: roomCode });
		});
		ws.on('room_joined', (data) => {
			isHost = data.is_host;
			topic = data.topic || '';
			mode = data.mode || 'classic';
			if (data.players) {
				players = data.players.map((p: any) => ({
					name: p.display_name || p.username,
					score: p.score || 0,
					is_host: p.is_host || false,
					is_ready: p.is_ready || false,
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
					is_ready: p.is_ready || false,
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
					is_ready: data.player.is_ready || false,
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
				finalLeaderboard = data.final_leaderboard;
				players = data.final_leaderboard.map((p: any) => ({
					name: p.username,
					score: p.score,
					is_host: false,
					is_ready: true,
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
		if (timeLeft > 15) return 'bg-green-500';
		if (timeLeft > 5) return 'bg-yellow-500';
		return 'bg-red-500';
	}

	function getModeLabel(m: string): string {
		const labels: Record<string, string> = {
			classic: '🎯 Классика',
			ffa: '⚔️ FFA',
			team: '👥 Команды',
			lives: '❤️ На выживание',
			coop: '🤝 Кооператив',
			svoyaigra: '🎲 Своя Игра',
		};
		return labels[m] || m;
	}
</script>

<div class="mx-auto max-w-7xl px-6 py-8">
	<!-- Header -->
	<div class="mb-6 flex items-center justify-between">
		<div>
			<div class="flex items-center gap-3">
				<div class="font-mono text-sm text-slate-500 tracking-wider bg-white/5 rounded-lg px-3 py-1 border border-white/10">{roomCode}</div>
				{#if mode}
					<span class="text-sm text-slate-400">{getModeLabel(mode)}</span>
				{/if}
			</div>
			<h1 class="mt-2 text-2xl font-bold text-white">{topic || 'Без темы'}</h1>
		</div>
		<div class="flex items-center gap-4">
			<div class="text-right">
				<div class="text-xs text-slate-500">Игроков</div>
				<div class="text-2xl font-bold text-white">{players.length}</div>
			</div>
		</div>
	</div>

	{#if error}
		<div class="mb-4 flex items-center gap-3 rounded-xl border border-red-500/20 bg-red-500/10 p-4 text-red-400">
			<svg class="w-5 h-5 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path></svg>
			{error}
		</div>
	{/if}

	<div class="grid grid-cols-1 gap-6 lg:grid-cols-3">
		<!-- Main area -->
		<div class="lg:col-span-2">
			{#if gamePhase === 'waiting'}
				<div class="relative overflow-hidden rounded-2xl border border-white/5 bg-white/[0.02] p-12 text-center backdrop-blur-sm">
					<div class="absolute inset-0 bg-gradient-to-br from-blue-500/5 to-purple-500/5"></div>
					<div class="relative z-10">
						<div class="mb-6 text-6xl">⏳</div>
						<h2 class="mb-2 text-2xl font-bold text-white">Ожидание игроков</h2>
						<p class="mb-6 text-slate-400">Поделитесь кодом комнаты: <span class="font-mono text-blue-400">{roomCode}</span></p>

						{#if isHost && players.length > 0}
							<button
								onclick={startGame}
								class="relative group"
							>
								<div class="absolute inset-0 bg-gradient-to-r from-green-500 to-emerald-600 rounded-xl blur-lg opacity-40 group-hover:opacity-70 transition-opacity"></div>
								<div class="relative flex items-center gap-3 rounded-xl bg-gradient-to-r from-green-500 to-emerald-600 px-8 py-4 text-lg font-semibold text-white shadow-xl">
									<svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z"></path></svg>
									Начать игру
								</div>
							</button>
						{:else if !isHost}
							<p class="text-sm text-slate-500">Ожидаем, пока хост начнёт игру...</p>
						{:else}
							<p class="text-sm text-slate-500">Нужен хотя бы 1 игрок для старта</p>
						{/if}
					</div>
				</div>

			{:else if gamePhase === 'playing' && question}
				<div class="space-y-6">
					<!-- Timer -->
					<div class="relative overflow-hidden rounded-xl border border-white/5 bg-white/[0.02] p-4 backdrop-blur-sm">
						<div class="flex items-center justify-between mb-2">
							<span class="text-sm text-slate-400">Время</span>
							<span class="text-2xl font-bold font-mono {getTimerColor()}">{timeLeft}с</span>
						</div>
						<div class="h-2 rounded-full bg-white/5 overflow-hidden">
							<div class="h-full rounded-full {getTimerBarColor()} transition-all duration-1000" style="width: {getTimerPercent()}%"></div>
						</div>
					</div>

					<!-- Question -->
					<div class="relative overflow-hidden rounded-2xl border border-white/5 bg-white/[0.02] p-8 backdrop-blur-sm">
						<div class="absolute inset-0 bg-gradient-to-br from-purple-500/5 to-blue-500/5"></div>
						<div class="relative z-10">
							{#if question.category}
								<span class="inline-block mb-3 rounded-full bg-purple-500/10 border border-purple-500/20 px-3 py-1 text-xs font-medium text-purple-400">
									{question.category}
								</span>
							{/if}
							<h2 class="text-xl font-bold text-white leading-relaxed">{question.question}</h2>
						</div>
					</div>

					<!-- Options -->
					<div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
						{#each question.options as option, i}
							{@const isSelected = selectedAnswer === i}
							<button
								onclick={() => submitAnswer(i)}
								disabled={selectedAnswer !== null}
								class="group relative overflow-hidden rounded-xl border p-5 text-left transition-all {
									isSelected
										? 'border-blue-500 bg-blue-500/10 shadow-lg shadow-blue-500/10'
										: selectedAnswer !== null
											? 'border-white/5 bg-white/[0.02] opacity-50'
											: 'border-white/10 bg-white/[0.02] hover:border-blue-500/50 hover:bg-blue-500/5 hover:scale-[1.02]'
								} disabled:cursor-default"
							>
								<div class="flex items-center gap-4">
									<div class="flex h-10 w-10 items-center justify-center rounded-lg font-bold text-lg {
										isSelected
											? 'bg-blue-500 text-white'
											: 'bg-white/5 text-slate-400 group-hover:bg-blue-500/20 group-hover:text-blue-400'
									} transition-colors">
										{String.fromCharCode(65 + i)}
									</div>
									<span class="font-medium {isSelected ? 'text-blue-300' : 'text-white'}">{option}</span>
								</div>
							</button>
						{/each}
					</div>

					{#if selectedAnswer !== null}
						<div class="flex items-center justify-center gap-2 text-sm text-blue-400">
							<svg class="w-4 h-4 animate-pulse" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"></path></svg>
							Ответ принят, ожидаем результат...
						</div>
					{/if}
				</div>

			{:else if gamePhase === 'result'}
				<div class="space-y-6">
					<!-- Result card -->
					<div class="relative overflow-hidden rounded-2xl border border-white/5 bg-white/[0.02] p-8 backdrop-blur-sm">
						<div class="absolute inset-0 {lastResult?.correct ? 'bg-gradient-to-br from-green-500/10 to-emerald-500/5' : 'bg-gradient-to-br from-red-500/10 to-rose-500/5'}"></div>
						<div class="relative z-10 text-center">
							<div class="mb-4 text-5xl">{lastResult?.correct ? '✅' : '❌'}</div>
							<h2 class="text-2xl font-bold {lastResult?.correct ? 'text-green-400' : 'text-red-400'}">
								{lastResult?.correct ? 'Правильно!' : 'Неправильно'}
							</h2>
							{#if lastResult}
								<p class="mt-2 text-slate-400">
									Правильный ответ: <span class="font-semibold text-white">{String.fromCharCode(65 + lastResult.correctIndex)}</span>
								</p>
								{#if lastResult.explanation}
									<div class="mt-4 rounded-xl bg-white/5 border border-white/10 p-4 text-left">
										<p class="text-sm text-slate-300">{lastResult.explanation}</p>
									</div>
								{/if}
							{/if}
						</div>
					</div>

					<!-- Leaderboard -->
					<div class="rounded-2xl border border-white/5 bg-white/[0.02] p-6 backdrop-blur-sm">
						<h3 class="mb-4 text-lg font-semibold text-white">Таблица лидеров</h3>
						<div class="space-y-2">
							{#each players.sort((a, b) => b.score - a.score) as player, i}
								<div class="flex items-center justify-between rounded-xl bg-white/[0.02] border border-white/5 p-3">
									<div class="flex items-center gap-3">
										<span class="flex h-8 w-8 items-center justify-center rounded-lg {i === 0 ? 'bg-yellow-500/20 text-yellow-400' : i === 1 ? 'bg-slate-400/20 text-slate-300' : i === 2 ? 'bg-orange-500/20 text-orange-400' : 'bg-white/5 text-slate-500'} font-bold text-sm">
											{i + 1}
										</span>
										<span class="font-medium text-white">{player.name}</span>
									</div>
									<span class="font-bold text-lg text-white">{player.score}</span>
								</div>
							{/each}
						</div>
					</div>
				</div>

			{:else if gamePhase === 'finished'}
				<div class="relative overflow-hidden rounded-2xl border border-white/5 bg-white/[0.02] p-12 text-center backdrop-blur-sm">
					<div class="absolute inset-0 bg-gradient-to-br from-yellow-500/10 to-orange-500/5"></div>
					<div class="relative z-10">
						<div class="mb-6 text-6xl">🏆</div>
						<h2 class="mb-2 text-3xl font-bold text-white">Игра окончена!</h2>
						{#if players.length > 0}
							<p class="text-slate-400">
								Победитель: <span class="font-semibold text-yellow-400">{players[0]?.name}</span>
							</p>
						{/if}

						<!-- Final leaderboard -->
						<div class="mt-8 space-y-3">
							{#each players.sort((a, b) => b.score - a.score).slice(0, 5) as player, i}
								<div class="flex items-center justify-between rounded-xl bg-white/5 border border-white/10 p-4 {i === 0 ? 'ring-2 ring-yellow-500/30' : ''}">
									<div class="flex items-center gap-4">
										<span class="text-2xl">
											{i === 0 ? '🥇' : i === 1 ? '🥈' : i === 2 ? '🥉' : `#${i + 1}`}
										</span>
										<span class="font-semibold text-white">{player.name}</span>
									</div>
									<span class="text-xl font-bold text-white">{player.score} <span class="text-sm text-slate-400">pts</span></span>
								</div>
							{/each}
						</div>

						<div class="mt-8 flex justify-center gap-4">
							<a href="/lobby" class="rounded-xl bg-white/5 border border-white/10 px-6 py-3 font-medium text-white hover:bg-white/10 transition-all">
								В лобби
							</a>
							<button onclick={startGame} class="relative group">
								<div class="absolute inset-0 bg-gradient-to-r from-blue-500 to-purple-600 rounded-xl blur-md opacity-40 group-hover:opacity-70 transition-opacity"></div>
								<div class="relative rounded-xl bg-gradient-to-r from-blue-500 to-purple-600 px-6 py-3 font-medium text-white">
									Играть снова
								</div>
							</button>
						</div>
					</div>
				</div>
			{/if}
		</div>

		<!-- Sidebar: Players -->
		<div>
			<div class="sticky top-24 rounded-2xl border border-white/5 bg-white/[0.02] p-5 backdrop-blur-sm">
				<h3 class="mb-4 text-sm font-semibold uppercase tracking-wider text-slate-400">Игроки</h3>
				<div class="space-y-2">
					{#each players.sort((a, b) => b.score - a.score) as player}
						<div class="flex items-center justify-between rounded-xl bg-white/[0.02] border border-white/5 p-3 transition-all hover:bg-white/[0.05]">
							<div class="flex items-center gap-3">
								<div class="flex h-8 w-8 items-center justify-center rounded-lg bg-gradient-to-br from-blue-500 to-purple-600 text-sm font-bold text-white">
									{player.name.charAt(0).toUpperCase()}
								</div>
								<div>
									<div class="flex items-center gap-2">
										<span class="text-sm font-medium text-white">{player.name}</span>
										{#if player.is_host}
											<span class="rounded bg-yellow-500/20 px-1.5 py-0.5 text-[10px] font-bold text-yellow-400">HOST</span>
										{/if}
									</div>
								</div>
							</div>
							<span class="text-sm font-bold text-white">{player.score}</span>
						</div>
					{/each}
				</div>
			</div>
		</div>
	</div>
</div>
