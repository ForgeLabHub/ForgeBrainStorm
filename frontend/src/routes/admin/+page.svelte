<script lang="ts">
import {
api,
type AdminAuditEntry,
type AdminRoom,
type AdminRoomDetail,
type AdminRoomPlayer,
type AdminSession,
type AdminStats,
type AdminUser,
type Role
} from '$lib/api';
import { onMount } from 'svelte';

type Tab = 'overview' | 'users' | 'rooms' | 'sessions' | 'audit';

let loading = $state(true);
let denied = $state(false);
let tab = $state<Tab>('overview');
let notice = $state('');
let busy = $state(false);

let stats = $state<AdminStats | null>(null);
let users = $state<AdminUser[]>([]);
let rooms = $state<AdminRoom[]>([]);
let roomDetail = $state<AdminRoomDetail | null>(null);
let roomAnnounce = $state('');
let sessions = $state<AdminSession[]>([]);
let sessionFilter = $state('');
let roleFilter = $state<Role | ''>('');
let userPage = $state(0);
let audit = $state<AdminAuditEntry[]>([]);
let query = $state('');
let broadcastText = $state('');
let providers = $state<string[]>([]);
let activeProvider = $state('');
let cached = $state({ questions_cached: 0, hints_cached: 0 });

const ROLE_LABEL: Record<Role, string> = { player: 'Игрок', cheater: 'Читер', admin: 'Админ' };
const ROLE_STYLE: Record<Role, string> = {
player: 'bg-white/5 text-dim border-line',
cheater: 'bg-coral/10 text-coral border-coral/30',
admin: 'bg-amber/10 text-amber border-amber/30'
};

onMount(load);

async function load() {
loading = true;
try {
const me = await api.me();
if (me.user?.role !== 'admin') {
denied = true;
return;
}
await Promise.all([refreshStats(), refreshUsers(), refreshRooms(), refreshSessions(), refreshAudit(), refreshAi()]);
} catch {
denied = true;
} finally {
loading = false;
}
}

async function refreshStats() {
stats = await api.adminStats();
}
async function refreshUsers() {
users = await api.adminUsers(query, roleFilter || undefined, 50, userPage * 50);
}

async function refreshSessions() {
sessions = await api.adminSessions();
}

async function refreshRoomDetail() {
if (!roomDetail) return;
roomDetail = await api.adminRoomDetail(roomDetail.code);
}
async function refreshRooms() {
rooms = await api.adminRooms();
}
async function refreshAudit() {
audit = await api.adminAuditLog();
}
async function refreshAi() {
const status = await api.adminAiStatus();
providers = status.providers;
activeProvider = status.default;
cached = { questions_cached: status.questions_cached, hints_cached: status.hints_cached };
}

async function act(run: () => Promise<unknown>, message: string) {
busy = true;
notice = '';
try {
await run();
notice = message;
await Promise.all([
refreshStats(),
refreshUsers(),
refreshRooms(),
refreshSessions(),
refreshAudit(),
refreshRoomDetail()
]);
} catch (error) {
notice = error instanceof Error ? error.message : 'Не удалось выполнить действие';
} finally {
busy = false;
}
}

function setRole(user: AdminUser, role: Role) {
if (user.role === role) return;
act(() => api.adminSetRole(user.user_id, role), `${user.username}: роль «${ROLE_LABEL[role]}»`);
}

function ban(user: AdminUser, minutes: number) {
act(() => api.adminBan(String(user.user_id), minutes, 'нарушение правил'), `${user.username} заблокирован`);
}

function unban(user: AdminUser) {
act(() => api.adminUnban(String(user.user_id)), `${user.username} разблокирован`);
}

function reset(user: AdminUser) {
act(() => api.adminResetUser(user.user_id), `${user.username}: статистика обнулена`);
}

function remove(user: AdminUser) {
if (!confirm(`Удалить аккаунт ${user.username}? Это необратимо.`)) return;
act(() => api.adminDeleteUser(user.user_id), `${user.username} удалён`);
}

