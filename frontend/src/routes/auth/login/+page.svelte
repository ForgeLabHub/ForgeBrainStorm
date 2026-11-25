<script lang="ts">
	import { api } from '$lib/api';
	import { goto } from '$app/navigation';

	let username = '';
	let password = '';
	let loading = false;
	let error = '';

	async function handleLogin() {
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
			console.error('Login failed:', e);
			error = 'Ошибка соединения с сервером';
		} finally {
			loading = false;
		}
	}
</script>

<div class="flex min-h-[70vh] items-center justify-center">
	<div class="w-full max-w-md rounded-2xl border border-slate-700 bg-slate-800 p-8 shadow-xl">
		<div class="mb-6 text-center">
			<div class="mb-3 text-5xl">🧠</div>
			<h1 class="text-2xl font-bold text-white">Вход в BrainStorm</h1>
			<p class="mt-1 text-sm text-slate-400">Войдите, чтобы играть и отслеживать прогресс</p>
		</div>

		<form on:submit|preventDefault={handleLogin} class="space-y-4">
			<div>
				<label for="username" class="mb-1 block text-sm font-medium text-slate-300">Имя пользователя</label>
				<input
					id="username"
					type="text"
					bind:value={username}
					placeholder="your_name"
					class="w-full rounded-lg border border-slate-600 bg-slate-700 px-4 py-2.5 text-white placeholder-slate-400 focus:border-blue-500 focus:outline-none"
					autocomplete="username"
				/>
			</div>

			<div>
				<label for="password" class="mb-1 block text-sm font-medium text-slate-300">Пароль</label>
				<input
					id="password"
					type="password"
					bind:value={password}
					placeholder="••••••••"
					class="w-full rounded-lg border border-slate-600 bg-slate-700 px-4 py-2.5 text-white placeholder-slate-400 focus:border-blue-500 focus:outline-none"
					autocomplete="current-password"
				/>
			</div>

			{#if error}
				<div class="rounded-lg bg-red-500/10 p-3 text-sm text-red-400">
					{error}
				</div>
			{/if}

			<button
				type="submit"
				disabled={loading}
				class="w-full rounded-lg bg-blue-600 py-2.5 font-semibold text-white transition hover:bg-blue-700 disabled:cursor-not-allowed disabled:opacity-50"
			>
				{#if loading}
					<span class="flex items-center justify-center gap-2">
						<div class="h-4 w-4 animate-spin rounded-full border-2 border-white border-t-transparent"></div>
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
