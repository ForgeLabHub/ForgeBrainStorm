<script lang="ts">
	import '../app.css';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { currentUser, authChecked, refreshAuth, logout } from '$lib/stores/auth';

	$: currentPath = $page.url.pathname;
	$: if (currentPath) {
		refreshAuth();
	}

	async function handleLogout() {
		await logout();
		await goto('/auth/login');
	}
</script>

<div class="min-h-screen bg-slate-900 text-slate-50">
	<header class="border-b border-slate-700 px-6 py-4">
		<nav class="mx-auto flex max-w-6xl items-center justify-between">
			<a href="/" class="text-2xl font-bold text-blue-400">🧠 BrainStorm</a>

			<div class="flex items-center gap-4">
				<a href="/lobby" class="text-slate-300 hover:text-white">Комнаты</a>
				<a href="/settings" class="text-slate-300 hover:text-white">Настройки</a>

				{#if !$authChecked}
					<span class="h-9 w-24 animate-pulse rounded-lg bg-slate-700"></span>
				{:else if $currentUser}
					<a
						href="/profile"
						class="flex items-center gap-2 rounded-lg px-2 py-1 text-slate-200 transition hover:bg-slate-800"
					>
						<span
							class="flex h-8 w-8 items-center justify-center rounded-full bg-gradient-to-br from-blue-500 to-purple-600 text-sm font-bold text-white"
						>
							{$currentUser.username.charAt(0).toUpperCase()}
						</span>
						<span class="max-w-[10rem] truncate font-medium">{$currentUser.username}</span>
					</a>
					<button
						on:click={handleLogout}
						class="rounded-lg border border-slate-600 px-3 py-1.5 text-sm text-slate-300 transition hover:border-red-500 hover:text-red-400"
					>
						Выйти
					</button>
				{:else}
					<a
						href="/auth/login"
						class="rounded-lg bg-blue-600 px-4 py-2 text-white hover:bg-blue-700"
					>
						Войти
					</a>
				{/if}
			</div>
		</nav>
	</header>

	<main class="mx-auto max-w-6xl px-6 py-8">
		<slot />
	</main>

	<footer class="border-t border-slate-700 px-6 py-4 text-center text-slate-400">
		<p>BrainStorm — интеллектуальная викторина с AI</p>
	</footer>
</div>
