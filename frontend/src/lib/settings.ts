/** Room defaults the host picks once in /settings instead of every time. */

export interface RoomDefaults {
  mode: string;
  difficulty: string;
  questions: number;
  options: number;
  timer: number;
}

const KEY = 'brainstorm:room-defaults';

export const DEFAULT_ROOM: RoomDefaults = {
  mode: 'classic',
  difficulty: 'medium',
  questions: 10,
  options: 4,
  timer: 30
};

const QUESTIONS_RANGE = [5, 30];
const OPTIONS_RANGE = [2, 6];
const TIMER_RANGE = [10, 60];

const clamp = (value: unknown, [min, max]: number[], fallback: number): number => {
  const n = Number(value);
  if (!Number.isFinite(n)) return fallback;
  return Math.min(max, Math.max(min, Math.round(n)));
};

/** Reads the saved defaults, ignoring anything a hand-edited storage could break. */
export function loadRoomDefaults(): RoomDefaults {
  if (typeof localStorage === 'undefined') return { ...DEFAULT_ROOM };
  try {
    const raw = JSON.parse(localStorage.getItem(KEY) ?? '{}') as Partial<RoomDefaults>;
    return {
      mode: typeof raw.mode === 'string' ? raw.mode : DEFAULT_ROOM.mode,
      difficulty: typeof raw.difficulty === 'string' ? raw.difficulty : DEFAULT_ROOM.difficulty,
      questions: clamp(raw.questions, QUESTIONS_RANGE, DEFAULT_ROOM.questions),
      options: clamp(raw.options, OPTIONS_RANGE, DEFAULT_ROOM.options),
      timer: clamp(raw.timer, TIMER_RANGE, DEFAULT_ROOM.timer)
    };
  } catch {
    return { ...DEFAULT_ROOM };
  }
}

export function saveRoomDefaults(defaults: RoomDefaults): void {
  if (typeof localStorage === 'undefined') return;
  localStorage.setItem(KEY, JSON.stringify(defaults));
}

export function clearRoomDefaults(): void {
  if (typeof localStorage === 'undefined') return;
  localStorage.removeItem(KEY);
}
