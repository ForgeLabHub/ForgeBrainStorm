<script lang="ts">
<<<<<<< HEAD
	import { onMount } from 'svelte';
	import { api } from '$lib/api';

	const catalog: Record<string, { title: string; description: string; mark: string; color: string }> = {
		openrouter: { title: 'OpenRouter', description: 'Доступ к разным моделям через единый шлюз.', mark: '↗', color: '#b5a2ff' },
		gigachat: { title: 'GigaChat', description: 'Генерация вопросов и подсказок на русском языке.', mark: 'G', color: '#86efc9' },
		openai: { title: 'OpenAI', description: 'Семейство моделей GPT для генерации контента.', mark: 'O', color: '#9edce7' },
		ollama: { title: 'Ollama', description: 'Модели, запущенные на вашем сервере.', mark: '◎', color: '#fbc68e' },
		anthropic: { title: 'Anthropic', description: 'Семейство моделей Claude.', mark: 'A', color: '#ffb6ad' }
	};
	const number = new Intl.NumberFormat('ru-RU');
	let providers: string[] = $state([]);
	let active = $state('');
	let stats: Awaited<ReturnType<typeof api.getAiCacheStats>>['cache'] | null = $state(null);
	let providersState = $state<'loading' | 'ready' | 'error'>('loading');
	let cacheState = $state<'loading' | 'ready' | 'error'>('loading');
	let switching = $state('');
	let clearing = $state(false);
	let confirmClear = $state(false);
	let providerNotice = $state<{ error: boolean; text: string } | null>(null);
	let cacheNotice = $state<{ error: boolean; text: string } | null>(null);

	onMount(() => { void loadProviders(); void loadCache(); });

	async function loadProviders() {
		providersState = 'loading';
		try {
			const response = await api.getAiProviders();
			providers = [...new Set(response.providers)];
			active = response.default;
			providersState = 'ready';
		} catch { providersState = 'error'; }
	}

	async function loadCache() {
		cacheState = 'loading';
		try {
			const result = await api.getAiCacheStats();
			if (!result.cache || !Number.isFinite(result.cache.questions_cached) || !Number.isFinite(result.cache.hints_cached)) throw new Error('Invalid cache statistics');
			stats = result.cache;
			cacheState = 'ready';
		} catch { cacheState = 'error'; }
	}

	async function switchProvider(provider: string) {
		if (switching || provider === active) return;
		switching = provider;
		providerNotice = null;
		try {
			await api.switchAiProvider(provider);
			active = provider;
			providerNotice = { error: false, text: `Провайдер изменён: ${catalog[provider]?.title ?? provider}.` };
		} catch {
			providerNotice = { error: true, text: 'Не удалось сменить провайдера. Попробуйте ещё раз.' };
		} finally { switching = ''; }
	}

	async function clearCache() {
		if (!confirmClear || clearing) return;
		clearing = true;
		cacheNotice = null;
		try {
			await api.clearAiCache();
			confirmClear = false;
			cacheNotice = { error: false, text: 'Кэш очищен.' };
			await loadCache();
		} catch {
			cacheNotice = { error: true, text: 'Не удалось очистить кэш. Попробуйте ещё раз.' };
		} finally { clearing = false; }
	}
</script>

