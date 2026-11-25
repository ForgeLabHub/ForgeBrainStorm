<script lang="ts">
	import { api, type Room } from '$lib/api';
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import Icon from '$lib/components/Icon.svelte';
	import { currentUser } from '$lib/stores/auth';
	import { modeLabel } from '$lib/ui';

	let rooms: Room[] = [];
	let loading = true;
	let loadError = '';
	let joinCode = '';
	let joinError = '';
	let joining = false;

	onMount(() => {
		loadRooms();
		const interval = setInterval(() => {
			if (!document.hidden) loadRooms();
		}, 5000);
		return () => clearInterval(interval);
	});

	async function loadRooms() {
		try {
			const response = await api.listRooms();
			rooms = response.rooms || [];
			loadError = '';
		} catch {
			loadError = 'Не удалось загрузить список комнат';
		} finally {
			loading = false;
		}
	}

	function normalizeCode(value: string) {
		joinCode = value.replace(/[^a-zA-Z0-9]/g, '').toUpperCase().slice(0, 6);
	}

	async function joinRoom() {
		const code = joinCode.trim().toUpperCase();
		if (code.length !== 6) {
			joinError = 'В коде ровно 6 символов';
			return;
		}
		if (!$currentUser) {
			goto('/auth/login');
			return;
		}

		joining = true;
		joinError = '';
		try {
			const response = await api.getRoom(code);
			if (response.ok && response.room) {
				goto(`/room/${code}`);
			} else {
				joinError = 'Комнаты с таким кодом нет';
			}
		} catch {
			joinError = 'Комнаты с таким кодом нет';
		} finally {
			joining = false;
		}
	}
</script>

<div class="mb-6 flex items-start justify-between gap-4">
	<div>
		<h1 class="font-display text-[26px] tracking-tight text-chalk sm:text-3xl">Комнаты</h1>
		<p class="mt-1 text-sm text-dim">Зайдите по коду или начните свою игру.</p>
	</div>
	<a href="/room/create" class="btn btn-primary shrink-0 px-4 text-sm sm:px-5">
		<Icon name="plus" size={16} />
		<span class="hidden sm:inline">Создать комнату</span>
		<span class="sm:hidden">Создать</span>
	</a>
</div>

{#if !$currentUser}
	<p class="mb-5 flex flex-wrap items-center gap-x-2 gap-y-1 rounded-[14px] border border-amber/30 bg-amber/10 px-4 py-3 text-sm text-chalk">
		Чтобы играть, нужен аккаунт.
		<a href="/auth/login" class="font-semibold underline decoration-amber/60 underline-offset-4">Войти</a>
		<span class="text-dim">или</span>
		<a href="/auth/register" class="font-semibold underline decoration-amber/60 underline-offset-4">зарегистрироваться</a>
	</p>
{/if}

<section class="card card-pad mb-6">
	<h2 class="section-title mb-1">Войти по коду</h2>
	<p class="mb-4 text-sm text-dim">Код комнаты покажет хост — шесть букв или цифр.</p>

	<form on:submit|preventDefault={joinRoom} class="flex flex-col gap-2.5 sm:flex-row">
		<label class="sr-only" for="room-code">Код комнаты</label>
		<input
			id="room-code"
			type="text"
			inputmode="text"
			autocapitalize="characters"
			autocomplete="off"
			spellcheck="false"
			maxlength="6"
			value={joinCode}
			on:input={(event) => normalizeCode(event.currentTarget.value)}
			placeholder="ABC123"
			class="field flex-1 text-center font-display text-lg tracking-[0.28em] uppercase placeholder:tracking-[0.28em]"
		/>
		<button
			type="submit"
			disabled={joining || joinCode.length !== 6}
			class="btn sm:w-36 {joinCode.length === 6 ? 'btn-primary' : 'btn-quiet opacity-60'}"
		>
			{joining ? 'Проверяем…' : 'Войти'}
		</button>
	</form>

	{#if joinError}
		<p class="mt-2.5 text-sm text-coral" role="alert">{joinError}</p>
	{/if}
</section>

<section>
	<div class="mb-3 flex items-baseline justify-between">
		<h2 class="section-title">Открытые комнаты</h2>
		<span class="text-sm text-dim">{rooms.length}</span>
	</div>

	{#if loading}
		<div class="space-y-2">
			{#each [0, 1, 2] as row}
				<div class="h-[74px] animate-pulse rounded-[16px] border border-line/50 bg-ink-850" class:opacity-60={row > 0}></div>
			{/each}
		</div>
	{:else if loadError}
		<p class="rounded-[14px] border border-coral/30 bg-coral/10 px-4 py-3 text-sm text-coral">{loadError}</p>
	{:else if rooms.length === 0}
		<div class="card card-pad text-center">
			<p class="font-medium text-chalk">Пока никто не играет</p>
			<p class="mx-auto mt-1 max-w-[38ch] text-sm text-dim">
				Создайте комнату — она сразу появится здесь, и друзьям будет достаточно ввести код.
			</p>
			<a href="/room/create" class="btn btn-primary mt-4 w-full sm:w-auto sm:px-6">Создать комнату</a>
		</div>
	{:else}
		<ul class="space-y-2">
			{#each rooms as room (room.code)}
				<li>
					<a
						href="/room/{room.code}"
						class="flex items-center gap-3 rounded-[16px] border border-line/70 bg-ink-850 px-3.5 py-3 transition-colors hover:border-line hover:bg-ink-800 sm:gap-4 sm:px-4"
					>
						<span class="digits shrink-0 rounded-[10px] bg-ink-900 px-2.5 py-1.5 text-[12px] tracking-[0.12em] text-dim">
							{room.code}
						</span>

						<div class="min-w-0 flex-1">
							<p class="truncate font-medium text-chalk">{room.topic || 'Без темы'}</p>
							<div class="mt-1 flex flex-wrap items-center gap-1.5">
								<span class="chip">{modeLabel(room.mode)}</span>
								<span class="text-xs text-dim">Хост {room.host}</span>
							</div>
						</div>

						<span class="flex shrink-0 items-center gap-1.5 text-sm text-dim">
							<Icon name="users" size={16} />
							{room.player_count ?? 0}
						</span>
					</a>
				</li>
			{/each}
		</ul>
	{/if}
</section>
