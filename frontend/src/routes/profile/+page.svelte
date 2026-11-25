<script lang="ts">
	import { api } from '$lib/api';
	import type { User } from '$lib/api';
	import { onMount } from 'svelte';

	let user: User | null = null;
	let loading = true;
	let error = '';

	onMount(async () => {
		try {
			const response = await api.me();
			if (response.user) {
				user = response.user;
			} else {
				error = 'Не удалось загрузить профиль';
			}
		} catch (e) {
			console.error('Failed to load profile:', e);
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

	function getWinRate(user: User): string {
		if (user.games_played === 0) return '0%';
		return Math.round((user.wins / user.games_played) * 100) + '%';
	}

	function getXpToNextLevel(user: User): { current: number; next: number; percent: number } {
		const currentLevel = user.level;
		const currentXpNeeded = 100 * (currentLevel * currentLevel);
		const nextLevelXp = 100 * ((currentLevel + 1) * (currentLevel + 1));
		const xpInCurrentLevel = user.xp - currentXpNeeded;
		const xpToNext = nextLevelXp - currentXpNeeded;
		const percent = Math.min(100, Math.round((xpInCurrentLevel / xpToNext) * 100));
		return { current: xpInCurrentLevel, next: xpToNext, percent };
	}
</script>

<div class="mx-auto max-w-3xl px-6 py-8">
	{#if loading}
		<div class="flex items-center justify-center py-20">
			<div class="h-8 w-8 animate-spin rounded-full border-4 border-blue-500 border-t-transparent"></div>
		</div>
	{:else if error}
		<div class="rounded-xl border border-red-500/30 bg-red-500/10 p-8 text-center">
			<div class="mb-3 text-4xl">🔒</div>
			<p class="mb-4 text-red-400">{error}</p>
			<a href="/auth/login" class="rounded-lg bg-blue-600 px-6 py-2 font-semibold text-white hover:bg-blue-700">
				Войти
			</a>
		</div>
	{:else if user}
		<div class="rounded-2xl border border-slate-700 bg-slate-800 p-8">
			<!-- Header -->
			<div class="mb-6 flex items-center justify-between">
				<div class="flex items-center gap-4">
					<div class="flex h-16 w-16 items-center justify-center rounded-full bg-gradient-to-br from-blue-500 to-purple-600 text-2xl font-bold text-white">
						{user.username.charAt(0).toUpperCase()}
					</div>
					<div>
						<h1 class="text-2xl font-bold text-white">{user.username}</h1>
						<div class="mt-1 flex items-center gap-2">
							<span class="rounded-full bg-blue-500/20 px-3 py-0.5 text-sm font-medium text-blue-400">
								Уровень {user.level}
							</span>
						</div>
					</div>
				</div>
				<button
					on:click={handleLogout}
					class="rounded-lg border border-slate-600 px-4 py-2 text-sm text-slate-300 transition hover:bg-slate-700"
				>
					Выйти
				</button>
			</div>

			<!-- XP Progress -->
			<div class="mb-8">
				<div class="mb-2 flex justify-between text-sm">
					<span class="text-slate-400">Прогресс до уровня {user.level + 1}</span>
					<span class="text-slate-400">{getXpToNextLevel(user).current} / {getXpToNextLevel(user).next} XP</span>
				</div>
				<div class="h-3 overflow-hidden rounded-full bg-slate-700">
					<div
						class="h-full bg-gradient-to-r from-blue-500 to-purple-500 transition-all duration-500"
						style="width: {getXpToNextLevel(user).percent}%"
					></div>
				</div>
			</div>

			<!-- Stats grid -->
			<div class="grid grid-cols-2 gap-4 md:grid-cols-4">
				<div class="rounded-xl border border-slate-700 bg-slate-900 p-4">
					<div class="text-xs uppercase tracking-wide text-slate-500">Опыт</div>
					<div class="mt-1 text-2xl font-bold text-white">{user.xp.toLocaleString()}</div>
				</div>
				<div class="rounded-xl border border-slate-700 bg-slate-900 p-4">
					<div class="text-xs uppercase tracking-wide text-slate-500">Монеты</div>
					<div class="mt-1 text-2xl font-bold text-yellow-400">{user.coins}</div>
				</div>
				<div class="rounded-xl border border-slate-700 bg-slate-900 p-4">
					<div class="text-xs uppercase tracking-wide text-slate-500">Игр сыграно</div>
					<div class="mt-1 text-2xl font-bold text-white">{user.games_played}</div>
				</div>
				<div class="rounded-xl border border-slate-700 bg-slate-900 p-4">
					<div class="text-xs uppercase tracking-wide text-slate-500">Процент побед</div>
					<div class="mt-1 text-2xl font-bold text-green-400">{getWinRate(user)}</div>
				</div>
			</div>

			<!-- Additional info -->
			<div class="mt-6 rounded-xl border border-slate-700 bg-slate-900 p-4">
				<div class="flex items-center justify-between">
					<span class="text-slate-400">Всего побед</span>
					<span class="font-semibold text-white">{user.wins}</span>
				</div>
			</div>
		</div>

		<!-- Quick actions -->
		<div class="mt-6 grid grid-cols-1 gap-4 md:grid-cols-2">
			<a href="/lobby" class="rounded-xl border border-slate-700 bg-slate-800 p-5 transition hover:border-blue-500">
				<div class="mb-1 text-2xl">🎮</div>
				<div class="font-semibold text-white">В лобби</div>
				<div class="text-sm text-slate-400">Найти или создать комнату</div>
			</a>
			<a href="/settings" class="rounded-xl border border-slate-700 bg-slate-800 p-5 transition hover:border-blue-500">
				<div class="mb-1 text-2xl">⚙️</div>
				<div class="font-semibold text-white">Настройки AI</div>
				<div class="text-sm text-slate-400">Переключение AI-провайдеров</div>
			</a>
		</div>
	{/if}
</div>
