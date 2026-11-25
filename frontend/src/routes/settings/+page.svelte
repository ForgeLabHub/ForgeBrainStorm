<script lang="ts">
	import { api } from '$lib/api';
	import { onMount } from 'svelte';

	let providers: string[] = $state([]);
	let defaultProvider = $state('');
	let cacheStats: { questions_cached: number; hints_cached: number } | null = $state(null);
	let loading = $state(true);
	let switching = $state(false);
	let message = $state('');
	let visible = $state(false);

	$: if (typeof window !== 'undefined') {
		setTimeout(() => (visible = true), 100);
	}

	onMount(async () => {
		await loadProviders();
		await loadCacheStats();
	});

	async function loadProviders() {
		try {
			const response = await api.getAiProviders();
			providers = response.providers || [];
			defaultProvider = response.default || '';
		} catch (error) {
			message = 'Ошибка загрузки списка провайдеров';
		} finally {
			loading = false;
		}
	}

	async function loadCacheStats() {
		try {
			const response = await api.getAiCacheStats();
			cacheStats = response as any;
		} catch (error) {
			console.error('Failed to load cache stats:', error);
		}
	}

	async function switchProvider(provider: string) {
		switching = true;
		message = '';
		try {
			await api.switchAiProvider(provider);
			defaultProvider = provider;
			message = `✓ Переключено на ${provider}`;
			setTimeout(() => (message = ''), 3000);
		} catch (error) {
			message = 'Ошибка переключения провайдера';
		} finally {
			switching = false;
		}
	}

	async function clearCache() {
		try {
			await api.clearAiCache();
			message = '✓ Кэш очищен';
			cacheStats = { questions_cached: 0, hints_cached: 0 };
			setTimeout(() => (message = ''), 3000);
		} catch (error) {
			message = 'Ошибка очистки кэша';
		}
	}

	function getProviderInfo(name: string): { icon: string; description: string; gradient: string; badge?: string } {
		const info: Record<string, any> = {
			openrouter: {
				icon: '🌐',
				description: 'OpenRouter — доступ к GPT-4o-mini, Claude и другим моделям',
				gradient: 'from-cyan-500 to-blue-500',
				badge: 'Рекомендуется',
			},
			gigachat: {
				icon: '🇷🇺',
				description: 'Sber GigaChat — оптимизирован для русского языка',
				gradient: 'from-green-500 to-emerald-500',
			},
			openai: {
				icon: '🤖',
				description: 'OpenAI GPT-4 — высочайшее качество генерации',
				gradient: 'from-purple-500 to-pink-500',
				badge: 'Премиум',
			},
			ollama: {
				icon: '🏠',
				description: 'Ollama — локальная модель, бесплатно и приватно',
				gradient: 'from-orange-500 to-red-500',
				badge: 'Бесплатно',
			},
			anthropic: {
				icon: '🧠',
				description: 'Anthropic Claude — отличное качество и безопасность',
				gradient: 'from-amber-500 to-yellow-500',
			},
		};
		return info[name] || { icon: '⚡', description: name, gradient: 'from-slate-500 to-slate-600' };
	}
</script>

