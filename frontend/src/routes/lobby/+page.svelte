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
		loadRooms();
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
			blitz: '⚡ Блиц',
			sandbox: '🎓 Тренировка',
		};
		return labels[mode] || mode;
	}

	function getModeColor(mode: string): string {
		const colors: Record<string, string> = {
			classic: 'from-blue-500 to-cyan-500',
			ffa: 'from-orange-500 to-red-500',
			team: 'from-purple-500 to-pink-500',
			lives: 'from-red-500 to-rose-500',
			blitz: 'from-amber-400 to-orange-500',
			sandbox: 'from-teal-500 to-emerald-500',
		};
		return colors[mode] || 'from-slate-500 to-slate-600';
	}

	function getModeBadge(mode: string): string {
		const badges: Record<string, string> = {
			classic: 'bg-blue-500/10 text-blue-400 border-blue-500/20',
			ffa: 'bg-orange-500/10 text-orange-400 border-orange-500/20',
			team: 'bg-purple-500/10 text-purple-400 border-purple-500/20',
			lives: 'bg-red-500/10 text-red-400 border-red-500/20',
			blitz: 'bg-amber-500/10 text-amber-400 border-amber-500/20',
			sandbox: 'bg-teal-500/10 text-teal-400 border-teal-500/20',
		};
		return badges[mode] || 'bg-slate-500/10 text-slate-400 border-slate-500/20';
	}
</script>

