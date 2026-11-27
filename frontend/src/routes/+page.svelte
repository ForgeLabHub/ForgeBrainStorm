<script lang="ts">
import { onMount } from 'svelte';

let visible = $state(false);

onMount(() => {
const timer = setTimeout(() => (visible = true), 100);
return () => clearTimeout(timer);
});

const features = [
{
icon: '🎯',
title: '10 режимов игры',
text: 'Классика, блиц, команды, выживание, кооператив, «Своя игра», марафон и другие',
accent: 'hover:border-blue-500/30'
},
{
icon: '👥',
title: 'Игра в компании',
text: 'До 30 человек в комнате, общий счёт в командах и кооперативе, объявления ведущего',
accent: 'hover:border-purple-500/30'
},
{
icon: '🏆',
title: 'Рейтинг',
text: 'Соревнуйтесь, зарабатывайте XP и монеты, поднимайтесь в таблице лидеров',
accent: 'hover:border-cyan-500/30'
}
];
</script>

<div class="relative overflow-hidden">
<div class="flex min-h-[70vh] flex-col items-center justify-center px-4 text-center">
<div
class="absolute left-1/2 top-1/2 h-[600px] w-[600px] -translate-x-1/2 -translate-y-1/2 rounded-full bg-gradient-to-r from-blue-500/10 via-purple-500/10 to-cyan-500/10 blur-3xl"
></div>

<div
class="relative z-10 transition-all duration-1000 {visible
? 'translate-y-0 opacity-100'
: 'translate-y-8 opacity-0'}"
>
<div
class="mb-6 inline-flex items-center gap-2 rounded-full border border-white/10 bg-white/5 px-4 py-1.5 text-sm text-slate-300 backdrop-blur-sm"
>
<span class="flex h-2 w-2 rounded-full bg-green-500 animate-pulse"></span>
Викторина для компании
</div>

<h1 class="mb-6 text-5xl font-black tracking-tight sm:text-7xl">
<span class="bg-gradient-to-r from-blue-400 via-purple-400 to-cyan-400 bg-clip-text text-transparent">
BrainStorm
</span>
</h1>

<p class="mx-auto mb-10 max-w-2xl text-lg leading-relaxed text-slate-400 sm:text-xl">
Создайте комнату, отправьте друзьям код и играйте
в <span class="font-semibold text-purple-400">10 режимов</span> — от блица до «Своей игры».
</p>

<div class="flex flex-col justify-center gap-4 sm:flex-row">
<a href="/lobby" class="group relative">
<div
class="absolute inset-0 rounded-2xl bg-gradient-to-r from-blue-500 to-purple-600 opacity-50 blur-lg transition-opacity group-hover:opacity-80"
></div>
<div
class="relative flex items-center gap-3 rounded-2xl bg-gradient-to-r from-blue-500 to-purple-600 px-8 py-4 text-lg font-semibold text-white shadow-2xl"
>
<svg class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"
><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z"></path><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path></svg>
Начать игру
</div>
</a>
<a
href="/auth/register"
class="flex items-center justify-center gap-3 rounded-2xl border border-white/10 bg-white/5 px-8 py-4 text-lg font-semibold text-white backdrop-blur-sm transition-all hover:border-white/20 hover:bg-white/10"
>
<svg class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M18 9v3m0 0v3m0-3h3m-3 0h-3m-2-5a4 4 0 11-8 0 4 4 0 018 0zM3 20a6 6 0 0112 0v1H3v-1z"></path></svg>
Регистрация
</a>
</div>
</div>
</div>

<div class="relative z-10 mx-auto max-w-6xl px-4 pb-20">
<div class="grid grid-cols-1 gap-6 md:grid-cols-3">
{#each features as feature (feature.title)}
<div
class="group relative overflow-hidden rounded-2xl border border-white/5 bg-white/[0.02] p-8 backdrop-blur-sm transition-all duration-300 {feature.accent}"
>
<div class="relative z-10">
<div class="mb-4 flex h-12 w-12 items-center justify-center rounded-xl bg-white/5 text-2xl">
{feature.icon}
</div>
<h3 class="mb-2 text-xl font-bold text-white">{feature.title}</h3>
<p class="text-sm leading-relaxed text-slate-400">{feature.text}</p>
</div>
</div>
{/each}
</div>
</div>
</div>
