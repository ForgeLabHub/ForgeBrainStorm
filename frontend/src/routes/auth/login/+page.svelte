<script lang="ts">
	import { api } from '$lib/api';
	import { goto } from '$app/navigation';
	import { refreshAuth } from '$lib/stores/auth';

	let username = '';
	let password = '';
	let loading = false;
	let error = '';

	async function handleLogin() {
		if (!username.trim() || !password) {
			error = 'Заполните оба поля';
			return;
		}

		loading = true;
		error = '';

		try {
			const response = await api.login(username.trim(), password);
			if (response.ok) {
				await refreshAuth();
				goto('/lobby');
			} else {
				error = response.error || 'Неверное имя или пароль';
			}
		} catch (e: any) {
			error = e.message || 'Сервер не отвечает. Попробуйте ещё раз.';
		} finally {
			loading = false;
		}
	}
</script>

<div class="mx-auto flex min-h-[70vh] w-full max-w-[400px] flex-col justify-center py-4">
	<div class="card card-pad">
		<h1 class="font-display text-2xl tracking-tight text-chalk">Вход</h1>
		<p class="mt-1 text-sm text-dim">Войдите, чтобы играть и сохранять прогресс.</p>

		<form on:submit|preventDefault={handleLogin} class="mt-6 space-y-4">
			<div>
				<label for="username" class="label">Имя пользователя</label>
				<input
					id="username"
					type="text"
					bind:value={username}
					placeholder="например, anna"
					class="field"
					autocomplete="username"
					autocapitalize="none"
					spellcheck="false"
				/>
			</div>

			<div>
				<label for="password" class="label">Пароль</label>
				<input
					id="password"
					type="password"
					bind:value={password}
					placeholder="••••••••••••"
					class="field"
					autocomplete="current-password"
				/>
			</div>

			{#if error}
				<p class="rounded-[12px] border border-coral/30 bg-coral/10 px-3 py-2.5 text-sm text-coral" role="alert">
					{error}
				</p>
			{/if}

			<button type="submit" disabled={loading} class="btn btn-primary w-full">
				{#if loading}
					<span class="h-4 w-4 animate-spin rounded-full border-2 border-ink-950/40 border-t-ink-950"></span>
					Вход…
				{:else}
					Войти
				{/if}
			</button>
		</form>

		<p class="mt-5 text-sm text-dim">
			Нет аккаунта?
			<a href="/auth/register" class="font-medium text-chalk underline decoration-line underline-offset-4 hover:decoration-amber">
				Зарегистрироваться
			</a>
		</p>
	</div>
</div>
