<script lang="ts">
	// Landing page
</script>

<div class="flex min-h-[60vh] flex-col items-center justify-center text-center">
	<h1 class="mb-6 text-6xl font-bold text-blue-400">
		🧠 BrainStorm
	</h1>
	<p class="mb-8 max-w-2xl text-xl text-slate-300">
		Интерактивная викторина с AI-генерацией вопросов.
		Создайте комнату, пригласите друзей и проверьте свои знания!
	</p>

	<div class="flex gap-4">
		<a
			href="/lobby"
			class="rounded-lg bg-blue-600 px-8 py-4 text-lg font-semibold text-white transition hover:bg-blue-700"
		>
			Создать комнату
		</a>
		<a
			href="/auth/register"
			class="rounded-lg border-2 border-blue-600 px-8 py-4 text-lg font-semibold text-blue-400 transition hover:bg-blue-600 hover:text-white"
		>
			Регистрация
		</a>
	</div>

	<div class="mt-16 grid grid-cols-1 gap-8 md:grid-cols-3">
		<div class="rounded-lg bg-slate-800 p-6">
			<h3 class="mb-2 text-xl font-semibold text-purple-400">🎯 6 режимов игры</h3>
			<p class="text-slate-400">Classic, FFA, Team, Lives, Coop, Своя Игра</p>
		</div>
		<div class="rounded-lg bg-slate-800 p-6">
			<h3 class="mb-2 text-xl font-semibold text-purple-400">🤖 AI-вопросы</h3>
				<p class="text-slate-400">AI-провайдер генерирует уникальные вопросы на любую тему</p>
		</div>
		<div class="rounded-lg bg-slate-800 p-6">
			<h3 class="mb-2 text-xl font-semibold text-purple-400">🏆 Лидерборд</h3>
			<p class="text-slate-400">Соревнуйтесь с другими игроками, зарабатывайте XP и монеты</p>
		</div>
	</div>
</div>
