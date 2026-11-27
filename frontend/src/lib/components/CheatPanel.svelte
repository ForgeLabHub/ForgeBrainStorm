<script lang="ts">
	import Icon, { type IconName } from '$lib/Icon.svelte';
	import {
		TARGET_POWERS,
		type CheatBudget,
		type CheatPowerKind,
		type CheatTarget
	} from '$lib/chat';

	let {
		budget,
		targets = [],
		notice = '',
		canUse = false,
		onUse
	}: {
		budget: CheatBudget;
		targets?: CheatTarget[];
		notice?: string;
		canUse?: boolean;
		onUse?: (power: CheatPowerKind, target: string | null, payload?: unknown) => void;
	} = $props();

	const META: Record<CheatPowerKind, { icon: IconName; label: string; hint: string }> = {
		peek: { icon: 'eye', label: 'Подсмотреть', hint: 'Верный ответ видите только вы' },
		freeze: { icon: 'hourglass', label: 'Заморозка', hint: '+10 секунд всем' },
		steal: { icon: 'bolt', label: 'Кража', hint: '50 очков у лидера' },
		double: { icon: 'spark', label: '×2', hint: 'Двойные очки за ответ' },
		hijack: { icon: 'mask', label: 'Угон аккаунта', hint: 'Забрать очки игрока и выбить его' },
		rewrite: { icon: 'pencil', label: 'Подмена вопроса', hint: 'Заменить вопрос на свой' },
		mute: { icon: 'mute', label: 'Мут', hint: 'Заглушить чат игрока' },
		jinx: { icon: 'skull', label: 'Проклятие', hint: 'Следующий ответ жертвы — 0 очков' }
	};

	let victim = $state('');
let hint = $state('');
	let rewriting = $state(false);
	let question = $state('');
	let options = $state(['', '', '', '']);
	let correct = $state(0);

	
	function use(power: CheatPowerKind) {
		if (!canUse || budget[power] <= 0) return;
		if (TARGET_POWERS.includes(power)) {
			if (!victim) {
				hint = 'Сначала выберите жертву';
				return;
			}
			onUse?.(power, victim);
			return;
		}
		if (power === 'rewrite') {
			rewriting = !rewriting;
			return;
		}
		onUse?.(power, null);
	}

	function submitRewrite() {
		const filled = options.map((option) => option.trim()).filter((option) => option.length > 0);
		if (question.trim().length < 3 || filled.length < 2) return;
		const index = Math.min(correct, filled.length - 1);
		onUse?.('rewrite', null, {
			question: question.trim(),
			options: filled,
			correct_index: index
		});
		rewriting = false;
		question = '';
		options = ['', '', '', ''];
		correct = 0;
	}
</script>

<section class="rounded-2xl border border-red-500/25 bg-red-500/[0.06] p-4">
	<div class="flex flex-wrap items-center justify-between gap-2">
		<p class="text-xs font-semibold uppercase tracking-widest text-red-300">Читерский набор</p>
		{#if hint || notice}
			<p class="truncate text-xs text-red-200">{hint || notice}</p>
		{/if}
	</div>

	{#if targets.length > 0}
		<label class="mt-3 flex items-center gap-2 text-xs text-red-200/80">
			<span class="shrink-0">Жертва:</span>
			<select
				bind:value={victim}
				class="min-w-0 flex-1 rounded-lg border border-red-500/30 bg-slate-900 px-2 py-1 text-xs text-red-100"
			>
				<option value="">— выберите игрока —</option>
				{#each targets as player (player.id)}
					<option value={player.id}>{player.name}</option>
				{/each}
			</select>
		</label>
	{/if}

	<div class="mt-3 grid grid-cols-2 gap-2 sm:grid-cols-4">
		{#each Object.entries(META) as [power, meta] (power)}
			{@const kind = power as CheatPowerKind}
			{@const left = budget[kind] ?? 0}
			{@const needsVictim = TARGET_POWERS.includes(kind)}
			<button
				type="button"
				onclick={() => use(kind)}
				disabled={left <= 0 || !canUse}
				class="flex flex-col items-start gap-1 rounded-xl border border-red-500/20 bg-red-500/10 px-3 py-2 text-left text-red-200 transition hover:border-red-400/50 disabled:opacity-40"
				title={meta.hint}
			>
				<span class="flex items-center gap-2 text-sm font-semibold">
					<Icon name={meta.icon} class="h-4 w-4" />
					{meta.label}
				</span>
				<span class="text-[11px] text-red-300/70">
					{meta.hint} · осталось {left}
					{#if needsVictim && !victim}
						<span class="text-amber-300"> · нужна цель</span>
					{/if}
				</span>
			</button>
		{/each}
	</div>

	{#if rewriting}
		<form
			class="mt-3 space-y-2 rounded-xl border border-red-500/25 bg-slate-950/40 p-3"
			onsubmit={(event) => {
				event.preventDefault();
				submitRewrite();
			}}
		>
			<p class="text-xs text-red-200">
				Свой вопрос для всей комнаты. Верный ответ увидите только вы — остальным
				придётся угадывать.
			</p>
			<input
				bind:value={question}
				maxlength={200}
				placeholder="Текст вопроса"
				class="w-full rounded-lg border border-white/10 bg-white/5 px-3 py-2 text-sm text-white placeholder:text-slate-500 focus:border-red-400/50 focus:outline-none"
			/>
			<div class="grid grid-cols-1 gap-2 sm:grid-cols-2">
				{#each options as _, index (index)}
					<label class="flex items-center gap-2 text-xs text-slate-300">
						<input
							type="radio"
							name="correct-option"
							checked={correct === index}
							onchange={() => (correct = index)}
							class="accent-red-500"
							aria-label="Верный вариант {index + 1}"
						/>
						<input
							bind:value={options[index]}
							maxlength={80}
							placeholder="Вариант {String.fromCharCode(65 + index)}"
							class="min-w-0 flex-1 rounded-lg border border-white/10 bg-white/5 px-2 py-1.5 text-sm text-white placeholder:text-slate-500 focus:border-red-400/50 focus:outline-none"
						/>
					</label>
				{/each}
			</div>
			<div class="flex items-center justify-between gap-2">
				<span class="text-[11px] text-slate-500">2–4 непустых варианта, кружок — верный</span>
				<button
					type="submit"
					disabled={question.trim().length < 3}
					class="rounded-lg bg-gradient-to-r from-red-500 to-orange-600 px-3 py-1.5 text-xs font-semibold text-white disabled:opacity-40"
				>
					Подменить вопрос
				</button>
			</div>
		</form>
	{/if}
</section>