function closeRoom(room: AdminRoom) {
act(() => api.adminCloseRoom(room.code), `Комната ${room.code} закрыта`);
}

function sendBroadcast() {
if (!broadcastText.trim()) return;
act(() => api.adminBroadcast(broadcastText), 'Объявление отправлено');
broadcastText = '';
}

function switchProvider(name: string) {
act(async () => {
await api.adminSwitchProvider(name);
await refreshAi();
}, `Провайдер: ${name}`);
}

function clearCache() {
act(async () => {
await api.adminClearAiCache();
await refreshAi();
}, 'Кэш очищен');
}

function openRoom(room: AdminRoom) {
roomDetail = null;
act(async () => {
roomDetail = await api.adminRoomDetail(room.code);
}, `Комната ${room.code}: детали загружены`);
}

function kickPlayer(player: AdminRoomPlayer) {
if (!roomDetail) return;
act(
() =>
api.adminKick(
roomDetail!.code,
{ user_id: player.user_id ?? undefined, username: player.name },
'Удалён администратором'
),
`${player.name} удалён из комнаты`
);
}

function sendRoomAnnounce() {
if (!roomDetail || !roomAnnounce.trim()) return;
const text = roomAnnounce;
roomAnnounce = '';
act(() => api.adminRoomAnnounce(roomDetail!.code, text), 'Объявление отправлено в комнату');
}

function toggleChatLock() {
if (!roomDetail) return;
const next = !roomDetail.chat_locked;
act(() => api.adminChatLock(roomDetail!.code, next), next ? 'Чат заморожен' : 'Чат разморожен');
}

function clearRoomChat() {
if (!roomDetail) return;
act(() => api.adminChatClear(roomDetail!.code), 'Чат комнаты очищен');
}

function revokeSessions(userId: number, name: string) {
act(() => api.adminRevokeSessions(userId), `${name}: сессии завершены`);
}

function changeRoleFilter(value: string) {
roleFilter = value as Role | '';
userPage = 0;
refreshUsers();
}

function changeUserPage(delta: number) {
const next = Math.max(0, userPage + delta);
if (next === userPage) return;
userPage = next;
refreshUsers();
}

const visibleSessions = $derived(
sessionFilter.trim()
? sessions.filter((session) =>
session.username.toLowerCase().includes(sessionFilter.trim().toLowerCase())
)
: sessions
);

const stateLabel: Record<string, string> = {
waiting: 'Ожидание',
starting: 'Подготовка',
playing: 'Игра',
finished: 'Итоги'
};
</script>

