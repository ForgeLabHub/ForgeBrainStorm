/** Types and constants shared by the in-room chat and emoji UI. */

export type ChatMessage = {
	seq: number;
	from: string;
	text: string;
	kind: 'text' | 'emoji';
};

/** One emoji in flight across the screen. `left` is a percentage. */
export type EmojiThrow = {
	id: number;
	emoji: string;
	from: string;
	target?: string | null;
	left: number;
};

/**
 * Must match `EMOJI_PALETTE` in `crates/server/src/domain/chat.rs`: the server
 * rejects anything else, so a drifting list would silently stop working.
 */
export const EMOJI_PALETTE = ['😂', '🔥', '😱', '👏', '💀', '🤡', '❤️', '🎯'];

export const MAX_CHAT_CHARS = 200;

export type CheatPowerKind =
	| 'peek'
	| 'freeze'
	| 'steal'
	| 'double'
	| 'hijack'
	| 'rewrite'
	| 'mute'
	| 'jinx'
	| 'leech'
	| 'blind'
	| 'swap'
	| 'hint';

export type CheatBudget = Record<CheatPowerKind, number>;

/** Powers that need a victim picked from the player list. */
export const TARGET_POWERS: CheatPowerKind[] = ['hijack', 'mute', 'jinx', 'blind'];

export type CheatTarget = { id: string; name: string };
