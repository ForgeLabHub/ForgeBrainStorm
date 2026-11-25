<script lang="ts">
	import { api } from '$lib/api';
	import { goto } from '$app/navigation';

	let username = $state('');
	let password = $state('');
	let loading = $state(false);
	let error = $state('');
	let visible = $state(false);

	$: if (typeof window !== 'undefined') {
		setTimeout(() => (visible = true), 100);
	}

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

<div class="relative flex min-h-[80vh] items-center justify-center px-4 py-12">
	<!-- Decorative elements -->
	<div class="pointer-events-none absolute inset-0 overflow-hidden">
		<div class="absolute -top-20 left-1/4 h-72 w-72 rounded-full bg-blue-500/10 blur-3xl"></div>
		<div class="absolute bottom-0 right-1/4 h-72 w-72 rounded-full bg-purple-500/10 blur-3xl"></div>
	</div>

	<div
		class="relative w-full max-w-md transition-all duration-700 {visible ? 'translate-y-0 opacity-100' : 'translate-y-8 opacity-0'}"
	>
		<!-- Glow border -->
		<div class="absolute -inset-px rounded-3xl bg-gradient-to-r from-blue-500 via-purple-500 to-cyan-500 opacity-20 blur-sm"></div>

		<div class="relative overflow-hidden rounded-3xl border border-white/10 bg-white/[0.03] p-8 shadow-2xl backdrop-blur-xl sm:p-10">
			<!-- Inner gradient -->
			<div class="absolute inset-0 bg-gradient-to-br from-blue-500/5 via-transparent to-purple-500/5"></div>

			<div class="relative z-10">
				<!-- Header -->
				<div class="mb-8 text-center">
					<div class="relative mx-auto mb-5 inline-flex">
						<div class="absolute inset-0 animate-pulse rounded-2xl bg-gradient-to-r from-blue-500 to-purple-500 blur-lg opacity-50"></div>
						<div class="relative flex h-16 w-16 items-center justify-center rounded-2xl bg-gradient-to-br from-blue-500 to-purple-600 text-3xl shadow-lg">
							🧠
						</div>
					</div>
					<h1 class="text-3xl font-bold text-white">С возвращением</h1>
					<p class="mt-2 text-sm text-slate-400">Войдите, чтобы продолжить игру</p>
				</div>

				<form onsubmit={handleLogin} class="space-y-5">
					<div>
						<label for="username" class="mb-2 block text-xs font-medium uppercase tracking-wider text-slate-400">
							Имя пользователя
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
								autocomplete="username"
								class="w-full rounded-xl border border-white/10 bg-white/5 py-3 pl-11 pr-4 text-white placeholder-slate-500 backdrop-blur-sm transition focus:border-blue-500/50 focus:bg-white/10 focus:outline-none focus:ring-2 focus:ring-blue-500/20"
							/>
						</div>
					</div>

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
								placeholder="••••••••"
								autocomplete="current-password"
								class="w-full rounded-xl border border-white/10 bg-white/5 py-3 pl-11 pr-4 text-white placeholder-slate-500 backdrop-blur-sm transition focus:border-blue-500/50 focus:bg-white/10 focus:outline-none focus:ring-2 focus:ring-blue-500/20"
							/>
						</div>
					</div>

					{#if error}
						<div class="flex items-center gap-2 rounded-xl border border-red-500/20 bg-red-500/10 p-3 text-sm text-red-400">
							<svg class="h-4 w-4 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>
							</svg>
							{error}
						</div>
					{/if}

					<button
						type="submit"
						disabled={loading}
						class="group relative w-full overflow-hidden rounded-xl py-3 font-semibold text-white shadow-lg transition-all disabled:cursor-not-allowed disabled:opacity-50"
					>
						<div class="absolute inset-0 bg-gradient-to-r from-blue-500 to-purple-600 transition-transform group-hover:scale-105"></div>
						<div class="absolute inset-0 bg-gradient-to-r from-blue-500 to-purple-600 opacity-0 blur-xl transition-opacity group-hover:opacity-70"></div>
						<div class="relative flex items-center justify-center gap-2">
							{#if loading}
								<svg class="h-5 w-5 animate-spin" fill="none" viewBox="0 0 24 24">
									<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
									<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"></path>
								</svg>
								<span>Вход...</span>
							{:else}
								<span>Войти</span>
								<svg class="h-5 w-5 transition-transform group-hover:translate-x-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
									<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7l5 5m0 0l-5 5m5-5H6"></path>
								</svg>
							{/if}
						</div>
					</button>
				</form>

				<div class="relative mt-8">
					<div class="absolute inset-0 flex items-center">
						<div class="w-full border-t border-white/10"></div>
					</div>
					<div class="relative flex justify-center text-xs">
						<span class="bg-transparent px-2 text-slate-500">НОВОЕ ЗДЕСЬ?</span>
					</div>
				</div>

				<a
					href="/auth/register"
					class="mt-4 block w-full rounded-xl border border-white/10 bg-white/5 py-3 text-center font-medium text-white backdrop-blur-sm transition hover:border-white/20 hover:bg-white/10"
				>
					Создать аккаунт
				</a>
			</div>
		</div>
	</div>
</div>
