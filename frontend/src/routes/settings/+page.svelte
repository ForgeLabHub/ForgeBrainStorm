<script lang="ts">
	import { onMount } from 'svelte';
	import { api } from '$lib/api';
	import { currentUser } from '$lib/stores/auth';
	import Icon from '$lib/components/Icon.svelte';
	import { plural } from '$lib/ui';

	const providerInfo: Record<string, { title: string; note: string }> = {
		openrouter: { title: 'OpenRouter', note: 'Один ключ — десятки моделей. Провайдер по умолчанию.' },
		openai: { title: 'OpenAI', note: 'GPT-4o mini, стабильное качество на английском.' },
		gigachat: { title: 'GigaChat', note: 'Сбер, хорошо работает с русским языком.' },
		ollama: { title: 'Ollama', note: 'Локальная модель, без оплаты и без ключа.' },
		anthropic: { title: 'Anthropic', note: 'Claude, сильный в длинных формулировках.' }
	};

	let providers: string[] = [];
	let defaultProvider = '';
	let cacheCount = 0;
	let loading = true;
	let switching = false;
	let message = '';

	onMount(async () => {
		await Promise.all([loadProviders(), loadCache()]);
		loading = false;
	});

	async function loadProviders() {
		try {
			const response = await api.getAiProviders();
			providers = response.providers || [];
			defaultProvider = response.default || '';
		} catch {
			message = 'Не удалось получить список провайдеров';
		}
	}

	async function loadCache() {
		try {
			const response = await api.getAiCacheStats();
			cacheCount = response.cache?.questions_cached ?? 0;
		} catch {
			cacheCount = 0;
		}
	}

	async function switchProvider(provider: string) {
		switching = true;
		message = '';
		try {
			await api.switchAiProvider(provider);
			defaultProvider = provider;
			message = `Теперь вопросы генерирует ${provider}`;
		} catch {
			message = 'Не удалось переключить провайдера';
		} finally {
			switching = false;
		}
	}

	async function clearCache() {
		try {
			await api.clearAiCache();
			cacheCount = 0;
			message = 'Кэш вопросов очищен';
		} catch {
			message = 'Не удалось очистить кэш';
		}
	}
</script>

<div class="mx-auto w-full max-w-2xl">
	<div class="mb-6">
		<h1 class="font-display text-[26px] tracking-tight text-chalk sm:text-3xl">Настройки AI</h1>
		<p class="mt-1 text-sm text-dim">Откуда берутся вопросы и как часто они обновляются.</p>
	</div>

	{#if loading}
		<div class="card card-pad">
			<div class="h-5 w-40 animate-pulse rounded bg-ink-800"></div>
		</div>
	{:else}
		<section class="card card-pad">
			<h2 class="section-title mb-1">Провайдер</h2>
			<p class="mb-4 text-sm text-dim">
				{#if providers.length > 0}
					Вопросы генерирует первый доступный провайдер. Переключение действует на новые комнаты.
				{:else}
					Ключи не настроены, поэтому вопросы берутся из встроенного набора. Игра работает, но вопросы повторяются.
				{/if}
			</p>

			{#if providers.length === 0}
				<div class="rounded-[14px] border border-line/70 bg-ink-900 px-4 py-4 text-sm leading-relaxed text-dim">
					<p class="mb-2 text-chalk">Как включить AI-вопросы</p>
					<ol class="list-decimal space-y-1 pl-4">
						<li>Получите ключ на openrouter.ai/keys</li>
						<li>Добавьте в <code class="rounded bg-ink-800 px-1.5 py-0.5 text-xs text-chalk">.env</code> строку
							<code class="rounded bg-ink-800 px-1.5 py-0.5 text-xs text-chalk">OPENROUTER_API_KEY=…</code></li>
						<li>Перезапустите сервер — провайдер появится в этом списке</li>
					</ol>
				</div>
			{:else}
				<ul class="space-y-1.5">
					{#each providers as provider}
						{@const info = providerInfo[provider]}
						<li>
							<button
								on:click={() => switchProvider(provider)}
								disabled={switching || provider === defaultProvider}
								class="flex w-full items-center gap-3 rounded-[14px] border px-3.5 py-3 text-left transition-colors {provider ===
								defaultProvider
									? 'border-amber bg-ink-800'
									: 'border-line hover:bg-ink-800'}"
							>
								<span class="grid h-9 w-9 shrink-0 place-items-center rounded-[10px] bg-ink-700 text-dim">
									<Icon name="bolt" size={17} />
								</span>
								<span class="min-w-0 flex-1">
									<span class="block text-sm font-medium text-chalk">{info?.title ?? provider}</span>
									<span class="block text-xs text-dim">{info?.note ?? 'Провайдер AI-вопросов'}</span>
								</span>
								{#if provider === defaultProvider}
									<span class="shrink-0 text-xs font-medium text-amber">Выбран</span>
								{/if}
							</button>
						</li>
					{/each}
				</ul>
			{/if}
		</section>

		<section class="card card-pad mt-4">
			<div class="flex items-center justify-between gap-4">
				<div>
					<h2 class="section-title">Кэш вопросов</h2>
					<p class="mt-1 text-sm text-dim">
						{cacheCount} {plural(cacheCount, 'вопрос', 'вопроса', 'вопросов')} сохранено — повторы не тратят запросы к AI.
					</p>
				</div>
				<button on:click={clearCache} class="btn btn-quiet shrink-0 px-4 text-sm">Очистить</button>
			</div>
		</section>

		{#if message}
			<p class="mt-4 rounded-[12px] border border-line/70 bg-ink-900 px-4 py-2.5 text-sm text-dim" role="status">
				{message}
			</p>
		{/if}

		{#if !$currentUser}
			<p class="mt-4 text-sm text-dim">
				<a href="/auth/login" class="underline decoration-line underline-offset-4 hover:decoration-amber">Войдите</a>,
				чтобы менять настройки от своего имени.
			</p>
		{/if}
	{/if}
</div>
