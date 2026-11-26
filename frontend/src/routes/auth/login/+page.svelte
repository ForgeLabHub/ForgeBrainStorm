<script lang="ts">
	import { api } from '$lib/api';
	import { goto } from '$app/navigation';

	let username = $state('');
	let password = $state('');
	let error = $state('');
	let loading = $state(false);

	async function handleSubmit() {
		error = '';
		if (!username.trim() || !password) {
			error = 'Заполните все поля';
			return;
		}

		loading = true;
		try {
			const response = await api.login(username.trim(), password);
			if (response.ok) {
				goto('/lobby');
			} else {
				error = response.error || 'Неверные учётные данные';
			}
		} catch (e) {
			error = 'Ошибка соединения';
		} finally {
			loading = false;
		}
	}
</script>

<div class="flex min-h-[80vh] items-center justify-center px-4 animate-fade-in">
	<div class="w-full max-w-md">
		<!-- Header -->
		<div class="text-center mb-8">
			<div class="inline-flex items-center justify-center h-16 w-16 rounded-2xl bg-gradient-to-br from-blue-500 to-purple-600 mb-4 shadow-lg shadow-blue-500/30">
				<svg class="h-8 w-8 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 16l-4-4m0 0l4-4m-4 4h14m-5 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h7a3 3 0 013 3v1"/>
				</svg>
			</div>
			<h1 class="text-3xl font-bold text-white">Вход</h1>
			<p class="mt-2 text-slate-400">Войдите в свой аккаунт</p>
		</div>

		<!-- Form card -->
		<div class="glass-card rounded-2xl p-8">
			<form onsubmit={(e) => { e.preventDefault(); handleSubmit(); }} class="space-y-5">
				<div>
					<label for="username" class="block text-sm font-medium text-slate-300 mb-2">Имя пользователя</label>
					<input
						id="username"
						type="text"
						bind:value={username}
						placeholder="your_username"
						class="input-modern w-full rounded-xl px-4 py-3 text-white placeholder-slate-600"
						disabled={loading}
					/>
				</div>

				<div>
					<label for="password" class="block text-sm font-medium text-slate-300 mb-2">Пароль</label>
					<input
						id="password"
						type="password"
						bind:value={password}
						placeholder="••••••••"
						class="input-modern w-full rounded-xl px-4 py-3 text-white placeholder-slate-600"
						disabled={loading}
					/>
				</div>

				{#if error}
					<div class="flex items-center gap-2 rounded-xl bg-red-500/10 border border-red-500/20 px-4 py-3 text-sm text-red-400">
						<svg class="h-5 w-5 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>
						</svg>
						{error}
					</div>
				{/if}

				<button
					type="submit"
					disabled={loading}
					class="btn-primary w-full rounded-xl px-4 py-3 font-semibold text-white disabled:opacity-50"
				>
					{#if loading}
						<span class="flex items-center justify-center gap-2">
							<svg class="h-5 w-5 animate-spin" fill="none" viewBox="0 0 24 24">
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
				<a href="/auth/register" class="text-blue-400 hover:text-blue-300 font-medium transition">
					Зарегистрироваться
				</a>
			</div>
		</div>
	</div>
</div>
