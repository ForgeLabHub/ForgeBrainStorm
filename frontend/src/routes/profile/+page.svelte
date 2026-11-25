<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { api, type User } from '$lib/api';
	import { currentUser, authChecked, refreshAuth, logout } from '$lib/stores/auth';
	import Icon from '$lib/components/Icon.svelte';
	import { initial, levelFromXp, levelBounds, levelProgress, plural } from '$lib/ui';

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
	$: level = user ? levelFromXp(user.xp) : 1;
	$: nextLevelXp = levelBounds(level).to;
	$: progress = user ? levelProgress(user.xp) : 0;
	$: winRate = user && user.games_played > 0 ? Math.round((user.wins / user.games_played) * 100) : 0;

	async function handleLogout() {
		await logout();
		await goto('/auth/login');
	}
</script>

{#if loading || !$authChecked}
	<div class="py-16">
		<div class="mx-auto h-6 w-6 animate-spin rounded-full border-2 border-amber/30 border-t-amber"></div>
	</div>
{:else if !user}
	<div class="card card-pad mx-auto max-w-md text-center">
		<h1 class="font-display text-xl text-chalk">Профиль недоступен</h1>
		<p class="mt-1 text-sm text-dim">Войдите, чтобы видеть статистику и прогресс.</p>
		<a href="/auth/login" class="btn btn-primary mt-5 w-full">Войти</a>
	</div>
{:else}
	<div class="mx-auto w-full max-w-2xl">
		<section class="card card-pad">
			<div class="flex items-center gap-4">
				<span class="grid h-16 w-16 shrink-0 place-items-center rounded-full bg-ink-700 font-display text-2xl text-chalk">
					{initial(user.username)}
				</span>
				<div class="min-w-0 flex-1">
					<h1 class="truncate font-display text-[22px] tracking-tight text-chalk">{user.username}</h1>
					<p class="mt-0.5 text-sm text-dim">Уровень {level}</p>
				</div>
			</div>

			<div class="mt-5">
				<div class="mb-1.5 flex items-baseline justify-between text-xs text-dim">
					<span class="digits text-sm text-chalk">{user.xp} XP</span>
					<span>до уровня {level + 1} — {Math.max(0, nextLevelXp - user.xp)} XP</span>
				</div>
				<div class="h-2 overflow-hidden rounded-full bg-ink-700">
					<div class="h-full rounded-full bg-amber transition-all duration-500" style="width: {progress}%"></div>
				</div>
			</div>
		</section>

		<section class="card mt-4">
			<div class="grid grid-cols-2 divide-line/70 sm:grid-cols-4 sm:divide-x">
				<div class="border-b border-line/70 px-4 py-4 sm:border-b-0">
					<p class="text-xs text-dim">Опыт</p>
					<p class="digits mt-1 text-xl text-chalk">{user.xp}</p>
				</div>
				<div class="border-b border-line/70 px-4 py-4 sm:border-b-0">
					<p class="text-xs text-dim">Монеты</p>
					<p class="digits mt-1 text-xl text-amber">{user.coins}</p>
				</div>
				<div class="border-b border-line/70 px-4 py-4 sm:border-b-0">
					<p class="text-xs text-dim">Игр</p>
					<p class="digits mt-1 text-xl text-chalk">{user.games_played}</p>
				</div>
				<div class="px-4 py-4">
					<p class="text-xs text-dim">Побед</p>
					<p class="digits mt-1 text-xl text-mint">{winRate}%</p>
				</div>
			</div>
		</section>

		<section class="mt-6">
			<h2 class="section-title mb-3">Дальше</h2>
			<div class="divide-y divide-line/70 overflow-hidden rounded-xl2 border border-line/80">
				<a href="/room/create" class="flex items-center gap-3 bg-ink-850 px-4 py-3.5 hover:bg-ink-800">
					<Icon name="play" size={18} />
					<span class="flex-1 text-sm text-chalk">Создать комнату</span>
					<span class="text-xs text-dim">новая игра</span>
				</a>
				<a href="/leaderboard" class="flex items-center gap-3 bg-ink-850 px-4 py-3.5 hover:bg-ink-800">
					<Icon name="trophy" size={18} />
					<span class="flex-1 text-sm text-chalk">Рейтинг игроков</span>
					<span class="text-xs text-dim">всего {user.games_played}
						{plural(user.games_played, 'игра', 'игры', 'игр')}</span>
				</a>
				<a href="/settings" class="flex items-center gap-3 bg-ink-850 px-4 py-3.5 hover:bg-ink-800">
					<Icon name="sliders" size={18} />
					<span class="flex-1 text-sm text-chalk">Настройки AI</span>
					<span class="text-xs text-dim">провайдер вопросов</span>
				</a>
				<button on:click={handleLogout} class="flex w-full items-center gap-3 bg-ink-850 px-4 py-3.5 text-left hover:bg-ink-800">
					<Icon name="logout" size={18} />
					<span class="flex-1 text-sm text-coral">Выйти из аккаунта</span>
				</button>
			</div>
		</section>
	</div>
{/if}