<div class="mx-auto max-w-7xl px-6 py-8">
	<!-- Header -->
	<div class="mb-8 flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
		<div>
			<h1 class="text-3xl font-bold text-white">Игровые комнаты</h1>
			<p class="mt-1 text-slate-400">Присоединитесь к игре или создайте свою</p>
		</div>
		<a href="/room/create" class="relative group">
			<div class="absolute inset-0 bg-gradient-to-r from-blue-500 to-purple-600 rounded-xl blur-md opacity-40 group-hover:opacity-70 transition-opacity"></div>
			<div class="relative flex items-center gap-2 rounded-xl bg-gradient-to-r from-blue-500 to-purple-600 px-5 py-2.5 font-semibold text-white shadow-lg">
				<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"></path></svg>
				Создать комнату
			</div>
		</a>
	</div>

	<!-- Join room -->
	<div class="mb-8 relative overflow-hidden rounded-2xl border border-white/5 bg-white/[0.02] p-6 backdrop-blur-sm">
		<div class="absolute inset-0 bg-gradient-to-br from-green-500/5 to-transparent"></div>
		<div class="relative z-10">
			<h2 class="mb-4 text-lg font-semibold text-white flex items-center gap-2">
				<svg class="w-5 h-5 text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z"></path></svg>
				Присоединиться по коду
			</h2>
			<form onsubmit={(e) => { e.preventDefault(); joinRoom(); }} class="flex gap-3">
				<input
					type="text"
					bind:value={joinCode}
					placeholder="ABCDEF"
					maxlength="6"
					class="flex-1 rounded-xl border border-white/10 bg-white/5 px-4 py-3 text-center font-mono text-lg tracking-[0.3em] text-white uppercase placeholder-slate-600 focus:border-green-500/50 focus:outline-none focus:ring-2 focus:ring-green-500/20 transition-all"
				/>
				<button
					type="submit"
					disabled={joining || joinCode.length !== 6}
					class="rounded-xl bg-gradient-to-r from-green-500 to-emerald-600 px-8 py-3 font-semibold text-white shadow-lg transition-all hover:shadow-green-500/20 disabled:cursor-not-allowed disabled:opacity-50 disabled:shadow-none"
				>
					{#if joining}
						<span class="flex items-center gap-2">
							<svg class="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24"><circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle><path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"></path></svg>
							Поиск...
						</span>
					{:else}
						Войти
					{/if}
				</button>
			</form>
			{#if joinError}
				<div class="mt-3 flex items-center gap-2 text-sm text-red-400">
					<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path></svg>
					{joinError}
				</div>
			{/if}
		</div>
	</div>

	<!-- Rooms list -->
	<div class="relative overflow-hidden rounded-2xl border border-white/5 bg-white/[0.02] p-6 backdrop-blur-sm">
		<div class="absolute inset-0 bg-gradient-to-br from-blue-500/3 to-transparent"></div>
		<div class="relative z-10">
			<div class="mb-6 flex items-center justify-between">
				<h2 class="text-lg font-semibold text-white flex items-center gap-2">
					<svg class="w-5 h-5 text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10"></path></svg>
					Доступные комнаты
					<span class="ml-2 rounded-full bg-white/10 px-2.5 py-0.5 text-sm font-normal text-slate-300">{rooms.length}</span>
				</h2>
				{#if !loading && rooms.length > 0}
					<div class="flex items-center gap-2 text-xs text-slate-500">
						<span class="w-1.5 h-1.5 rounded-full bg-green-500 animate-pulse"></span>
						Обновляется автоматически
					</div>
				{/if}
			</div>

			{#if loading}
				<div class="py-16 text-center">
					<div class="mx-auto h-10 w-10 animate-spin rounded-full border-4 border-blue-500/30 border-t-blue-500"></div>
					<p class="mt-4 text-slate-500">Загрузка комнат...</p>
				</div>
			{:else if rooms.length === 0}
				<div class="py-16 text-center">
					<div class="mb-4 text-5xl">🎮</div>
					<p class="text-lg text-slate-300">Пока нет активных комнат</p>
					<p class="mt-2 text-sm text-slate-500">Создайте свою и пригласите друзей!</p>
				</div>
			{:else}
				<div class="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
					{#each rooms as room}
						<a
							href="/room/{room.code}"
							class="group relative overflow-hidden rounded-xl border border-white/5 bg-white/[0.02] p-5 transition-all hover:border-white/20 hover:bg-white/[0.05] hover:scale-[1.02]"
						>
							<!-- Gradient accent -->
							<div class="absolute top-0 left-0 right-0 h-1 bg-gradient-to-r {getModeColor(room.mode)} opacity-0 group-hover:opacity-100 transition-opacity"></div>

							<div class="flex items-start justify-between mb-3">
								<div>
									<div class="font-mono text-xs text-slate-500 tracking-wider">{room.code}</div>
									<div class="mt-1 font-semibold text-white group-hover:text-blue-300 transition-colors">
										{room.topic || 'Без темы'}
									</div>
								</div>
								<div class="opacity-0 group-hover:opacity-100 transition-all transform translate-x-2 group-hover:translate-x-0">
									<svg class="w-5 h-5 text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7l5 5m0 0l-5 5m5-5H6"></path></svg>
								</div>
							</div>

							<div class="flex items-center justify-between">
								<span class="inline-flex items-center rounded-lg border px-2.5 py-1 text-xs font-medium {getModeBadge(room.mode)}">
									{getModeLabel(room.mode)}
								</span>
								<span class="flex items-center gap-1.5 text-sm text-slate-400">
									<svg class="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
										<path d="M9 6a3 3 0 11-6 0 3 3 0 016 0zM17 6a3 3 0 11-6 0 3 3 0 016 0zM12.93 17c.046-.327.07-.66.07-1a6.97 6.97 0 00-1.5-4.33A5 5 0 0119 16v1h-6.07zM6 11a5 5 0 015 5v1H1v-1a5 5 0 015-5z"/>
									</svg>
									{room.player_count || 0}
								</span>
							</div>

							<div class="mt-3 flex items-center gap-2 text-xs text-slate-500">
								<div class="flex h-5 w-5 items-center justify-center rounded-full bg-gradient-to-br from-blue-500 to-purple-600 text-[10px] font-bold text-white">
									{room.host?.charAt(0).toUpperCase() || '?'}
								</div>
								<span>{room.host || 'Unknown'}</span>
							</div>
						</a>
					{/each}
				</div>
			{/if}
		</div>
	</div>
</div>