<div class="settings">
	<nav class="crumb" aria-label="Хлебные крошки"><a href="/">BrainStorm</a><span aria-hidden="true">/</span><span aria-current="page">Настройки ИИ</span></nav>
	<header class="hero">
		<div class="hero-art" aria-hidden="true"><svg viewBox="0 0 320 230" fill="none"><defs><linearGradient id="signal" x2="1" y2="1" gradientUnits="objectBoundingBox"><stop stop-color="#bcaaff"/><stop offset="1" stop-color="#7762ea" stop-opacity=".16"/></linearGradient></defs><g stroke="url(#signal)"><circle cx="168" cy="116" r="96"/><circle cx="168" cy="116" r="72" stroke-dasharray="3 7"/><circle cx="168" cy="116" r="50"/><path d="M36 116h264M168 5v220M100 48l136 136m0-136L100 184" opacity=".45"/></g><path d="M168 76q5 35 40 40-35 5-40 40-5-35-40-40 35-5 40-40Z" fill="#c7b7ff"/><circle cx="238" cy="50" r="5" fill="#bcaaff"/><circle cx="96" cy="159" r="3" fill="#8ef0d0"/><circle cx="265" cy="116" r="3" fill="#bcaaff"/></svg></div>
		<div class="eyebrow"><span aria-hidden="true">✦</span> AI STUDIO</div>
		<h1>Настройки ИИ</h1>
		<p>Выберите, какой сервис создаёт вопросы и подсказки, и управляйте кэшем генерации.</p>
	</header>

	<div class="panels">
		<section class="panel" aria-labelledby="providers-heading" aria-busy={providersState === 'loading'}>
			<header class="section-head"><span class="section-icon" aria-hidden="true">✧</span><div><h2 id="providers-heading">Провайдеры</h2><p>Сервис генерации по умолчанию</p></div></header>
			{#if providersState === 'loading'}
				<p class="state" role="status">Загружаем провайдеров…</p>
			{:else if providersState === 'error'}
				<div class="state"><strong>Не удалось загрузить провайдеров</strong><p>Проверьте подключение к серверу.</p><button class="action" onclick={loadProviders}>Повторить</button></div>
			{:else if providers.length === 0}
				<div class="state"><strong>Нет доступных провайдеров</strong><p>Попросите администратора настроить сервис генерации.</p><button class="action" onclick={loadProviders}>Обновить список</button></div>
			{:else}
				<ul class="providers">
					{#each providers as provider (provider)}
						{@const info = catalog[provider]}
						<li><button class="provider" class:selected={active === provider} type="button" aria-pressed={active === provider} disabled={Boolean(switching) || active === provider} onclick={() => switchProvider(provider)}>
							<span class="provider-mark" style:color={info?.color ?? '#c7b7ff'} aria-hidden="true">{info?.mark ?? '✦'}</span>
							<span class="provider-text"><strong>{info?.title ?? provider}</strong><small>{info?.description ?? 'Провайдер генерации'}</small></span>
							<span class="badge">{active === provider ? 'Активен' : switching === provider ? 'Подключаем…' : 'Выбрать'}</span>
						</button></li>
					{/each}
				</ul>
			{/if}
			{#if providerNotice}<p class="notice" class:error={providerNotice.error} role={providerNotice.error ? 'alert' : 'status'}>{providerNotice.text}</p>{/if}
		</section>

		<section class="panel" aria-labelledby="cache-heading" aria-busy={cacheState === 'loading'}>
			<header class="section-head"><span class="section-icon mint" aria-hidden="true"><svg width="19" height="19" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6"><ellipse cx="12" cy="5" rx="8" ry="3"/><path d="M4 5v14c0 4 16 4 16 0V5M4 12c0 4 16 4 16 0"/></svg></span><div><h2 id="cache-heading">Кэш генерации</h2><p>Сохранённые результаты</p></div></header>
			<dl class="metrics">
				<div><dt>Вопросы</dt><dd>{cacheState === 'ready' && stats ? number.format(stats.questions_cached) : '—'}</dd></div>
				<div><dt>Подсказки</dt><dd>{cacheState === 'ready' && stats ? number.format(stats.hints_cached) : '—'}</dd></div>
			</dl>
			{#if cacheState === 'loading'}<p class="detail" role="status">Загружаем статистику…</p>
			{:else if cacheState === 'error'}<p class="detail">Статистика недоступна.</p><button class="action" onclick={loadCache} disabled={clearing}>Повторить</button>
			{:else if confirmClear}
				<div class="confirmation" role="group" aria-label="Подтверждение очистки"><p class="detail">Удалить сохранённые вопросы и подсказки из кэша? При следующем запросе они будут сгенерированы заново.</p><div class="actions"><button class="action danger" onclick={clearCache} disabled={clearing}>{clearing ? 'Очищаем…' : 'Да, очистить'}</button><button class="action" onclick={() => (confirmClear = false)} disabled={clearing}>Отмена</button></div></div>
			{:else}
				<p class="detail">Кэш ускоряет повторные запросы на генерацию.</p><button class="action" onclick={() => { confirmClear = true; cacheNotice = null; }} disabled={clearing || !stats || stats.questions_cached + stats.hints_cached === 0}>Очистить кэш</button>
			{/if}
			{#if cacheNotice}<p class="notice" class:error={cacheNotice.error} role={cacheNotice.error ? 'alert' : 'status'}>{cacheNotice.text}</p>{/if}
		</section>
	</div>
</div>

<style>
	.settings { max-width: 1080px; margin: 0 auto; color: #f3efff; }
	.crumb { display: flex; gap: .75rem; align-items: center; margin-bottom: 1rem; color: #a39ab8; font-size: .81rem; }
	.crumb a:hover { color: #fff; }
	.crumb [aria-current] { color: #ddd4ef; }
	.hero { isolation: isolate; position: relative; overflow: hidden; padding: clamp(1.4rem, 5vw, 3.6rem); border: 1px solid #413461; border-radius: 1.6rem; background: radial-gradient(ellipse at 88% 38%, #3d276c 0%, transparent 54%), linear-gradient(125deg,#1b1530,#141225); }
	.hero-art { position: absolute; inset: 0 2% 0 auto; width: 36%; z-index: -1; display: grid; place-items: center; pointer-events: none; }
	.hero-art svg { width: 100%; max-height: 100%; }
	.eyebrow { display: inline-flex; align-items: center; gap: .5rem; padding: .35rem .7rem; border: 1px solid #61488c; border-radius: 99px; background: #2d2048; color: #ddc9ff; letter-spacing: .15em; font-size: .68rem; font-weight: 750; }
	h1 { font-size: clamp(2.25rem, 6vw, 4.4rem); line-height: 1.04; letter-spacing: -.06em; font-weight: 800; margin: .9rem 0 .8rem; }
	.hero p { max-width: 34rem; color: #d1c5e4; line-height: 1.6; }
	.panels { display: grid; grid-template-columns: minmax(0,1.45fr) minmax(0,1fr); gap: 1rem; margin-top: 1rem; align-items: start; }
	.panel { min-width: 0; padding: clamp(1rem, 2.6vw, 1.65rem); border: 1px solid #312943; border-radius: 1.35rem; background: #151224; }
	.section-head { display: flex; align-items: center; gap: .85rem; margin-bottom: 1.2rem; }
	.section-icon,.provider-mark { flex: none; width: 2.6rem; height: 2.6rem; display: grid; place-items: center; border-radius: .8rem; background: #30234c; color: #d4bdff; font-size: 1.4rem; }
	.section-icon.mint { color: #8ef0d0; background: #1d3437; }
	h2 { font-size: 1.15rem; font-weight: 750; }
	.section-head p,.detail,.state p { color: #b5abc9; font-size: .86rem; line-height: 1.5; }
	.providers { display: grid; gap: .6rem; }
	.provider { display: flex; width: 100%; align-items: center; gap: .7rem; text-align: left; padding: .7rem; border: 1px solid #362d49; border-radius: .9rem; background: #1b172c; }
	.provider:not(:disabled):hover,.provider.selected { border-color: #8664ca; background: #29203f; }
	.provider-text { flex: 1; min-width: 0; overflow-wrap: anywhere; }
	.provider-text strong,.provider-text small { display: block; }
	.provider-text small { color: #beb3d1; line-height: 1.4; margin-top: .15rem; }
	.badge { flex: none; color: #ded0f8; font-size: .72rem; border-radius: 99px; padding: .28rem .55rem; background: #35294c; }
	.selected .badge { color: #a5f3db; background: #203e3c; }
	.metrics { display: grid; grid-template-columns: repeat(2, minmax(0,1fr)); gap: .65rem; }
	.metrics div,.state { padding: .9rem; border: 1px solid #352d47; border-radius: .85rem; background: #1c172c; }
	dt { color: #c5b9d8; font-size: .79rem; }
	dd { font-size: clamp(1.25rem, 3vw, 2rem); font-weight: 750; font-variant-numeric: tabular-nums; overflow-wrap: anywhere; }
	.detail,.state p { margin: .7rem 0; }
	.action { min-height: 2.45rem; padding: .4rem .75rem; border: 1px solid #57456f; border-radius: .65rem; font-size: .83rem; color: #eadfff; background: #2a203d; }
	.action:not(:disabled):hover { background: #422c61; }
	.action:disabled { opacity: .57; }
	.actions { display: flex; flex-wrap: wrap; gap: .45rem; }
	.danger { border-color: #935665; color: #ffd5dd; }
	.notice { margin-top: .75rem; padding: .65rem .8rem; border-radius: .65rem; color: #a5f3db; background: #203631; font-size: .85rem; }
	.notice.error { color: #ffd0d9; background: #3e2535; }
	@media (max-width: 720px) { .panels { grid-template-columns: 1fr; } .hero-art { opacity: .26; width: 52%; right: 0; } }
	@media (max-width: 390px) { .provider { flex-wrap: wrap; } .provider-text { flex-basis: 65%; } }
</style>
=======
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
>>>>>>> 4808142 (Merge pull request #15 from ForgeLabHub/arena/ca3aa14f-forgebrainstorm)
