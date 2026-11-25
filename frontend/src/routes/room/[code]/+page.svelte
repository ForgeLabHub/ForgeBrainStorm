<script lang="ts">
	import { page } from '$app/stores';
	import { api } from '$lib/api';
	import { ws } from '$lib/ws';
	import { onMount, onDestroy } from 'svelte';
	import { goto } from '$app/navigation';
	import Icon from '$lib/components/Icon.svelte';
	import { currentUser } from '$lib/stores/auth';
	import { copyText, initial, plural } from '$lib/ui';

	type Player = { name: string; score: number; is_host: boolean; is_ready: boolean };
	type Question = { question: string; options: string[]; time_limit: number };
	type Verdict = { correct: boolean | null; correctIndex: number; explanation: string };

	$: roomCode = ($page.params.code ?? '').toUpperCase();

	let players: Player[] = [];
	let question: Question | null = null;
	let qIndex = 0;
	let qTotal = 0;
	let selected: number | null = null;
	let phase: 'waiting' | 'preparing' | 'playing' | 'result' | 'finished' = 'waiting';
	let questionSource = '';
	let preparingCount = 0;
	let isHost = false;
	let topic = '';
	let error = '';
	let connected = false;
	let verdict: Verdict | null = null;
	let deadline = 0;
	let timeLimit = 30;
	let now = Date.now();
	let copied = false;

	let ticker: ReturnType<typeof setInterval> | null = null;
	let copyTimer: ReturnType<typeof setTimeout> | null = null;

	$: timeLeft = Math.max(0, Math.ceil((deadline - now) / 1000));
	$: timePct = Math.max(0, Math.min(100, ((deadline - now) / (timeLimit * 1000)) * 100));
	$: lowTime = phase === 'playing' && timeLeft <= 5;
	$: ranked = [...players].sort((a, b) => b.score - a.score);
	$: myName = $currentUser?.username ?? '';
	$: myScore = players.find((player) => player.name === myName)?.score ?? 0;

	function onPlayers(data: any) {
		players = data.players || [];
		// The host restarting the game resets every score to zero.
		if (phase === 'finished' && players.length > 0 && players.every((player) => player.score === 0)) {
			questionSource = '';
			phase = 'waiting';
			question = null;
			verdict = null;
			selected = null;
			qIndex = 0;
			qTotal = 0;
		}
	}

	function onQuestion(data: any) {
		question = data.question;
		questionSource = typeof data.source === 'string' ? data.source : '';
		timeLimit = data.time_limit || data.question?.time_limit || 30;
		qIndex = typeof data.index === 'number' ? data.index : 0;
		qTotal = typeof data.total === 'number' ? data.total : 0;
		deadline = Date.now() + timeLimit * 1000;
		selected = null;
		verdict = null;
		error = '';
		phase = 'playing';
	}

	function onResult(data: any) {
		const correctIndex =
			typeof data.correctIndex === 'number' ? data.correctIndex : (data.correct_index ?? 0);
		verdict = {
			correct: selected === null ? null : selected === correctIndex,
			correctIndex,
			explanation: data.explanation || ''
		};
		phase = 'result';
	}

	function onGameOver() {
		phase = 'finished';
		verdict = null;
	}

	const handlers: Record<string, (data: any) => void> = {
		connected: () => {
			connected = true;
			ws.send('join_room', { code: roomCode });
		},
		disconnected: () => {
			connected = false;
		},
		room_joined: (data) => {
			isHost = Boolean(data.is_host);
			if (data.topic) topic = data.topic;
		},
		players_updated: onPlayers,
		game_preparing: () => {
			phase = 'preparing';
			verdict = null;
			selected = null;
			error = '';
		},
		question_emitted: onQuestion,
		question_result: onResult,
		game_over: onGameOver,
		error: (data) => {
			error = data.message || 'Что-то пошло не так';
		}
	};

	function onKeydown(event: KeyboardEvent) {
		if (phase !== 'playing' || !question || event.metaKey || event.ctrlKey || event.altKey) return;
		const index = Number.parseInt(event.key, 10);
		if (Number.isNaN(index) || index < 1 || index > question.options.length) return;
		event.preventDefault();
		submitAnswer(index - 1);
	}

	onMount(() => {
		api
			.getRoom(roomCode)
			.then((response) => {
				if (response.room) topic = response.room.topic;
			})
			.catch(() => {});

		for (const [event, handler] of Object.entries(handlers)) ws.on(event, handler);
		window.addEventListener('keydown', onKeydown);
		ws.connect();
		ticker = setInterval(() => (now = Date.now()), 200);

		return () => {
			for (const [event, handler] of Object.entries(handlers)) ws.off(event, handler);
			window.removeEventListener('keydown', onKeydown);
		};
	});

	onDestroy(() => {
		if (ticker) clearInterval(ticker);
		if (copyTimer) clearTimeout(copyTimer);
		ws.disconnect();
	});

	function submitAnswer(index: number) {
		if (phase !== 'playing' || selected !== null) return;
		selected = index;
		ws.send('submit_answer', { answer_index: index });
	}

	function startGame() {
		error = '';
		ws.send('start_game', {});
	}

	function playAgain() {
		ws.send('play_again', {});
	}

	async function copyCode() {
		if (await copyText(roomCode)) {
			copied = true;
			if (copyTimer) clearTimeout(copyTimer);
			copyTimer = setTimeout(() => (copied = false), 1800);
		}
	}

	function letter(index: number): string {
		return String.fromCharCode(65 + index);
	}
