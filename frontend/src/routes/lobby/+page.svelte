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

	onMount(() => {
		void loadRooms();
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
			classic: 'from-blue-500/20 to-blue-600/20 border-blue-500/30 text-blue-400',
			ffa: 'from-orange-500/20 to-orange-600/20 border-orange-500/30 text-orange-400',
			team: 'from-purple-500/20 to-purple-600/20 border-purple-500/30 text-purple-400',
			lives: 'from-red-500/20 to-red-600/20 border-red-500/30 text-red-400',
			coop: 'from-green-500/20 to-green-600/20 border-green-500/30 text-green-400',
			svoyaigra: 'from-yellow-500/20 to-yellow-600/20 border-yellow-500/30 text-yellow-400',
		};
		return colors[mode] || 'from-slate-500/20 to-slate-600/20 border-slate-500/30 text-slate-400';
	}
</script>

<div class="mx-auto max-w-5xl animate-fade-in">
	<!-- Header -->
	<div class="mb-8 flex items-center justify-between">
		<div>
			<h1 class="text-3xl font-bold text-white">Комнаты</h1>
			<p class="mt-1 text-sm text-slate-500">Присоединяйтесь к игре или создайте свою</p>
		</div>
		<a href="/room/create" class="btn-primary rounded-xl px-5 py-2.5 text-sm font-semibold text-white">
			<span class="flex items-center gap-2">
				<svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"/>
				</svg>
				Создать
			</span>
		</a>
	</div>

	<!-- Join room card -->
	<div class="glass-card rounded-2xl p-6 sm:p-8 mb-6">
		<h2 class="mb-4 text-sm font-semibold text-slate-300 uppercase tracking-wider">Присоединиться по коду</h2>
		<form onsubmit={(e) => { e.preventDefault(); joinRoom(); }} class="flex gap-3">
			<input
				type="text"
				bind:value={joinCode}
				placeholder="ABCDEF"
				maxlength="6"
				class="input-modern flex-1 rounded-xl px-4 py-3 font-mono text-center text-xl uppercase placeholder-slate-600 tracking-widest"
			/>
			<button
				type="submit"
				disabled={joining || joinCode.length !== 6}
				class="btn-secondary rounded-xl px-8 py-3 font-semibold text-white disabled:cursor-not-allowed disabled:opacity-50"
			>
				{#if joining}
					<svg class="h-5 w-5 animate-spin" fill="none" viewBox="0 0 24 24">
						<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
						<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"></path>
					</svg>
				{:else}
					Войти
				{/if}
			</button>
		</form>
		{#if joinError}
			<p class="mt-3 text-sm text-red-400 flex items-center gap-2">
				<svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>
				</svg>
				{joinError}
			</p>
		{/if}
	</div>

	<!-- Rooms list -->
	<div class="glass-card rounded-2xl p-6 sm:p-8">
		<div class="mb-6 flex items-center justify-between">
			<h2 class="text-sm font-semibold text-slate-300 uppercase tracking-wider">Доступные</h2>
			<span class="px-3 py-1 rounded-full bg-white/5 border border-white/10 text-xs font-medium text-slate-400">
				{rooms.length} {rooms.length === 1 ? 'комната' : 'комнат'}
			</span>
		</div>

		{#if loading}
			<div class="py-16 text-center">
				<svg class="mx-auto h-10 w-10 animate-spin text-blue-500" fill="none" viewBox="0 0 24 24">
					<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
					<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"></path>
				</svg>
			</div>
		{:else if rooms.length === 0}
			<div class="py-16 text-center">
				<div class="mx-auto mb-4 flex h-16 w-16 items-center justify-center rounded-2xl bg-white/5 border border-white/10">
					<svg class="h-8 w-8 text-slate-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z"/>
					</svg>
				</div>
				<p class="text-slate-400 font-medium">Нет активных комнат</p>
				<p class="mt-1 text-sm text-slate-500">Создайте свою первую комнату</p>
			</div>
		{:else}
			<div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
				{#each rooms as room, i}
					<a
						href="/room/{room.code}"
						class="group glass-card rounded-xl p-5 hover:scale-[1.02] transition-all duration-300 animate-fade-in"
						style="animation-delay: {i * 50}ms"
					>
						<div class="mb-3 flex items-start justify-between">
							<div>
								<div class="font-mono text-xs text-slate-500 tracking-wider">{room.code}</div>
								<div class="mt-1 font-semibold text-white group-hover:text-blue-400 transition">
									{room.topic || 'Без темы'}
								</div>
							</div>
							<svg class="h-5 w-5 text-slate-600 transition-all group-hover:text-white group-hover:translate-x-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"/>
							</svg>
						</div>

						<div class="flex items-center justify-between">
							<span class="inline-flex items-center rounded-lg px-3 py-1 text-xs font-semibold bg-gradient-to-r border {getModeColor(room.mode)}">
								{getModeLabel(room.mode)}
							</span>
							<span class="flex items-center gap-1.5 text-sm text-slate-400">
								<svg class="h-4 w-4" fill="currentColor" viewBox="0 0 20 20">
									<path d="M9 6a3 3 0 11-6 0 3 3 0 016 0zM17 6a3 3 0 11-6 0 3 3 0 016 0zM12.93 17c.046-.327.07-.66.07-1a6.97 6.97 0 00-1.5-4.33A5 5 0 0119 16v1h-6.07zM6 11a5 5 0 015 5v1H1v-1a5 5 0 015-5z"/>
								</svg>
								<span class="font-medium">{room.player_count || 0}</span>
							</span>
						</div>

						<div class="mt-3 text-xs text-slate-500 flex items-center gap-1.5">
							<svg class="h-3.5 w-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z"/>
							</svg>
							{room.host}
						</div>
					</a>
				{/each}
			</div>
		{/if}
	</div>
</div>
