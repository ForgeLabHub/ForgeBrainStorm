<script lang="ts">
	import Icon from '$lib/components/Icon.svelte';
	import { currentUser } from '$lib/stores/auth';

	const letters = ['A', 'B', 'C', 'D'];

	const steps = [
		{ title: 'Создайте комнату', text: 'Тема, сложность и число вопросов — три клика.' },
		{ title: 'Отправьте код', text: 'Друзья открывают сайт и вводят шесть символов.' },
		{ title: 'Отвечайте первыми', text: 'Очки начисляются за правильный ответ на скорость.' }
	];

	const facts = [
		{ icon: 'bolt', title: 'Вопросы по вашей теме', text: 'AI придумывает их за секунды, без повторов из сборников.' },
		{ icon: 'users', title: 'До 30 человек в комнате', text: 'Счёт и таймер считает сервер, поэтому всем честно.' },
		{ icon: 'trophy', title: 'Прогресс остаётся', text: 'Опыт, монеты и место в общем рейтинге — после каждой игры.' }
	];

	const demoOptions = ['Меркурий', 'Венера', 'Земля', 'Марс'];
</script>

<section class="grid gap-10 pb-4 pt-2 lg:grid-cols-[1.05fr_0.95fr] lg:items-center lg:gap-14 lg:pb-10 lg:pt-6">
	<div>
		<h1 class="font-display text-[30px] leading-[1.1] tracking-tight text-chalk sm:text-[38px] lg:text-[42px]">
			Викторина с вопросами от AI
		</h1>
		<p class="mt-4 max-w-[48ch] text-base leading-relaxed text-dim sm:text-lg">
			Соберите друзей в одной комнате, отвечайте на скорость и набирайте очки. Тему задаёте вы —
			остальное берёт на себя генератор вопросов.
		</p>

		<div class="mt-7 flex flex-col gap-3 sm:flex-row">
			<a href="/room/create" class="btn btn-primary flex-1 sm:flex-none sm:px-6">
				<Icon name="play" size={16} />
				Создать комнату
			</a>
			<a href="/lobby" class="btn btn-quiet flex-1 sm:flex-none sm:px-6">Войти по коду</a>
		</div>

		<p class="mt-4 text-sm text-dim">
			{#if $currentUser}
				Вы вошли как <span class="text-chalk">{$currentUser.username}</span>.
			{:else}
				Играть можно без установки — прямо в браузере.
			{/if}
		</p>
	</div>

	<div class="card card-pad animate-rise lg:order-none">
		<div class="mb-3 flex items-center justify-between">
			<span class="chip">Вопрос 3 из 10</span>
			<span class="digits text-lg text-amber">18 с</span>
		</div>

		<div class="mb-5 h-1.5 overflow-hidden rounded-full bg-ink-700">
			<div class="h-full w-3/5 rounded-full bg-amber"></div>
		</div>

		<p class="mb-4 font-display text-[19px] leading-snug text-chalk">
			Какая планета ближе всего к Солнцу?
		</p>

		<div class="grid grid-cols-1 gap-2">
			{#each demoOptions as option, i}
				<div
					class="flex items-center gap-3 rounded-[14px] border px-3 py-2.5 {i === 0
						? 'border-amber bg-ink-800'
						: 'border-line bg-ink-900'}"
				>
					<span
						class="grid h-7 w-7 shrink-0 place-items-center rounded-lg bg-ink-700 text-xs font-semibold text-dim"
					>
						{letters[i]}
					</span>
					<span class="text-sm text-chalk">{option}</span>
				</div>
			{/each}
		</div>

		<p class="mt-4 text-xs leading-relaxed text-dim">
			Экран вопроса: таймер, четыре варианта и счёт остальных игроков.
		</p>
	</div>
</section>

<section class="mt-12 border-t border-line/60 pt-8 sm:mt-16">
	<h2 class="section-title">Как проходит игра</h2>
	<ol class="mt-5 grid gap-6 sm:grid-cols-3 sm:gap-8">
		{#each steps as step, i}
			<li class="flex gap-4 sm:block">
				<span class="digits shrink-0 text-2xl text-amber">{i + 1}</span>
				<div class="sm:mt-3">
					<h3 class="font-semibold text-chalk">{step.title}</h3>
					<p class="mt-1 text-sm leading-relaxed text-dim">{step.text}</p>
				</div>
			</li>
		{/each}
	</ol>
</section>

<section class="mt-10 border-t border-line/60 pt-8">
	<h2 class="section-title">Что внутри</h2>
	<ul class="mt-4 divide-y divide-line/60">
		{#each facts as fact}
			<li class="flex items-start gap-4 py-4">
				<span class="grid h-10 w-10 shrink-0 place-items-center rounded-[12px] border border-line bg-ink-850 text-dim">
					<Icon name={fact.icon} size={19} />
				</span>
				<div>
					<h3 class="font-medium text-chalk">{fact.title}</h3>
					<p class="mt-0.5 max-w-[60ch] text-sm leading-relaxed text-dim">{fact.text}</p>
				</div>
			</li>
		{/each}
	</ul>
</section>
