<script module lang="ts">
	export type IconName =
		| 'target'
		| 'swords'
		| 'users'
		| 'heart'
		| 'bolt'
		| 'cap'
		| 'seedling'
		| 'flame'
		| 'clock'
		| 'check'
		| 'cross'
		| 'trophy'
		| 'medal'
		| 'scissors'
		| 'hourglass'
		| 'gamepad'
		| 'eye'
		| 'spark'
		| 'chat'
		| 'mask'
		| 'pencil'
		| 'mute'
		| 'skull'
		| 'blind'
		| 'swap'
		| 'drain'
		| 'bulb';
</script>

<script lang="ts">
	/**
	 * Stroke icons (24×24, currentColor) used instead of emoji: emoji depend on a
	 * colour-emoji font being installed and fall back to empty boxes without one.
	 */

	const paths: Record<IconName, string[]> = {
		chat: ['M21 11.5a8.4 8.4 0 01-9 8.4 9 9 0 01-3.3-.6L3 21l1.7-4.6A8.4 8.4 0 0121 11.5z'],
		mask: ['M4 6h16l-1.2 8.4A6 6 0 0113 20h-2a6 6 0 01-5.8-5.6L4 6z', 'M9.5 11.5h.01', 'M14.5 11.5h.01'],
		pencil: ['M4 20h4l10-10-4-4L4 16v4z', 'M14 6l4 4'],
		mute: ['M11 5L6 9H3v6h3l5 4V5z', 'M16 9l5 6', 'M21 9l-5 6'],
		skull: ['M12 3a7 7 0 00-7 7v3.5L7 16v4h10v-4l2-2.5V10a7 7 0 00-7-7z', 'M9.5 12h.01', 'M14.5 12h.01'],
		blind: ['M3 3l18 18', 'M10.6 10.6a2 2 0 002.8 2.8', 'M9.9 5.2A9.7 9.7 0 0112 5c5 0 9 4.5 9 7 0 .9-.6 2.1-1.6 3.2', 'M6.6 6.7C4.5 8 3 9.8 3 12c0 2.5 4 7 9 7 1.4 0 2.7-.3 3.9-.9'],
		swap: ['M7 7h11l-3-3', 'M17 17H6l3 3'],
		drain: ['M12 3v10', 'M8 9l4 4 4-4', 'M4 17h16', 'M6 21h12'],
		bulb: ['M9 18h6', 'M10 21h4', 'M12 3a6 6 0 00-3 11.2V16h6v-1.8A6 6 0 0012 3z'],
		target: ['M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z'],
		swords: ['M4 20l4-4m0 0l4-4m-4 4l-2-2m8-8l4-4m0 0h-4m4 0v4', 'M14 6l4 4', 'M6 6l12 12'],
		users: ['M17 20v-1a4 4 0 00-4-4H7a4 4 0 00-4 4v1', 'M10 11a4 4 0 100-8 4 4 0 000 8', 'M21 20v-1a4 4 0 00-3-3.87', 'M16 3.13a4 4 0 010 7.75'],
		heart: ['M20.8 5.6a5.5 5.5 0 00-7.8 0L12 6.6l-1-1a5.5 5.5 0 10-7.8 7.8L12 22l8.8-8.6a5.5 5.5 0 000-7.8z'],
		bolt: ['M13 2L4 14h7l-1 8 9-12h-7l1-8z'],
		cap: ['M22 9L12 4 2 9l10 5 10-5z', 'M6 11.5V16c0 1.7 2.7 3 6 3s6-1.3 6-3v-4.5'],
		seedling: ['M12 21v-8', 'M12 13c0-4 3-7 7-7 0 4-3 7-7 7z', 'M12 13C12 9.7 9.3 7 6 7c0 3.3 2.7 6 6 6z'],
		flame: ['M12 22c4 0 7-2.7 7-6.5 0-4.5-4-6.5-4-10.5-2 1-3 2.5-3 4.5-1-.5-1.5-1.5-1.5-2.5C8 9 5 11.5 5 15.5 5 19.3 8 22 12 22z'],
		clock: ['M12 21a9 9 0 100-18 9 9 0 000 18z', 'M12 7v5l3 2'],
		check: ['M20 6L9 17l-5-5'],
		cross: ['M18 6L6 18', 'M6 6l12 12'],
		trophy: ['M8 21h8', 'M12 17v4', 'M7 4h10v5a5 5 0 01-10 0V4z', 'M17 5h3a2 2 0 01-2 4', 'M7 5H4a2 2 0 002 4'],
		medal: ['M12 15a5 5 0 100-10 5 5 0 000 10z', 'M8.5 14L7 22l5-3 5 3-1.5-8'],
		scissors: ['M6 6l12 12', 'M18 6L9.5 14.5', 'M6 9a3 3 0 100-6 3 3 0 000 6z', 'M6 21a3 3 0 100-6 3 3 0 000 6z'],
		hourglass: ['M7 3h10', 'M7 21h10', 'M7 3c0 5 5 6 5 9s-5 4-5 9', 'M17 3c0 5-5 6-5 9s5 4 5 9'],
		eye: ['M2 12s3.6-7 10-7 10 7 10 7-3.6 7-10 7-10-7-10-7z', 'M12 15a3 3 0 100-6 3 3 0 000 6'],
		spark: ['M12 3l2.6 5.7 6.2.8-4.6 4.3 1.2 6.2L12 17l-5.4 3 1.2-6.2L3.2 9.5l6.2-.8z'],
		gamepad: ['M7 12h4m-2-2v4', 'M16 11h.01M18 13h.01', 'M17 6H7a5 5 0 00-5 5v2a4 4 0 007 2.6L10 15h4l1 0.6A4 4 0 0022 13v-2a5 5 0 00-5-5z']
	};

	interface Props {
		name: IconName;
		class?: string;
	}

	let { name, class: className = 'h-5 w-5' }: Props = $props();
</script>

<svg
	class={className}
	fill="none"
	stroke="currentColor"
	viewBox="0 0 24 24"
	aria-hidden="true"
	stroke-linecap="round"
	stroke-linejoin="round"
	stroke-width="2"
>
	{#each paths[name] as d}
		<path {d} />
	{/each}
</svg>
