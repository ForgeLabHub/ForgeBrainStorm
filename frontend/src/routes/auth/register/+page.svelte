<script lang="ts">
	import { api } from '$lib/api';
	import { goto } from '$app/navigation';
	import { refreshAuth } from '$lib/stores/auth';

	let username = '';
	let password = '';
	let confirmPassword = '';
	let loading = false;
	let error = '';

	$: passwordMatch = password === confirmPassword;
	$: strength = scorePassword(password);

	function scorePassword(value: string): { score: number; label: string; tone: string } {
		let score = 0;
		if (value.length >= 8) score++;
		if (value.length >= 12) score++;
		if (/[a-z]/.test(value) && /[A-Z]/.test(value)) score++;
		if (/\d/.test(value)) score++;
		if (/[^A-Za-z0-9]/.test(value)) score++;

		if (score <= 1) return { score, label: 'Слабый', tone: 'bg-coral' };
		if (score <= 2) return { score, label: 'Средний', tone: 'bg-amber' };
		if (score <= 3) return { score, label: 'Хороший', tone: 'bg-azure' };
		return { score, label: 'Надёжный', tone: 'bg-mint' };
	}

	async function handleRegister() {
		if (!username.trim()) {
			error = 'Введите имя пользователя';
			return;
		}
		if (username.trim().length < 2 || username.trim().length > 20) {
			error = 'Имя — от 2 до 20 символов';
			return;
		}
		if (password.length < 12) {
			error = 'Пароль — не короче 12 символов';
			return;
		}
		if (!passwordMatch) {
			error = 'Пароли не совпадают';
			return;
		}

		loading = true;
		error = '';

		try {
			const response = await api.register(username.trim(), password);
			if (response.ok) {
				await refreshAuth();
				goto('/lobby');
			} else {
				error = response.error || 'Не удалось создать аккаунт';
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
		<h1 class="font-display text-2xl tracking-tight text-chalk">Регистрация</h1>
		<p class="mt-1 text-sm text-dim">Аккаунт нужен, чтобы играть под своим именем и копить опыт.</p>

		<form on:submit|preventDefault={handleRegister} class="mt-6 space-y-4">
			<div>
				<label for="username" class="label">Имя пользователя</label>
				<input
					id="username"
					type="text"
					bind:value={username}
					placeholder="например, anna"
					maxlength="20"
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
					placeholder="Минимум 12 символов"
					class="field"
					autocomplete="new-password"
				/>
				{#if password.length > 0}
					<div class="mt-2 flex items-center gap-3">
						<div class="h-1.5 flex-1 overflow-hidden rounded-full bg-ink-700">
							<div class="h-full transition-all duration-300 {strength.tone}" style="width: {(strength.score / 5) * 100}%"></div>
						</div>
						<span class="w-20 text-right text-xs text-dim">{strength.label}</span>
					</div>
				{/if}
			</div>

			<div>
				<label for="confirmPassword" class="label">Повторите пароль</label>
				<input
					id="confirmPassword"
					type="password"
					bind:value={confirmPassword}
					placeholder="Ещё раз"
					class="field {confirmPassword && !passwordMatch ? 'border-coral/60 focus:border-coral' : ''}"
					autocomplete="new-password"
				/>
				{#if confirmPassword && !passwordMatch}
					<p class="mt-1.5 text-xs text-coral">Пароли не совпадают</p>
				{/if}
			</div>

			{#if error}
				<p class="rounded-[12px] border border-coral/30 bg-coral/10 px-3 py-2.5 text-sm text-coral" role="alert">
					{error}
				</p>
			{/if}

			<button
				type="submit"
				disabled={loading || !passwordMatch || password.length < 12 || !username.trim()}
				class="btn btn-primary w-full"
			>
				{#if loading}
					<span class="h-4 w-4 animate-spin rounded-full border-2 border-ink-950/40 border-t-ink-950"></span>
					Создаём…
				{:else}
					Создать аккаунт
				{/if}
			</button>
		</form>

		<p class="mt-5 text-sm text-dim">
			Уже есть аккаунт?
			<a href="/auth/login" class="font-medium text-chalk underline decoration-line underline-offset-4 hover:decoration-amber">
				Войти
			</a>
		</p>
	</div>
</div>
