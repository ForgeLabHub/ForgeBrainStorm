<script lang="ts">
	import '../app.css';
	import { api } from '$lib/api';
	import { onMount } from 'svelte';

	let { children } = $props();
	import { page } from '$app/stores';
	import { afterNavigate } from '$app/navigation';

	let isLoggedIn = $state(false);
	let username = $state('');
	let userLevel = $state(1);
	let userXp = $state(0);
	let userCoins = $state(0);
	let mobileMenuOpen = $state(false);

	async function loadUser() {
		try {
			const response = await api.me();
			if (response.user) {
				isLoggedIn = true;
				username = response.user.username;
				userLevel = response.user.level || 1;
				userXp = response.user.xp || 0;
				userCoins = response.user.coins || 0;
				return;
			}
		} catch {
			// Not logged in
		}
		isLoggedIn = false;
	}

	onMount(loadUser);
	afterNavigate(loadUser);

	function closeMobileMenu() {
		mobileMenuOpen = false;
	}
</script>

<div class="min-h-screen bg-[#0a0b1a] text-slate-50">
	<!-- Animated background -->
	<div class="fixed inset-0 overflow-hidden pointer-events-none">
		<div class="absolute -top-40 -right-40 w-96 h-96 bg-purple-600/10 rounded-full blur-3xl animate-pulse"></div>
		<div class="absolute top-1/2 -left-40 w-80 h-80 bg-blue-600/10 rounded-full blur-3xl animate-pulse" style="animation-delay: 1s"></div>
		<div class="absolute -bottom-20 right-1/3 w-72 h-72 bg-cyan-500/8 rounded-full blur-3xl animate-pulse" style="animation-delay: 2s"></div>
	</div>

	<!-- Header -->
	<header class="relative z-50 border-b border-white/5 bg-[#0a0b1a]/80 backdrop-blur-xl sticky top-0">
		<nav class="mx-auto flex max-w-7xl items-center justify-between px-4 py-3 sm:px-6">
			<!-- Logo -->
			<a href="/" class="flex items-center gap-3 group" onclick={closeMobileMenu}>
				<div class="relative">
					<div class="absolute inset-0 bg-gradient-to-r from-blue-500 to-purple-500 rounded-xl blur-md opacity-50 group-hover:opacity-80 transition-opacity"></div>
					<div class="relative flex h-9 w-9 sm:h-10 sm:w-10 items-center justify-center rounded-xl bg-gradient-to-br from-blue-500 to-purple-600 text-lg sm:text-xl">
						🧠
					</div>
				</div>
				<span class="hidden sm:block text-xl font-bold bg-gradient-to-r from-blue-400 to-purple-400 bg-clip-text text-transparent">
					BrainStorm
				</span>
			</a>

			<!-- Desktop nav -->
			<div class="hidden md:flex items-center gap-3">
				<a href="/lobby" class="flex items-center gap-2 rounded-xl px-4 py-2 text-sm font-medium text-slate-300 hover:text-white hover:bg-white/5 transition-all">
					<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z"></path></svg>
					Комнаты
				</a>

				<a href="/settings" class="flex items-center gap-2 rounded-xl px-4 py-2 text-sm font-medium text-slate-300 hover:text-white hover:bg-white/5 transition-all">
					<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"></path><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"></path></svg>
					AI
				</a>

				{#if isLoggedIn}
					<a href="/profile" class="flex items-center gap-3 rounded-xl bg-white/5 border border-white/10 px-3 py-2 hover:bg-white/10 hover:border-white/20 transition-all group">
						<div class="relative">
							<div class="absolute inset-0 bg-gradient-to-r from-blue-500 to-purple-500 rounded-lg blur-sm opacity-0 group-hover:opacity-50 transition-opacity"></div>
							<div class="relative flex h-8 w-8 items-center justify-center rounded-lg bg-gradient-to-br from-blue-500 to-purple-600 text-sm font-bold text-white">
								{username.charAt(0).toUpperCase()}
							</div>
						</div>
						<div class="flex flex-col">
							<span class="text-sm font-medium text-white leading-tight">{username}</span>
							<span class="text-xs text-slate-400 leading-tight">Ур. {userLevel} · 🪙 {userCoins}</span>
						</div>
					</a>
				{:else}
					<a href="/auth/login" class="relative group">
						<div class="absolute inset-0 bg-gradient-to-r from-blue-500 to-purple-600 rounded-xl blur-md opacity-50 group-hover:opacity-80 transition-opacity"></div>
						<div class="relative rounded-xl bg-gradient-to-r from-blue-500 to-purple-600 px-5 py-2 text-sm font-semibold text-white shadow-lg">
							Войти
						</div>
					</a>
				{/if}
			</div>

			<!-- Mobile menu button -->
			<div class="flex items-center gap-3 md:hidden">
				{#if isLoggedIn}
					<a href="/profile" class="flex h-9 w-9 items-center justify-center rounded-lg bg-gradient-to-br from-blue-500 to-purple-600 text-sm font-bold text-white">
						{username.charAt(0).toUpperCase()}
					</a>
				{:else}
					<a href="/auth/login" class="rounded-xl bg-gradient-to-r from-blue-500 to-purple-600 px-4 py-2 text-sm font-semibold text-white">
						Войти
					</a>
				{/if}

				<button
					onclick={() => (mobileMenuOpen = !mobileMenuOpen)}
					class="flex h-9 w-9 items-center justify-center rounded-xl bg-white/5 border border-white/10 text-white"
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
			</div>
		</nav>

		<!-- Mobile menu -->
		{#if mobileMenuOpen}
			<div class="md:hidden border-t border-white/5 bg-[#0a0b1a]/95 backdrop-blur-xl">
				<div class="mx-auto max-w-7xl px-4 py-4 space-y-2">
					<a
						href="/lobby"
						onclick={closeMobileMenu}
						class="flex items-center gap-3 rounded-xl px-4 py-3 text-slate-300 hover:bg-white/5 transition-all"
					>
						<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z"></path></svg>
						Комнаты
					</a>
					<a
						href="/settings"
						onclick={closeMobileMenu}
						class="flex items-center gap-3 rounded-xl px-4 py-3 text-slate-300 hover:bg-white/5 transition-all"
					>
						<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"></path><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"></path></svg>
						Настройки AI
					</a>
					{#if isLoggedIn}
						<div class="border-t border-white/5 pt-2 mt-2">
							<div class="flex items-center gap-3 px-4 py-2">
								<div class="flex h-8 w-8 items-center justify-center rounded-lg bg-gradient-to-br from-blue-500 to-purple-600 text-sm font-bold text-white">
									{username.charAt(0).toUpperCase()}
								</div>
								<div>
									<div class="text-sm font-medium text-white">{username}</div>
									<div class="text-xs text-slate-400">Ур. {userLevel} · 🪙 {userCoins} · XP {userXp}</div>
								</div>
							</div>
						</div>
					{/if}
				</div>
			</div>
		{/if}
	</header>

	<!-- Main content -->
	<main class="relative z-10 mx-auto max-w-7xl px-4 py-6 sm:px-6 sm:py-8">
		{@render children()}
	</main>

	<!-- Footer -->
	<footer class="relative z-10 border-t border-white/5 bg-[#0a0b1a]/80 backdrop-blur-xl px-4 py-6 sm:px-6">
		<div class="mx-auto max-w-7xl flex flex-col sm:flex-row items-center justify-between gap-4">
			<p class="text-sm text-slate-500">
				BrainStorm — интеллектуальная викторина с AI
			</p>
			<div class="flex items-center gap-4 text-sm text-slate-500">
				<span>v0.2.0</span>
				<span class="flex items-center gap-1">
					<span class="w-2 h-2 rounded-full bg-green-500 animate-pulse"></span>
					Online
				</span>
			</div>
		</div>
	</footer>
</div>

<style>
	@keyframes float {
		0%, 100% { transform: translateY(0px); }
		50% { transform: translateY(-10px); }
	}
</style>
