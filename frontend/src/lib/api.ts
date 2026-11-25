/**
 * HTTP API client for BrainStorm backend.
 */

const API_BASE = import.meta.env.VITE_API_URL || '';

export interface User {
username: string;
xp: number;
coins: number;
games_played: number;
wins: number;
level: number;
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

// AI
generateQuestions: (topic: string, count = 10, difficulty = 'medium', numOptions = 4) =>
request<GenerateResponse>('/api/ai/generate', {
method: 'POST',
body: JSON.stringify({
topic,
count,
difficulty,
num_options: numOptions,
}),
}),

getAiProviders: () => request<ProvidersResponse>('/api/ai/providers'),

switchAiProvider: (provider: string) =>
request<ApiBase>('/api/ai/providers/switch', {
method: 'POST',
body: JSON.stringify({ provider }),
}),

getAiCacheStats: () => request<CacheStatsResponse>('/api/ai/cache/stats'),

clearAiCache: () => request<ApiBase>('/api/ai/cache/clear', { method: 'POST' }),
};
