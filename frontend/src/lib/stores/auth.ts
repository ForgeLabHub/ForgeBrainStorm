import { writable } from 'svelte/store';
import { api, type User } from '$lib/api';

/** Currently authenticated user, or `null` when logged out. */
export const currentUser = writable<User | null>(null);

/** Becomes true once the first `/api/auth/me` probe has settled. */
export const authChecked = writable(false);

export async function refreshAuth(): Promise<void> {
	try {
		const response = await api.me();
		currentUser.set(response.user ?? null);
	} catch {
		currentUser.set(null);
	} finally {
		authChecked.set(true);
	}
}

export async function logout(): Promise<void> {
	try {
		await api.logout();
	} catch {
		// Session may already be gone — clearing local state is enough.
	}
	currentUser.set(null);
}
