<script lang="ts">
	import { onMount } from 'svelte';
	import { api } from '$lib/api';
	import { currentUser } from '$lib/stores/auth';
	import Icon from '$lib/components/Icon.svelte';
	import { initial } from '$lib/ui';

	type Row = {
		rank: number;
		username: string;
		total_score: number;
		games_played: number;
		wins: number;
	};

	let rows: Row[] = [];
	let loading = true;
	let error = '';

	onMount(async () => {
		try {
			const response = await api.topPlayers(50);
			rows = (response.players || []) as unknown as Row[];
		} catch {
			error = 'Не удалось загрузить рейтинг';
		} finally {
			loading = false;
		}
	});

	$: me = $currentUser?.username ?? '';
</script>

<div class="mx-auto w-full max-w-2xl">
	<div class="mb-6">
		<h1 class="font-display text-[26px] tracking-tight text-chalk sm:text-3xl">Рейтинг</h1>
		<p class="mt-1 text-sm text-dim">Общий счёт по всем сыгранным играм. Обновляется после каждой партии.</p>
	</div>

	{#if loading}
		<div class="space-y-2">
			{#each [0, 1, 2, 3] as row}
				<div class="h-[62px] animate-pulse rounded-[14px] border border-line/50 bg-ink-850" class:opacity-50={row > 1}></div>
			{/each}
		</div>
	{:else if error}
		<p class="rounded-[14px] border border-coral/30 bg-coral/10 px-4 py-3 text-sm text-coral">{error}</p>
	{:else if rows.length === 0}
		<div class="card card-pad text-center">
			<span class="mx-auto grid h-12 w-12 place-items-center rounded-full border border-line bg-ink-900 text-dim">
				<Icon name="trophy" size={22} />
			</span>
			<p class="mt-3 font-medium text-chalk">Рейтинг пока пуст</p>
			<p class="mx-auto mt-1 max-w-[38ch] text-sm text-dim">
				Сыграйте первую игру до конца — результат попадёт сюда автоматически.
			</p>
			<a href="/room/create" class="btn btn-primary mt-4 w-full sm:w-auto sm:px-6">Создать комнату</a>
		</div>
	{:else}
		<ol class="space-y-1.5">
			{#each rows as row}
				<li
					class="flex items-center gap-3 rounded-[14px] border px-3 py-2.5 {row.username === me
						? 'border-amber/60 bg-ink-800'
						: 'border-line/70 bg-ink-850'}"
				>
					<span class="digits w-7 text-center text-sm {row.rank <= 3 ? 'text-amber' : 'text-dim'}">
						{row.rank}
					</span>
					<span class="grid h-9 w-9 shrink-0 place-items-center rounded-full bg-ink-700 text-xs font-semibold text-chalk">
						{initial(row.username)}
					</span>
					<div class="min-w-0 flex-1">
						<p class="truncate text-sm font-medium text-chalk">
							{row.username}
							{#if row.username === me}<span class="ml-1.5 text-xs text-amber">вы</span>{/if}
						</p>
						<p class="text-xs text-dim">{row.games_played} игр · {row.wins} побед</p>
					</div>
					<span class="digits text-base text-chalk">{row.total_score}</span>
				</li>
			{/each}
		</ol>
	{/if}
</div>
