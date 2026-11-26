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
	let gamePhase: 'loading' | 'waiting' | 'playing' | 'result' | 'finished' = $state('loading');
	let isHost = $state(false);
	let topic = $state('');
	let mode = $state('');
	let error = $state('');
	let timerInterval: ReturnType<typeof setInterval> | null = null;
	let lastResult: { correct: boolean; correctIndex: number; explanation: string; player_answers?: any[] } | null = $state(null);
	let finalLeaderboard: any[] | null = $state(null);
	let showConfetti = $state(false);
	let questionNumber = $state(0);
	let totalQuestions = $state(0);
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
			questionNumber = (data.question_index || 0) + 1;
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
			// Show confetti if correct
			if (data.correct) {
				showConfetti = true;
				setTimeout(() => showConfetti = false, 2000);
			}
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
			classic: '🎯 Классика',
			ffa: '⚔️ FFA',
			team: '👥 Команды',
			lives: '❤️ На выживание',
			coop: '🤝 Кооператив',
			svoyaigra: '🎲 Своя Игра',
		};
		return labels[m] || m;
	}

	function getCircleProgress(): number {
		if (!question) return 0;
		return (timeLeft / question.time_limit) * 100;
	}
</script>

<!-- Confetti effect -->
{#if showConfetti}
	<div class="fixed inset-0 pointer-events-none z-50 overflow-hidden">
		<div class="absolute top-0 left-1/4 w-2 h-2 bg-yellow-400 rounded-full animate-confetti-1"></div>
		<div class="absolute top-0 left-1/2 w-2 h-2 bg-blue-400 rounded-full animate-confetti-2"></div>
		<div class="absolute top-0 right-1/4 w-2 h-2 bg-purple-400 rounded-full animate-confetti-3"></div>
		<div class="absolute top-0 left-1/3 w-2 h-2 bg-green-400 rounded-full animate-confetti-4"></div>
		<div class="absolute top-0 right-1/3 w-2 h-2 bg-pink-400 rounded-full animate-confetti-5"></div>
	</div>
{/if}

<div class="mx-auto max-w-7xl px-4 py-6 sm:px-6 sm:py-8">
	{#if gamePhase === 'loading'}
		<!-- Skeleton loader -->
		<div class="space-y-6">
			<div class="h-8 w-48 animate-pulse rounded-lg bg-white/5"></div>
			<div class="h-12 w-96 animate-pulse rounded-lg bg-white/5"></div>
			<div class="grid grid-cols-1 gap-6 lg:grid-cols-3">
				<div class="lg:col-span-2 space-y-6">
					<div class="h-32 animate-pulse rounded-2xl bg-white/5"></div>
					<div class="h-64 animate-pulse rounded-2xl bg-white/5"></div>
					<div class="grid grid-cols-2 gap-3">
						<div class="h-20 animate-pulse rounded-xl bg-white/5"></div>
						<div class="h-20 animate-pulse rounded-xl bg-white/5"></div>
						<div class="h-20 animate-pulse rounded-xl bg-white/5"></div>
						<div class="h-20 animate-pulse rounded-xl bg-white/5"></div>
					</div>
				</div>
				<div class="h-96 animate-pulse rounded-2xl bg-white/5"></div>
			</div>
		</div>
	{:else}
		<!-- Header -->
		<div class="mb-6 flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
			<div>
				<div class="flex items-center gap-3">
					<button
						onclick={copyCode}
						class="group relative font-mono text-sm text-slate-400 tracking-wider bg-white/5 rounded-lg px-3 py-1.5 border border-white/10 hover:border-blue-500/50 hover:bg-blue-500/5 transition-all"
					>
						{roomCode}
						{#if copiedCode}
							<span class="absolute -top-8 left-1/2 -translate-x-1/2 rounded-lg bg-green-500 px-2 py-1 text-xs text-white whitespace-nowrap">
								Скопировано!
							</span>
						{/if}
					</button>
					{#if mode}
						<span class="text-sm text-slate-400">{getModeLabel(mode)}</span>
					{/if}
				</div>
				<h1 class="mt-2 text-xl sm:text-2xl font-bold text-white">{topic || 'Без темы'}</h1>
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
				<svg class="w-5 h-5 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>
				</svg>
				{error}
			</div>
		{/if}

		<div class="grid grid-cols-1 gap-6 lg:grid-cols-3">
			<!-- Main area -->
			<div class="lg:col-span-2">
				{#if gamePhase === 'waiting'}
					<div class="relative overflow-hidden rounded-2xl border border-white/5 bg-white/[0.02] p-8 sm:p-12 text-center backdrop-blur-sm">
						<div class="absolute inset-0 bg-gradient-to-br from-blue-500/5 to-purple-500/5"></div>
						<div class="relative z-10">
							<div class="mb-6 text-6xl animate-bounce-slow">⏳</div>
							<h2 class="mb-2 text-2xl font-bold text-white">Ожидание игроков</h2>
							<p class="mb-6 text-slate-400">
								Поделитесь кодом комнаты:
								<button
									onclick={copyCode}
									class="font-mono text-blue-400 hover:text-blue-300 transition-colors"
								>
									{roomCode}
								</button>
							</p>

							<!-- Player list preview -->
							{#if players.length > 0}
								<div class="mb-6 flex flex-wrap justify-center gap-2">
									{#each players as player}
										<div class="flex items-center gap-2 rounded-full bg-white/5 border border-white/10 px-3 py-1.5">
											<div class="flex h-6 w-6 items-center justify-center rounded-full bg-gradient-to-br from-blue-500 to-purple-600 text-xs font-bold text-white">
												{player.name.charAt(0).toUpperCase()}
											</div>
											<span class="text-sm text-white">{player.name}</span>
											{#if player.is_host}
												<span class="rounded bg-yellow-500/20 px-1.5 py-0.5 text-[10px] font-bold text-yellow-400">HOST</span>
											{/if}
										</div>
									{/each}
								</div>
							{/if}

							{#if isHost && players.length > 0}
								<button
									onclick={startGame}
									class="relative group"
								>
									<div class="absolute inset-0 bg-gradient-to-r from-green-500 to-emerald-600 rounded-xl blur-lg opacity-40 group-hover:opacity-70 transition-opacity"></div>
									<div class="relative flex items-center gap-3 rounded-xl bg-gradient-to-r from-green-500 to-emerald-600 px-8 py-4 text-lg font-semibold text-white shadow-xl">
										<svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
											<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z"></path>
										</svg>
										Начать игру
									</div>
								</button>
							{:else if !isHost}
								<div class="flex items-center justify-center gap-2 text-sm text-slate-500">
									<svg class="w-4 h-4 animate-pulse" fill="none" stroke="currentColor" viewBox="0 0 24 24">
										<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"></path>
									</svg>
									Ожидаем, пока хост начнёт игру...
								</div>
							{:else}
								<p class="text-sm text-slate-500">Нужен хотя бы 1 игрок для старта</p>
							{/if}
						</div>
					</div>

				{:else if gamePhase === 'playing' && question}
					<div class="space-y-6">
						<!-- Question progress -->
						{#if totalQuestions > 0}
							<div class="flex items-center justify-between text-sm text-slate-400">
								<span>Вопрос {questionNumber} из {totalQuestions}</span>
							</div>
						{/if}

						<!-- Timer with circular progress -->
						<div class="relative overflow-hidden rounded-xl border border-white/5 bg-white/[0.02] p-4 backdrop-blur-sm">
							<div class="flex items-center gap-4">
								<!-- Circular timer -->
								<div class="relative flex-shrink-0">
									<svg class="w-16 h-16 -rotate-90 transform">
										<circle
											cx="32"
											cy="32"
											r="28"
											fill="none"
											stroke="rgba(255,255,255,0.05)"
											stroke-width="4"
										/>
										<circle
											cx="32"
											cy="32"
											r="28"
											fill="none"
											stroke="url(#timerGradient)"
											stroke-width="4"
											stroke-linecap="round"
											stroke-dasharray="{2 * Math.PI * 28}"
											stroke-dashoffset="{2 * Math.PI * 28 * (1 - getCircleProgress() / 100)}"
											class="transition-all duration-1000"
										/>
										<defs>
											<linearGradient id="timerGradient" x1="0%" y1="0%" x2="100%" y2="100%">
												{#if timeLeft > 15}
													<stop offset="0%" stop-color="#10b981" />
													<stop offset="100%" stop-color="#059669" />
												{:else if timeLeft > 5}
													<stop offset="0%" stop-color="#f59e0b" />
													<stop offset="100%" stop-color="#d97706" />
												{:else}
													<stop offset="0%" stop-color="#ef4444" />
													<stop offset="100%" stop-color="#dc2626" />
												{/if}
											</linearGradient>
										</defs>
									</svg>
									<div class="absolute inset-0 flex items-center justify-center">
										<span class="text-xl font-bold font-mono {getTimerColor()}">{timeLeft}</span>
									</div>
								</div>

								<!-- Timer bar -->
								<div class="flex-1">
									<div class="h-2 rounded-full bg-white/5 overflow-hidden">
										<div class="h-full rounded-full bg-gradient-to-r {getTimerBarColor()} transition-all duration-1000" style="width: {getTimerPercent()}%"></div>
									</div>
									<div class="mt-2 text-sm text-slate-400">Осталось времени</div>
								</div>
							</div>
						</div>

						<!-- Question -->
						<div class="relative overflow-hidden rounded-2xl border border-white/5 bg-white/[0.02] p-6 sm:p-8 backdrop-blur-sm animate-fade-in">
							<div class="absolute inset-0 bg-gradient-to-br from-purple-500/5 to-blue-500/5"></div>
							<div class="relative z-10">
								{#if question.category}
									<span class="inline-block mb-3 rounded-full bg-purple-500/10 border border-purple-500/20 px-3 py-1 text-xs font-medium text-purple-400">
										{question.category}
									</span>
								{/if}
								<h2 class="text-lg sm:text-xl font-bold text-white leading-relaxed">{question.question}</h2>
							</div>
						</div>

						<!-- Options -->
						<div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
							{#each question.options as option, i}
								{@const isSelected = selectedAnswer === i}
								<button
									onclick={() => submitAnswer(i)}
									disabled={selectedAnswer !== null}
									class="group relative overflow-hidden rounded-xl border p-4 sm:p-5 text-left transition-all {
										isSelected
											? 'border-blue-500 bg-blue-500/10 shadow-lg shadow-blue-500/10 scale-[1.02]'
											: selectedAnswer !== null
												? 'border-white/5 bg-white/[0.02] opacity-50'
												: 'border-white/10 bg-white/[0.02] hover:border-blue-500/50 hover:bg-blue-500/5 hover:scale-[1.02]'
									} disabled:cursor-default"
									style="animation-delay: {i * 100}ms"
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
									{#if isSelected}
										<div class="absolute top-2 right-2">
											<svg class="w-5 h-5 text-blue-400" fill="currentColor" viewBox="0 0 20 20">
												<path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd" />
											</svg>
										</div>
									{/if}
								</button>
							{/each}
						</div>

						{#if selectedAnswer !== null}
							<div class="flex items-center justify-center gap-2 text-sm text-blue-400">
								<svg class="w-4 h-4 animate-pulse" fill="none" stroke="currentColor" viewBox="0 0 24 24">
									<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"></path>
								</svg>
								Ответ принят, ожидаем результат...
							</div>
						{/if}
					</div>

				{:else if gamePhase === 'result'}
					<div class="space-y-6 animate-fade-in">
						<!-- Result card -->
						<div class="relative overflow-hidden rounded-2xl border border-white/5 bg-white/[0.02] p-6 sm:p-8 backdrop-blur-sm">
							<div class="absolute inset-0 {lastResult?.correct ? 'bg-gradient-to-br from-green-500/10 to-emerald-500/5' : 'bg-gradient-to-br from-red-500/10 to-rose-500/5'}"></div>
							<div class="relative z-10 text-center">
								<div class="mb-4 text-6xl animate-bounce-slow">{lastResult?.correct ? '✅' : '❌'}</div>
								<h2 class="text-2xl font-bold {lastResult?.correct ? 'text-green-400' : 'text-red-400'}">
									{lastResult?.correct ? 'Правильно!' : 'Неправильно'}
								</h2>
								{#if lastResult}
									<p class="mt-2 text-slate-400">
										Правильный ответ: <span class="font-semibold text-white">{String.fromCharCode(65 + lastResult.correctIndex)}</span>
									</p>
									{#if lastResult.explanation}
										<div class="mt-4 rounded-xl bg-white/5 border border-white/10 p-4 text-left">
											<div class="flex items-start gap-3">
												<svg class="w-5 h-5 text-blue-400 flex-shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
													<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>
												</svg>
												<p class="text-sm text-slate-300">{lastResult.explanation}</p>
											</div>
										</div>
									{/if}
								{/if}
							</div>
						</div>

						<!-- Leaderboard -->
						<div class="rounded-2xl border border-white/5 bg-white/[0.02] p-6 backdrop-blur-sm">
							<h3 class="mb-4 flex items-center gap-2 text-lg font-semibold text-white">
								<svg class="w-5 h-5 text-yellow-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
									<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"></path>
								</svg>
								Таблица лидеров
							</h3>
							<div class="space-y-2">
								{#each players.sort((a, b) => b.score - a.score) as player, i}
									<div class="flex items-center justify-between rounded-xl bg-white/[0.02] border border-white/5 p-3 transition-all hover:bg-white/[0.05]">
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
					<div class="relative overflow-hidden rounded-2xl border border-white/5 bg-white/[0.02] p-8 sm:p-12 text-center backdrop-blur-sm animate-fade-in">
						<div class="absolute inset-0 bg-gradient-to-br from-yellow-500/10 to-orange-500/5"></div>
						<div class="relative z-10">
							<div class="mb-6 text-7xl animate-bounce-slow">🏆</div>
							<h2 class="mb-2 text-3xl font-bold text-white">Игра окончена!</h2>
							{#if players.length > 0}
								<p class="text-lg text-slate-400">
									Победитель: <span class="font-semibold text-yellow-400">{players[0]?.name}</span>
								</p>
							{/if}

							<!-- Final leaderboard -->
							<div class="mt-8 space-y-3">
								{#each players.sort((a, b) => b.score - a.score).slice(0, 5) as player, i}
									<div class="flex items-center justify-between rounded-xl bg-white/5 border border-white/10 p-4 {i === 0 ? 'ring-2 ring-yellow-500/30' : ''} transition-all hover:bg-white/10">
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

							<div class="mt-8 flex flex-col sm:flex-row justify-center gap-4">
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
					<h3 class="mb-4 flex items-center gap-2 text-sm font-semibold uppercase tracking-wider text-slate-400">
						<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z"></path>
						</svg>
						Игроки ({players.length})
					</h3>
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
	{/if}
</div>

<style>
	@keyframes confetti-1 {
		0% { transform: translateY(0) rotate(0deg); opacity: 1; }
		100% { transform: translateY(100vh) rotate(720deg); opacity: 0; }
	}
	@keyframes confetti-2 {
		0% { transform: translateY(0) rotate(0deg); opacity: 1; }
		100% { transform: translateY(100vh) rotate(540deg); opacity: 0; }
	}
	@keyframes confetti-3 {
		0% { transform: translateY(0) rotate(0deg); opacity: 1; }
		100% { transform: translateY(100vh) rotate(630deg); opacity: 0; }
	}
	@keyframes confetti-4 {
		0% { transform: translateY(0) rotate(0deg); opacity: 1; }
		100% { transform: translateY(100vh) rotate(450deg); opacity: 0; }
	}
	@keyframes confetti-5 {
		0% { transform: translateY(0) rotate(0deg); opacity: 1; }
		100% { transform: translateY(100vh) rotate(810deg); opacity: 0; }
	}
	@keyframes bounce-slow {
		0%, 100% { transform: translateY(0); }
		50% { transform: translateY(-10px); }
	}
	@keyframes fade-in {
		from { opacity: 0; transform: translateY(10px); }
		to { opacity: 1; transform: translateY(0); }
	}

	:global(.animate-confetti-1) { animation: confetti-1 2s ease-out forwards; }
	:global(.animate-confetti-2) { animation: confetti-2 2s ease-out 0.1s forwards; }
	:global(.animate-confetti-3) { animation: confetti-3 2s ease-out 0.2s forwards; }
	:global(.animate-confetti-4) { animation: confetti-4 2s ease-out 0.3s forwards; }
	:global(.animate-confetti-5) { animation: confetti-5 2s ease-out 0.4s forwards; }
	:global(.animate-bounce-slow) { animation: bounce-slow 2s ease-in-out infinite; }
	:global(.animate-fade-in) { animation: fade-in 0.5s ease-out; }
</style>
