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

export interface ApiResponse<T> {
	ok: boolean;
	user?: T;
	room?: T;
	rooms?: T[];
	players?: T[];
	error?: string;
}

async function request<T>(path: string, options?: RequestInit): Promise<ApiResponse<T>> {
	const response = await fetch(`${API_BASE}${path}`, {
		credentials: 'include',
		headers: {
			'Content-Type': 'application/json',
			...options?.headers,
		},
		...options,
	});

	if (!response.ok) {
		const error = await response.json().catch(() => ({ error: 'Request failed' }));
		throw new Error(error.error || `HTTP ${response.status}`);
	}

	return response.json();
}

export const api = {
	// Auth
	register: (username: string, password: string) =>
		request<User>('/api/auth/register', {
			method: 'POST',
			body: JSON.stringify({ username, password }),
		}),

	login: (username: string, password: string) =>
		request<User>('/api/auth/login', {
			method: 'POST',
			body: JSON.stringify({ username, password }),
		}),

	logout: () =>
		request<void>('/api/auth/logout', { method: 'POST' }),

	me: () =>
		request<User>('/api/auth/me'),

	// Rooms
	listRooms: () =>
		request<Room>('/api/rooms'),

	createRoom: (topic?: string, mode?: string) =>
		request<Room>('/api/rooms', {
			method: 'POST',
			body: JSON.stringify({ topic, mode }),
		}),

	getRoom: (code: string) =>
		request<Room>(`/api/rooms/${code}`),

	// Leaderboard
	topPlayers: (limit = 50) =>
		request<User>(`/api/leaderboard/top?limit=${limit}`),

	playerRank: (username: string) =>
		request<User>(`/api/leaderboard/rank/${username}`),
};
