<script lang="ts">
	import '../app.css';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import Icon from '$lib/components/Icon.svelte';
	import { currentUser, authChecked, refreshAuth, logout } from '$lib/stores/auth';
	import { initial } from '$lib/ui';

	const nav = [
		{ href: '/lobby', label: 'Комнаты', icon: 'users' },
		{ href: '/leaderboard', label: 'Лидеры', icon: 'trophy' },
		{ href: '/settings', label: 'Настройки', icon: 'sliders' }
	];

	$: path = $page.url.pathname;
	// The room screen renders its own shell so the game gets the whole viewport.
	$: isGame = /^\/room\/(?!create$)[^/]+$/.test(path);
	$: if (path) refreshAuth();

	function isActive(href: string): boolean {
		return path === href || path.startsWith(href + '/');
	}

	async function handleLogout() {
		await logout();
		await goto('/auth/login');
	}
</script>

<div class="flex min-h-[100dvh] flex-col">
	{#if !isGame}
		<header class="sticky top-0 z-30 border-b border-line/60 bg-ink-950/80 backdrop-blur-md">
			<div class="mx-auto flex h-14 w-full max-w-5xl items-center gap-2 px-4 sm:h-16 sm:px-6">
				<a href="/" class="flex items-center gap-2.5 rounded-lg py-1 pr-2">
					<span
						class="grid h-8 w-8 place-items-center rounded-[11px] bg-amber font-display text-[17px] font-bold leading-none text-ink-950"
					>
						?
					</span>
					<span class="font-display text-[15px] font-semibold tracking-tight text-chalk sm:text-base">
						BrainStorm
					</span>
				</a>

				<nav class="ml-3 hidden items-center gap-1 md:flex" aria-label="Основная навигация">
					{#each nav as item}
						<a
							href={item.href}
							aria-current={isActive(item.href) ? 'page' : undefined}
							class="rounded-[11px] px-3 py-2 text-sm transition-colors {isActive(item.href)
								? 'bg-ink-800 text-chalk'
								: 'text-dim hover:bg-ink-900 hover:text-chalk'}"
						>
							{item.label}
						</a>
					{/each}
				</nav>

				<div class="ml-auto flex items-center gap-1.5">
					{#if !$authChecked}
						<span class="h-9 w-24 animate-pulse rounded-[11px] bg-ink-800"></span>
					{:else if $currentUser}
						<a
							href="/profile"
							class="flex min-h-[40px] items-center gap-2 rounded-[11px] py-1 pl-1 pr-2.5 transition-colors hover:bg-ink-900"
							title="Профиль"
						>
							<span class="grid h-8 w-8 place-items-center rounded-full bg-ink-700 text-sm font-semibold text-chalk">
								{initial($currentUser.username)}
							</span>
							<span class="hidden max-w-[9rem] truncate text-sm font-medium text-chalk sm:block">
								{$currentUser.username}
							</span>
						</a>
						<button
							on:click={handleLogout}
							class="btn-icon hidden sm:inline-flex"
							title="Выйти"
							aria-label="Выйти"
						>
							<Icon name="logout" size={18} />
						</button>
					{:else}
						<a href="/auth/login" class="btn btn-primary h-10 min-h-[40px] px-4 text-sm">Войти</a>
					{/if}
				</div>
			</div>
		</header>
	{/if}

	<main class="w-full flex-1 {isGame ? '' : 'mx-auto max-w-5xl px-4 pb-28 pt-5 sm:px-6 sm:pb-10 sm:pt-8'}">
		<slot />
	</main>

	{#if !isGame}
		<footer class="hidden border-t border-line/60 px-6 py-6 text-center text-xs text-dim md:block">
			BrainStorm — викторина с вопросами от AI
		</footer>

		<nav
			class="fixed inset-x-0 bottom-0 z-40 border-t border-line/70 bg-ink-900/95 pb-[env(safe-area-inset-bottom)] backdrop-blur-md md:hidden"
			aria-label="Навигация"
		>
			<div class="mx-auto grid max-w-md grid-cols-3">
				{#each nav as item}
					<a
						href={item.href}
						aria-current={isActive(item.href) ? 'page' : undefined}
						class="flex min-h-[58px] flex-col items-center justify-center gap-1 text-[11px] transition-colors {isActive(
							item.href
						)
							? 'text-amber'
							: 'text-dim'}"
					>
						<Icon name={item.icon} size={22} />
						{item.label}
					</a>
				{/each}
			</div>
		</nav>
	{/if}
</div>
