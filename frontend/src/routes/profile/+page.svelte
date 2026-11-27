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

	function getRankTitle(level: number): { title: string; color: string; gradient: string } {
		if (level >= 20) return { title: 'Мастер', color: 'text-purple-400', gradient: 'from-purple-500 to-pink-500' };
		if (level >= 15) return { title: 'Алмаз', color: 'text-cyan-400', gradient: 'from-cyan-500 to-blue-500' };
		if (level >= 10) return { title: 'Золото', color: 'text-yellow-400', gradient: 'from-yellow-500 to-orange-500' };
		if (level >= 5) return { title: 'Серебро', color: 'text-slate-300', gradient: 'from-slate-400 to-slate-500' };
		return { title: 'Новичок', color: 'text-slate-400', gradient: 'from-slate-500 to-slate-600' };
	}
</script>

<div class="mx-auto max-w-3xl animate-fade-in">
	{#if loading}
		<div class="py-20 text-center">
			<svg class="mx-auto h-10 w-10 animate-spin text-blue-500" fill="none" viewBox="0 0 24 24">
				<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
				<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"></path>
			</svg>
		</div>
	{:else if error}
		<div class="glass-card rounded-2xl p-8 text-center">
			<div class="mx-auto mb-4 flex h-16 w-16 items-center justify-center rounded-2xl bg-red-500/10 border border-red-500/20">
				<svg class="h-8 w-8 text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>
				</svg>
			</div>
			<p class="mb-4 text-red-400">{error}</p>
			<a href="/auth/login" class="btn-primary inline-block rounded-xl px-6 py-3 font-semibold text-white">
				Войти
			</a>
		</div>
	{:else if user}
		{@const xp = getXpToNextLevel(user)}
		{@const winRate = getWinRate(user)}
		{@const rank = getRankTitle(user.level)}

		<!-- Profile header -->
		<div class="glass-card rounded-2xl p-5 sm:p-6 md:p-8 mb-6">
			<div class="flex flex-col sm:flex-row items-center sm:items-start gap-4 sm:gap-6">
				<!-- Avatar -->
				<div class="relative">
					<div class="absolute inset-0 bg-gradient-to-br from-blue-500 to-purple-600 rounded-2xl blur opacity-50"></div>
					<div class="relative flex h-16 w-16 sm:h-20 sm:w-20 items-center justify-center rounded-2xl bg-gradient-to-br from-blue-500 to-purple-600 text-2xl sm:text-3xl font-black text-white shadow-xl">
						{user.username.charAt(0).toUpperCase()}
					</div>
				</div>

				<!-- Info -->
				<div class="flex-1 text-center sm:text-left">
					<h1 class="text-xl sm:text-2xl font-bold text-white truncate">{user.username}</h1>
					<div class="mt-2 flex flex-wrap items-center justify-center sm:justify-start gap-2">
						<span class="px-3 py-1 rounded-lg bg-gradient-to-r {rank.gradient} text-white text-xs font-bold shadow-lg">
							{rank.title}
						</span>
						<span class="px-3 py-1 rounded-lg bg-white/5 border border-white/10 text-xs font-medium text-slate-300">
							Уровень {user.level}
						</span>
					</div>
				</div>

			<!-- Logout button -->
			<button
				onclick={handleLogout}
				class="btn-secondary rounded-xl px-4 py-2 text-sm font-medium text-slate-300 hover:border-red-500/30 hover:bg-red-500/10 hover:text-red-400 w-full sm:w-auto"
			>
				<span class="flex items-center justify-center gap-2">
					<svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1"/>
					</svg>
					Выйти
				</span>
			</button>
			</div>

			<!-- XP Progress -->
			<div class="mt-6 pt-6 border-t border-white/5">
				<div class="flex items-center justify-between mb-2">
					<span class="text-sm font-medium text-slate-300">Прогресс до уровня {user.level + 1}</span>
					<span class="text-sm text-slate-500">{xp.current} / {xp.next} XP</span>
				</div>
				<div class="h-2.5 overflow-hidden rounded-full bg-white/5">
					<div
						class="h-full rounded-full bg-gradient-to-r from-blue-500 to-purple-500 transition-all duration-500 shadow-lg shadow-blue-500/50"
						style="width: {xp.percent}%"
					></div>
				</div>
			</div>
		</div>

	<!-- Stats grid -->
	<div class="grid grid-cols-2 gap-3 sm:gap-4 mb-6 sm:grid-cols-4">
		<div class="glass-card rounded-xl p-4 sm:p-5 transition-all duration-300">
			<div class="flex items-center gap-2 mb-2">
				<div class="flex h-8 w-8 items-center justify-center rounded-lg bg-blue-500/20 border border-blue-500/30">
					<svg class="h-4 w-4 text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z"/>
					</svg>
				</div>
				<span class="text-xs font-semibold uppercase tracking-wide text-blue-400">Опыт</span>
			</div>
			<div class="text-xl sm:text-2xl font-black text-white">{user.xp.toLocaleString()}</div>
		</div>

		<div class="glass-card rounded-xl p-4 sm:p-5 transition-all duration-300">
			<div class="flex items-center gap-2 mb-2">
				<div class="flex h-8 w-8 items-center justify-center rounded-lg bg-yellow-500/20 border border-yellow-500/30">
					<svg class="h-4 w-4 text-yellow-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>
					</svg>
				</div>
				<span class="text-xs font-semibold uppercase tracking-wide text-yellow-400">Монеты</span>
			</div>
			<div class="text-xl sm:text-2xl font-black text-yellow-400">{user.coins}</div>
			</div>

		<div class="glass-card rounded-xl p-4 sm:p-5 transition-all duration-300">
			<div class="flex items-center gap-2 mb-2">
				<div class="flex h-8 w-8 items-center justify-center rounded-lg bg-purple-500/20 border border-purple-500/30">
					<svg class="h-4 w-4 text-purple-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z"/>
					</svg>
				</div>
				<span class="text-xs font-semibold uppercase tracking-wide text-purple-400">Игр</span>
			</div>
			<div class="text-xl sm:text-2xl font-black text-white">{user.games_played}</div>
		</div>

		<div class="glass-card rounded-xl p-4 sm:p-5 transition-all duration-300">
			<div class="flex items-center gap-2 mb-2">
				<div class="flex h-8 w-8 items-center justify-center rounded-lg bg-green-500/20 border border-green-500/30">
					<svg class="h-4 w-4 text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"/>
					</svg>
				</div>
				<span class="text-xs font-semibold uppercase tracking-wide text-green-400">Победы</span>
			</div>
			<div class="text-xl sm:text-2xl font-black text-green-400">{winRate}%</div>
		</div>
	</div>

	<!-- Quick actions -->
	<div class="grid grid-cols-1 gap-3 sm:gap-4 sm:grid-cols-2">
		<a href="/lobby" class="glass-card group rounded-xl p-4 sm:p-5 transition-all duration-300">
				<div class="flex items-center gap-4">
					<div class="flex h-12 w-12 items-center justify-center rounded-xl bg-gradient-to-br from-blue-500/20 to-blue-600/20 border border-blue-500/30 group-hover:border-blue-500/50 transition">
						<svg class="h-6 w-6 text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z"/>
						</svg>
					</div>
					<div>
						<div class="font-semibold text-white group-hover:text-blue-400 transition">В лобби</div>
						<div class="text-sm text-slate-400">Найти или создать комнату</div>
					</div>
				</div>
			</a>

		<a href="/settings" class="glass-card group rounded-xl p-4 sm:p-5 transition-all duration-300">
			<div class="flex items-center gap-4">
				<div class="flex h-12 w-12 items-center justify-center rounded-xl bg-gradient-to-br from-purple-500/20 to-purple-600/20 border border-purple-500/30 group-hover:border-purple-500/50 transition">
					<svg class="h-6 w-6 text-purple-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"/>
					</svg>
				</div>
				<div>
					<div class="font-semibold text-white group-hover:text-purple-400 transition">Настройки AI</div>
					<div class="text-sm text-slate-400">Переключение провайдеров</div>
				</div>
			</div>
		</a>
		</div>
	{/if}
</div>