<div class="mx-auto max-w-4xl px-4 py-8 sm:px-6">
	<!-- Header -->
	<div class="mb-8 transition-all duration-700 {visible ? 'translate-y-0 opacity-100' : 'translate-y-8 opacity-0'}">
		<div class="flex items-center gap-3">
			<div class="flex h-10 w-10 items-center justify-center rounded-xl bg-gradient-to-br from-purple-500 to-cyan-500 text-xl">
				⚡
			</div>
			<div>
				<h1 class="text-2xl font-bold text-white sm:text-3xl">Настройки AI</h1>
				<p class="text-sm text-slate-400 sm:text-base">Управление AI-провайдерами для генерации вопросов</p>
			</div>
		</div>
	</div>

	{#if loading}
		<!-- Skeleton -->
		<div class="space-y-4">
			{#each Array(3) as _}
				<div class="h-24 animate-pulse rounded-2xl bg-white/5"></div>
			{/each}
		</div>
	{:else}
		<!-- Provider list -->
		<div class="space-y-4 transition-all duration-700 delay-100 {visible ? 'translate-y-0 opacity-100' : 'translate-y-8 opacity-0'}">
			<h2 class="flex items-center gap-2 text-sm font-medium uppercase tracking-wider text-slate-400">
				<svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z"></path>
				</svg>
				Провайдеры
			</h2>

			{#if providers.length === 0}
				<div class="relative overflow-hidden rounded-2xl border border-white/5 bg-white/[0.02] p-8 text-center backdrop-blur-sm">
					<div class="absolute inset-0 bg-gradient-to-br from-red-500/5 to-transparent"></div>
					<div class="relative z-10">
						<div class="mb-4 text-5xl">🔌</div>
						<h3 class="text-lg font-semibold text-white">Нет доступных провайдеров</h3>
						<p class="mt-2 text-sm text-slate-400">
							Проверьте конфигурацию и API-ключи в <code class="rounded bg-white/10 px-1.5 py-0.5 text-xs">.env</code>
						</p>
					</div>
				</div>
			{:else}
				<div class="grid gap-3">
					{#each providers as provider}
						{@const info = getProviderInfo(provider)}
						{@const isActive = provider === defaultProvider}

						<button
							onclick={() => switchProvider(provider)}
							disabled={switching || isActive}
							class="group relative overflow-hidden rounded-2xl border p-4 text-left backdrop-blur-sm transition-all sm:p-5 {
								isActive
									? 'border-white/20 bg-white/[0.05]'
									: 'border-white/5 bg-white/[0.02] hover:border-white/20 hover:bg-white/[0.04]'
							} {switching && !isActive ? 'cursor-wait opacity-50' : ''}"
						>
							{#if isActive}
								<div class="absolute inset-0 bg-gradient-to-r {info.gradient} opacity-5"></div>
							{/if}

							<div class="relative z-10 flex items-center gap-4">
								<!-- Icon -->
								<div class="flex h-12 w-12 flex-shrink-0 items-center justify-center rounded-xl {isActive ? `bg-gradient-to-br ${info.gradient}` : 'bg-white/5'} text-xl transition-transform group-hover:scale-110 sm:h-14 sm:w-14">
									{info.icon}
								</div>

								<!-- Info -->
								<div class="min-w-0 flex-1">
									<div class="flex flex-wrap items-center gap-2">
										<span class="font-semibold text-white">{provider}</span>
										{#if info.badge}
											<span class="rounded-full bg-white/10 px-2 py-0.5 text-[10px] font-medium text-slate-300 sm:text-xs">
												{info.badge}
											</span>
										{/if}
									</div>
									<p class="mt-0.5 text-xs text-slate-400 sm:text-sm">{info.description}</p>
								</div>

								<!-- Status -->
								<div class="flex-shrink-0">
									{#if isActive}
										<div class="flex items-center gap-2 rounded-full bg-green-500/10 border border-green-500/20 px-3 py-1.5">
											<svg class="h-4 w-4 text-green-400" fill="currentColor" viewBox="0 0 20 20">
												<path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd" />
											</svg>
											<span class="text-xs font-medium text-green-400">Активен</span>
										</div>
									{:else}
										<div class="rounded-full bg-white/5 border border-white/10 px-3 py-1.5 text-xs text-slate-400 opacity-0 transition-opacity group-hover:opacity-100">
											Выбрать
										</div>
									{/if}
								</div>
							</div>
						</button>
					{/each}
				</div>
			{/if}

			<!-- Message -->
			{#if message}
				<div class="flex items-center gap-2 rounded-xl border border-green-500/20 bg-green-500/10 p-3 text-sm text-green-400">
					{message}
				</div>
			{/if}
		</div>

		<!-- Cache section -->
		<div class="mt-8 transition-all duration-700 delay-200 {visible ? 'translate-y-0 opacity-100' : 'translate-y-8 opacity-0'}">
			<h2 class="mb-4 flex items-center gap-2 text-sm font-medium uppercase tracking-wider text-slate-400">
				<svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4m0 5c0 2.21-3.582 4-8 4s-8-1.79-8-4"></path>
				</svg>
				Кэш
			</h2>

			<div class="rounded-2xl border border-white/5 bg-white/[0.02] p-5 backdrop-blur-sm sm:p-6">
				<div class="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
					<div class="grid grid-cols-2 gap-4 sm:flex sm:gap-6">
						<div>
							<div class="text-xs font-medium uppercase tracking-wider text-slate-500">Вопросов</div>
							<div class="mt-1 text-2xl font-bold text-white">{cacheStats?.questions_cached ?? 0}</div>
						</div>
						<div>
							<div class="text-xs font-medium uppercase tracking-wider text-slate-500">Подсказок</div>
							<div class="mt-1 text-2xl font-bold text-white">{cacheStats?.hints_cached ?? 0}</div>
						</div>
					</div>

					<button
						onclick={clearCache}
						class="flex items-center justify-center gap-2 rounded-xl border border-white/10 bg-white/5 px-4 py-2.5 text-sm font-medium text-white backdrop-blur-sm transition hover:border-red-500/30 hover:bg-red-500/10 hover:text-red-400"
					>
						<svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"></path>
						</svg>
						Очистить кэш
					</button>
				</div>
			</div>
		</div>

		<!-- Info section -->
		<div class="mt-8 transition-all duration-700 delay-300 {visible ? 'translate-y-0 opacity-100' : 'translate-y-8 opacity-0'}">
			<h2 class="mb-4 flex items-center gap-2 text-sm font-medium uppercase tracking-wider text-slate-400">
				<svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>
				</svg>
				Информация
			</h2>

			<div class="rounded-2xl border border-white/5 bg-white/[0.02] p-5 backdrop-blur-sm sm:p-6">
				<div class="space-y-4">
					<div class="flex items-start gap-3">
						<div class="mt-0.5 flex h-6 w-6 flex-shrink-0 items-center justify-center rounded-lg bg-blue-500/10 text-xs">
							🔄
						</div>
						<div>
							<div class="text-sm font-medium text-white">Автоматическое переключение</div>
							<div class="text-xs text-slate-400">Система автоматически переключается между провайдерами при недоступности</div>
						</div>
					</div>

					<div class="flex items-start gap-3">
						<div class="mt-0.5 flex h-6 w-6 flex-shrink-0 items-center justify-center rounded-lg bg-green-500/10 text-xs">
							⚡
						</div>
						<div>
							<div class="text-sm font-medium text-white">Кэширование на 1 час</div>
							<div class="text-xs text-slate-400">Ответы кэшируются для экономии API-вызовов и ускорения работы</div>
						</div>
					</div>

					<div class="flex items-start gap-3">
						<div class="mt-0.5 flex h-6 w-6 flex-shrink-0 items-center justify-center rounded-lg bg-purple-500/10 text-xs">
							🏠
						</div>
						<div>
							<div class="text-sm font-medium text-white">Локальные модели</div>
							<div class="text-xs text-slate-400">Ollama — бесплатная локальная модель, требует установки на сервер</div>
						</div>
					</div>

					<div class="flex items-start gap-3">
						<div class="mt-0.5 flex h-6 w-6 flex-shrink-0 items-center justify-center rounded-lg bg-orange-500/10 text-xs">
							🇷🇺
						</div>
						<div>
							<div class="text-sm font-medium text-white">Оптимизация для русского</div>
							<div class="text-xs text-slate-400">GigaChat специально обучен для работы с русским языком</div>
						</div>
					</div>
				</div>
			</div>
		</div>
	{/if}
</div>
