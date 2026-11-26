<script lang="ts">
	import '../app.css';
	import { api } from '$lib/api';
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import type { Snippet } from 'svelte';
	let { children }: { children: Snippet } = $props();

	let isLoggedIn = $state(false);
	let username = $state('');
	let mobileMenuOpen = $state(false);

	onMount(async () => {
		try {
			const response = await api.me();
			if (response.user) {
				isLoggedIn = true;
				username = response.user.username;
			}
		} catch (e) {
			// Not logged in
		}
	});

	function closeMobileMenu() {
		mobileMenuOpen = false;
	}
</script>

<svelte:head>
	<title>{$page.url.pathname === '/settings' ? 'Настройки ИИ · BrainStorm' : 'BrainStorm — интеллектуальная викторина'}</title>
	<meta name="description" content={$page.url.pathname === '/settings' ? 'Управление провайдерами и кэшем генерации BrainStorm.' : 'Интерактивная викторина с ИИ-генерацией вопросов'} />
</svelte:head>

<div class="min-h-screen bg-[#0a0b1a] text-slate-50">
	<!-- Header -->
	<header class="sticky top-0 z-50 border-b border-white/5 bg-[#0a0b1a]/80 backdrop-blur-xl">
		<nav class="mx-auto flex max-w-7xl items-center justify-between px-4 py-4 sm:px-6">
			<a href="/" class="flex items-center gap-2" onclick={closeMobileMenu}>
				<svg class="h-8 w-8" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
					<path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 18c-4.41 0-8-3.59-8-8s3.59-8 8-8 8 3.59 8 8-3.59 8-8 8zm-1-13h2v6h-2zm0 8h2v2h-2z" fill="currentColor"/>
				</svg>
				<span class="text-lg font-semibold text-white">BrainStorm</span>
			</a>

			<!-- Desktop nav -->
			<div class="hidden items-center gap-6 md:flex">
				<a href="/lobby" class="text-sm text-slate-400 hover:text-white transition">Комнаты</a>
				<a href="/settings" aria-current={$page.url.pathname === '/settings' ? 'page' : undefined} class="text-sm text-slate-400 hover:text-white transition">Настройки</a>
				{#if isLoggedIn}
					<a href="/profile" class="flex items-center gap-2 rounded-lg bg-white/5 px-3 py-1.5 text-sm hover:bg-white/10 transition">
						<div class="h-6 w-6 rounded-full bg-gradient-to-br from-blue-500 to-purple-600 flex items-center justify-center text-xs font-semibold">
							{username.charAt(0).toUpperCase()}
						</div>
						<span>{username}</span>
					</a>
				{:else}
					<a href="/auth/login" class="rounded-lg bg-blue-500 px-4 py-2 text-sm font-medium hover:bg-blue-600 transition">
						Войти
					</a>
				{/if}
			</div>

			<!-- Mobile menu button -->
			<button
				onclick={() => (mobileMenuOpen = !mobileMenuOpen)}
				class="flex h-9 w-9 items-center justify-center rounded-lg bg-white/5 md:hidden"
				aria-label="Меню"
				aria-expanded={mobileMenuOpen}
			>
				{#if mobileMenuOpen}
					<svg class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
					</svg>
				{:else}
					<svg class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16"></path>
					</svg>
				{/if}
			</button>
		</nav>

		<!-- Mobile menu -->
		{#if mobileMenuOpen}
			<div class="border-t border-white/5 md:hidden">
				<div class="mx-auto max-w-7xl px-4 py-4 space-y-1">
					<a href="/lobby" onclick={closeMobileMenu} class="block rounded-lg px-4 py-3 text-slate-300 hover:bg-white/5">
						Комнаты
					</a>
					<a href="/settings" aria-current={$page.url.pathname === '/settings' ? 'page' : undefined} onclick={closeMobileMenu} class="block rounded-lg px-4 py-3 text-slate-300 hover:bg-white/5">
						Настройки
					</a>
					{#if isLoggedIn}
						<a href="/profile" onclick={closeMobileMenu} class="block rounded-lg px-4 py-3 text-slate-300 hover:bg-white/5">
							Профиль
						</a>
					{:else}
						<a href="/auth/login" onclick={closeMobileMenu} class="block rounded-lg bg-blue-500 px-4 py-3 text-center font-medium">
							Войти
						</a>
					{/if}
				</div>
			</div>
		{/if}
	</header>

	<main class="mx-auto max-w-7xl px-4 py-8 sm:px-6">
		{@render children()}
	</main>

	<footer class="border-t border-white/5 px-4 py-6 sm:px-6">
		<div class="mx-auto max-w-7xl text-center text-sm text-slate-500">
			BrainStorm
		</div>
	</footer>
</div>

<style>
	a[aria-current="page"] { color: #dfccff; text-decoration: underline; text-underline-offset: .35em; }
</style>
