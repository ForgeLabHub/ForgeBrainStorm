<script lang="ts">
	import { api } from '$lib/api';
	import { goto } from '$app/navigation';

	let username = $state('');
	let password = $state('');
	let confirmPassword = $state('');
	let loading = $state(false);
	let error = $state('');
	let visible = $state(false);

	$: if (typeof window !== 'undefined') {
		setTimeout(() => (visible = true), 100);
	}

	$: passwordMatch = password === confirmPassword;
	$: passwordStrength = checkPasswordStrength(password);

	function checkPasswordStrength(pwd: string): { score: number; label: string; color: string; bg: string } {
		let score = 0;
		if (pwd.length >= 12) score++;
		if (pwd.length >= 16) score++;
		if (/[A-Z]/.test(pwd) && /[a-z]/.test(pwd)) score++;
		if (/\d/.test(pwd)) score++;
		if (/[^A-Za-z0-9]/.test(pwd)) score++;

		if (score <= 1) return { score, label: 'Слабый', color: 'text-red-400', bg: 'bg-red-500' };
		if (score <= 2) return { score, label: 'Средний', color: 'text-yellow-400', bg: 'bg-yellow-500' };
		if (score <= 3) return { score, label: 'Хороший', color: 'text-blue-400', bg: 'bg-blue-500' };
		return { score, label: 'Отличный', color: 'text-green-400', bg: 'bg-green-500' };
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
			error = e.message || 'Ошибка соединения с сервером';
		} finally {
			loading = false;
		}
	}
</script>

