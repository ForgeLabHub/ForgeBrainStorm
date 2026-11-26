<script lang="ts">
	import { api } from '$lib/api';
	import { goto } from '$app/navigation';

	let username = $state('');
	let password = $state('');
	let confirmPassword = $state('');
	let loading = $state(false);
	let error = $state('');

<<<<<<< HEAD
	const passwordMatch = $derived(password === confirmPassword);
	const passwordStrength = $derived(checkPasswordStrength(password));
=======
	$: passwordMatch = password === confirmPassword;
	$: passwordStrength = checkPasswordStrength(password);
>>>>>>> 4808142 (Merge pull request #15 from ForgeLabHub/arena/ca3aa14f-forgebrainstorm)

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
		return { score, label: 'Сильный', color: 'bg-green-500' };
	}

	async function handleRegister(e: Event) {
		e.preventDefault();
		if (!username.trim()) {
			error = 'Введите имя пользователя';
			return;
		}
		if (username.length < 2 || username.length > 20) {
			error = 'Имя: от 2 до 20 символов';
			return;
		}
		if (password.length < 12) {
			error = 'Пароль: минимум 12 символов';
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
			error = e.message || 'Ошибка соединения с сервером';
		} finally {
			loading = false;
		}
	}
</script>

<div class="flex min-h-[70vh] items-center justify-center px-4">
	<div class="w-full max-w-sm">
		<div class="mb-8 text-center">
			<h1 class="text-2xl font-bold text-white">Регистрация</h1>
			<p class="mt-2 text-sm text-slate-400">Создайте аккаунт для игры</p>
		</div>

		<form onsubmit={handleRegister} class="space-y-4">
			<div>
				<label for="username" class="mb-2 block text-sm font-medium text-slate-300">
					Имя пользователя
				</label>
				<input
					id="username"
					type="text"
					bind:value={username}
					placeholder="your_name"
					maxlength="20"
					autocomplete="username"
					class="w-full rounded-lg border border-white/10 bg-white/5 px-4 py-2.5 text-white placeholder-slate-500 transition focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
				/>
				<p class="mt-1 text-xs text-slate-500">{username.length}/20</p>
			</div>

			<div>
				<label for="password" class="mb-2 block text-sm font-medium text-slate-300">
					Пароль
				</label>
				<input
					id="password"
					type="password"
					bind:value={password}
					placeholder="Минимум 12 символов"
					autocomplete="new-password"
					class="w-full rounded-lg border border-white/10 bg-white/5 px-4 py-2.5 text-white placeholder-slate-500 transition focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
				/>
				{#if password.length > 0}
					<div class="mt-2 flex items-center gap-2">
						<div class="h-1 flex-1 overflow-hidden rounded-full bg-white/5">
							<div
								class="h-full transition-all duration-300 {passwordStrength.color}"
								style="width: {(passwordStrength.score / 5) * 100}%"
							></div>
						</div>
						<span class="text-xs text-slate-400">{passwordStrength.label}</span>
					</div>
				{/if}
			</div>

			<div>
				<label for="confirmPassword" class="mb-2 block text-sm font-medium text-slate-300">
					Подтвердите пароль
				</label>
				<input
					id="confirmPassword"
					type="password"
					bind:value={confirmPassword}
					placeholder="Повторите пароль"
					autocomplete="new-password"
					class="w-full rounded-lg border px-4 py-2.5 text-white placeholder-slate-500 transition focus:outline-none focus:ring-1 {
						confirmPassword && !passwordMatch
							? 'border-red-500/50 bg-red-500/5 focus:border-red-500 focus:ring-red-500'
							: confirmPassword && passwordMatch
								? 'border-green-500/50 bg-green-500/5 focus:border-green-500 focus:ring-green-500'
								: 'border-white/10 bg-white/5 focus:border-blue-500 focus:ring-blue-500'
					}"
				/>
			</div>

			{#if error}
				<div class="rounded-lg border border-red-500/20 bg-red-500/10 p-3 text-sm text-red-400">
					{error}
				</div>
			{/if}

			<button
				type="submit"
				disabled={loading || !passwordMatch || password.length < 12}
				class="w-full rounded-lg bg-blue-500 py-2.5 font-medium text-white transition hover:bg-blue-600 disabled:cursor-not-allowed disabled:opacity-50"
			>
				{#if loading}
					<span class="flex items-center justify-center gap-2">
						<svg class="h-4 w-4 animate-spin" fill="none" viewBox="0 0 24 24">
							<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
							<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"></path>
						</svg>
						Создание...
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
