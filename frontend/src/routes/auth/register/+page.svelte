<script lang="ts">
	import { api } from '$lib/api';
	import { goto } from '$app/navigation';

	let username = '';
	let password = '';
	let confirmPassword = '';
	let loading = false;
	let error = '';

	$: passwordMatch = password === confirmPassword;
	$: passwordStrength = checkPasswordStrength(password);

	function checkPasswordStrength(pwd: string): { score: number; label: string; color: string } {
		let score = 0;
		if (pwd.length >= 12) score++;
		if (pwd.length >= 16) score++;
		if (/[A-Z]/.test(pwd) && /[a-z]/.test(pwd)) score++;
		if (/\d/.test(pwd)) score++;
		if (/[^A-Za-z0-9]/.test(pwd)) score++;

		if (score <= 1) return { score, label: 'Слабый', color: 'bg-red-500' };
		if (score <= 2) return { score, label: 'Средний', color: 'bg-yellow-500' };
		if (score <= 3) return { score, label: 'Хороший', color: 'bg-blue-500' };
		return { score, label: 'Отличный', color: 'bg-green-500' };
	}

	async function handleRegister() {
		if (!username.trim()) {
			error = 'Введите имя пользователя';
			return;
		}
		if (username.length < 2 || username.length > 20) {
			error = 'Имя: от 2 до 20 символов';
			return;
		}
		if (password.length < 12) {
			error = 'Пароль должен быть не менее 12 символов';
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
				goto('/lobby');
			} else {
				error = response.error || 'Не удалось зарегистрироваться';
			}
		} catch (e: any) {
			console.error('Registration failed:', e);
			error = e.message || 'Ошибка соединения с сервером';
		} finally {
			loading = false;
		}
	}
</script>

<div class="flex min-h-[70vh] items-center justify-center">
	<div class="w-full max-w-md rounded-2xl border border-slate-700 bg-slate-800 p-8 shadow-xl">
		<div class="mb-6 text-center">
			<div class="mb-3 text-5xl">🧠</div>
			<h1 class="text-2xl font-bold text-white">Регистрация</h1>
			<p class="mt-1 text-sm text-slate-400">Создайте аккаунт, чтобы играть</p>
		</div>

		<form on:submit|preventDefault={handleRegister} class="space-y-4">
			<div>
				<label for="username" class="mb-1 block text-sm font-medium text-slate-300">Имя пользователя</label>
				<input
					id="username"
					type="text"
					bind:value={username}
					placeholder="your_name"
					maxlength="20"
					class="w-full rounded-lg border border-slate-600 bg-slate-700 px-4 py-2.5 text-white placeholder-slate-400 focus:border-blue-500 focus:outline-none"
					autocomplete="username"
				/>
				<p class="mt-1 text-xs text-slate-500">{username.length}/20 символов</p>
			</div>

			<div>
				<label for="password" class="mb-1 block text-sm font-medium text-slate-300">Пароль</label>
				<input
					id="password"
					type="password"
					bind:value={password}
					placeholder="Минимум 12 символов"
					class="w-full rounded-lg border border-slate-600 bg-slate-700 px-4 py-2.5 text-white placeholder-slate-400 focus:border-blue-500 focus:outline-none"
					autocomplete="new-password"
				/>
				{#if password.length > 0}
					<div class="mt-2">
						<div class="flex items-center gap-2">
							<div class="h-1.5 flex-1 overflow-hidden rounded-full bg-slate-700">
								<div
									class="h-full transition-all duration-300 {passwordStrength.color}"
									style="width: {(passwordStrength.score / 5) * 100}%"
								></div>
							</div>
							<span class="text-xs text-slate-400">{passwordStrength.label}</span>
						</div>
					</div>
				{/if}
			</div>

			<div>
				<label for="confirmPassword" class="mb-1 block text-sm font-medium text-slate-300">Подтвердите пароль</label>
				<input
					id="confirmPassword"
					type="password"
					bind:value={confirmPassword}
					placeholder="Повторите пароль"
					class="w-full rounded-lg border px-4 py-2.5 text-white placeholder-slate-400 focus:outline-none
						{passwordMatch ? 'border-slate-600 bg-slate-700 focus:border-blue-500' : 'border-red-500/50 bg-red-500/5 focus:border-red-500'}"
					autocomplete="new-password"
				/>
				{#if confirmPassword && !passwordMatch}
					<p class="mt-1 text-xs text-red-400">Пароли не совпадают</p>
				{/if}
			</div>

			{#if error}
				<div class="rounded-lg bg-red-500/10 p-3 text-sm text-red-400">
					{error}
				</div>
			{/if}

			<button
				type="submit"
				disabled={loading || !passwordMatch || password.length < 12}
				class="w-full rounded-lg bg-blue-600 py-2.5 font-semibold text-white transition hover:bg-blue-700 disabled:cursor-not-allowed disabled:opacity-50"
			>
				{#if loading}
					<span class="flex items-center justify-center gap-2">
						<div class="h-4 w-4 animate-spin rounded-full border-2 border-white border-t-transparent"></div>
						Регистрация...
					</span>
				{:else}
					Создать аккаунт
				{/if}
			</button>
		</form>

		<div class="mt-6 text-center text-sm text-slate-400">
			Уже есть аккаунт?
			<a href="/auth/login" class="font-medium text-blue-400 hover:text-blue-300">Войти</a>
		</div>
	</div>
</div>
