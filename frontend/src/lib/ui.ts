/** Shared labels and small helpers for the quiz UI. */

export const MODES: Record<string, { label: string; hint: string }> = {
  classic: { label: 'Классика', hint: 'Все отвечают одновременно' },
  ffa: { label: 'Каждый за себя', hint: 'Чем быстрее ответ, тем больше очков' },
  lives: { label: 'На выживание', hint: 'Три жизни, ошибки выбивают' },
  blitz: { label: 'Блиц', hint: '10 секунд на вопрос, серия умножает очки' },
  team: { label: 'Команды', hint: 'Две команды на одном счёте' },
  sandbox: { label: 'Тренировка', hint: 'Без рейтинга и жизней' },
  coop: { label: 'Кооператив', hint: 'Одна общая копилка очков' },
  svoyaigra: { label: 'Своя Игра', hint: '15 секунд, очки растут с ответом' }
};

export const DIFFICULTIES: Record<string, string> = {
  easy: 'Лёгкая',
  medium: 'Средняя',
  hard: 'Сложная'
};

export function modeLabel(mode: string): string {
  return MODES[mode]?.label ?? mode;
}

export function difficultyLabel(difficulty: string): string {
  return DIFFICULTIES[difficulty] ?? difficulty;
}

/** Mirrors the backend: level = floor(sqrt(xp / 100)), never below 1. */
export function levelFromXp(xp: number): number {
  return Math.max(1, Math.floor(Math.sqrt(Math.max(xp, 0) / 100)));
}

export function levelBounds(level: number): { from: number; to: number } {
  // Level 1 starts from zero so the bar moves as soon as the first points arrive.
  return { from: level <= 1 ? 0 : level * level * 100, to: (level + 1) * (level + 1) * 100 };
}

export function levelProgress(xp: number): number {
  const { from, to } = levelBounds(levelFromXp(xp));
  return Math.min(100, Math.max(0, ((xp - from) / (to - from)) * 100));
}

export function initial(name: string): string {
  return (name.trim()[0] ?? '?').toUpperCase();
}

export function plural(n: number, one: string, few: string, many: string): string {
  const mod10 = n % 10;
  const mod100 = n % 100;
  if (mod10 === 1 && mod100 !== 11) return one;
  if (mod10 >= 2 && mod10 <= 4 && (mod100 < 12 || mod100 > 14)) return few;
  return many;
}

export async function copyText(text: string): Promise<boolean> {
  try {
    await navigator.clipboard.writeText(text);
    return true;
  } catch {
    return false;
  }
}
