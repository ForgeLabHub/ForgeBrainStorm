<script lang="ts">
	let visible = $state(false);

	$: if (typeof window !== 'undefined') {
		setTimeout(() => (visible = true), 100);
	}
</script>

<div class="relative overflow-hidden">
	<!-- Hero Section -->
	<div class="flex min-h-[70vh] flex-col items-center justify-center text-center px-4">
		<!-- Glow effect -->
		<div class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[600px] h-[600px] bg-gradient-to-r from-blue-500/10 via-purple-500/10 to-cyan-500/10 rounded-full blur-3xl"></div>

		<div class="relative z-10 transition-all duration-1000 {visible ? 'opacity-100 translate-y-0' : 'opacity-0 translate-y-8'}">
			<!-- Badge -->
			<div class="mb-6 inline-flex items-center gap-2 rounded-full border border-white/10 bg-white/5 px-4 py-1.5 text-sm text-slate-300 backdrop-blur-sm">
				<span class="flex h-2 w-2 rounded-full bg-green-500 animate-pulse"></span>
				AI-powered quiz platform
			</div>

			<h1 class="mb-6 text-5xl sm:text-7xl font-black tracking-tight">
				<span class="bg-gradient-to-r from-blue-400 via-purple-400 to-cyan-400 bg-clip-text text-transparent">
					BrainStorm
				</span>
			</h1>

			<p class="mb-10 max-w-2xl text-lg sm:text-xl text-slate-400 leading-relaxed mx-auto">
				Интерактивная викторина с <span class="text-purple-400 font-semibold">AI-генерацией</span> вопросов.
				Создайте комнату, пригласите друзей и проверьте свои знания!
			</p>

			<div class="flex flex-col sm:flex-row gap-4 justify-center">
				<a href="/lobby" class="relative group">
					<div class="absolute inset-0 bg-gradient-to-r from-blue-500 to-purple-600 rounded-2xl blur-lg opacity-50 group-hover:opacity-80 transition-opacity"></div>
					<div class="relative flex items-center gap-3 rounded-2xl bg-gradient-to-r from-blue-500 to-purple-600 px-8 py-4 text-lg font-semibold text-white shadow-2xl">
						<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z"></path><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path></svg>
						Начать игру
					</div>
				</a>
				<a href="/auth/register" class="flex items-center justify-center gap-3 rounded-2xl border border-white/10 bg-white/5 px-8 py-4 text-lg font-semibold text-white backdrop-blur-sm hover:bg-white/10 hover:border-white/20 transition-all">
					<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M18 9v3m0 0v3m0-3h3m-3 0h-3m-2-5a4 4 0 11-8 0 4 4 0 018 0zM3 20a6 6 0 0112 0v1H3v-1z"></path></svg>
					Регистрация
				</a>
			</div>
		</div>
	</div>

	<!-- Features Grid -->
	<div class="relative z-10 mx-auto max-w-6xl px-4 pb-20">
		<div class="grid grid-cols-1 md:grid-cols-3 gap-6">
			<!-- Feature 1 -->
			<div class="group relative overflow-hidden rounded-2xl border border-white/5 bg-white/[0.02] p-8 backdrop-blur-sm hover:border-blue-500/30 transition-all duration-300">
				<div class="absolute inset-0 bg-gradient-to-br from-blue-500/5 to-transparent opacity-0 group-hover:opacity-100 transition-opacity"></div>
				<div class="relative z-10">
					<div class="mb-4 flex h-12 w-12 items-center justify-center rounded-xl bg-blue-500/10 text-2xl">
						🎯
					</div>
					<h3 class="mb-2 text-xl font-bold text-white">6 режимов игры</h3>
					<p class="text-slate-400 text-sm leading-relaxed">
						Classic, FFA, Team, Lives, Coop, Своя Игра — каждый найдёт свой стиль
					</p>
				</div>
			</div>

			<!-- Feature 2 -->
			<div class="group relative overflow-hidden rounded-2xl border border-white/5 bg-white/[0.02] p-8 backdrop-blur-sm hover:border-purple-500/30 transition-all duration-300">
				<div class="absolute inset-0 bg-gradient-to-br from-purple-500/5 to-transparent opacity-0 group-hover:opacity-100 transition-opacity"></div>
				<div class="relative z-10">
					<div class="mb-4 flex h-12 w-12 items-center justify-center rounded-xl bg-purple-500/10 text-2xl">
						🤖
					</div>
					<h3 class="mb-2 text-xl font-bold text-white">AI-вопросы</h3>
					<p class="text-slate-400 text-sm leading-relaxed">
						GPT-4o-mini генерирует уникальные вопросы на любую тему в реальном времени
					</p>
				</div>
			</div>

			<!-- Feature 3 -->
			<div class="group relative overflow-hidden rounded-2xl border border-white/5 bg-white/[0.02] p-8 backdrop-blur-sm hover:border-cyan-500/30 transition-all duration-300">
				<div class="absolute inset-0 bg-gradient-to-br from-cyan-500/5 to-transparent opacity-0 group-hover:opacity-100 transition-opacity"></div>
				<div class="relative z-10">
					<div class="mb-4 flex h-12 w-12 items-center justify-center rounded-xl bg-cyan-500/10 text-2xl">
						🏆
					</div>
					<h3 class="mb-2 text-xl font-bold text-white">Лидерборд</h3>
					<p class="text-slate-400 text-sm leading-relaxed">
						Соревнуйтесь, зарабатывайте XP и монеты, поднимайтесь в рейтинге
					</p>
				</div>
			</div>
		</div>

		<!-- Stats row -->
		<div class="mt-12 grid grid-cols-2 md:grid-cols-4 gap-4">
			<div class="rounded-xl border border-white/5 bg-white/[0.02] p-5 text-center backdrop-blur-sm">
				<div class="text-2xl font-bold bg-gradient-to-r from-blue-400 to-cyan-400 bg-clip-text text-transparent">Rust</div>
				<div class="text-xs text-slate-500 mt-1">Backend</div>
			</div>
			<div class="rounded-xl border border-white/5 bg-white/[0.02] p-5 text-center backdrop-blur-sm">
				<div class="text-2xl font-bold bg-gradient-to-r from-purple-400 to-pink-400 bg-clip-text text-transparent">SvelteKit</div>
				<div class="text-xs text-slate-500 mt-1">Frontend</div>
			</div>
			<div class="rounded-xl border border-white/5 bg-white/[0.02] p-5 text-center backdrop-blur-sm">
				<div class="text-2xl font-bold bg-gradient-to-r from-green-400 to-emerald-400 bg-clip-text text-transparent">WebSocket</div>
				<div class="text-xs text-slate-500 mt-1">Real-time</div>
			</div>
			<div class="rounded-xl border border-white/5 bg-white/[0.02] p-5 text-center backdrop-blur-sm">
				<div class="text-2xl font-bold bg-gradient-to-r from-orange-400 to-red-400 bg-clip-text text-transparent">PostgreSQL</div>
				<div class="text-xs text-slate-500 mt-1">Database</div>
			</div>
		</div>
	</div>
</div>
