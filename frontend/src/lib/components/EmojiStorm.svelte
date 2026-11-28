<script lang="ts">
	import type { EmojiThrow } from '$lib/chat';

	let { throws = [] }: { throws?: EmojiThrow[] } = $props();
</script>

<!-- Flying emoji live above everything else and never intercept clicks. -->
<div class="pointer-events-none fixed inset-0 z-50 overflow-hidden" aria-hidden="true">
	{#each throws as item (item.id)}
		<div
			class="emoji-throw"
			style="left: {item.left}%; --drift: {(item.left - 50) / 4}px"
			title="{item.from}{item.target ? ` → ${item.target}` : ''}"
		>
			<span class="text-5xl drop-shadow-[0_2px_12px_rgba(0,0,0,0.45)]">{item.emoji}</span>
			<span class="mt-1 block text-center text-[10px] font-semibold text-white/70">{item.from}</span>
		</div>
	{/each}
</div>

<style>
	.emoji-throw {
		position: absolute;
		bottom: -10%;
		animation: emoji-flight 3s cubic-bezier(0.2, 0.7, 0.3, 1) forwards;
		text-align: center;
	}

	@keyframes emoji-flight {
		0% {
			opacity: 0;
			transform: translateY(0) scale(0.6) rotate(-12deg);
		}
		15% {
			opacity: 1;
			transform: translateY(-12vh) scale(1.15) rotate(6deg);
		}
		70% {
			opacity: 1;
			transform: translateY(-62vh) translateX(var(--drift)) scale(1) rotate(-4deg);
		}
		100% {
			opacity: 0;
			transform: translateY(-88vh) translateX(calc(var(--drift) * 1.6)) scale(0.85) rotate(8deg);
		}
	}

	@media (prefers-reduced-motion: reduce) {
		.emoji-throw {
			animation-duration: 1.2s;
			animation-timing-function: ease-out;
		}

		@keyframes emoji-flight {
			0%,
			100% {
				opacity: 0;
				transform: none;
			}
			30%,
			70% {
				opacity: 1;
				transform: translateY(-40vh);
			}
		}
	}
</style>
