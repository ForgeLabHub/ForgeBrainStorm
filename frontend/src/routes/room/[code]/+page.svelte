<script lang="ts">
	import { modeLabel } from '$lib/ui';
	import Icon, { type IconName } from '$lib/Icon.svelte';
	import { page } from '$app/stores';
	import { api } from '$lib/api';
        import { ws } from '$lib/ws';
	import { onMount, onDestroy } from 'svelte';
	import ChatPanel from '$lib/components/ChatPanel.svelte';
	import CheatPanel from '$lib/components/CheatPanel.svelte';
	import EmojiStorm from '$lib/components/EmojiStorm.svelte';
	import {
		EMOJI_PALETTE,
		TARGET_POWERS,
		type ChatMessage,
		type CheatBudget,
		type CheatPowerKind,
		type CheatTarget,
		type EmojiThrow
	} from '$lib/chat';

	const roomCode = ($page.params.code ?? '').toUpperCase();

	type Player = {
		id?: string;
		name: string;
		score: number;
		lives?: number;
		infinite_lives?: boolean;
		team?: number | null;
		streak?: number;
		eliminated?: boolean;
		disconnected?: boolean;
		is_host: boolean;
		is_ready: boolean;
		muted?: boolean;
		hijacked?: boolean;
		username?: string;
	};

	type TeamScore = { team: number; name: string; score: number };

	type PowerUpKind = 'double_points' | 'freeze_time' | 'remove_two_options';

	const POWER_UP_META: Record<PowerUpKind, { icon: IconName; label: string }> = {
		double_points: { icon: 'bolt', label: '×2 очки' },
		freeze_time: { icon: 'hourglass', label: '+10 сек' },
		remove_two_options: { icon: 'scissors', label: '−2 варианта' },
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
	let gamePhase: 'waiting' | 'preparing' | 'playing' | 'result' | 'finished' = $state('waiting');
	let questionSource = $state('');
	let isHost = $state(false);
	let topic = $state('');
	let mode = $state('');
	let error = $state('');
	let roomMissing = $state(false);
	let timerInterval: ReturnType<typeof setInterval> | null = null;
	type ResultKind = 'correct' | 'wrong' | 'timeout';
	let lastResult: { kind: ResultKind; correctIndex: number; explanation: string; player_answers?: any[] } | null = $state(null);
	let finalLeaderboard: any[] | null = $state(null);
	let powerUps: PowerUpKind[] = $state([]);
	let cheatBudget: CheatBudget | null = $state(null);
	let myId = $state('');
	let chatMessages = $state<ChatMessage[]>([]);
	let chatMuted = $state(false);
	let chatLocked = $state(false);
	let blindSeconds = $state(0);
	let blindTimer: ReturnType<typeof setInterval> | null = null;
	let throws = $state<EmojiThrow[]>([]);
	let throwSeq = 0;
	let cheatReveal: number | null = $state(null);
	let cheatNotice = $state('');
	let hiddenOptions: number[] = $state([]);
	let eliminated = $state(false);
	let modeTitle = $state('');
	let persisted = $state(true);

	type Proposal = {
		id: string;
		proposer_sid: string;
		proposer_name: string;
		proposal_type: string;
		value: string;
		description: string;
		vote_count: number;
		voted_by: string[];
		active: boolean;
		created_at_secs: number;
	};
	let proposals = $state<Proposal[]>([]);
	let proposalKind = $state('topic');
	let proposalValue = $state('');
	let proposalError = $state('');
	let providerOptions = $state<string[]>([]);
	let proposalNotice = $state('');

	// Sorting `players` in place inside markup mutates state during render;
	// every view reads this derived copy instead.
	let ranked = $derived([...players].sort((a, b) => b.score - a.score));
	// Everyone still in the game except me: the cheater's victim pool.
	let cheatTargets: CheatTarget[] = $derived(
		players
			.filter((player) => player.id && player.id !== myId && !player.eliminated)
			.map((player) => ({ id: player.id as string, name: player.name }))
	);
	// The server sends authoritative totals in `teams` (two sides in team mode,
	// one shared side in coop). The client only computes them before game_over.
	let serverTeams: TeamScore[] = $state([]);
	let isCoop = $derived(mode === 'coop');
	let teamScores = $derived<TeamScore[]>(
		serverTeams.length > 0
			? [...serverTeams].sort((a, b) => b.score - a.score)
			: isCoop
				? [{ team: 0, name: 'Все вместе', score: players.reduce((sum, p) => sum + p.score, 0) }]
			: mode === 'team'
				? [
						{ team: 0, name: 'Синие', score: players.filter((p) => p.team === 0).reduce((sum, p) => sum + p.score, 0) },
						{ team: 1, name: 'Красные', score: players.filter((p) => p.team === 1).reduce((sum, p) => sum + p.score, 0) }
					].sort((a, b) => b.score - a.score)
				: []
	);

	const PROPOSAL_KINDS = [
		{ value: 'topic', label: 'Тема' },
		{ value: 'mode', label: 'Режим' },
		{ value: 'difficulty', label: 'Сложность' },
		{ value: 'question_count', label: 'Вопросов' },
		{ value: 'num_options', label: 'Вариантов' },
		{ value: 'timer_secs', label: 'Секунд на вопрос' },
		{ value: 'ai_provider', label: 'ИИ-провайдер' }
	];
	const PROPOSAL_MODES = [
		{ value: 'classic', label: 'Классика' },
		{ value: 'ffa', label: 'Каждый за себя' },
		{ value: 'lives', label: 'На выживание' },
		{ value: 'blitz', label: 'Блиц' },
		{ value: 'team', label: 'Команды' },
		{ value: 'sandbox', label: 'Тренировка' },
		{ value: 'coop', label: 'Кооператив' },
		{ value: 'svoyaigra', label: 'Своя Игра' },
		{ value: 'sudden', label: 'Внезапная смерть' },
		{ value: 'marathon', label: 'Марафон' }
	];
	const PROPOSAL_RANGES: Record<string, [number, number]> = {
		question_count: [5, 30],
		num_options: [2, 6],
		timer_secs: [10, 120]
	};
	const SETTING_LABELS: Record<string, string> = Object.fromEntries(
		PROPOSAL_KINDS.map((kind) => [kind.value, kind.label])
	);

	onMount(() => {
		ws.connect();
		ws.on('connected', () => {
			ws.send('join_room', { code: roomCode });
		});
		ws.on('room_joined', (data: any) => {
			isHost = data.is_host;
			myId = typeof data.sid === 'string' ? data.sid : '';
			topic = data.topic || '';
			mode = data.mode || 'classic';
			modeTitle = data.mode_title || '';
			if (data.players) {
				players = data.players.map((p: any) => ({
					name: p.name ?? p.display_name ?? p.username,
					score: p.score || 0,
					lives: p.lives,
					infinite_lives: p.infinite_lives,
					team: p.team ?? null,
					streak: p.streak ?? 0,
					eliminated: p.eliminated || false,
						id: p.id,
												muted: p.muted || false,
						hijacked: p.hijacked || false,
					is_host: p.is_host || false,
					is_ready: p.is_ready || false,
					username: p.name ?? p.username,
				}));
			}
		});
		ws.on('players_updated', (data: any) => {
			if (data?.players) {
				players = data.players.map((p: any) => ({
					name: p.name ?? p.display_name ?? p.username,
					score: p.score || 0,
					lives: p.lives,
					infinite_lives: p.infinite_lives,
					eliminated: p.eliminated || false,
						id: p.id,
												muted: p.muted || false,
						hijacked: p.hijacked || false,
					is_host: p.is_host || false,
					is_ready: p.is_ready || false,
					username: p.name ?? p.username,
				}));
			}
		});
		ws.on('player_joined', (data: any) => {
			if (data?.player) {
				players = [...players, {
					name: data.player.name ?? data.player.display_name ?? data.player.username,
					score: data.player.score || 0,
					lives: data.player.lives,
					infinite_lives: data.player.infinite_lives,
					team: data.player.team ?? null,
					streak: data.player.streak ?? 0,
					eliminated: data.player.eliminated || false,
						id: data.player.id,
												muted: data.player.muted || false,
						hijacked: data.player.hijacked || false,
					is_host: data.player.is_host || false,
					is_ready: data.player.is_ready || false,
					username: data.player.username,
				}];
			}
		});
		ws.on('player_left', (data: any) => {
			players = players.filter(p => p.name !== (data?.name ?? data?.username));
		});
		void api
			.aiProviders()
			.then((response) => {
				providerOptions = response.providers ?? [];
			})
			.catch(() => {
				providerOptions = [];
			});
		ws.on('power_ups', (data: any) => {
			powerUps = Array.isArray(data?.items) ? data.items : [];
		});
		ws.on('cheat_state', (data: any) => {
			const powers = data?.powers;
			// The server sends the full budget; the panel hides what is at zero.
			cheatBudget =
				powers && typeof powers.peek === 'number' ? (powers as CheatBudget) : null;
		});
		ws.on('cheat_reveal', (data: any) => {
			cheatReveal = typeof data?.correct_index === 'number' ? data.correct_index : null;
		});
		ws.on('cheat_armed', () => {
			cheatNotice = 'Следующий верный ответ даст двойные очки';
		});
		ws.on('cheat_used', (data: any) => {
			if (data?.power === 'freeze') {
				cheatNotice = `Заморозка: +${data.seconds} секунд`;
				timeLeft += Number(data.seconds) || 0;
			}
			if (data?.power === 'steal') {
				cheatNotice = `Кража: ${data.points} очков у лидера`;
			}
			if (data?.power === 'hijack') {
				cheatNotice = `Угон: аккаунт ${data.target} с ${data.points} очками теперь ваш`;
			}
			if (data?.power === 'rewrite') {
				cheatNotice = 'Вопрос подменён';
			}
			if (data?.power === 'mute') {
				cheatNotice = `${data.target} больше не может писать в чат`;
			}
			if (data?.power === 'jinx') {
				cheatNotice = `${data.target} проклят: следующий ответ — 0 очков`;
			}
		});
		ws.on('chat_history', (data: any) => {
			chatMessages = Array.isArray(data?.messages) ? data.messages : [];
		});
		ws.on('chat_message', (data: any) => {
			if (data && typeof data.seq === 'number') chatMessages = [...chatMessages, data];
		});
		ws.on('emoji', (data: any) => {
			if (!data?.emoji) return;
			throwSeq += 1;
			const id = throwSeq;
			throws = [
				...throws,
				{
					id,
					emoji: data.emoji,
					from: data.from || '',
					target: data.target ?? null,
					left: 8 + Math.random() * 84
				}
			];
			setTimeout(() => {
				throws = throws.filter((item) => item.id !== id);
			}, 3200);
		});
		ws.on('hijacked', (data: any) => {
			eliminated = true;
			error = `${data?.by ?? 'Читер'} угнал ваш аккаунт и забрал ${data?.points ?? 0} очков`;
		});
		ws.on('muted', (data: any) => {
			chatMuted = true;
			cheatNotice = `${data?.by ?? 'Читер'} заглушил ваш чат`;
		});
		ws.on('jinxed', (data: any) => {
			cheatNotice = `${data?.by ?? 'Читер'} проклял вас: следующий ответ не принесёт очков`;
		});
		ws.on('blinded', (data: any) => {
			// The server refuses the answer anyway; this only makes the wait
			// visible and disables the buttons.
			blindSeconds = typeof data?.seconds === 'number' ? data.seconds : 10;
			cheatNotice = `${data?.by ?? 'Читер'} ослепил вас на ${blindSeconds} с`;
			if (blindTimer) clearInterval(blindTimer);
			blindTimer = setInterval(() => {
				blindSeconds = Math.max(0, blindSeconds - 1);
				if (blindSeconds === 0 && blindTimer) {
					clearInterval(blindTimer);
					blindTimer = null;
				}
			}, 1000);
		});
		ws.on('status', (data: any) => {
			// Sent on join: a reload must not launder a mute, a curse or a blind.
			if (typeof data?.muted === 'boolean') chatMuted = data.muted;
			if (typeof data?.eliminated === 'boolean') eliminated = data.eliminated;
			if (data?.jinxed) cheatNotice = 'На вас лежит проклятие: следующий ответ не принесёт очков';
			if (typeof data?.blind_seconds === 'number' && data.blind_seconds > 0) {
				blindSeconds = data.blind_seconds;
			}
		});
		ws.on('chat_locked', (data: any) => {
			chatLocked = Boolean(data?.locked);
		});
		ws.on('chat_cleared', () => {
			chatMessages = [];
		});
		ws.on('kicked', (data: any) => {
			roomMissing = true;
			error = data?.message || 'Вас удалили из комнаты';
		});
		ws.on('announcement', (data: any) => {
			if (data?.message) cheatNotice = data.message;
		});
		ws.on('options_hidden', (data: any) => {
			hiddenOptions = Array.isArray(data?.indexes) ? data.indexes : [];
		});
		ws.on('power_up_used', (data: any) => {
			// The server owns the clock; mirror its extension locally.
			if (data?.power_up === 'freeze_time' && typeof data.seconds === 'number') {
				timeLeft += data.seconds;
			}
		});
		ws.on('game_preparing', () => {
			gamePhase = 'preparing';
			chatMessages = [];
			chatMuted = false;
			lastResult = null;
			error = '';
			eliminated = false;
			stopTimer();
		});
		ws.on('question_emitted', (data: any) => {
			question = { ...data.question, time_limit: data.time_limit || 30 };
			questionSource = typeof data.source === 'string' ? data.source : '';
			timeLeft = data.time_limit || 30;
			selectedAnswer = null;
			hiddenOptions = [];
			cheatReveal = null;
			gamePhase = 'playing';
			startTimer();
		});
		ws.on('timer_update', (data: any) => {
			timeLeft = data.seconds_remaining || 0;
		});
		ws.on('question_result', (data: any) => {
			gamePhase = 'result';
			stopTimer();
			const correctIndex =
				typeof data.correct_index === 'number' ? data.correct_index : (data.correctIndex ?? 0);
			lastResult = {
				kind:
					selectedAnswer === null
						? 'timeout'
						: selectedAnswer === correctIndex
							? 'correct'
							: 'wrong',
				correctIndex,
				explanation: data.explanation || '',
				player_answers: data.player_answers
			};
		});
		ws.on('game_over', (data: any) => {
			gamePhase = 'finished';
			stopTimer();
			persisted = data?.persisted !== false;
			serverTeams = Array.isArray(data?.teams) ? data.teams : [];
			const board = data?.final_leaderboard ?? data?.scores;
			if (board) {
				finalLeaderboard = board;
				players = board.map((p: any) => ({
					name: p.name ?? p.username,
					score: p.score,
					lives: p.lives,
					team: p.team ?? null,
					eliminated: p.eliminated ?? false,
					is_host: false,
					is_ready: true,
					username: p.name ?? p.username,
				}));
			}
		});
		ws.on('eliminated', (data: any) => {
			eliminated = true;
			error = data?.message || 'Жизни кончились';
		});
		ws.on('game_started', () => {
			gamePhase = 'playing';
			serverTeams = [];
		});
		ws.on('proposals_list', (data: any) => {
			proposals = Array.isArray(data?.proposals) ? data.proposals : [];
		});
		ws.on('proposal_accepted', (data: any) => {
			const change = data?.change;
			if (!change?.setting) return;
			if (change.setting === 'topic') topic = change.value;
			if (change.setting === 'mode') {
				mode = change.value;
				modeTitle = PROPOSAL_MODES.find((option) => option.value === change.value)?.label ?? modeTitle;
			}
			proposalNotice = `Настройки обновлены: ${SETTING_LABELS[change.setting] ?? change.setting} → ${change.value}`;
		});
		ws.on('error', (data: any) => {
			if (data?.code === 'room_not_found') {
				roomMissing = true;
				return;
			}
			if (typeof data?.code === 'string' && data.code.startsWith('cheat')) {
				cheatNotice = data.message || 'Способность не сработала';
				return;
			}
			if (data?.code === 'blinded') {
				cheatNotice = data.message || 'Вас ослепили';
				return;
			}
			error = data.message || 'Ошибка';
		});
	});

	onDestroy(() => {
		stopTimer();
		if (blindTimer) clearInterval(blindTimer);
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
		if (blindSeconds > 0) return;
		selectedAnswer = index;
		ws.send('submit_answer', { answer_index: index });
	}

	function useCheatPower(power: CheatPowerKind, target: string | null = null, payload?: unknown) {
		if (gamePhase !== 'playing' || !cheatBudget || cheatBudget[power] <= 0) return;
		if (TARGET_POWERS.includes(power) && !target) return;
		ws.send('cheat_power', { power, target, payload });
	}

	function sendChat(text: string) {
		ws.send('chat_send', { text });
	}

	function throwEmoji(emoji: string, target: string | null) {
		if (!EMOJI_PALETTE.includes(emoji)) return;
		ws.send('emoji', { emoji, target });
	}

	function usePowerUp(kind: PowerUpKind) {
		if (gamePhase !== 'playing' || !powerUps.includes(kind)) return;
		ws.send('use_power_up', { power_up: kind });
	}

	function startGame() {
		ws.send('start_game', {});
	}

	function proposalLabel(proposal: Proposal): string {
		const setting = SETTING_LABELS[proposal.proposal_type] ?? proposal.proposal_type;
		const value =
			proposal.proposal_type === 'mode'
				? (PROPOSAL_MODES.find((option) => option.value === proposal.value)?.label ?? proposal.value)
				: proposal.value;
		return `${setting}: ${value}`;
	}

	function votedByMe(proposal: Proposal): boolean {
		return proposal.voted_by.includes(myId);
	}

	function propose(event: Event) {
		event.preventDefault();
		const value = proposalValue.trim();
		const range = PROPOSAL_RANGES[proposalKind];
		if (!value) {
			proposalError = 'Укажите значение';
			return;
		}
		if (range) {
			const number = Number(value);
			if (!Number.isInteger(number) || number < range[0] || number > range[1]) {
				proposalError = `Нужно число от ${range[0]} до ${range[1]}`;
				return;
			}
		}
		proposalError = '';
		ws.send('propose', {
			proposal_type: proposalKind,
			value,
			description: `${SETTING_LABELS[proposalKind] ?? proposalKind} → ${value}`
		});
		proposalValue = '';
	}

	function toggleVote(proposal: Proposal) {
		ws.send(votedByMe(proposal) ? 'unvote_proposal' : 'vote_proposal', { proposal_id: proposal.id });
	}

	function acceptProposal(id: string) {
		ws.send('accept_proposal', { proposal_id: id });
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

</script>

<div class="mx-auto max-w-7xl px-6 py-8">
	<!-- Header -->
	<div class="mb-6 flex items-center justify-between">
		<div>
			<div class="flex items-center gap-3">
				<div class="font-mono text-sm text-slate-500 tracking-wider bg-white/5 rounded-lg px-3 py-1 border border-white/10">{roomCode}</div>
				{#if mode}
					<span class="text-sm text-slate-400">{modeTitle || modeLabel(mode)}</span>
				{/if}
			</div>
			{#if !roomMissing}
				<h1 class="mt-2 text-2xl font-bold text-white">{topic || 'Без темы'}</h1>
			{/if}
		</div>
		{#if !roomMissing}
			<div class="flex items-center gap-4">
				<div class="text-right">
					<div class="text-xs text-slate-500">Игроков</div>
					<div class="text-2xl font-bold text-white">{players.length}</div>
				</div>
			</div>
		{/if}
	</div>

	{#if roomMissing}
		<div class="rounded-2xl border border-white/5 bg-white/[0.02] p-10 text-center">
			<div class="mb-4 flex justify-center text-slate-500"><Icon name="cross" class="h-12 w-12" /></div>
			<h2 class="mb-2 text-xl font-bold text-white">Комнаты {roomCode} нет</h2>
			<p class="mb-6 text-slate-400">
				Код мог устареть, а комнату — закрыть хост. Проверьте код или создайте свою.
			</p>
			<div class="flex flex-wrap justify-center gap-3">
				<a href="/lobby" class="rounded-xl bg-gradient-to-r from-blue-500 to-purple-600 px-5 py-2.5 font-semibold text-white shadow-lg transition hover:shadow-blue-500/20">
					К комнатам
				</a>
				<a href="/room/create" class="rounded-xl border border-white/10 bg-white/5 px-5 py-2.5 font-semibold text-slate-200 transition hover:bg-white/10">
					Создать комнату
				</a>
			</div>
		</div>
	{:else}
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
						<div class="mb-6 flex justify-center text-blue-400"><Icon name="clock" class="h-14 w-14" /></div>
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

					<div class="mt-6 rounded-2xl border border-white/5 bg-white/[0.02] p-4 backdrop-blur-sm sm:p-5">
					<div class="mb-4 flex flex-wrap items-center justify-between gap-2">
						<h3 class="text-sm font-semibold uppercase tracking-wider text-slate-400">Предложения</h3>
						<span class="text-xs text-slate-500">
							{isHost ? 'Принимайте — настройки применятся сразу' : 'Предложите изменение, остальные голосуют'}
						</span>
					</div>

					<form class="mb-4 flex flex-wrap items-end gap-3" onsubmit={propose}>
						<label class="min-w-40 flex-1 text-xs text-slate-400">
							<span class="mb-1 block">Что меняем</span>
							<select
								bind:value={proposalKind}
								class="w-full rounded-xl border border-white/10 bg-white/5 px-3 py-2 text-sm text-white focus:border-blue-500/50 focus:outline-none"
							>
								{#each PROPOSAL_KINDS as kind}
									{#if kind.value !== 'ai_provider' || providerOptions.length > 0}
										<option value={kind.value}>{kind.label}</option>
									{/if}
								{/each}
							</select>
						</label>
						<label class="min-w-40 flex-1 text-xs text-slate-400">
							<span class="mb-1 block">Значение</span>
							{#if proposalKind === 'mode'}
								<select bind:value={proposalValue} class="w-full rounded-xl border border-white/10 bg-white/5 px-3 py-2 text-sm text-white focus:border-blue-500/50 focus:outline-none">
									<option value="">— выберите —</option>
									{#each PROPOSAL_MODES as option}
										<option value={option.value}>{option.label}</option>
									{/each}
								</select>
							{:else if proposalKind === 'difficulty'}
								<select bind:value={proposalValue} class="w-full rounded-xl border border-white/10 bg-white/5 px-3 py-2 text-sm text-white focus:border-blue-500/50 focus:outline-none">
									<option value="">— выберите —</option>
									<option value="easy">Лёгкая</option>
									<option value="medium">Средняя</option>
									<option value="hard">Сложная</option>
								</select>
							{:else if proposalKind === 'ai_provider'}
								<select bind:value={proposalValue} class="w-full rounded-xl border border-white/10 bg-white/5 px-3 py-2 text-sm text-white focus:border-blue-500/50 focus:outline-none">
									<option value="">— выберите —</option>
									{#each providerOptions as provider}
										<option value={provider}>{provider}</option>
									{/each}
								</select>
							{:else if PROPOSAL_RANGES[proposalKind]}
								<input
									type="number"
									bind:value={proposalValue}
									min={PROPOSAL_RANGES[proposalKind][0]}
									max={PROPOSAL_RANGES[proposalKind][1]}
									class="w-full rounded-xl border border-white/10 bg-white/5 px-3 py-2 text-sm text-white focus:border-blue-500/50 focus:outline-none"
								/>
							{:else}
								<input
									type="text"
									bind:value={proposalValue}
									maxlength="60"
									placeholder="Например: Космос"
									class="w-full rounded-xl border border-white/10 bg-white/5 px-3 py-2 text-sm text-white focus:border-blue-500/50 focus:outline-none"
								/>
							{/if}
						</label>
						<button
							type="submit"
							class="rounded-xl bg-blue-600 px-4 py-2 text-sm font-semibold text-white transition-colors hover:bg-blue-500"
						>
							Предложить
						</button>
					</form>

					{#if proposalError}
						<p class="mb-3 text-xs text-red-400">{proposalError}</p>
					{/if}
					{#if proposalNotice}
						<p class="mb-3 text-xs text-emerald-400">{proposalNotice}</p>
					{/if}

					{#if proposals.length === 0}
						<p class="text-sm text-slate-500">Пока нет предложений — станьте первым.</p>
					{:else}
						<ul class="space-y-2">
							{#each proposals as proposal (proposal.id)}
								<li
									class="flex flex-wrap items-center justify-between gap-3 rounded-xl border border-white/5 bg-white/[0.03] px-3 py-2 {proposal.active
										? ''
										: 'opacity-60'}"
								>
									<div>
										<p class="text-sm font-medium text-white">{proposalLabel(proposal)}</p>
										<p class="text-xs text-slate-500">
											{proposal.proposer_name} · голосов: {proposal.vote_count}{proposal.active ? '' : ' · принято'}
										</p>
									</div>
									{#if proposal.active}
										<div class="flex items-center gap-2">
											{#if proposal.proposer_sid !== myId}
												<button
													type="button"
													onclick={() => toggleVote(proposal)}
													class="rounded-lg border border-white/10 px-3 py-1.5 text-xs font-semibold {votedByMe(
														proposal
													)
														? 'bg-blue-600/20 text-blue-300'
														: 'text-slate-300 hover:bg-white/5'}"
												>
													{votedByMe(proposal) ? 'Убрать голос' : 'За'}
												</button>
											{/if}
											{#if isHost}
												<button
													type="button"
													onclick={() => acceptProposal(proposal.id)}
													class="rounded-lg bg-emerald-600 px-3 py-1.5 text-xs font-semibold text-white transition-colors hover:bg-emerald-500"
												>
													Принять
												</button>
											{/if}
										</div>
									{/if}
								</li>
							{/each}
						</ul>
					{/if}
				</div>

		{:else if gamePhase === 'preparing'}
			<div class="relative overflow-hidden rounded-2xl border border-white/5 bg-white/[0.02] p-12 text-center backdrop-blur-sm">
				<div class="absolute inset-0 bg-gradient-to-br from-purple-500/5 to-blue-500/5"></div>
				<div class="relative z-10">
					<div class="mx-auto mb-6 h-12 w-12 animate-spin rounded-full border-4 border-white/10 border-t-purple-500"></div>
					<h2 class="mb-2 text-2xl font-bold text-white">Готовим вопросы</h2>
					<p class="text-slate-400">ИИ подбирает вопросы по теме «{topic}» — это займёт несколько секунд</p>
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

					{#if eliminated}
						<div class="rounded-xl border border-red-500/20 bg-red-500/10 p-4 text-center text-sm text-red-300">
							Жизни кончились — вы выбыли. Досмотрите игру до конца, очки сохранятся в таблице.
						</div>
					{/if}
					<!-- Power-ups: the hand is private to this player -->
					{#if powerUps.length > 0}
						<div class="flex flex-wrap gap-2">
							{#each powerUps as kind (kind)}
								<button
									onclick={() => usePowerUp(kind)}
									class="flex items-center gap-2 rounded-xl border border-purple-500/20 bg-purple-500/10 px-4 py-2 text-sm font-medium text-purple-300 transition-all hover:border-purple-400/50 hover:bg-purple-500/20"
								>
									<Icon name={POWER_UP_META[kind].icon} class="h-4 w-4" />
									<span>{POWER_UP_META[kind].label}</span>
								</button>
							{/each}
						</div>
					{/if}

					{#if cheatBudget}
						<CheatPanel
							budget={cheatBudget}
							targets={cheatTargets}
							notice={cheatNotice}
							canUse={gamePhase === 'playing' && !eliminated}
							onUse={useCheatPower}
						/>
					{/if}

					<!-- Question -->
					<div class="relative overflow-hidden rounded-2xl border border-white/5 bg-white/[0.02] p-5 backdrop-blur-sm sm:p-8">
						<div class="absolute inset-0 bg-gradient-to-br from-purple-500/5 to-blue-500/5"></div>
						<div class="relative z-10">
							{#if question.category}
								<span class="mb-3 mr-2 inline-block rounded-full border border-blue-500/20 bg-blue-500/10 px-3 py-1 text-xs font-medium text-blue-400">
									{question.category}
								</span>
							{/if}

							<h2 class="text-lg font-bold leading-relaxed text-white sm:text-xl">{question.question}</h2>
						</div>
					</div>

					{#if blindSeconds > 0}
						<div class="flex items-center gap-3 rounded-xl border border-red-500/30 bg-red-500/10 px-4 py-3 text-sm text-red-200">
							<Icon name="blind" class="h-5 w-5 shrink-0" />
							<span>Вас ослепили: отвечать можно через {blindSeconds} с</span>
						</div>
					{/if}

					<!-- Options -->
					<div class="grid grid-cols-1 gap-3 sm:grid-cols-2 {blindSeconds > 0 ? 'pointer-events-none blur-sm' : ''}">
						{#each question.options as option, i}
							{@const isSelected = selectedAnswer === i}
							{@const isHidden = hiddenOptions.includes(i)}
							<button
								onclick={() => submitAnswer(i)}
								disabled={selectedAnswer !== null || isHidden || eliminated || blindSeconds > 0}
								class="group relative overflow-hidden rounded-xl border p-5 text-left transition-all {
									isHidden
										? 'border-white/5 bg-white/[0.01] opacity-30 line-through'
										: isSelected
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
									{#if cheatReveal === i}
										<span class="ml-auto shrink-0 rounded-md border border-red-400/40 bg-red-500/15 px-2 py-0.5 text-[11px] font-semibold text-red-300">верный</span>
									{/if}
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
					<div class="relative overflow-hidden rounded-2xl border border-white/5 bg-white/[0.02] p-5 backdrop-blur-sm sm:p-8">
						<div class="absolute inset-0 {lastResult?.kind === 'correct' ? 'bg-gradient-to-br from-green-500/10 to-emerald-500/5' : lastResult?.kind === 'timeout' ? 'bg-gradient-to-br from-amber-500/10 to-orange-500/5' : 'bg-gradient-to-br from-red-500/10 to-rose-500/5'}"></div>
						<div class="relative z-10 text-center">
							<div class="mb-4 flex justify-center {lastResult?.kind === 'correct' ? 'text-green-400' : lastResult?.kind === 'timeout' ? 'text-amber-400' : 'text-red-400'}"><Icon name={lastResult?.kind === 'correct' ? 'check' : 'cross'} class="h-12 w-12" /></div>
							<h2 class="text-2xl font-bold {lastResult?.kind === 'correct' ? 'text-green-400' : lastResult?.kind === 'timeout' ? 'text-amber-400' : 'text-red-400'}">
								{lastResult?.kind === 'correct' ? 'Правильно!' : lastResult?.kind === 'timeout' ? 'Время вышло' : 'Неправильно'}
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
							{#each ranked as player, i}
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
						<div class="mb-6 flex justify-center text-yellow-400"><Icon name="trophy" class="h-14 w-14" /></div>
						<h2 class="mb-2 text-3xl font-bold text-white">Игра окончена!</h2>
						{#if isCoop && teamScores.length > 0}
							<p class="text-slate-400">
								Общими усилиями вы набрали
								<span class="font-semibold text-emerald-400">{teamScores[0].score}</span> очков
							</p>
						{:else if mode === 'team' && teamScores.length > 1}
							<p class="text-slate-400">
								{#if teamScores[0].score === teamScores[1].score}
									Ничья: обе команды набрали {teamScores[0].score}
								{:else}
									Победитель: <span class="font-semibold text-yellow-400">{teamScores[0].name}</span>
									<span class="text-slate-500">({teamScores[0].score} против {teamScores[1].score})</span>
								{/if}
							</p>
						{:else if players.length > 0}
							<p class="text-slate-400">
								{#if players.every(p => p.eliminated)}
									Никто не дожил до конца
								{:else}
									Победитель: <span class="font-semibold text-yellow-400">{players[0]?.name}</span>
								{/if}
							</p>
						{/if}
						{#if !persisted}
							<p class="mt-3 inline-block rounded-lg bg-blue-500/10 px-3 py-1.5 text-xs text-blue-300">
								Тренировка: результат не попал в рейтинг и историю
							</p>
						{/if}

						<!-- Final leaderboard -->
						<div class="mt-8 space-y-3">
							{#each ranked.slice(0, 5) as player, i}
								<div class="flex items-center justify-between rounded-xl bg-white/5 border border-white/10 p-4 {i === 0 ? 'ring-2 ring-yellow-500/30' : ''}">
									<div class="flex items-center gap-4">
										<span class="flex h-8 w-8 items-center justify-center rounded-lg text-sm font-bold {i === 0 ? 'bg-yellow-500/20 text-yellow-300' : i === 1 ? 'bg-slate-400/20 text-slate-200' : i === 2 ? 'bg-orange-500/20 text-orange-300' : 'bg-white/5 text-slate-400'}">
											{i + 1}
										</span>
										<span class="font-semibold text-white">{player.name}</span>
										{#if player.eliminated}
											<span class="rounded bg-red-500/20 px-1.5 py-0.5 text-[10px] font-bold text-red-400">ВЫБЫЛ</span>
										{/if}
									{#if (mode === 'team' || isCoop) && player.team !== null && player.team !== undefined}
										{#if isCoop}
											<span class="rounded bg-emerald-500/20 px-1.5 py-0.5 text-[10px] font-bold text-emerald-300">ВМЕСТЕ</span>
										{:else}
											<span class="rounded px-1.5 py-0.5 text-[10px] font-bold {player.team === 0 ? 'bg-blue-500/20 text-blue-300' : 'bg-rose-500/20 text-rose-300'}">
												{player.team === 0 ? 'СИНИЕ' : 'КРАСНЫЕ'}
											</span>
										{/if}
									{/if}
										{#if (player.streak ?? 0) > 1}
											<span class="rounded bg-orange-500/20 px-1.5 py-0.5 text-[10px] font-bold text-orange-300">×{player.streak} серия</span>
										{/if}
									</div>
									<span class="text-xl font-bold text-white">{player.score} <span class="text-sm text-slate-400">pts</span></span>
								</div>
							{/each}
						</div>

						<div class="mt-8 flex justify-center gap-4">
							<a href="/lobby" class="rounded-xl bg-white/5 border border-white/10 px-6 py-3 font-medium text-white hover:bg-white/10 transition-all">
								В лобби
							</a>
							{#if isHost}
								<button onclick={startGame} class="relative group">
									<div class="absolute inset-0 bg-gradient-to-r from-blue-500 to-purple-600 rounded-xl blur-md opacity-40 group-hover:opacity-70 transition-opacity"></div>
									<div class="relative rounded-xl bg-gradient-to-r from-blue-500 to-purple-600 px-6 py-3 font-medium text-white">
										Играть снова
									</div>
								</button>
							{:else}
								<p class="self-center text-sm text-slate-400">Новую игру начнёт хост — останьтесь в комнате</p>
							{/if}
						</div>
					</div>
				</div>
			{/if}
		</div>

		<!-- Sidebar: players, then chat -->
		<div class="space-y-4">
			<div class="sticky top-24 rounded-2xl border border-white/5 bg-white/[0.02] p-5 backdrop-blur-sm">
					{#if (mode === 'team' || isCoop) && teamScores.length > 0}
						<div class="mb-4 rounded-xl border border-white/10 bg-white/[0.03] p-3">
							<div class="mb-2 text-[11px] font-semibold uppercase tracking-wider text-slate-500">
								{isCoop ? 'Общий счёт' : 'Счёт команд'}
							</div>
							{#each teamScores as team, index}
								<div class="flex items-center justify-between py-1">
									<span class="flex items-center gap-2 text-sm {isCoop ? 'text-emerald-300' : team.team === 0 ? 'text-blue-300' : 'text-rose-300'}">
										<span class="h-2 w-2 rounded-full {isCoop ? 'bg-emerald-400' : team.team === 0 ? 'bg-blue-400' : 'bg-rose-400'}"></span>
										{!isCoop && index === 0 && team.score > 0 ? '★ ' : ''}{team.name}
									</span>
									<span class="text-sm font-bold text-white">{team.score}</span>
								</div>
							{/each}
						</div>
					{/if}
				<h3 class="mb-4 text-sm font-semibold uppercase tracking-wider text-slate-400">Игроки</h3>
				<div class="space-y-2">
					{#each ranked as player}
						<div class="flex items-center justify-between rounded-xl bg-white/[0.02] border border-white/5 p-3 transition-all hover:bg-white/[0.05] {player.disconnected ? 'opacity-50' : ''}">
							<div class="flex items-center gap-3">
								<div class="flex h-8 w-8 items-center justify-center rounded-lg bg-gradient-to-br from-blue-500 to-purple-600 text-sm font-bold text-white">
									{player.name.charAt(0).toUpperCase()}
								</div>
								<div>
									<div class="flex items-center gap-2">
										<span class="text-sm font-medium text-white">{player.name}</span>
										{#if player.disconnected}
										<span class="rounded bg-slate-500/20 px-1.5 py-0.5 text-[10px] font-bold text-slate-400">ОФФЛАЙН</span>
									{/if}
									{#if player.is_host}
											<span class="rounded bg-yellow-500/20 px-1.5 py-0.5 text-[10px] font-bold text-yellow-400">HOST</span>
										{/if}
										{#if player.eliminated}
											<span class="rounded bg-red-500/20 px-1.5 py-0.5 text-[10px] font-bold text-red-400">ВЫБЫЛ</span>
										{/if}
									</div>
								</div>
							</div>
							{#if (mode === 'lives' || mode === 'sudden') && !player.infinite_lives && typeof player.lives === 'number'}
								<div class="mb-1 text-right text-xs tracking-tight">
									{#each Array(3) as _, i}
										<Icon name="heart" class="h-3.5 w-3.5 {i < (player.lives ?? 0) ? 'text-red-400' : 'text-slate-700'}" />
									{/each}
								</div>
							{/if}
							<span class="text-sm font-bold text-white">{player.score}</span>
						</div>
					{/each}
				</div>
			</div>
			<ChatPanel
				messages={chatMessages}
				muted={chatMuted}
				locked={chatLocked}
				canChat={gamePhase === 'playing' || gamePhase === 'waiting'}
				targets={cheatTargets}
				onSend={sendChat}
				onEmoji={throwEmoji}
			/>
		</div>
	</div>
{/if}

<EmojiStorm throws={throws} />
</div>
