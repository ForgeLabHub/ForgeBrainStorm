<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { api, type User } from '$lib/api';
	import { currentUser, authChecked, refreshAuth, logout } from '$lib/stores/auth';

	let stats: User | null = null;
	let loading = true;

	onMount(async () => {
		await refreshAuth();
		try {
			const response = await api.me();
			stats = response.user ?? null;
		} catch {
			stats = null;
		} finally {
			loading = false;
		}
	});

	$: user = stats ?? $currentUser;

	// Mirrors the backend: level = floor(sqrt(xp / 100)), min 1.
	$: level = user ? Math.max(1, Math.floor(Math.sqrt(user.xp / 100))) : 1;
	$: levelFloor = level * level * 100;
	$: levelCeil = (level + 1) * (level + 1) * 100;
	$: levelProgress = user
		? Math.min(100, Math.max(0, ((user.xp - levelFloor) / (levelCeil - levelFloor)) * 100))
		: 0;
	$: winRate = user && user.games_played > 0 ? Math.round((user.wins / user.games_played) * 100) : 0;

	async function handleLogout() {
		await logout();
		await goto('/auth/login');
	}
</script>

<div class="mx-auto max-w-4xl px-6 py-8">
	{#if loading || !$authChecked}
		<div class="flex items-center justify-center py-20">
			<div class="h-8 w-8 animate-spin rounded-full border-4 border-blue-500 border-t-transparent"></div>
		</div>
	{:else if !user}
		<div class="rounded-2xl border border-slate-700 bg-slate-800 p-10 text-center">
			<div class="mb-3 text-5xl">🔒</div>
			<h1 class="mb-2 text-2xl font-bold text-white">Профиль недоступен</h1>
			<p class="mb-6 text-slate-400">Войдите, чтобы увидеть статистику и прогресс.</p>
			<a
				href="/auth/login"
				class="inline-block rounded-lg bg-blue-600 px-6 py-3 font-semibold text-white transition hover:bg-blue-700"
			>
				Войти
			</a>
		</div>
	{:else}
		<div class="mb-6 rounded-2xl border border-slate-700 bg-slate-800 p-6">
			<div class="flex flex-col items-center gap-6 sm:flex-row">
				<div
					class="flex h-24 w-24 shrink-0 items-center justify-center rounded-full bg-gradient-to-br from-blue-500 to-purple-600 text-4xl font-bold text-white"
				>
					{user.username.charAt(0).toUpperCase()}
				</div>
				<div class="flex-1 text-center sm:text-left">
					<h1 class="text-3xl font-bold text-white">{user.username}</h1>
					<p class="mt-1 text-slate-400">Уровень {level}</p>
					<div class="mt-3">
						<div class="mb-1 flex justify-between text-xs text-slate-400">
							<span>{user.xp} XP</span>
							<span>до уровня {level + 1}: {levelCeil - user.xp} XP</span>
						</div>
						<div class="h-2.5 overflow-hidden rounded-full bg-slate-700">
							<div
								class="h-full rounded-full bg-gradient-to-r from-blue-500 to-purple-500 transition-all duration-500"
								style="width: {levelProgress}%"
							></div>
						</div>
					</div>
				</div>
			</div>
		</div>

		<div class="mb-6 grid grid-cols-2 gap-4 md:grid-cols-4">
			<div class="rounded-xl border border-slate-700 bg-slate-800 p-5">
				<div class="text-sm text-slate-400">Опыт</div>
				<div class="mt-1 text-2xl font-bold text-blue-400">{user.xp}</div>
			</div>
			<div class="rounded-xl border border-slate-700 bg-slate-800 p-5">
				<div class="text-sm text-slate-400">Монеты</div>
				<div class="mt-1 text-2xl font-bold text-yellow-400">{user.coins}</div>
			</div>
			<div class="rounded-xl border border-slate-700 bg-slate-800 p-5">
				<div class="text-sm text-slate-400">Игр сыграно</div>
				<div class="mt-1 text-2xl font-bold text-white">{user.games_played}</div>
			</div>
			<div class="rounded-xl border border-slate-700 bg-slate-800 p-5">
				<div class="text-sm text-slate-400">Побед</div>
				<div class="mt-1 text-2xl font-bold text-green-400">{winRate}%</div>
			</div>
		</div>

		<div class="rounded-2xl border border-slate-700 bg-slate-800 p-6">
			<h2 class="mb-4 text-lg font-semibold text-white">Быстрые действия</h2>
			<div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
				<a
					href="/room/create"
					class="rounded-lg border border-slate-600 bg-slate-700 p-4 text-left transition hover:border-blue-500 hover:bg-slate-600"
				>
					<div class="font-semibold text-white">🎮 Создать комнату</div>
					<div class="text-sm text-slate-400">Начать новую викторину</div>
				</a>
				<a
					href="/lobby"
					class="rounded-lg border border-slate-600 bg-slate-700 p-4 text-left transition hover:border-blue-500 hover:bg-slate-600"
				>
					<div class="font-semibold text-white">🧩 Лобби</div>
					<div class="text-sm text-slate-400">Присоединиться к игре</div>
				</a>
				<a
					href="/settings"
					class="rounded-lg border border-slate-600 bg-slate-700 p-4 text-left transition hover:border-blue-500 hover:bg-slate-600"
				>
					<div class="font-semibold text-white">⚙️ Настройки AI</div>
					<div class="text-sm text-slate-400">Выбрать провайдера вопросов</div>
				</a>
				<button
					on:click={handleLogout}
					class="rounded-lg border border-red-500/40 bg-red-500/10 p-4 text-left transition hover:bg-red-500/20"
				>
					<div class="font-semibold text-red-300">🚪 Выйти</div>
					<div class="text-sm text-red-400/80">Завершить сессию</div>
				</button>
			</div>
		</div>
	{/if}
</div>
