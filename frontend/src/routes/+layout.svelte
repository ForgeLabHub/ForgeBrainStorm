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
	let scrolled = $state(false);

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

		// Track scroll for header effect
		const handleScroll = () => {
			scrolled = window.scrollY > 20;
		};
		window.addEventListener('scroll', handleScroll);
		return () => window.removeEventListener('scroll', handleScroll);
	});

	function closeMobileMenu() {
		mobileMenuOpen = false;
	}
</script>

<svelte:head>
	<title>{$page.url.pathname === '/settings' ? 'Настройки ИИ · BrainStorm' : 'BrainStorm — интеллектуальная викторина'}</title>
	<meta name="description" content={$page.url.pathname === '/settings' ? 'Управление провайдерами и кэшем генерации BrainStorm.' : 'Интерактивная викторина с ИИ-генерацией вопросов'} />
</svelte:head>

<div class="min-h-screen bg-[#0a0a0f] text-slate-50 relative">
	<!-- Background gradient effects -->
	<div class="fixed inset-0 pointer-events-none">
		<div class="absolute top-0 left-1/4 w-96 h-96 bg-blue-500/10 rounded-full blur-3xl"></div>
		<div class="absolute bottom-0 right-1/4 w-96 h-96 bg-purple-500/10 rounded-full blur-3xl"></div>
	</div>

	<!-- Header -->
	<header class="sticky top-0 z-50 transition-all duration-300 {scrolled ? 'bg-[#0a0a0f]/90 backdrop-blur-xl shadow-lg shadow-black/20' : 'bg-transparent'}">
		<nav class="mx-auto flex max-w-6xl items-center justify-between px-4 py-4 sm:px-6">
			<a href="/" class="flex items-center gap-3 group" onclick={closeMobileMenu}>
				<div class="relative">
					<div class="absolute inset-0 bg-gradient-to-br from-blue-500 to-purple-600 rounded-lg blur opacity-50 group-hover:opacity-75 transition"></div>
					<div class="relative h-9 w-9 rounded-lg bg-gradient-to-br from-blue-500 to-purple-600 flex items-center justify-center">
						<svg class="h-5 w-5 text-white" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
							<path stroke-linecap="round" stroke-linejoin="round" d="M13 10V3L4 14h7v7l9-11h-7z"/>
						</svg>
					</div>
				</div>
				<span class="text-lg font-bold bg-gradient-to-r from-white to-slate-300 bg-clip-text text-transparent">BrainStorm</span>
			</a>

			<!-- Desktop nav -->
			<div class="hidden items-center gap-2 md:flex">
				<a href="/lobby" class="px-4 py-2 text-sm text-slate-400 hover:text-white hover:bg-white/5 rounded-lg transition-all">
					Комнаты
				</a>
				<a href="/settings" aria-current={$page.url.pathname === '/settings' ? 'page' : undefined} class="px-4 py-2 text-sm text-slate-400 hover:text-white hover:bg-white/5 rounded-lg transition-all">
					Настройки
				</a>
				{#if isLoggedIn}
					<a href="/profile" class="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-white/5 border border-white/10 hover:bg-white/10 hover:border-white/20 transition-all">
						<div class="h-7 w-7 rounded-full bg-gradient-to-br from-blue-500 to-purple-600 flex items-center justify-center text-xs font-bold shadow-lg shadow-blue-500/20">
							{username.charAt(0).toUpperCase()}
						</div>
						<span class="text-sm font-medium">{username}</span>
					</a>
				{:else}
					<a href="/auth/login" class="btn-primary rounded-lg px-5 py-2 text-sm font-medium text-white">
						Войти
					</a>
				{/if}
			</div>

			<!-- Mobile menu button -->
			<button
				onclick={() => (mobileMenuOpen = !mobileMenuOpen)}
				class="flex h-10 w-10 items-center justify-center rounded-lg bg-white/5 border border-white/10 md:hidden hover:bg-white/10 transition"
				aria-label="Menu"
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
			<div class="border-t border-white/5 bg-[#0a0a0f]/95 backdrop-blur-xl md:hidden animate-fade-in">
				<div class="mx-auto max-w-6xl px-4 py-4 space-y-1">
					<a href="/lobby" onclick={closeMobileMenu} class="flex items-center gap-3 rounded-lg px-4 py-3 text-slate-300 hover:bg-white/5 transition">
						<svg class="h-5 w-5 text-slate-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z"/>
						</svg>
						Комнаты
					</a>
					<a href="/settings" onclick={closeMobileMenu} class="flex items-center gap-3 rounded-lg px-4 py-3 text-slate-300 hover:bg-white/5 transition">
						<svg class="h-5 w-5 text-slate-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"/>
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"/>
						</svg>
						Настройки
					</a>
					{#if isLoggedIn}
						<a href="/profile" onclick={closeMobileMenu} class="flex items-center gap-3 rounded-lg px-4 py-3 text-slate-300 hover:bg-white/5 transition">
							<svg class="h-5 w-5 text-slate-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z"/>
							</svg>
							Профиль
						</a>
					{:else}
						<a href="/auth/login" onclick={closeMobileMenu} class="flex items-center justify-center gap-2 rounded-lg btn-primary px-4 py-3 font-medium text-white mt-2">
							<svg class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 16l-4-4m0 0l4-4m-4 4h14m-5 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h7a3 3 0 013 3v1"/>
							</svg>
							Войти
						</a>
					{/if}
				</div>
			</div>
		{/if}
	</header>

	<main class="mx-auto max-w-6xl px-4 py-8 sm:px-6 relative">
		{@render children()}
	</main>

	<footer class="border-t border-white/5 px-4 py-8 sm:px-6 mt-12">
		<div class="mx-auto max-w-6xl flex flex-col sm:flex-row items-center justify-between gap-4">
			<div class="flex items-center gap-2 text-sm text-slate-500">
				<div class="h-5 w-5 rounded bg-gradient-to-br from-blue-500 to-purple-600 flex items-center justify-center">
					<svg class="h-3 w-3 text-white" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
						<path stroke-linecap="round" stroke-linejoin="round" d="M13 10V3L4 14h7v7l9-11h-7z"/>
					</svg>
				</div>
				<span>BrainStorm</span>
			</div>
			<div class="text-xs text-slate-600">
				Интеллектуальная викторина с AI
			</div>
		</div>
	</footer>
</div>
