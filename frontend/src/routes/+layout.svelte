<script lang="ts">
	import '../app.css';
</script>

<div class="min-h-screen bg-slate-900 text-slate-50">
	<header class="border-b border-slate-700 px-6 py-4">
		<nav class="mx-auto flex max-w-6xl items-center justify-between">
			<a href="/" class="text-2xl font-bold text-blue-400">
				🧠 BrainStorm
			</a>
			<div class="flex items-center gap-4">
				<a href="/lobby" class="text-slate-300 hover:text-white">Комнаты</a>
				<a href="/profile" class="text-slate-300 hover:text-white">Профиль</a>
				<a href="/auth/login" class="rounded-lg bg-blue-600 px-4 py-2 text-white hover:bg-blue-700">
					Войти
				</a>
			</div>
		</nav>
	</header>

	<main class="mx-auto max-w-6xl px-6 py-8">
		<slot />
	</main>

	<footer class="border-t border-slate-700 px-6 py-4 text-center text-slate-400">
		<p>BrainStorm — интеллектуальная викторина с AI</p>
	</footer>
</div>
