<script lang="ts">
	import Icon from '$lib/Icon.svelte';
	import { EMOJI_PALETTE, MAX_CHAT_CHARS, type ChatMessage, type CheatTarget } from '$lib/chat';

	let {
		messages = [],
		muted = false,
		locked = false,
		canChat = true,
		targets = [],
		onSend,
		onEmoji
	}: {
		messages?: ChatMessage[];
		muted?: boolean;
		locked?: boolean;
		canChat?: boolean;
		targets?: CheatTarget[];
		onSend?: (text: string) => void;
		onEmoji?: (emoji: string, target: string | null) => void;
	} = $props();

	let draft = $state('');
	let victim = $state('');
	let list = $state<HTMLDivElement | null>(null);
	let open = $state(true);

	$effect(() => {
		// Track the last message so the panel follows the conversation.
		messages.at(-1)?.seq;
		if (list) list.scrollTop = list.scrollHeight;
	});

	let left = $derived(MAX_CHAT_CHARS - draft.length);

	function send() {
		const text = draft.trim();
		if (!text || !canChat || muted || locked) return;
		onSend?.(text);
		draft = '';
	}

	function throwEmoji(emoji: string) {
		if (!canChat || muted || locked) return;
		onEmoji?.(emoji, victim || null);
	}
</script>

<section class="rounded-2xl border border-white/5 bg-white/[0.02] backdrop-blur-sm">
	<button
		type="button"
		onclick={() => (open = !open)}
		class="flex w-full items-center justify-between gap-2 px-4 py-3 text-left"
		aria-expanded={open}
	>
		<span class="flex items-center gap-2 text-sm font-semibold uppercase tracking-wider text-slate-400">
			<Icon name="chat" class="h-4 w-4" />
			Чат
		</span>
		<span class="flex items-center gap-2">
			{#if muted}
				<span class="rounded bg-red-500/20 px-1.5 py-0.5 text-[10px] font-bold text-red-300">ЗАГЛУШЁН</span>
			{:else if locked}
				<span class="rounded bg-amber-500/20 px-1.5 py-0.5 text-[10px] font-bold text-amber-300">ЗАМОРОЖЕН</span>
			{/if}
			<span class="text-xs text-slate-500">{open ? 'свернуть' : 'открыть'}</span>
		</span>
	</button>

	{#if open}
		<div class="border-t border-white/5 px-4 pb-4 pt-3">
			<div
				bind:this={list}
				class="mb-3 h-56 space-y-1.5 overflow-y-auto pr-1 text-sm"
				aria-live="polite"
			>
				{#if messages.length === 0}
					<p class="py-6 text-center text-xs text-slate-500">
						Пока тихо. Напишите первым — или киньте эмодзи.
					</p>
				{/if}
				{#each messages as message (message.seq)}
					{#if message.kind === 'emoji'}
						<p class="text-xs text-slate-500">
							<span class="font-medium text-slate-300">{message.from}</span>
							кинул(а) <span class="text-lg leading-none">{message.text}</span>
						</p>
					{:else}
						<p class="leading-snug">
							<span class="font-medium text-blue-300">{message.from}</span>
							<span class="text-slate-500">·</span>
							<span class="text-slate-200">{message.text}</span>
						</p>
					{/if}
				{/each}
			</div>

			{#if muted}
				<p class="mb-3 rounded-lg border border-red-500/20 bg-red-500/10 px-3 py-2 text-xs text-red-300">
					Читер запретил вам писать в чат до конца игры.
				</p>
			{/if}

			<div class="mb-2 flex flex-wrap items-center gap-1">
				{#each EMOJI_PALETTE as emoji (emoji)}
					<button
						type="button"
						onclick={() => throwEmoji(emoji)}
						disabled={muted || locked || !canChat}
						class="rounded-lg border border-white/10 bg-white/5 px-2 py-1 text-base leading-none transition hover:border-blue-400/40 hover:bg-blue-500/10 disabled:opacity-40"
						title="Кинуть {emoji}"
					>
						{emoji}
					</button>
				{/each}
				{#if targets.length > 0}
					<select
						bind:value={victim}
						disabled={muted || locked || !canChat}
						class="ml-auto rounded-lg border border-white/10 bg-slate-900 px-2 py-1 text-xs text-slate-300"
						aria-label="В кого кинуть эмодзи"
					>
						<option value="">всем</option>
						{#each targets as player (player.id)}
							<option value={player.id}>{player.name}</option>
						{/each}
					</select>
				{/if}
			</div>

			<form
				class="flex items-end gap-2"
				onsubmit={(event) => {
					event.preventDefault();
					send();
				}}
			>
				<div class="relative flex-1">
					<input
						bind:value={draft}
						maxlength={MAX_CHAT_CHARS}
						disabled={muted || locked || !canChat}
						placeholder={muted ? 'Вас заглушили' : locked ? 'Чат заморожен' : 'Сообщение…'}
						class="w-full rounded-xl border border-white/10 bg-white/5 px-3 py-2 text-sm text-white placeholder:text-slate-500 focus:border-blue-400/50 focus:outline-none disabled:opacity-50"
						aria-label="Сообщение в чат"
					/>
					{#if draft.length > MAX_CHAT_CHARS - 30}
						<span class="absolute -top-5 right-1 text-[10px] text-slate-500">{left}</span>
					{/if}
				</div>
				<button
					type="submit"
					disabled={muted || locked || !canChat || draft.trim().length === 0}
					class="rounded-xl bg-gradient-to-r from-blue-500 to-purple-600 px-3 py-2 text-sm font-semibold text-white transition disabled:opacity-40"
				>
					Отправить
				</button>
			</form>
		</div>
	{/if}
</section>
