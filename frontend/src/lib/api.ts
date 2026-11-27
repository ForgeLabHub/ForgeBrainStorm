/**
 * HTTP API client for BrainStorm backend.
 */

const API_BASE = import.meta.env.VITE_API_URL || '';

export type Role = 'player' | 'cheater' | 'admin';

export interface User {
username: string;
xp: number;
coins: number;
games_played: number;
wins: number;
total_score?: number;
level: number;
role?: Role;
}

export interface Room {
code: string;
host: string;
mode: string;
topic: string;
player_count?: number;
}

export interface AiQuestion {
question: string;
options: string[];
correct_index: number;
explanation: string;
hint: string;
difficulty: string;
category?: string | null;
}

export interface CreateRoomInput {
topic?: string;
mode?: string;
difficulty?: string;
max_players?: number;
question_count?: number;
num_options?: number;
timer_seconds?: number;
ai_provider?: string;
}

export interface ApiBase {
ok: boolean;
error?: string;
}

export interface MeResponse extends ApiBase {
logged_in?: boolean;
user?: User;
}

export interface LoginResponse extends ApiBase {
user?: User;
}

export interface RoomsResponse extends ApiBase {
rooms: Room[];
}

export interface RoomResponse extends ApiBase {
room: Room;
}

export interface ProvidersResponse extends ApiBase {
providers: string[];
default: string;
}

export interface CacheStatsResponse extends ApiBase {
cache: { questions_cached: number; hints_cached: number };
}

export interface GenerateResponse extends ApiBase {
provider: string;
tokens_used: number;
latency_ms: number;
questions: AiQuestion[];
}

export interface LeaderboardResponse extends ApiBase {
players: User[];
}

export interface RecentGame {
room_code: string;
topic: string;
mode: string;
score: number;
place: number;
players: number;
duration_secs: number;
played_at: string;
}

export interface RecentGamesResponse extends ApiBase {
games: RecentGame[];
}

export interface PlayerRankResponse extends ApiBase {
player: User;
}

async function request<T>(path: string, options?: RequestInit): Promise<T> {
const response = await fetch(`${API_BASE}${path}`, {
credentials: 'include',
headers: {
'Content-Type': 'application/json',
...options?.headers,
},
...options,
});

if (!response.ok) {
const body = await response.json().catch(() => null);
const raw = body?.error;
const message = typeof raw === 'string' ? raw : raw?.message;
throw new Error(message || `HTTP ${response.status}`);
}

return response.json();
}

