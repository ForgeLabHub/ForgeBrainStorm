<script lang="ts">
	import { api } from '$lib/api';
	import type { Room } from '$lib/api';
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';

	let rooms: Room[] = [];
	let loading = true;
	let joinCode = '';
	let joinError = '';
	let joining = false;

	onMount(async () => {
		await loadRooms();
		// Refresh rooms every 5 seconds
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
			joinError = 'Ошибка соединения с сервером';
		} finally {
			joining = false;
		}
	}

	function getModeLabel(mode: string): string {
		const labels: Record<string, string> = {
			classic: '🎯 Классика',
			ffa: '⚔️ Каждый за себя',
			team: '👥 Команды',
			lives: '❤️ На выживание',
			coop: '🤝 Кооператив',
			svoyaigra: '🎲 Своя Игра',
		};
		return labels[mode] || mode;
	}

	function getModeColor(mode: string): string {
		const colors: Record<string, string> = {
			classic: 'text-blue-400',
			ffa: 'text-orange-400',
			team: 'text-purple-400',
			lives: 'text-red-400',
			coop: 'text-green-400',
			svoyaigra: 'text-yellow-400',
		};
		return colors[mode] || 'text-slate-400';
	}
</script>

<div class="mx-auto max-w-6xl px-6 py-8">
	<!-- Header -->
	<div class="mb-8 flex items-center justify-between">
		<div>
			<h1 class="text-3xl font-bold text-white">Комнаты</h1>
			<p class="mt-1 text-slate-400">Присоединитесь к игре или создайте свою</p>
		</div>
		<a
			href="/room/create"
			class="rounded-lg bg-blue-600 px-6 py-3 font-semibold text-white transition hover:bg-blue-700"
		>
			+ Создать комнату
		</a>
	</div>

	<!-- Join room -->
	<div class="mb-8 rounded-xl border border-slate-700 bg-slate-800 p-6">
		<h2 class="mb-4 text-lg font-semibold text-white">Присоединиться по коду</h2>
		<form on:submit|preventDefault={joinRoom} class="flex gap-3">
			<input
				type="text"
				bind:value={joinCode}
				placeholder="ABCDEF"
				maxlength="6"
				class="flex-1 rounded-lg border border-slate-600 bg-slate-700 px-4 py-2.5 text-center font-mono text-lg tracking-widest text-white uppercase placeholder-slate-500 focus:border-blue-500 focus:outline-none"
			/>
			<button
				type="submit"
				disabled={joining || joinCode.length !== 6}
				class="rounded-lg bg-green-600 px-6 py-2.5 font-semibold text-white transition hover:bg-green-700 disabled:cursor-not-allowed disabled:opacity-50"
			>
				{joining ? '...' : 'Войти'}
			</button>
		</form>
		{#if joinError}
			<p class="mt-2 text-sm text-red-400">{joinError}</p>
		{/if}
	</div>

	<!-- Rooms list -->
	<div class="rounded-xl border border-slate-700 bg-slate-800 p-6">
		<h2 class="mb-4 text-lg font-semibold text-white">Доступные комнаты ({rooms.length})</h2>

		{#if loading}
			<div class="py-12 text-center">
				<div class="mx-auto h-8 w-8 animate-spin rounded-full border-4 border-blue-500 border-t-transparent"></div>
			</div>
		{:else if rooms.length === 0}
			<div class="py-12 text-center text-slate-400">
				<div class="mb-3 text-4xl">🎮</div>
				<p>Нет активных комнат</p>
				<p class="mt-1 text-sm">Создайте свою и пригласите друзей!</p>
			</div>
		{:else}
			<div class="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
				{#each rooms as room}
					<a
						href="/room/{room.code}"
						class="group rounded-lg border border-slate-600 bg-slate-700 p-4 transition hover:border-blue-500 hover:bg-slate-600"
					>
						<div class="mb-3 flex items-start justify-between">
							<div>
								<div class="font-mono text-sm text-slate-400">{room.code}</div>
								<div class="mt-1 font-semibold text-white">{room.topic || 'Без темы'}</div>
							</div>
							<div class="text-2xl opacity-0 transition group-hover:opacity-100">→</div>
						</div>

						<div class="flex items-center justify-between text-sm">
							<span class="{getModeColor(room.mode)}">
								{getModeLabel(room.mode)}
							</span>
							<span class="flex items-center gap-1 text-slate-400">
								<svg class="h-4 w-4" fill="currentColor" viewBox="0 0 20 20">
									<path d="M9 6a3 3 0 11-6 0 3 3 0 016 0zM17 6a3 3 0 11-6 0 3 3 0 016 0zM12.93 17c.046-.327.07-.66.07-1a6.97 6.97 0 00-1.5-4.33A5 5 0 0119 16v1h-6.07zM6 11a5 5 0 015 5v1H1v-1a5 5 0 015-5z"/>
								</svg>
								{room.player_count || 0}
							</span>
						</div>

						<div class="mt-2 text-xs text-slate-500">
							Хост: {room.host}
						</div>
					</a>
				{/each}
			</div>
		{/if}
	</div>
</div>