{#if loading}
<div class="flex min-h-[40vh] items-center justify-center text-dim">Загрузка…</div>
{:else if denied}
<div class="mx-auto max-w-md rounded-xl2 border border-line bg-ink-900 p-6 text-center">
<div class="text-3xl">🔒</div>
<h1 class="mt-3 font-display text-xl">Только для администратора</h1>
<p class="mt-2 text-sm text-dim">Эта страница недоступна для вашей роли.</p>
<a href="/lobby" class="mt-5 inline-block rounded-lg bg-amber px-4 py-2 text-sm font-semibold text-ink-950">В комнаты</a>
</div>
{:else}
<div class="space-y-5">
<header class="flex flex-wrap items-end justify-between gap-3">
<div>
<p class="text-xs uppercase tracking-widest text-dim">Администрирование</p>
<h1 class="font-display text-2xl sm:text-3xl">Пульт управления</h1>
</div>
{#if notice}
<p class="rounded-lg border border-mint/30 bg-mint/10 px-3 py-2 text-sm text-mint">{notice}</p>
{/if}
</header>

<div class="-mx-1 flex gap-2 overflow-x-auto px-1 pb-1">
{#each [['overview', 'Обзор'], ['users', 'Игроки'], ['rooms', 'Комнаты'], ['sessions', 'Сессии'], ['audit', 'Журнал']] as const as [key, label]}
<button
class="shrink-0 rounded-lg border px-4 py-2 text-sm transition
{tab === key ? 'border-amber/40 bg-amber/10 text-amber' : 'border-line bg-ink-900 text-dim hover:text-chalk'}"
onclick={() => (tab = key)}
>{label}</button>
{/each}
</div>

{#if tab === 'overview' && stats}
<div class="grid grid-cols-2 gap-3 sm:grid-cols-3 lg:grid-cols-4">
{#each [
['Игроков', stats.users],
['Админов', stats.admins],
['Читеров', stats.cheaters],
['Партий', stats.games],
['Живых комнат', stats.live_rooms],
['В игре', stats.live_players],
['Активных банов', stats.active_bans],
['Живых сессий', stats.live_sessions],
['Очков всего', stats.total_score]
] as [label, value]}
<div class="rounded-xl2 border border-line bg-ink-900 p-4">
<p class="text-xs text-dim">{label}</p>
<p class="mt-1 font-display text-2xl">{value}</p>
</div>
{/each}
</div>

<section class="rounded-xl2 border border-line bg-ink-900 p-4 sm:p-5">
<h2 class="font-display text-lg">Объявление во все комнаты</h2>
<div class="mt-3 flex flex-col gap-2 sm:flex-row">
<input
bind:value={broadcastText}
maxlength="200"
placeholder="Например: через 5 минут турнир"
class="min-w-0 flex-1 rounded-lg border border-line bg-ink-850 px-3 py-2 text-sm outline-none focus:border-amber/50"
/>
<button
onclick={sendBroadcast}
disabled={busy}
class="rounded-lg bg-amber px-4 py-2 text-sm font-semibold text-ink-950 disabled:opacity-50"
>Отправить</button>
</div>
</section>

<section class="rounded-xl2 border border-line bg-ink-900 p-4 sm:p-5">
<h2 class="font-display text-lg">Генератор вопросов</h2>
<p class="mt-1 text-sm text-dim">Активный провайдер: <span class="text-chalk">{activeProvider || '—'}</span></p>
<div class="mt-3 flex flex-wrap gap-2">
{#each providers as provider}
<button
onclick={() => switchProvider(provider)}
disabled={busy || provider === activeProvider}
class="rounded-lg border px-3 py-2 text-sm transition
{provider === activeProvider ? 'border-mint/40 bg-mint/10 text-mint' : 'border-line bg-ink-850 text-dim hover:text-chalk'}"
>{provider}</button>
{/each}
<button onclick={clearCache} disabled={busy} class="rounded-lg border border-line bg-ink-850 px-3 py-2 text-sm text-dim hover:text-chalk">
Очистить кэш ({cached.questions_cached})
</button>
</div>
</section>
{/if}

{#if tab === 'users'}
<section class="space-y-3">
<div class="flex flex-col gap-2 sm:flex-row">
<input
bind:value={query}
placeholder="Поиск по имени"
class="min-w-0 flex-1 rounded-lg border border-line bg-ink-900 px-3 py-2 text-sm outline-none focus:border-amber/50"
/>
<select
value={roleFilter}
onchange={(event) => changeRoleFilter((event.currentTarget as HTMLSelectElement).value)}
class="rounded-lg border border-line bg-ink-850 px-3 py-2 text-sm"
>
<option value="">Все роли</option>
<option value="player">Игроки</option>
<option value="cheater">Читеры</option>
<option value="admin">Админы</option>
</select>
<button onclick={refreshUsers} class="rounded-lg border border-line bg-ink-850 px-4 py-2 text-sm text-dim hover:text-chalk">Найти</button>
</div>

<div class="flex items-center justify-between text-xs text-dim">
<span>Страница {userPage + 1} · по 50 на странице</span>
<div class="flex gap-2">
<button
onclick={() => changeUserPage(-1)}
disabled={busy || userPage === 0}
class="rounded-md border border-line px-2 py-1 disabled:opacity-40"
>Назад</button>
<button
onclick={() => changeUserPage(1)}
disabled={busy || users.length < 50}
class="rounded-md border border-line px-2 py-1 disabled:opacity-40"
>Вперёд</button>
</div>
</div>

<div class="space-y-2 sm:hidden">
{#each users as user}
<article class="rounded-xl2 border border-line bg-ink-900 p-4">
<div class="flex items-start justify-between gap-2">
<div class="min-w-0">
<p class="truncate font-medium">{user.username}</p>
<p class="text-xs text-dim">{user.games_played} партий · {user.wins} побед · {user.xp} XP</p>
</div>
<span class="shrink-0 rounded-md border px-2 py-0.5 text-xs {ROLE_STYLE[user.role]}">{ROLE_LABEL[user.role]}</span>
</div>
<div class="mt-3 flex flex-wrap gap-2 text-xs">
{#each ['player', 'cheater', 'admin'] as const as role}
<button onclick={() => setRole(user, role)} disabled={busy || user.role === role} class="rounded-md border border-line px-2 py-1 text-dim disabled:opacity-40">
{ROLE_LABEL[role]}
</button>
{/each}
{#if user.banned}
<button onclick={() => unban(user)} class="rounded-md border border-mint/40 px-2 py-1 text-mint">Разбан</button>
{:else}
<button onclick={() => ban(user, 60)} class="rounded-md border border-coral/40 px-2 py-1 text-coral">Бан 1ч</button>
{/if}
<button onclick={() => reset(user)} class="rounded-md border border-line px-2 py-1 text-dim">Сброс</button>
<button onclick={() => remove(user)} class="rounded-md border border-coral/30 px-2 py-1 text-coral">Удалить</button>
</div>
</article>
{/each}
</div>

<div class="hidden overflow-x-auto rounded-xl2 border border-line bg-ink-900 sm:block">
<table class="w-full text-left text-sm">
<thead class="border-b border-line text-xs uppercase tracking-wide text-dim">
<tr>
<th class="px-4 py-3">Игрок</th>
<th class="px-4 py-3">Роль</th>
<th class="px-4 py-3">Партии</th>
<th class="px-4 py-3">XP</th>
<th class="px-4 py-3">Действия</th>
</tr>
</thead>
<tbody>
{#each users as user}
<tr class="border-b border-line/60 last:border-0">
<td class="px-4 py-3">
<span class="font-medium">{user.username}</span>
{#if user.banned}<span class="ml-2 text-xs text-coral">забанен</span>{/if}
</td>
<td class="px-4 py-3">
<select
value={user.role}
disabled={busy}
onchange={(event) => setRole(user, (event.currentTarget as HTMLSelectElement).value as Role)}
class="rounded-md border border-line bg-ink-850 px-2 py-1 text-xs"
>
<option value="player">Игрок</option>
<option value="cheater">Читер</option>
<option value="admin">Админ</option>
</select>
</td>
<td class="px-4 py-3 text-dim">{user.games_played} · {user.wins} побед</td>
<td class="px-4 py-3 text-dim">{user.xp}</td>
<td class="px-4 py-3">
<div class="flex flex-wrap gap-2 text-xs">
{#if user.banned}
<button onclick={() => unban(user)} class="rounded-md border border-mint/40 px-2 py-1 text-mint">Разбан</button>
{:else}
<button onclick={() => ban(user, 60)} class="rounded-md border border-coral/40 px-2 py-1 text-coral">Бан 1ч</button>
{/if}
<button onclick={() => reset(user)} class="rounded-md border border-line px-2 py-1 text-dim">Сброс</button>
<button onclick={() => remove(user)} class="rounded-md border border-coral/30 px-2 py-1 text-coral">Удалить</button>
</div>
</td>
</tr>
{/each}
</tbody>
</table>
</div>
</section>
{/if}

{#if tab === 'rooms'}
{#if rooms.length === 0}
<p class="rounded-xl2 border border-line bg-ink-900 p-5 text-sm text-dim">Сейчас нет активных комнат.</p>
{:else}
<div class="grid gap-3 sm:grid-cols-2">
{#each rooms as room}
<article class="rounded-xl2 border border-line bg-ink-900 p-4">
<div class="flex items-start justify-between gap-2">
<div class="min-w-0">
<p class="font-display text-lg tracking-wider">{room.code}</p>
<p class="truncate text-sm text-dim">{room.topic}</p>
</div>
<span class="shrink-0 rounded-md border border-line px-2 py-0.5 text-xs text-dim">
{stateLabel[room.state] ?? room.state}
</span>
</div>
<p class="mt-2 text-xs text-dim">{room.mode_title} · {room.player_count} игроков · хост {room.host}</p>
<div class="mt-3 flex flex-wrap gap-2">
<button onclick={() => openRoom(room)} disabled={busy} class="rounded-md border border-line px-3 py-1.5 text-xs text-dim hover:text-chalk">
Детали
</button>
<button onclick={() => closeRoom(room)} class="rounded-md border border-coral/40 px-3 py-1.5 text-xs text-coral">
Закрыть комнату
</button>
</div>
</article>
{/each}
</div>
{/if}

{#if roomDetail}
<section class="mt-4 space-y-4 rounded-xl2 border border-amber/30 bg-ink-900 p-4 sm:p-5">
<div class="flex flex-wrap items-start justify-between gap-2">
<div>
<p class="font-display text-xl tracking-wider">{roomDetail.code}</p>
<p class="text-sm text-dim">
{roomDetail.topic} · {roomDetail.mode} · {roomDetail.difficulty} ·
{roomDetail.current_question + 1}/{roomDetail.question_count} вопросов ·
{stateLabel[roomDetail.state] ?? roomDetail.state}
</p>
<p class="text-xs text-dim">
Источник вопросов: {roomDetail.question_source} · сообщений в чате: {roomDetail.chat_messages}
{#if roomDetail.seconds_left !== null}· осталось {roomDetail.seconds_left} с{/if}
</p>
</div>
<button onclick={() => (roomDetail = null)} class="rounded-md border border-line px-2 py-1 text-xs text-dim">Свернуть</button>
</div>

<div class="flex flex-col gap-2 sm:flex-row">
<input
bind:value={roomAnnounce}
maxlength="200"
placeholder="Объявление только в эту комнату"
class="min-w-0 flex-1 rounded-lg border border-line bg-ink-850 px-3 py-2 text-sm outline-none focus:border-amber/50"
/>
<button onclick={sendRoomAnnounce} disabled={busy || !roomAnnounce.trim()} class="rounded-lg bg-amber px-4 py-2 text-sm font-semibold text-ink-950 disabled:opacity-50">
Отправить
</button>
</div>

<div class="flex flex-wrap gap-2">
<button
onclick={toggleChatLock}
disabled={busy}
class="rounded-md border px-3 py-1.5 text-xs {roomDetail.chat_locked ? 'border-mint/40 text-mint' : 'border-line text-dim'}"
>
{roomDetail.chat_locked ? 'Разморозить чат' : 'Заморозить чат'}
</button>
<button onclick={clearRoomChat} disabled={busy} class="rounded-md border border-coral/40 px-3 py-1.5 text-xs text-coral">
Очистить чат
</button>
</div>

<div class="overflow-x-auto">
<table class="w-full text-left text-sm">
<thead class="border-b border-line text-xs uppercase tracking-wide text-dim">
<tr>
<th class="px-3 py-2">Игрок</th>
<th class="px-3 py-2">Очки</th>
<th class="px-3 py-2">Состояние</th>
<th class="px-3 py-2">Действия</th>
</tr>
</thead>
<tbody>
{#each roomDetail.players as player (player.sid)}
<tr class="border-b border-line/60 last:border-0">
<td class="px-3 py-2">
<span class="font-medium">{player.name}</span>
{#if !player.connected}<span class="ml-2 text-xs text-dim">офлайн</span>{/if}
{#if player.user_id === roomDetail.host_user_id}<span class="ml-2 text-xs text-amber">хост</span>{/if}
</td>
<td class="px-3 py-2 text-dim">{player.score}</td>
<td class="px-3 py-2 text-xs text-dim">
{#if player.spectator}выбыл{:else}в игре{/if}
{#if player.muted}· мут{/if}
{#if player.jinxed}· проклят{/if}
{#if player.hijacked}· угнан{/if}
{#if player.blind_seconds}· слеп {player.blind_seconds} с{/if}
</td>
<td class="px-3 py-2">
<button
onclick={() => kickPlayer(player)}
disabled={busy}
class="rounded-md border border-coral/40 px-2 py-1 text-xs text-coral disabled:opacity-40"
>Выкинуть</button>
</td>
</tr>
{/each}
</tbody>
</table>
</div>
</section>
{/if}
{/if}

{#if tab === 'sessions'}
<section class="space-y-3">
<div class="flex flex-col gap-2 sm:flex-row">
<input
bind:value={sessionFilter}
placeholder="Фильтр по нику"
class="min-w-0 flex-1 rounded-lg border border-line bg-ink-900 px-3 py-2 text-sm outline-none focus:border-amber/50"
/>
<button onclick={refreshSessions} class="rounded-lg border border-line bg-ink-850 px-4 py-2 text-sm text-dim hover:text-chalk">Обновить</button>
</div>

{#if visibleSessions.length === 0}
<p class="rounded-xl2 border border-line bg-ink-900 p-5 text-sm text-dim">Активных сессий нет.</p>
{:else}
<div class="overflow-x-auto rounded-xl2 border border-line bg-ink-900">
<table class="w-full text-left text-sm">
<thead class="border-b border-line text-xs uppercase tracking-wide text-dim">
<tr>
<th class="px-4 py-3">Игрок</th>
<th class="px-4 py-3">Создана</th>
<th class="px-4 py-3">Истекает</th>
<th class="px-4 py-3">IP</th>
<th class="px-4 py-3">Действия</th>
</tr>
</thead>
<tbody>
{#each visibleSessions as session (session.id)}
<tr class="border-b border-line/60 last:border-0">
<td class="px-4 py-3">
<span class="font-medium">{session.username}</span>
{#if session.revoked_at}<span class="ml-2 text-xs text-coral">отозвана</span>{/if}
</td>
<td class="px-4 py-3 text-xs text-dim">{new Date(session.created_at).toLocaleString('ru-RU')}</td>
<td class="px-4 py-3 text-xs text-dim">{new Date(session.expires_at).toLocaleString('ru-RU')}</td>
<td class="px-4 py-3 text-xs text-dim">{session.ip ?? '—'}</td>
<td class="px-4 py-3">
<button
onclick={() => revokeSessions(session.user_id, session.username)}
disabled={busy || Boolean(session.revoked_at)}
class="rounded-md border border-coral/40 px-2 py-1 text-xs text-coral disabled:opacity-40"
>Завершить все</button>
</td>
</tr>
{/each}
</tbody>
</table>
</div>
{/if}
</section>
{/if}

{#if tab === 'audit'}
{#if audit.length === 0}
<p class="rounded-xl2 border border-line bg-ink-900 p-5 text-sm text-dim">Журнал пуст.</p>
{:else}
<ul class="space-y-2">
{#each audit as entry}
<li class="rounded-lg border border-line bg-ink-900 p-3 text-sm">
<div class="flex flex-wrap items-baseline justify-between gap-2">
<span class="font-medium">{entry.action}</span>
<span class="text-xs text-dim">{new Date(entry.created_at).toLocaleString('ru-RU')}</span>
</div>
<p class="mt-1 text-xs text-dim">
{entry.actor_name ?? 'система'} → {entry.target}{#if entry.reason} · {entry.reason}{/if}
</p>
</li>
{/each}
</ul>
{/if}
{/if}
</div>
{/if}
