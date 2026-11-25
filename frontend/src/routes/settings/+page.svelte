<script lang="ts">
	import { api } from '$lib/api';
	import { onMount } from 'svelte';

	let providers: string[] = [];
	let defaultProvider = '';
	let loading = true;
	let switching = false;
	let message = '';

	onMount(async () => {
		await loadProviders();
	});

	async function loadProviders() {
		try {
			const response = await api.getAiProviders();
			providers = response.providers || [];
			defaultProvider = response.default || '';
		} catch (error) {
			console.error('Failed to load AI providers:', error);
			message = 'Ошибка загрузки списка провайдеров';
		} finally {
			loading = false;
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
			console.error('Failed to switch provider:', error);
			message = 'Ошибка переключения провайдера';
		} finally {
			switching = false;
		}
	}
</script>

<div class="mx-auto max-w-4xl px-6 py-8">
	<div class="mb-8">
		<h1 class="mb-2 text-3xl font-bold text-white">Настройки AI</h1>
		<p class="text-slate-400">Управление AI-провайдерами для генерации вопросов</p>
	</div>

	{#if loading}
		<div class="flex items-center justify-center py-12">
			<div class="h-8 w-8 animate-spin rounded-full border-4 border-blue-500 border-t-transparent"></div>
		</div>
	{:else}
		<div class="rounded-xl border border-slate-700 bg-slate-800 p-6">
			<h2 class="mb-4 text-xl font-semibold text-white">Доступные провайдеры</h2>
			
			{#if providers.length === 0}
				<p class="text-slate-400">Нет доступных провайдеров. Проверьте конфигурацию.</p>
			{:else}
				<div class="space-y-3">
					{#each providers as provider}
						<button
							on:click={() => switchProvider(provider)}
							disabled={switching || provider === defaultProvider}
							class="w-full rounded-lg border p-4 text-left transition
								{provider === defaultProvider
								? 'border-blue-500 bg-blue-500/10'
								: 'border-slate-600 bg-slate-700 hover:border-blue-500 hover:bg-slate-600'}
								{switching ? 'cursor-wait opacity-50' : ''}"
						>
							<div class="flex items-center justify-between">
								<div>
									<div class="font-semibold text-white">
										{provider}
									</div>
									<div class="text-sm text-slate-400">
										{provider === 'gigachat' && 'Sber GigaChat (рекомендуется для русского языка)'}
										{provider === 'openai' && 'OpenAI GPT-4 (высокое качество)'}
										{provider === 'ollama' && 'Ollama (локальная модель, бесплатно)'}
										{provider === 'anthropic' && 'Anthropic Claude (отличное качество)'}
									</div>
								</div>
								{#if provider === defaultProvider}
									<div class="flex items-center gap-2 text-blue-400">
										<svg class="h-5 w-5" fill="currentColor" viewBox="0 0 20 20">
											<path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd"/>
										</svg>
										<span class="text-sm font-medium">По умолчанию</span>
									</div>
								{/if}
							</div>
						</button>
					{/each}
				</div>
			{/if}

			{#if message}
				<div class="mt-4 rounded-lg bg-blue-500/10 p-3 text-blue-400">
					{message}
				</div>
			{/if}
		</div>

		<div class="mt-6 rounded-xl border border-slate-700 bg-slate-800 p-6">
			<h2 class="mb-4 text-xl font-semibold text-white">Информация</h2>
			<div class="space-y-2 text-sm text-slate-400">
				<p>• Система автоматически переключается между провайдерами при недоступности</p>
				<p>• Ответы кэшируются на 1 час для экономии API-вызовов</p>
				<p>• Локальные модели (Ollama) бесплатны, но требуют установки</p>
				<p>• GigaChat оптимизирован для русского языка</p>
			</div>
		</div>
	{/if}
</div>
