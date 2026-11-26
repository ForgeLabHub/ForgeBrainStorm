<script lang="ts">
	import { api } from '$lib/api';
	import type { User } from '$lib/api';
	import { onMount } from 'svelte';

	let user: User | null = $state(null);
	let loading = $state(true);
	let error = $state('');

	onMount(async () => {
		try {
			const response = await api.me();
			if (response.user) {
				user = response.user;
			} else {
				error = 'Не удалось загрузить профиль';
			}
		} catch (e) {
			error = 'Ошибка соединения с сервером';
		} finally {
			loading = false;
		}
	});

	async function handleLogout() {
		try {
			await api.logout();
			window.location.href = '/auth/login';
		} catch (e) {
			console.error('Logout failed:', e);
		}
	}

	function getWinRate(user: User): number {
		if (user.games_played === 0) return 0;
		return Math.round((user.wins / user.games_played) * 100);
	}

	function getXpToNextLevel(user: User): { current: number; next: number; percent: number } {
		const currentLevel = user.level;
		const currentXpNeeded = 100 * (currentLevel * currentLevel);
		const nextLevelXp = 100 * ((currentLevel + 1) * (currentLevel + 1));
		const xpInCurrentLevel = user.xp - currentXpNeeded;
		const xpToNext = nextLevelXp - currentXpNeeded;
		const percent = Math.min(100, Math.round((xpInCurrentLevel / xpToNext) * 100));
		return { current: Math.max(0, xpInCurrentLevel), next: xpToNext, percent };
	}

	function getRankTitle(level: number): string {
		if (level >= 20) return 'Мастер';
		if (level >= 15) return 'Алмаз';
		if (level >= 10) return 'Золото';
		if (level >= 5) return 'Серебро';
		return 'Новичок';
	}
</script>

<div class="mx-auto max-w-2xl">
	{#if loading}
		<div class="py-20 text-center">
			<svg class="mx-auto h-8 w-8 animate-spin text-blue-500" fill="none" viewBox="0 0 24 24">
				<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
				<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"></path>
			</svg>
		</div>
	{:else if error}
		<div class="rounded-xl border border-red-500/20 bg-red-500/5 p-8 text-center">
			<p class="mb-4 text-red-400">{error}</p>
			<a href="/auth/login" class="rounded-lg bg-blue-500 px-6 py-2 font-medium text-white hover:bg-blue-600">
				Войти
			</a>
		</div>
	{:else if user}
		{@const xp = getXpToNextLevel(user)}
		{@const winRate = getWinRate(user)}
		{@const rankTitle = getRankTitle(user.level)}

		<!-- Profile header -->
		<div class="mb-6 flex items-center justify-between">
			<div class="flex items-center gap-4">
				<div class="flex h-14 w-14 items-center justify-center rounded-full bg-gradient-to-br from-blue-500 to-purple-600 text-xl font-bold text-white">
					{user.username.charAt(0).toUpperCase()}
				</div>
				<div>
					<h1 class="text-xl font-bold text-white">{user.username}</h1>
					<div class="mt-1 flex items-center gap-2">
						<span class="rounded-full bg-white/5 border border-white/10 px-2.5 py-0.5 text-xs font-medium text-slate-300">
							{rankTitle}
						</span>
						<span class="text-xs text-slate-500">Уровень {user.level}</span>
					</div>
				</div>
			</div>

			<button
				onclick={handleLogout}
				class="rounded-lg border border-white/10 bg-white/5 px-3 py-1.5 text-sm text-slate-300 transition hover:border-red-500/30 hover:bg-red-500/10 hover:text-red-400"
			>
				Выйти
			</button>
		</div>

		<!-- XP Progress -->
		<div class="mb-6 rounded-xl border border-white/5 bg-white/[0.02] p-4">
			<div class="mb-2 flex items-center justify-between text-sm">
				<span class="text-slate-400">Прогресс до уровня {user.level + 1}</span>
				<span class="text-slate-500">{xp.current} / {xp.next} XP</span>
			</div>
			<div class="h-2 overflow-hidden rounded-full bg-white/5">
				<div
					class="h-full rounded-full bg-gradient-to-r from-blue-500 to-purple-500 transition-all duration-500"
					style="width: {xp.percent}%"
				></div>
			</div>
		</div>

		<!-- Stats -->
		<div class="mb-6 grid grid-cols-2 gap-3 sm:grid-cols-4">
			<div class="rounded-xl border border-white/5 bg-white/[0.02] p-4">
				<div class="mb-1 flex items-center gap-1.5 text-xs font-medium uppercase tracking-wide text-blue-400">
					<svg class="h-3.5 w-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z"/>
					</svg>
					Опыт
				</div>
				<div class="text-2xl font-bold text-white">{user.xp.toLocaleString()}</div>
			</div>

			<div class="rounded-xl border border-white/5 bg-white/[0.02] p-4">
				<div class="mb-1 flex items-center gap-1.5 text-xs font-medium uppercase tracking-wide text-yellow-400">
					<svg class="h-3.5 w-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>
					</svg>
					Монеты
				</div>
				<div class="text-2xl font-bold text-yellow-400">{user.coins}</div>
			</div>

			<div class="rounded-xl border border-white/5 bg-white/[0.02] p-4">
				<div class="mb-1 flex items-center gap-1.5 text-xs font-medium uppercase tracking-wide text-purple-400">
					<svg class="h-3.5 w-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z"/>
					</svg>
					Игр
				</div>
				<div class="text-2xl font-bold text-white">{user.games_played}</div>
			</div>

			<div class="rounded-xl border border-white/5 bg-white/[0.02] p-4">
				<div class="mb-1 flex items-center gap-1.5 text-xs font-medium uppercase tracking-wide text-green-400">
					<svg class="h-3.5 w-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"/>
					</svg>
					Победы
				</div>
				<div class="text-2xl font-bold text-green-400">{winRate}%</div>
			</div>
		</div>

		<!-- Quick actions -->
		<div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
			<a href="/lobby" class="group rounded-xl border border-white/5 bg-white/[0.02] p-4 transition hover:border-white/20">
				<div class="flex items-center gap-3">
					<svg class="h-5 w-5 text-slate-400 transition group-hover:text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z"/>
					</svg>
					<div>
						<div class="font-medium text-white">В лобби</div>
						<div class="text-sm text-slate-400">Найти или создать комнату</div>
					</div>
				</div>
			</a>

			<a href="/settings" class="group rounded-xl border border-white/5 bg-white/[0.02] p-4 transition hover:border-white/20">
				<div class="flex items-center gap-3">
					<svg class="h-5 w-5 text-slate-400 transition group-hover:text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"/>
					</svg>
					<div>
						<div class="font-medium text-white">Настройки AI</div>
						<div class="text-sm text-slate-400">Переключение провайдеров</div>
					</div>
				</div>
			</a>
		</div>
	{/if}
</div>
