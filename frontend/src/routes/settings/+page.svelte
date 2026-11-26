<script lang="ts">
	import { api } from '$lib/api';
	import { onMount } from 'svelte';

	let providers: string[] = $state([]);
	let defaultProvider = $state('');
	let cacheStats: { questions_cached: number; hints_cached: number } | null = $state(null);
	let loading = $state(true);
	let switching = $state(false);
	let message = $state('');

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
			message = 'Ошибка загрузки провайдеров';
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
			message = `Переключено на ${provider}`;
			setTimeout(() => (message = ''), 3000);
		} catch (error) {
			message = 'Ошибка переключения';
		} finally {
			switching = false;
		}
	}

	async function clearCache() {
		try {
			await api.clearAiCache();
			message = 'Кэш очищен';
			cacheStats = { questions_cached: 0, hints_cached: 0 };
			setTimeout(() => (message = ''), 3000);
		} catch (error) {
			message = 'Ошибка очистки кэша';
		}
	}

	function getProviderInfo(name: string): { description: string } {
		const info: Record<string, any> = {
			openrouter: { description: 'OpenRouter — доступ к GPT-4o-mini, Claude и другим моделям' },
			gigachat: { description: 'Sber GigaChat — оптимизирован для русского языка' },
			openai: { description: 'OpenAI GPT-4 — высочайшее качество генерации' },
			ollama: { description: 'Ollama — локальная модель, бесплатно и приватно' },
			anthropic: { description: 'Anthropic Claude — отличное качество и безопасность' },
		};
		return info[name] || { description: name };
	}
</script>

<div class="mx-auto max-w-2xl">
	<div class="mb-8">
		<h1 class="text-2xl font-bold text-white">Настройки AI</h1>
		<p class="mt-2 text-sm text-slate-400">Управление AI-провайдерами</p>
	</div>

	{#if loading}
		<div class="space-y-3">
			{#each Array(3) as _}
				<div class="h-20 animate-pulse rounded-xl bg-white/5"></div>
			{/each}
		</div>
	{:else}
		<!-- Providers -->
		<div class="mb-6">
			<h2 class="mb-3 text-sm font-medium text-slate-300">Провайдеры</h2>

			{#if providers.length === 0}
				<div class="rounded-xl border border-white/5 bg-white/[0.02] p-6 text-center">
					<p class="text-slate-400">Нет доступных провайдеров</p>
					<p class="mt-1 text-sm text-slate-500">Проверьте конфигурацию в .env</p>
				</div>
			{:else}
				<div class="space-y-2">
					{#each providers as provider}
						{@const info = getProviderInfo(provider)}
						{@const isActive = provider === defaultProvider}

						<button
							onclick={() => switchProvider(provider)}
							disabled={switching || isActive}
							class="w-full rounded-xl border p-4 text-left transition {
								isActive
									? 'border-white/20 bg-white/[0.05]'
									: 'border-white/5 bg-white/[0.02] hover:border-white/20'
							} {switching && !isActive ? 'cursor-wait opacity-50' : ''}"
						>
							<div class="flex items-center justify-between">
								<div class="min-w-0 flex-1">
									<div class="flex items-center gap-2">
										<span class="font-medium text-white">{provider}</span>
									</div>
									<p class="mt-1 text-sm text-slate-400">{info.description}</p>
								</div>
								{#if isActive}
									<div class="flex items-center gap-2 rounded-full bg-green-500/10 border border-green-500/20 px-2.5 py-1">
										<svg class="h-3.5 w-3.5 text-green-400" fill="currentColor" viewBox="0 0 20 20">
											<path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd" />
										</svg>
										<span class="text-xs font-medium text-green-400">Активен</span>
									</div>
								{/if}
							</div>
						</button>
					{/each}
				</div>
			{/if}

			{#if message}
				<div class="mt-3 rounded-lg border border-green-500/20 bg-green-500/10 p-3 text-sm text-green-400">
					{message}
				</div>
			{/if}
		</div>

		<!-- Cache -->
		<div class="rounded-xl border border-white/5 bg-white/[0.02] p-4">
			<div class="flex items-center justify-between">
				<div>
					<h3 class="text-sm font-medium text-slate-300">Кэш</h3>
					<p class="mt-1 text-xs text-slate-500">
						{cacheStats?.questions_cached ?? 0} вопросов, {cacheStats?.hints_cached ?? 0} подсказок
					</p>
				</div>
				<button
					onclick={clearCache}
					class="rounded-lg border border-white/10 bg-white/5 px-3 py-1.5 text-sm text-slate-300 transition hover:border-red-500/30 hover:bg-red-500/10 hover:text-red-400"
				>
					Очистить
				</button>
			</div>
		</div>
	{/if}
</div>
