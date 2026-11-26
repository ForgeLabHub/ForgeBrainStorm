<script lang="ts">
	import { api } from '$lib/api';
	import { goto } from '$app/navigation';

	let username = $state('');
	let displayName = $state('');
	let password = $state('');
	let confirmPassword = $state('');
	let error = $state('');
	let loading = $state(false);

	function getPasswordStrength(pwd: string): { score: number; label: string; color: string } {
		if (pwd.length === 0) return { score: 0, label: '', color: '' };
		let score = 0;
		if (pwd.length >= 8) score++;
		if (pwd.length >= 12) score++;
		if (/[a-z]/.test(pwd) && /[A-Z]/.test(pwd)) score++;
		if (/\d/.test(pwd)) score++;
		if (/[^a-zA-Z\d]/.test(pwd)) score++;

		if (score <= 1) return { score: 20, label: 'Слабый', color: 'bg-red-500' };
		if (score <= 2) return { score: 40, label: 'Средний', color: 'bg-yellow-500' };
		if (score <= 3) return { score: 60, label: 'Хороший', color: 'bg-blue-500' };
		if (score <= 4) return { score: 80, label: 'Сильный', color: 'bg-green-500' };
		return { score: 100, label: 'Отличный', color: 'bg-emerald-500' };
	}

	$: passwordStrength = getPasswordStrength(password);
	$: passwordsMatch = password === confirmPassword && confirmPassword.length > 0;

	async function handleSubmit() {
		error = '';
		
		if (!username.trim() || !displayName.trim() || !password) {
			error = 'Заполните все поля';
			return;
		}

		if (password !== confirmPassword) {
			error = 'Пароли не совпадают';
			return;
		}

		if (password.length < 8) {
			error = 'Пароль должен быть не менее 8 символов';
			return;
		}

		loading = true;
		try {
			const response = await api.register(username.trim(), displayName.trim(), password);
			if (response.ok) {
				goto('/lobby');
			} else {
				error = response.error || 'Ошибка регистрации';
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
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M18 9v3m0 0v3m0-3h3m-3 0h-3m-2-5a4 4 0 11-8 0 4 4 0 018 0zM3 20a6 6 0 0112 0v1H3v-1z"/>
				</svg>
			</div>
			<h1 class="text-3xl font-bold text-white">Регистрация</h1>
			<p class="mt-2 text-slate-400">Создайте новый аккаунт</p>
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
					<label for="displayName" class="block text-sm font-medium text-slate-300 mb-2">Отображаемое имя</label>
					<input
						id="displayName"
						type="text"
						bind:value={displayName}
						placeholder="Ваше имя"
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
					{#if password.length > 0}
						<div class="mt-2">
							<div class="flex items-center justify-between mb-1">
								<div class="h-1.5 flex-1 bg-white/5 rounded-full overflow-hidden">
									<div class="h-full transition-all duration-300 {passwordStrength.color}" style="width: {passwordStrength.score}%"></div>
								</div>
							</div>
							<p class="text-xs text-slate-500">{passwordStrength.label}</p>
						</div>
					{/if}
				</div>

				<div>
					<label for="confirmPassword" class="block text-sm font-medium text-slate-300 mb-2">Подтвердите пароль</label>
					<input
						id="confirmPassword"
						type="password"
						bind:value={confirmPassword}
						placeholder="••••••••"
						class="input-modern w-full rounded-xl px-4 py-3 text-white placeholder-slate-600 {passwordsMatch ? 'border-green-500/50' : ''}"
						disabled={loading}
					/>
					{#if confirmPassword.length > 0}
						<p class="mt-1 text-xs {passwordsMatch ? 'text-green-400' : 'text-red-400'}">
							{passwordsMatch ? '✓ Пароли совпадают' : '✗ Пароли не совпадают'}
						</p>
					{/if}
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
							Создание...
						</span>
					{:else}
						Зарегистрироваться
					{/if}
				</button>
			</form>

			<div class="mt-6 text-center text-sm text-slate-400">
				Уже есть аккаунт?
				<a href="/auth/login" class="text-blue-400 hover:text-blue-300 font-medium transition">
					Войти
				</a>
			</div>
		</div>
	</div>
</div>
