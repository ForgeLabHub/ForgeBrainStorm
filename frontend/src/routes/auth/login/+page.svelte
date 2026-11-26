<script lang="ts">
	import { api } from '$lib/api';
	import { goto } from '$app/navigation';

	let username = $state('');
	let password = $state('');
	let loading = $state(false);
	let error = $state('');

	async function handleLogin(e: Event) {
		e.preventDefault();
		if (!username.trim() || !password) {
			error = 'Заполните все поля';
			return;
		}
		loading = true;
		error = '';
		try {
			const response = await api.login(username.trim(), password);
			if (response.ok) {
				goto('/lobby');
			} else {
				error = response.error || 'Неверные учётные данные';
			}
		} catch (e) {
			error = 'Ошибка соединения с сервером';
		} finally {
			loading = false;
		}
	}
</script>

<div class="flex min-h-[70vh] items-center justify-center px-4">
	<div class="w-full max-w-sm">
		<div class="mb-8 text-center">
			<h1 class="text-2xl font-bold text-white">Вход</h1>
			<p class="mt-2 text-sm text-slate-400">Войдите, чтобы продолжить</p>
		</div>

		<form onsubmit={handleLogin} class="space-y-4">
			<div>
				<label for="username" class="mb-2 block text-sm font-medium text-slate-300">
					Имя пользователя
				</label>
				<input
					id="username"
					type="text"
					bind:value={username}
					placeholder="your_name"
					autocomplete="username"
					class="w-full rounded-lg border border-white/10 bg-white/5 px-4 py-2.5 text-white placeholder-slate-500 transition focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
				/>
			</div>

			<div>
				<label for="password" class="mb-2 block text-sm font-medium text-slate-300">
					Пароль
				</label>
				<input
					id="password"
					type="password"
					bind:value={password}
					placeholder="••••••••"
					autocomplete="current-password"
					class="w-full rounded-lg border border-white/10 bg-white/5 px-4 py-2.5 text-white placeholder-slate-500 transition focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
				/>
			</div>

			{#if error}
				<div class="rounded-lg border border-red-500/20 bg-red-500/10 p-3 text-sm text-red-400">
					{error}
				</div>
			{/if}

			<button
				type="submit"
				disabled={loading}
				class="w-full rounded-lg bg-blue-500 py-2.5 font-medium text-white transition hover:bg-blue-600 disabled:cursor-not-allowed disabled:opacity-50"
			>
				{#if loading}
					<span class="flex items-center justify-center gap-2">
						<svg class="h-4 w-4 animate-spin" fill="none" viewBox="0 0 24 24">
							<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
							<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"></path>
						</svg>
						Вход...
					</span>
				{:else}
					Войти
				{/if}
			</button>
		</form>

		<div class="mt-6 text-center text-sm text-slate-400">
			Нет аккаунта?
			<a href="/auth/register" class="font-medium text-blue-400 hover:text-blue-300">Зарегистрироваться</a>
		</div>
	</div>
</div>