</script>

<div class="flex min-h-[100dvh] flex-col">
	<header class="sticky top-0 z-30 border-b border-line/60 bg-ink-950/85 backdrop-blur-md">
		<div class="mx-auto flex h-14 w-full max-w-5xl items-center gap-1.5 px-3 sm:px-6">
			<a href="/lobby" class="btn-icon h-10 w-10" aria-label="Вернуться в комнаты">
				<Icon name="back" size={18} />
			</a>

			<button
				on:click={copyCode}
				class="flex min-h-[40px] items-center gap-2 rounded-[11px] px-2 text-chalk transition-colors hover:bg-ink-900"
				title="Скопировать код комнаты"
			>
				<span class="digits text-sm tracking-[0.16em]">{copied ? 'Скопировано' : roomCode}</span>
				<Icon name={copied ? 'check' : 'copy'} size={15} />
			</button>

			{#if !connected}
				<span class="chip animate-pulse-soft text-amber">Переподключение</span>
			{/if}

			<div class="ml-auto flex items-center gap-2">
				{#if qTotal > 0 && (phase === 'playing' || phase === 'result')}
					<span class="chip hidden sm:inline-flex">Вопрос {qIndex + 1} из {qTotal}</span>
				{/if}
				{#if phase === 'playing'}
					<span class="digits text-lg {lowTime ? 'text-coral' : 'text-amber'}">{timeLeft} с</span>
				{/if}
			</div>
		</div>

		{#if phase === 'playing'}
			<div class="h-1 w-full bg-ink-800">
				<div
					class="h-full transition-[width] duration-200 ease-linear {lowTime ? 'bg-coral' : 'bg-amber'}"
					style="width: {timePct}%"
				></div>
			</div>
		{/if}
	</header>

	<main class="mx-auto w-full max-w-5xl flex-1 px-3 py-4 sm:px-6 sm:py-6">
		{#if error}
			<p class="mb-3 rounded-[12px] border border-coral/30 bg-coral/10 px-4 py-2.5 text-sm text-coral" role="alert">
				{error}
			</p>
		{/if}

		<div class="grid gap-4 lg:grid-cols-[minmax(0,1fr)_250px] lg:items-center">
			<div class="min-w-0 lg:flex lg:min-h-[calc(100vh-170px)] lg:flex-col lg:justify-center">
				{#if phase === 'waiting'}
					<section class="card card-pad animate-rise text-center">
						<p class="text-sm text-dim">{topic || 'Комната'}</p>
						<button
							on:click={copyCode}
							class="digits mx-auto mt-2 block rounded-[16px] border border-line bg-ink-900 px-5 py-3 text-[30px] tracking-[0.22em] text-chalk transition-colors hover:border-amber/60 sm:text-[38px]"
							title="Скопировать код"
						>
							{roomCode}
						</button>
						<p class="mt-2 text-sm text-dim">
							{copied ? 'Код скопирован — отправьте его друзьям' : 'Нажмите на код, чтобы скопировать'}
						</p>

						<div class="mx-auto mt-6 max-w-sm space-y-1.5 text-left text-sm text-dim">
							<p class="flex gap-2.5"><span class="digits text-amber">1</span> Отправьте код друзьям</p>
							<p class="flex gap-2.5"><span class="digits text-amber">2</span> Дождитесь их в списке игроков</p>
							<p class="flex gap-2.5"><span class="digits text-amber">3</span> Хост нажимает «Начать игру»</p>
						</div>

						<div class="mt-6">
							{#if isHost}
								<button on:click={startGame} disabled={players.length < 1} class="btn btn-primary w-full sm:w-auto sm:px-8">
									<Icon name="play" size={16} />
									Начать игру
								</button>
								{#if players.length < 1}
									<p class="mt-2 text-xs text-dim">Нужен хотя бы один игрок в комнате</p>
								{/if}
							{:else}
								<p class="text-sm text-dim">Ждём, когда хост начнёт игру</p>
							{/if}
						</div>
					</section>
				{:else if phase === 'preparing'}
					<section class="card card-pad animate-rise text-center flex min-h-[45vh] flex-col justify-center">
						<div class="mx-auto grid h-14 w-14 place-items-center rounded-full border border-amber/40 bg-amber/10">
							<span class="animate-pulse text-amber"><Icon name="sparkle" size={24} /></span>
						</div>
						<p class="mt-4 font-display text-lg text-chalk">Готовим вопросы</p>
						<p class="mt-1.5 text-sm text-dim">
							{topic} · {preparingCount || 5} вопросов · несколько секунд
						</p>
						<p class="mt-4 text-xs text-dim">Вопросы придумывает модель по вашей теме, поэтому они не повторяются между играми.</p>
					</section>
				{:else if phase === 'playing' && question}
					<section class="animate-rise">
						<div class="mb-3 flex items-center justify-between gap-3">
							<p class="flex min-w-0 items-center gap-2 text-sm text-dim">
								<span class="truncate">{topic}</span>
								{#if questionSource}
									<span class="chip shrink-0 py-0.5 text-[11px] {questionSource === 'fallback' ? '' : 'border-amber/40 text-amber'}">
										{questionSource === 'fallback' ? 'офлайн-банк' : 'AI · ' + questionSource.replace('ai:', '')}
									</span>
								{/if}
							</p>
							<span class="chip shrink-0 sm:hidden">Вопрос {qIndex + 1} из {qTotal}</span>
						</div>

						<h1 class="mb-5 font-display text-[21px] leading-snug text-chalk sm:text-[28px] lg:text-[31px]">
							{question.question}
						</h1>

						<div class="grid gap-2 sm:grid-cols-2">
							{#each question.options as option, i}
								<button
									on:click={() => submitAnswer(i)}
									disabled={selected !== null}
									class="flex min-h-[62px] items-center gap-3 rounded-[16px] border px-3.5 py-3 text-left text-[15px] transition-colors sm:text-base lg:text-[17px] {selected === i
										? 'border-amber bg-ink-800 text-chalk'
										: selected !== null
											? 'border-line/60 bg-ink-850 text-dim opacity-60'
											: 'border-line bg-ink-850 text-chalk hover:border-amber/60 hover:bg-ink-800'}"
								>
									<span
										class="grid h-8 w-8 shrink-0 place-items-center rounded-[10px] bg-ink-700 text-xs font-semibold {selected === i
											? 'text-amber'
											: 'text-dim'}"
									>
										{letter(i)}
									</span>
									<span class="min-w-0">{option}</span>
								</button>
							{/each}
						</div>

						<p class="mt-3 text-sm text-dim">
							{selected === null
								? 'Выберите ответ — изменить его будет нельзя.'
								: 'Ответ принят. Ждём остальных игроков…'}
						</p>
					</section>
				{:else if phase === 'result' && verdict}
					<section class="card card-pad animate-rise {verdict.correct === false ? 'border-coral/40' : 'border-mint/40'}">
						<p
							class="font-display text-xl sm:text-2xl {verdict.correct === true
								? 'text-mint'
								: verdict.correct === false
									? 'text-coral'
									: 'text-dim'}"
						>
							{#if verdict.correct === true}
								Верно
							{:else if verdict.correct === false}
								Мимо
							{:else}
								Время вышло
							{/if}
						</p>

						<div class="mt-4 rounded-[14px] border border-line/70 bg-ink-900 px-4 py-3">
							<p class="text-xs text-dim">Правильный ответ</p>
							<p class="mt-1 font-medium text-chalk">{question?.options?.[verdict.correctIndex] ?? '—'}</p>
						</div>

						{#if verdict.explanation}
							<div class="mt-2 rounded-[14px] border border-line/70 bg-ink-900 px-4 py-3">
								<p class="text-xs text-dim">Почему</p>
								<p class="mt-1 text-sm leading-relaxed text-chalk">{verdict.explanation}</p>
							</div>
						{/if}

						<p class="mt-4 text-sm text-dim">
							{qIndex + 1 < qTotal ? 'Следующий вопрос — через пару секунд.' : 'Это был последний вопрос, считаем итоги.'}
						</p>
					</section>
				{:else if phase === 'finished'}
					<section class="animate-rise space-y-4">
						<div class="card card-pad">
							<p class="font-display text-xl text-chalk sm:text-2xl">Игра окончена</p>
							<p class="mt-1 text-sm text-dim">{topic}</p>

							{#if ranked.length > 0}
								<ol class="mt-5 space-y-1.5">
									{#each ranked as player, i}
										<li
											class="flex items-center gap-3 rounded-[14px] border px-3.5 py-3 {player.name === myName
												? 'border-amber/60 bg-ink-800'
												: 'border-line/70 bg-ink-900'}"
										>
											<span class="digits w-6 text-center text-sm text-dim">{i + 1}</span>
											<span class="grid h-8 w-8 place-items-center rounded-full bg-ink-700 text-xs font-semibold text-chalk">
												{initial(player.name)}
											</span>
											<span class="min-w-0 flex-1 truncate text-sm font-medium text-chalk">
												{player.name}
												{#if player.is_host}<span class="ml-1 text-xs text-dim">хост</span>{/if}
											</span>
											<span class="digits text-base text-chalk">{player.score}</span>
										</li>
									{/each}
								</ol>
							{:else}
								<p class="mt-4 text-sm text-dim">В игре не было игроков.</p>
							{/if}

							<div class="mt-5 flex flex-col gap-2 sm:flex-row">
								{#if isHost}
									<button on:click={playAgain} class="btn btn-primary flex-1">Играть снова</button>
								{/if}
								<button on:click={() => goto('/lobby')} class="btn btn-quiet flex-1">В комнаты</button>
							</div>
						</div>
					</section>
				{:else}
					<section class="card card-pad text-center text-sm text-dim">Подключаемся к комнате…</section>
				{/if}
			</div>

			<aside class="card card-pad lg:sticky lg:top-20 {phase === 'waiting' ? '' : 'hidden lg:block'}">
				<div class="mb-3 flex items-baseline justify-between">
					<h2 class="text-sm font-semibold text-chalk">Игроки</h2>
					<span class="text-xs text-dim">
						{players.length} {plural(players.length, 'человек', 'человека', 'человек')}
					</span>
				</div>

				{#if phase !== 'waiting'}
					<p class="mb-3 flex items-center justify-between rounded-[12px] border border-line/70 bg-ink-900 px-3 py-2 text-sm">
						<span class="text-dim">Ваш счёт</span>
						<span class="digits text-chalk">{myScore}</span>
					</p>
				{/if}

				<ul class="space-y-1.5">
					{#each ranked as player}
						<li class="flex items-center gap-2.5 rounded-[12px] px-1.5 py-1.5">
							<span class="grid h-7 w-7 shrink-0 place-items-center rounded-full bg-ink-700 text-[11px] font-semibold text-chalk">
								{initial(player.name)}
							</span>
							<span class="min-w-0 flex-1 truncate text-sm {player.name === myName ? 'text-amber' : 'text-chalk'}">
								{player.name}
							</span>
							<span class="digits text-sm text-dim">
								{#if phase !== 'waiting'}{player.score}{/if}
							</span>
						</li>
					{/each}
				</ul>

				{#if players.length === 0}
					<p class="py-3 text-sm text-dim">Пока никого нет. Отправьте друзьям код комнаты.</p>
				{/if}
			</aside>
		</div>
	</main>
</div>
