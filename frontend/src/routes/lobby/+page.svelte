<script lang="ts">
	import { api } from '$lib/api';
	import type { Room } from '$lib/api';
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';

	let rooms: Room[] = $state([]);
	let loading = $state(true);
	let joinCode = $state('');
	let joinError = $state('');
	let joining = $state(false);

<<<<<<< HEAD
	onMount(() => {
		void loadRooms();
=======
	onMount(async () => {
		await loadRooms();
>>>>>>> 4808142 (Merge pull request #15 from ForgeLabHub/arena/ca3aa14f-forgebrainstorm)
		const interval = setInterval(loadRooms, 5000);
		return () => clearInterval(interval);
	});

	async function loadRooms() {
		try {
			const response = await api.listRooms();
			rooms = response.rooms || [];
		} catch (e) {
			console.error('Failed to load rooms:', e);
		} finally {
			loading = false;
		}
	}

	async function joinRoom() {
		const code = joinCode.trim().toUpperCase();
		if (code.length !== 6) {
			joinError = 'Код комнаты — 6 символов';
			return;
		}
		joining = true;
		joinError = '';
		try {
			const response = await api.getRoom(code);
			if (response.ok && response.room) {
				goto(`/room/${code}`);
			} else {
				joinError = 'Комната не найдена';
			}
		} catch (e) {
			joinError = 'Ошибка соединения';
		} finally {
			joining = false;
		}
	}

	function getModeLabel(mode: string): string {
		const labels: Record<string, string> = {
			classic: 'Классика',
			ffa: 'FFA',
			team: 'Команды',
			lives: 'Выживание',
			coop: 'Кооператив',
			svoyaigra: 'Своя Игра',
		};
		return labels[mode] || mode;
	}

	function getModeColor(mode: string): string {
		const colors: Record<string, string> = {
			classic: 'text-blue-400 bg-blue-500/10',
			ffa: 'text-orange-400 bg-orange-500/10',
			team: 'text-purple-400 bg-purple-500/10',
			lives: 'text-red-400 bg-red-500/10',
			coop: 'text-green-400 bg-green-500/10',
			svoyaigra: 'text-yellow-400 bg-yellow-500/10',
		};
		return colors[mode] || 'text-slate-400 bg-slate-500/10';
	}
</script>

<div class="mx-auto max-w-4xl">
	<div class="mb-8 flex items-center justify-between">
		<h1 class="text-2xl font-bold text-white">Комнаты</h1>
		<a href="/room/create" class="rounded-lg bg-blue-500 px-4 py-2 text-sm font-medium hover:bg-blue-600 transition">
			Создать
		</a>
	</div>

	<!-- Join room -->
	<div class="mb-6 rounded-xl border border-white/5 bg-white/[0.02] p-4 sm:p-6">
		<h2 class="mb-3 text-sm font-medium text-slate-300">Присоединиться по коду</h2>
		<form onsubmit={(e) => { e.preventDefault(); joinRoom(); }} class="flex gap-2">
			<input
				type="text"
				bind:value={joinCode}
				placeholder="ABCDEF"
				maxlength="6"
				class="flex-1 rounded-lg border border-white/10 bg-white/5 px-4 py-2 font-mono text-center text-lg uppercase placeholder-slate-600 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
			/>
			<button
				type="submit"
				disabled={joining || joinCode.length !== 6}
				class="rounded-lg bg-white/5 border border-white/10 px-6 py-2 font-medium text-white transition hover:bg-white/10 disabled:cursor-not-allowed disabled:opacity-50"
			>
				{joining ? '...' : 'Войти'}
			</button>
		</form>
		{#if joinError}
			<p class="mt-2 text-sm text-red-400">{joinError}</p>
		{/if}
	</div>

	<!-- Rooms list -->
	<div class="rounded-xl border border-white/5 bg-white/[0.02] p-4 sm:p-6">
		<div class="mb-4 flex items-center justify-between">
			<h2 class="text-sm font-medium text-slate-300">Доступные</h2>
			<span class="text-sm text-slate-500">{rooms.length}</span>
		</div>

		{#if loading}
			<div class="py-12 text-center">
				<svg class="mx-auto h-8 w-8 animate-spin text-blue-500" fill="none" viewBox="0 0 24 24">
					<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
					<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"></path>
				</svg>
			</div>
		{:else if rooms.length === 0}
			<div class="py-12 text-center text-slate-400">
				<p class="mb-1">Нет активных комнат</p>
				<p class="text-sm text-slate-500">Создайте свою</p>
			</div>
		{:else}
			<div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
				{#each rooms as room}
					<a
						href="/room/{room.code}"
						class="group rounded-lg border border-white/5 bg-white/[0.02] p-4 transition hover:border-white/20 hover:bg-white/[0.04]"
					>
						<div class="mb-2 flex items-start justify-between">
							<div>
								<div class="font-mono text-xs text-slate-500">{room.code}</div>
								<div class="mt-1 font-medium text-white group-hover:text-blue-400 transition">
									{room.topic || 'Без темы'}
								</div>
							</div>
							<svg class="h-4 w-4 text-slate-600 transition group-hover:text-white group-hover:translate-x-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"/>
							</svg>
						</div>

						<div class="flex items-center justify-between">
							<span class="inline-flex items-center rounded px-2 py-0.5 text-xs font-medium {getModeColor(room.mode)}">
								{getModeLabel(room.mode)}
							</span>
							<span class="flex items-center gap-1 text-xs text-slate-400">
								<svg class="h-3.5 w-3.5" fill="currentColor" viewBox="0 0 20 20">
									<path d="M9 6a3 3 0 11-6 0 3 3 0 016 0zM17 6a3 3 0 11-6 0 3 3 0 016 0zM12.93 17c.046-.327.07-.66.07-1a6.97 6.97 0 00-1.5-4.33A5 5 0 0119 16v1h-6.07zM6 11a5 5 0 015 5v1H1v-1a5 5 0 015-5z"/>
								</svg>
								{room.player_count || 0}
							</span>
						</div>

						<div class="mt-2 text-xs text-slate-500">
							{room.host}
						</div>
					</a>
				{/each}
			</div>
		{/if}
	</div>
</div>
