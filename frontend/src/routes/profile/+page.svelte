<script lang="ts">
	import { api } from '$lib/api';
	import type { User } from '$lib/api';
	import { onMount } from 'svelte';

	let user: User | null = $state(null);
	let loading = $state(true);
	let error = $state('');
	let visible = $state(false);

	$: if (typeof window !== 'undefined') {
		setTimeout(() => (visible = true), 100);
	}

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
		if (level >= 20) return '🏆 Мастер';
		if (level >= 15) return '💎 Алмаз';
		if (level >= 10) return '🥇 Золото';
		if (level >= 5) return '🥈 Серебро';
		return '🥉 Новичок';
	}

	function getRankGradient(level: number): string {
		if (level >= 20) return 'from-yellow-400 via-orange-500 to-red-500';
		if (level >= 15) return 'from-cyan-400 via-blue-500 to-purple-600';
		if (level >= 10) return 'from-yellow-400 via-yellow-500 to-amber-600';
		if (level >= 5) return 'from-slate-300 via-slate-400 to-slate-500';
		return 'from-orange-400 via-orange-500 to-amber-600';
	}
</script>

<div class="mx-auto max-w-4xl px-4 py-8 sm:px-6">
	{#if loading}
		<!-- Skeleton loader -->
		<div class="space-y-6">
			<div class="relative overflow-hidden rounded-3xl border border-white/5 bg-white/[0.02] p-8">
				<div class="flex items-center gap-4">
					<div class="h-20 w-20 animate-pulse rounded-2xl bg-white/5"></div>
					<div class="flex-1 space-y-3">
						<div class="h-6 w-48 animate-pulse rounded bg-white/5"></div>
						<div class="h-4 w-32 animate-pulse rounded bg-white/5"></div>
					</div>
				</div>
			</div>
			<div class="grid grid-cols-2 gap-4 md:grid-cols-4">
				{#each Array(4) as _}
					<div class="h-24 animate-pulse rounded-2xl bg-white/5"></div>
				{/each}
			</div>
		</div>
	{:else if error}
		<div class="relative overflow-hidden rounded-3xl border border-red-500/20 bg-red-500/5 p-12 text-center">
			<div class="absolute inset-0 bg-gradient-to-br from-red-500/5 to-transparent"></div>
			<div class="relative z-10">
				<div class="mb-4 text-6xl">🔒</div>
				<h2 class="mb-2 text-2xl font-bold text-white">Требуется авторизация</h2>
				<p class="mb-6 text-slate-400">{error}</p>
				<a
					href="/auth/login"
					class="inline-flex items-center gap-2 rounded-xl bg-gradient-to-r from-blue-500 to-purple-600 px-6 py-3 font-semibold text-white shadow-lg transition hover:shadow-blue-500/20"
				>
					<svg class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 16l-4-4m0 0l4-4m-4 4h14m-5 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h7a3 3 0 013 3v1"></path>
					</svg>
					Войти
				</a>
			</div>
		</div>
	{:else if user}
		{@const xp = getXpToNextLevel(user)}
		{@const winRate = getWinRate(user)}
		{@const rankTitle = getRankTitle(user.level)}
		{@const rankGradient = getRankGradient(user.level)}

		<!-- Profile header card -->
		<div
			class="relative overflow-hidden rounded-3xl border border-white/5 bg-white/[0.02] p-6 backdrop-blur-sm transition-all duration-700 sm:p-8 {visible ? 'translate-y-0 opacity-100' : 'translate-y-8 opacity-0'}"
		>
			<!-- Background gradient -->
			<div class="absolute inset-0 bg-gradient-to-br {rankGradient} opacity-5"></div>
			<div class="absolute -right-20 -top-20 h-64 w-64 rounded-full bg-gradient-to-br {rankGradient} opacity-10 blur-3xl"></div>

			<div class="relative z-10">
				<div class="flex flex-col items-start gap-6 sm:flex-row sm:items-center sm:justify-between">
					<div class="flex items-center gap-4 sm:gap-5">
						<!-- Avatar -->
						<div class="relative">
							<div class="absolute inset-0 animate-pulse rounded-2xl bg-gradient-to-r {rankGradient} blur-lg opacity-50"></div>
							<div class="relative flex h-16 w-16 items-center justify-center rounded-2xl bg-gradient-to-br {rankGradient} text-2xl font-bold text-white shadow-lg sm:h-20 sm:w-20 sm:text-3xl">
								{user.username.charAt(0).toUpperCase()}
							</div>
							<!-- Level badge -->
							<div class="absolute -bottom-2 -right-2 flex h-7 w-7 items-center justify-center rounded-lg border-2 border-[#0a0b1a] bg-white/10 text-xs font-bold text-white backdrop-blur-sm sm:h-8 sm:w-8 sm:text-sm">
								{user.level}
							</div>
						</div>

						<!-- User info -->
						<div>
							<h1 class="text-2xl font-bold text-white sm:text-3xl">{user.username}</h1>
							<div class="mt-1 flex flex-wrap items-center gap-2">
								<span class="rounded-full bg-white/5 border border-white/10 px-3 py-1 text-xs font-medium text-slate-300 sm:text-sm">
									{rankTitle}
								</span>
								<span class="rounded-full bg-white/5 border border-white/10 px-3 py-1 text-xs font-medium text-slate-400 sm:text-sm">
									Уровень {user.level}
								</span>
							</div>
						</div>
					</div>

					<!-- Logout button -->
					<button
						onclick={handleLogout}
						class="flex items-center gap-2 rounded-xl border border-white/10 bg-white/5 px-4 py-2 text-sm font-medium text-slate-300 backdrop-blur-sm transition hover:border-red-500/30 hover:bg-red-500/10 hover:text-red-400"
					>
						<svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1"></path>
						</svg>
						Выйти
					</button>
				</div>

				<!-- XP Progress -->
				<div class="mt-8">
					<div class="mb-3 flex items-center justify-between">
						<span class="text-sm text-slate-400">Прогресс до уровня {user.level + 1}</span>
						<span class="text-sm font-medium text-white">
							{xp.current.toLocaleString()} <span class="text-slate-500">/ {xp.next.toLocaleString()} XP</span>
						</span>
					</div>
					<div class="relative h-3 overflow-hidden rounded-full bg-white/5">
						<div class="absolute inset-0 bg-gradient-to-r {rankGradient} opacity-20 blur-sm"></div>
						<div
							class="relative h-full rounded-full bg-gradient-to-r {rankGradient} transition-all duration-1000"
							style="width: {xp.percent}%"
						></div>
					</div>
					<div class="mt-2 text-right text-xs text-slate-500">{xp.percent}%</div>
				</div>
			</div>
		</div>

		<!-- Stats grid -->
		<div
			class="mt-6 grid grid-cols-2 gap-3 sm:gap-4 md:grid-cols-4 transition-all duration-700 delay-100 {visible ? 'translate-y-0 opacity-100' : 'translate-y-8 opacity-0'}"
		>
			<!-- XP -->
			<div class="group relative overflow-hidden rounded-2xl border border-white/5 bg-white/[0.02] p-4 backdrop-blur-sm transition-all hover:border-blue-500/30 hover:bg-white/[0.04] sm:p-5">
				<div class="absolute inset-0 bg-gradient-to-br from-blue-500/5 to-transparent opacity-0 transition-opacity group-hover:opacity-100"></div>
				<div class="relative z-10">
					<div class="flex items-center gap-2 text-xs font-medium uppercase tracking-wider text-blue-400">
						<svg class="h-3.5 w-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z"></path>
						</svg>
						Опыт
					</div>
					<div class="mt-2 text-2xl font-bold text-white sm:text-3xl">{user.xp.toLocaleString()}</div>
				</div>
			</div>

			<!-- Coins -->
			<div class="group relative overflow-hidden rounded-2xl border border-white/5 bg-white/[0.02] p-4 backdrop-blur-sm transition-all hover:border-yellow-500/30 hover:bg-white/[0.04] sm:p-5">
				<div class="absolute inset-0 bg-gradient-to-br from-yellow-500/5 to-transparent opacity-0 transition-opacity group-hover:opacity-100"></div>
				<div class="relative z-10">
					<div class="flex items-center gap-2 text-xs font-medium uppercase tracking-wider text-yellow-400">
						<svg class="h-3.5 w-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>
						</svg>
						Монеты
					</div>
					<div class="mt-2 text-2xl font-bold text-yellow-400 sm:text-3xl">{user.coins.toLocaleString()}</div>
				</div>
			</div>

			<!-- Games -->
			<div class="group relative overflow-hidden rounded-2xl border border-white/5 bg-white/[0.02] p-4 backdrop-blur-sm transition-all hover:border-purple-500/30 hover:bg-white/[0.04] sm:p-5">
				<div class="absolute inset-0 bg-gradient-to-br from-purple-500/5 to-transparent opacity-0 transition-opacity group-hover:opacity-100"></div>
				<div class="relative z-10">
					<div class="flex items-center gap-2 text-xs font-medium uppercase tracking-wider text-purple-400">
						<svg class="h-3.5 w-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z"></path>
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>
						</svg>
						Игр
					</div>
					<div class="mt-2 text-2xl font-bold text-white sm:text-3xl">{user.games_played}</div>
				</div>
			</div>

			<!-- Win Rate -->
			<div class="group relative overflow-hidden rounded-2xl border border-white/5 bg-white/[0.02] p-4 backdrop-blur-sm transition-all hover:border-green-500/30 hover:bg-white/[0.04] sm:p-5">
				<div class="absolute inset-0 bg-gradient-to-br from-green-500/5 to-transparent opacity-0 transition-opacity group-hover:opacity-100"></div>
				<div class="relative z-10">
					<div class="flex items-center gap-2 text-xs font-medium uppercase tracking-wider text-green-400">
						<svg class="h-3.5 w-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"></path>
						</svg>
						Победы
					</div>
					<div class="mt-2 text-2xl font-bold text-green-400 sm:text-3xl">{winRate}%</div>
				</div>
			</div>
		</div>

		<!-- Detailed stats -->
		<div
			class="mt-6 rounded-2xl border border-white/5 bg-white/[0.02] p-6 backdrop-blur-sm transition-all duration-700 delay-200 {visible ? 'translate-y-0 opacity-100' : 'translate-y-8 opacity-0'}"
		>
			<h3 class="mb-4 flex items-center gap-2 text-lg font-semibold text-white">
				<svg class="h-5 w-5 text-slate-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"></path>
				</svg>
				Статистика
			</h3>

			<div class="space-y-4">
				<!-- Wins -->
				<div class="flex items-center justify-between rounded-xl bg-white/[0.02] border border-white/5 p-4">
					<div class="flex items-center gap-3">
						<div class="flex h-10 w-10 items-center justify-center rounded-lg bg-green-500/10 text-lg">
							🏆
						</div>
						<div>
							<div class="text-sm font-medium text-white">Побед</div>
							<div class="text-xs text-slate-500">Из {user.games_played} игр</div>
						</div>
					</div>
					<div class="text-2xl font-bold text-green-400">{user.wins}</div>
				</div>

				<!-- Losses -->
				<div class="flex items-center justify-between rounded-xl bg-white/[0.02] border border-white/5 p-4">
					<div class="flex items-center gap-3">
						<div class="flex h-10 w-10 items-center justify-center rounded-lg bg-red-500/10 text-lg">
							💔
						</div>
						<div>
							<div class="text-sm font-medium text-white">Поражений</div>
							<div class="text-xs text-slate-500">Продолжайте практиковаться!</div>
						</div>
					</div>
					<div class="text-2xl font-bold text-red-400">{Math.max(0, user.games_played - user.wins)}</div>
				</div>

				<!-- Average score -->
				<div class="flex items-center justify-between rounded-xl bg-white/[0.02] border border-white/5 p-4">
					<div class="flex items-center gap-3">
						<div class="flex h-10 w-10 items-center justify-center rounded-lg bg-blue-500/10 text-lg">
							📊
						</div>
						<div>
							<div class="text-sm font-medium text-white">Средний счёт</div>
							<div class="text-xs text-slate-500">Очков за игру</div>
						</div>
					</div>
					<div class="text-2xl font-bold text-blue-400">
						{user.games_played > 0 ? Math.round(user.xp / user.games_played) : 0}
					</div>
				</div>
			</div>
		</div>

		<!-- Quick actions -->
		<div
			class="mt-6 grid grid-cols-1 gap-4 sm:grid-cols-2 transition-all duration-700 delay-300 {visible ? 'translate-y-0 opacity-100' : 'translate-y-8 opacity-0'}"
		>
			<a
				href="/lobby"
				class="group relative overflow-hidden rounded-2xl border border-white/5 bg-white/[0.02] p-5 backdrop-blur-sm transition-all hover:border-blue-500/30"
			>
				<div class="absolute inset-0 bg-gradient-to-br from-blue-500/10 to-transparent opacity-0 transition-opacity group-hover:opacity-100"></div>
				<div class="relative flex items-center gap-4">
					<div class="flex h-12 w-12 items-center justify-center rounded-xl bg-blue-500/10 text-2xl transition-transform group-hover:scale-110">
						🎮
					</div>
					<div>
						<div class="font-semibold text-white">В лобби</div>
						<div class="text-sm text-slate-400">Найти или создать комнату</div>
					</div>
					<svg class="ml-auto h-5 w-5 text-slate-500 transition-transform group-hover:translate-x-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7l5 5m0 0l-5 5m5-5H6"></path>
					</svg>
				</div>
			</a>

			<a
				href="/settings"
				class="group relative overflow-hidden rounded-2xl border border-white/5 bg-white/[0.02] p-5 backdrop-blur-sm transition-all hover:border-purple-500/30"
			>
				<div class="absolute inset-0 bg-gradient-to-br from-purple-500/10 to-transparent opacity-0 transition-opacity group-hover:opacity-100"></div>
				<div class="relative flex items-center gap-4">
					<div class="flex h-12 w-12 items-center justify-center rounded-xl bg-purple-500/10 text-2xl transition-transform group-hover:scale-110">
						⚙️
					</div>
					<div>
						<div class="font-semibold text-white">Настройки AI</div>
						<div class="text-sm text-slate-400">Переключение провайдеров</div>
					</div>
					<svg class="ml-auto h-5 w-5 text-slate-500 transition-transform group-hover:translate-x-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7l5 5m0 0l-5 5m5-5H6"></path>
					</svg>
				</div>
			</a>
		</div>
	{/if}
</div>