<div class="relative flex min-h-[80vh] items-center justify-center px-4 py-12">
	<!-- Decorative elements -->
	<div class="pointer-events-none absolute inset-0 overflow-hidden">
		<div class="absolute -top-20 right-1/4 h-72 w-72 rounded-full bg-purple-500/10 blur-3xl"></div>
		<div class="absolute bottom-0 left-1/4 h-72 w-72 rounded-full bg-cyan-500/10 blur-3xl"></div>
	</div>

	<div
		class="relative w-full max-w-md transition-all duration-700 {visible ? 'translate-y-0 opacity-100' : 'translate-y-8 opacity-0'}"
	>
		<!-- Glow border -->
		<div class="absolute -inset-px rounded-3xl bg-gradient-to-r from-purple-500 via-cyan-500 to-blue-500 opacity-20 blur-sm"></div>

		<div class="relative overflow-hidden rounded-3xl border border-white/10 bg-white/[0.03] p-8 shadow-2xl backdrop-blur-xl sm:p-10">
			<!-- Inner gradient -->
			<div class="absolute inset-0 bg-gradient-to-br from-purple-500/5 via-transparent to-cyan-500/5"></div>

			<div class="relative z-10">
				<!-- Header -->
				<div class="mb-8 text-center">
					<div class="relative mx-auto mb-5 inline-flex">
						<div class="absolute inset-0 animate-pulse rounded-2xl bg-gradient-to-r from-purple-500 to-cyan-500 blur-lg opacity-50"></div>
						<div class="relative flex h-16 w-16 items-center justify-center rounded-2xl bg-gradient-to-br from-purple-500 to-cyan-600 text-3xl shadow-lg">
							✨
						</div>
					</div>
					<h1 class="text-3xl font-bold text-white">Создать аккаунт</h1>
					<p class="mt-2 text-sm text-slate-400">Присоединяйтесь к BrainStorm</p>
				</div>

				<form onsubmit={handleRegister} class="space-y-5">
					<!-- Username -->
					<div>
						<label for="username" class="mb-2 flex items-center justify-between text-xs font-medium uppercase tracking-wider text-slate-400">
							<span>Имя пользователя</span>
							<span class="normal-case tracking-normal {username.length > 18 ? 'text-orange-400' : 'text-slate-500'}">
								{username.length}/20
							</span>
						</label>
						<div class="relative">
							<svg class="absolute left-3.5 top-1/2 h-5 w-5 -translate-y-1/2 text-slate-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z"></path>
							</svg>
							<input
								id="username"
								type="text"
								bind:value={username}
								placeholder="your_name"
								maxlength="20"
								autocomplete="username"
								class="w-full rounded-xl border border-white/10 bg-white/5 py-3 pl-11 pr-4 text-white placeholder-slate-500 backdrop-blur-sm transition focus:border-purple-500/50 focus:bg-white/10 focus:outline-none focus:ring-2 focus:ring-purple-500/20"
							/>
						</div>
					</div>

					<!-- Password -->
					<div>
						<label for="password" class="mb-2 block text-xs font-medium uppercase tracking-wider text-slate-400">
							Пароль
						</label>
						<div class="relative">
							<svg class="absolute left-3.5 top-1/2 h-5 w-5 -translate-y-1/2 text-slate-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z"></path>
							</svg>
							<input
								id="password"
								type="password"
								bind:value={password}
								placeholder="Минимум 12 символов"
								autocomplete="new-password"
								class="w-full rounded-xl border border-white/10 bg-white/5 py-3 pl-11 pr-4 text-white placeholder-slate-500 backdrop-blur-sm transition focus:border-purple-500/50 focus:bg-white/10 focus:outline-none focus:ring-2 focus:ring-purple-500/20"
							/>
						</div>

						{#if password.length > 0}
							<div class="mt-3 flex items-center gap-3">
								<div class="h-1.5 flex-1 overflow-hidden rounded-full bg-white/5">
									<div
										class="h-full rounded-full transition-all duration-500 {passwordStrength.bg}"
										style="width: {(passwordStrength.score / 5) * 100}%"
									></div>
								</div>
								<span class="text-xs font-medium {passwordStrength.color}">
									{passwordStrength.label}
								</span>
							</div>
						{/if}
					</div>

					<!-- Confirm Password -->
					<div>
						<label for="confirmPassword" class="mb-2 block text-xs font-medium uppercase tracking-wider text-slate-400">
							Подтвердите пароль
						</label>
						<div class="relative">
							<svg
								class="absolute left-3.5 top-1/2 h-5 w-5 -translate-y-1/2 transition-colors {confirmPassword && !passwordMatch ? 'text-red-400' : confirmPassword && passwordMatch ? 'text-green-400' : 'text-slate-500'}"
								fill="none"
								stroke="currentColor"
								viewBox="0 0 24 24"
							>
								{#if confirmPassword && passwordMatch}
									<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"></path>
								{:else if confirmPassword && !passwordMatch}
									<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
								{:else}
									<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"></path>
								{/if}
							</svg>
							<input
								id="confirmPassword"
								type="password"
								bind:value={confirmPassword}
								placeholder="Повторите пароль"
								autocomplete="new-password"
								class="w-full rounded-xl border py-3 pl-11 pr-4 text-white placeholder-slate-500 backdrop-blur-sm transition focus:bg-white/10 focus:outline-none focus:ring-2 {
									confirmPassword && !passwordMatch
										? 'border-red-500/50 bg-red-500/5 focus:border-red-500/50 focus:ring-red-500/20'
										: confirmPassword && passwordMatch
											? 'border-green-500/50 bg-green-500/5 focus:border-green-500/50 focus:ring-green-500/20'
											: 'border-white/10 bg-white/5 focus:border-purple-500/50 focus:ring-purple-500/20'
								}"
							/>
						</div>
					</div>

					<!-- Error -->
					{#if error}
						<div class="flex items-center gap-2 rounded-xl border border-red-500/20 bg-red-500/10 p-3 text-sm text-red-400">
							<svg class="h-4 w-4 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>
							</svg>
							{error}
						</div>
					{/if}

					<!-- Submit -->
					<button
						type="submit"
						disabled={loading || !passwordMatch || password.length < 12}
						class="group relative w-full overflow-hidden rounded-xl py-3 font-semibold text-white shadow-lg transition-all disabled:cursor-not-allowed disabled:opacity-50"
					>
						<div class="absolute inset-0 bg-gradient-to-r from-purple-500 to-cyan-600 transition-transform group-hover:scale-105"></div>
						<div class="absolute inset-0 bg-gradient-to-r from-purple-500 to-cyan-600 opacity-0 blur-xl transition-opacity group-hover:opacity-70"></div>
						<div class="relative flex items-center justify-center gap-2">
							{#if loading}
								<svg class="h-5 w-5 animate-spin" fill="none" viewBox="0 0 24 24">
									<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
									<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"></path>
								</svg>
								<span>Создание...</span>
							{:else}
								<span>Создать аккаунт</span>
								<svg class="h-5 w-5 transition-transform group-hover:translate-x-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
									<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7l5 5m0 0l-5 5m5-5H6"></path>
								</svg>
							{/if}
						</div>
					</button>
				</form>

				<!-- Divider -->
				<div class="relative mt-8">
					<div class="absolute inset-0 flex items-center">
						<div class="w-full border-t border-white/10"></div>
					</div>
					<div class="relative flex justify-center text-xs">
						<span class="bg-transparent px-2 text-slate-500">УЖЕ ЕСТЬ АККАУНТ?</span>
					</div>
				</div>

				<a
					href="/auth/login"
					class="mt-4 block w-full rounded-xl border border-white/10 bg-white/5 py-3 text-center font-medium text-white backdrop-blur-sm transition hover:border-white/20 hover:bg-white/10"
				>
					Войти
				</a>
			</div>
		</div>
	</div>
</div>
