<script lang="ts">
	import '../app.css';
	import { api } from '$lib/api';
	import { onMount } from 'svelte';

	let isLoggedIn = false;
	let username = '';

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
</script>

<div class="min-h-screen bg-slate-900 text-slate-50">
	<header class="border-b border-slate-700 px-6 py-4">
		<nav class="mx-auto flex max-w-6xl items-center justify-between">
			<a href="/" class="text-2xl font-bold text-blue-400">
				🧠 BrainStorm
			</a>
			<div class="flex items-center gap-4">
				<a href="/lobby" class="text-slate-300 hover:text-white">Комнаты</a>
				<a href="/settings" class="text-slate-300 hover:text-white">AI</a>
				{#if isLoggedIn}
					<a href="/profile" class="flex items-center gap-2 rounded-full bg-slate-700 px-3 py-1 text-slate-200 hover:bg-slate-600">
						<span class="flex h-6 w-6 items-center justify-center rounded-full bg-blue-500 text-xs font-bold text-white">
							{username.charAt(0).toUpperCase()}
						</span>
						{username}
					</a>
				{:else}
					<a href="/auth/login" class="rounded-lg bg-blue-600 px-4 py-2 text-white hover:bg-blue-700">
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