export const api = {
// Auth
register: (username: string, password: string) =>
request<LoginResponse>('/api/auth/register', {
method: 'POST',
body: JSON.stringify({ username, password }),
}),

login: (username: string, password: string) =>
request<LoginResponse>('/api/auth/login', {
method: 'POST',
body: JSON.stringify({ username, password }),
}),

logout: () => request<ApiBase>('/api/auth/logout', { method: 'POST' }),

me: () => request<MeResponse>('/api/auth/me'),

// Rooms
listRooms: () => request<RoomsResponse>('/api/rooms'),

createRoom: (input: CreateRoomInput) =>
request<RoomResponse>('/api/rooms', {
method: 'POST',
body: JSON.stringify(input),
}),

getRoom: (code: string) => request<RoomResponse>(`/api/rooms/${code}`),

// Profile
recentGames: (limit = 5) => request<RecentGamesResponse>(`/api/profile/games?limit=${limit}`),

// Leaderboard
topPlayers: (limit = 50) => request<LeaderboardResponse>(`/api/leaderboard/top?limit=${limit}`),

playerRank: (username: string) => request<PlayerRankResponse>(`/api/leaderboard/rank/${username}`),

// Admin panel
adminStats: () => request<AdminStats>('/api/admin/stats'),

adminUsers: (query = '', role?: Role, limit = 50, offset = 0) => {
const params = new URLSearchParams({ q: query, limit: String(limit), offset: String(offset) });
if (role) params.set('role', role);
return request<AdminUser[]>(`/api/admin/users?${params.toString()}`);
},

adminSetRole: (userId: number, role: Role) =>
request<ApiBase>('/api/admin/users/role', {
method: 'POST',
body: JSON.stringify({ user_id: userId, role }),
}),

adminResetUser: (userId: number) =>
request<ApiBase>('/api/admin/users/reset', {
method: 'POST',
body: JSON.stringify({ user_id: userId }),
}),

adminDeleteUser: (userId: number) =>
request<ApiBase>('/api/admin/users/delete', {
method: 'POST',
body: JSON.stringify({ user_id: userId }),
}),

adminBan: (identifier: string, minutes = 60, reason = '') =>
request<ApiBase>('/api/admin/ban', {
method: 'POST',
body: JSON.stringify({ identifier, kind: 'user', duration_minutes: minutes, reason }),
}),

adminUnban: (identifier: string) =>
request<ApiBase>('/api/admin/unban', {
method: 'POST',
body: JSON.stringify({ identifier, kind: 'user' }),
}),

adminRooms: () => request<AdminRoom[]>('/api/admin/rooms'),

adminCloseRoom: (code: string) =>
request<ApiBase>('/api/admin/rooms/close', {
method: 'POST',
body: JSON.stringify({ code }),
}),

adminRoomDetail: (code: string) =>
request<AdminRoomDetail>(`/api/admin/rooms/detail?code=${encodeURIComponent(code)}`),

adminKick: (code: string, user: { user_id?: number; username?: string }, reason = '') =>
request<ApiBase>('/api/admin/rooms/kick', {
method: 'POST',
body: JSON.stringify({ code, ...user, reason }),
}),

adminRoomAnnounce: (code: string, message: string) =>
request<ApiBase>('/api/admin/rooms/announce', {
method: 'POST',
body: JSON.stringify({ code, message }),
}),

adminChatLock: (code: string, locked: boolean) =>
request<ApiBase>('/api/admin/rooms/chat_lock', {
method: 'POST',
body: JSON.stringify({ code, locked }),
}),

adminChatClear: (code: string) =>
request<ApiBase>('/api/admin/rooms/chat_clear', {
method: 'POST',
body: JSON.stringify({ code }),
}),

adminSessions: (userId?: number) =>
request<AdminSession[]>(`/api/admin/sessions${userId ? `?user_id=${userId}` : ''}`),

adminRevokeSessions: (userId: number) =>
request<ApiBase>('/api/admin/sessions/revoke', {
method: 'POST',
body: JSON.stringify({ user_id: userId }),
}),

adminBroadcast: (message: string) =>
request<ApiBase>('/api/admin/broadcast', {
method: 'POST',
body: JSON.stringify({ message }),
}),

adminAuditLog: () => request<AdminAuditEntry[]>('/api/admin/audit_log'),

adminAiStatus: () => request<AdminAiStatus>('/api/admin/ai'),

adminSwitchProvider: (provider: string) =>
request<ApiBase>('/api/admin/ai/switch', {
method: 'POST',
body: JSON.stringify({ provider }),
}),

adminClearAiCache: () => request<ApiBase>('/api/admin/ai/cache/clear', { method: 'POST' })
};

export interface AdminStats {
users: number;
admins: number;
cheaters: number;
games: number;
total_score: number;
active_bans: number;
live_sessions: number;
live_rooms: number;
live_players: number;
}

export interface AdminUser {
user_id: number;
username: string;
role: Role;
xp: number;
coins: number;
games_played: number;
wins: number;
banned: boolean;
}

export interface AdminRoom {
code: string;
host: string;
state: string;
mode: string;
mode_title: string;
topic: string;
player_count: number;
}

export interface AdminRoomPlayer {
sid: string;
user_id: number | null;
name: string;
score: number;
lives: number;
team: number | null;
spectator: boolean;
connected: boolean;
muted: boolean;
jinxed: boolean;
hijacked: boolean;
blind_seconds: number | null;
cheat_left: Record<string, number> | null;
}

export interface AdminRoomDetail {
code: string;
state: string;
host: string;
host_user_id: number;
topic: string;
mode: string;
difficulty: string;
question_count: number;
current_question: number;
seconds_left: number | null;
chat_locked: boolean;
chat_messages: number;
question_source: string;
players: AdminRoomPlayer[];
}

export interface AdminSession {
id: string;
user_id: number;
username: string;
created_at: string;
expires_at: string;
revoked_at: string | null;
ip: string | null;
}

export interface AdminAuditEntry {
id: number;
actor_id: number | null;
actor_name: string | null;
action: string;
target: string;
reason: string;
created_at: string;
}

export interface AdminAiStatus {
providers: string[];
default: string;
questions_cached: number;
hints_cached: number;
}
