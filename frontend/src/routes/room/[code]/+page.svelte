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
	};

	type Question = {
		question: string;
		options: string[];
		time_limit: number;
	};

	let players: Player[] = [];
	let question: Question | null = null;
	let selectedAnswer: number | null = null;
	let timeLeft = 0;
	let gamePhase: 'waiting' | 'playing' | 'result' | 'finished' = 'waiting';
	let isHost = false;
	let topic = '';
	let error = '';
	let timerInterval: ReturnType<typeof setInterval> | null = null;
	let lastResult: { correct: boolean; correctIndex: number; explanation: string } | null = null;

	onMount(() => {
		ws.connect();
		ws.on('connected', () => {
			ws.send('join_room', { code: roomCode });
		});
		ws.on('room_joined', (data) => {
			isHost = data.is_host;
			topic = data.topic || '';
		});
		ws.on('players_updated', (data) => {
			players = data.players || [];
		});
		ws.on('question_emitted', (data) => {
			question = data.question;
			timeLeft = data.time_limit || 30;
			selectedAnswer = null;
			gamePhase = 'playing';
			startTimer();
		});
		ws.on('question_result', (data) => {
			gamePhase = 'result';
			stopTimer();
			lastResult = data;
		});
		ws.on('game_over', () => {
			gamePhase = 'finished';
			stopTimer();
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
</script>

<div class="mx-auto max-w-6xl px-6 py-8">
	<!-- Header -->
	<div class="mb-6 flex items-center justify-between">
		<div>
			<div class="font-mono text-sm text-slate-400">Комната: {roomCode}</div>
			<h1 class="text-2xl font-bold text-white">{topic || 'Без темы'}</h1>
		</div>
		<div class="flex items-center gap-4">
			<div class="text-right">
				<div class="text-xs text-slate-400">Игроков</div>
				<div class="text-2xl font-bold text-white">{players.length}</div>
			</div>
		</div>
	</div>

	{#if error}
		<div class="mb-4 rounded-lg bg-red-500/10 p-4 text-red-400">{error}</div>
	{/if}

	<div class="grid grid-cols-1 gap-6 lg:grid-cols-3">
		<!-- Main area -->
		<div class="lg:col-span-2">
			{#if gamePhase === 'waiting'}
				<div class="rounded-xl border border-slate-700 bg-slate-800 p-8 text-center">
					<div class="mb-4 text-6xl">⏳</div>
					<h2 class="mb-2 text-xl font-semibold text-white">Ожидание игроков</h2>
					<p class="mb-6 text-slate-400">Поделитесь кодом комнаты: <span class="font-mono text-blue-400">{roomCode}</span></p>

					{#if isHost}
						<button
							on:click={startGame}
							disabled={players.length < 1}
							class="rounded-lg bg-green-600 px-8 py-3 font-semibold text-white transition hover:bg-green-700 disabled:cursor-not-allowed disabled:opacity-50"
						>
							▶ Начать игру
						</button>
						{#if players.length < 1}
							<p class="mt-2 text-sm text-slate-400">Нужен хотя бы 1 игрок</p>
						{/if}
					{:else}
						<div class="text-slate-400">Ожидание хоста...</div>
					{/if}
				</div>

			{:else if gamePhase === 'playing' && question}
				<div class="rounded-xl border border-slate-700 bg-slate-800 p-6">
					<!-- Timer -->
					<div class="mb-6">
						<div class="mb-2 flex items-center justify-between">
							<span class="text-sm text-slate-400">Осталось времени</span>
							<span class="font-mono text-2xl font-bold {getTimerColor()}">{timeLeft}с</span>
						</div>
						<div class="h-2 overflow-hidden rounded-full bg-slate-700">
							<div
								class="h-full bg-blue-500 transition-all duration-1000"
								style="width: {getTimerPercent()}%"
							></div>
						</div>
					</div>

					<!-- Question -->
					<h2 class="mb-6 text-xl font-semibold text-white">{question.question}</h2>

					<!-- Options -->
					<div class="space-y-3">
						{#each question.options as option, i}
							<button
								on:click={() => submitAnswer(i)}
								disabled={selectedAnswer !== null}
								class="w-full rounded-lg border-2 p-4 text-left transition
									{selectedAnswer === i
									? 'border-blue-500 bg-blue-500/20 text-white'
									: selectedAnswer !== null
									? 'border-slate-600 bg-slate-700 opacity-50'
									: 'border-slate-600 bg-slate-700 text-slate-200 hover:border-blue-500 hover:bg-slate-600'}"
							>
								<span class="mr-3 inline-flex h-8 w-8 items-center justify-center rounded-full bg-slate-600 font-bold">
									{String.fromCharCode(65 + i)}
								</span>
								{option}
							</button>
						{/each}
					</div>
				</div>

			{:else if gamePhase === 'result' && lastResult}
				<div class="rounded-xl border-2 border-blue-500/30 bg-slate-800 p-6">
					<div class="mb-4 text-center">
						<div class="text-5xl">
							{lastResult.correct ? '✅' : '❌'}
						</div>
						<div class="mt-2 text-2xl font-bold text-white">
							{lastResult.correct ? 'Правильно!' : 'Неправильно'}
						</div>
					</div>

					<div class="rounded-lg bg-slate-700 p-4">
						<div class="mb-2 text-sm text-slate-400">Правильный ответ:</div>
						<div class="text-lg font-semibold text-green-400">
							{question?.options?.[lastResult.correctIndex] || '—'}
						</div>
					</div>

					{#if lastResult.explanation}
						<div class="mt-4 rounded-lg bg-slate-700/50 p-4">
							<div class="mb-1 text-sm text-slate-400">Пояснение:</div>
							<div class="text-slate-200">{lastResult.explanation}</div>
						</div>
					{/if}
				</div>

			{:else if gamePhase === 'finished'}
				<div class="rounded-xl border-2 border-yellow-500/30 bg-slate-800 p-8 text-center">
					<div class="mb-4 text-6xl">🏆</div>
					<h2 class="mb-2 text-2xl font-bold text-white">Игра окончена!</h2>
					<p class="text-slate-400">Посмотрите результаты</p>

					<div class="mt-6 space-y-2">
						{#each players.sort((a, b) => b.score - a.score).slice(0, 3) as player, i}
							<div class="flex items-center justify-between rounded-lg bg-slate-700 p-4">
								<div class="flex items-center gap-3">
									<span class="text-2xl">
										{i === 0 ? '🥇' : i === 1 ? '🥈' : '🥉'}
									</span>
									<span class="font-semibold text-white">{player.name}</span>
								</div>
								<span class="text-xl font-bold text-yellow-400">{player.score}</span>
							</div>
						{/each}
					</div>

					<div class="mt-6 flex justify-center gap-3">
						<a href="/lobby" class="rounded-lg bg-blue-600 px-6 py-2 font-semibold text-white hover:bg-blue-700">
							В лобби
						</a>
						<button
							on:click={() => ws.send('play_again', {})}
							class="rounded-lg bg-green-600 px-6 py-2 font-semibold text-white hover:bg-green-700"
						>
							Ещё раз
						</button>
					</div>
				</div>

			{:else}
				<div class="rounded-xl border border-slate-700 bg-slate-800 p-8 text-center text-slate-400">
					Подключение...
				</div>
			{/if}
		</div>

		<!-- Sidebar: players -->
		<div class="rounded-xl border border-slate-700 bg-slate-800 p-4">
			<h3 class="mb-3 text-sm font-semibold uppercase tracking-wide text-slate-400">Игроки</h3>
			<div class="space-y-2">
				{#each players as player}
					<div class="flex items-center justify-between rounded-lg bg-slate-700 p-3">
						<div class="flex items-center gap-2">
							{#if player.is_host}
								<span class="text-yellow-400">👑</span>
							{/if}
							<span class="font-medium text-white">{player.name}</span>
						</div>
						<span class="font-mono text-sm text-blue-400">{player.score}</span>
					</div>
				{/each}
				{#if players.length === 0}
					<div class="py-4 text-center text-sm text-slate-500">Пока никого нет</div>
				{/if}
			</div>
		</div>
	</div>
</div>
