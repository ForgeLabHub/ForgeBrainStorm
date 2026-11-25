<script lang="ts">
  export let name: string;
  export let size = 20;
  export let strokeWidth = 1.8;

  const glyphs: Record<string, string> = {
    play: '<path d="M8 5.2v13.6L19 12 8 5.2Z" fill="currentColor" stroke="none" />',
    users:
      '<circle cx="9" cy="7.5" r="3.2" /><path d="M3 19.5v-1.2a5 5 0 0 1 5-5h2a5 5 0 0 1 5 5v1.2" /><path d="M16.5 4.6a3.2 3.2 0 0 1 0 6.3" /><path d="M21 19.5v-1.2a4.4 4.4 0 0 0-3-4.2" />',
    trophy:
      '<path d="M7 4h10v4.5a5 5 0 0 1-10 0V4Z" /><path d="M7 5.5H4.6A2.4 2.4 0 0 0 7 9" /><path d="M17 5.5h2.4A2.4 2.4 0 0 1 17 9" /><path d="M12 13.5V17" /><path d="M9 20.5h6" /><path d="M10 17h4l.6 3.5H9.4L10 17Z" />',
    sliders:
      '<path d="M4 7h8" /><path d="M16 7h4" /><path d="M4 17h4" /><path d="M12 17h8" /><circle cx="14" cy="7" r="2.2" /><circle cx="10" cy="17" r="2.2" />',
    user: '<circle cx="12" cy="8.5" r="3.6" /><path d="M5 20a7 7 0 0 1 14 0" />',
    copy:
      '<rect x="8.5" y="8.5" width="11.5" height="11.5" rx="2.5" /><path d="M15.5 5.5A2 2 0 0 0 13.5 3.5H6a2.5 2.5 0 0 0-2.5 2.5v7.5a2 2 0 0 0 2 2" />',
    check: '<path d="M4.5 12.5l4.8 4.8L19.5 7" />',
    back: '<path d="M14.5 5.5 8 12l6.5 6.5" />',
    bolt: '<path d="M13.2 3 5.5 13.8h4.6L9.5 21l7.9-11.2h-4.7L13.2 3Z" fill="currentColor" stroke="none" />',
    lock:
      '<rect x="4.8" y="10.5" width="14.4" height="9.7" rx="2.6" /><path d="M8.3 10.5V8a3.7 3.7 0 0 1 7.4 0v2.5" />',
    logout: '<path d="M14.5 8V5.5a2 2 0 0 0-2-2H6a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2h6.5a2 2 0 0 0 2-2V16" /><path d="M9.5 12h11" /><path d="M17.5 8.5 21 12l-3.5 3.5" />',
    plus: '<path d="M12 5v14" /><path d="M5 12h14" />'
  };
</script>

<svg
  xmlns="http://www.w3.org/2000/svg"
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width={strokeWidth}
  stroke-linecap="round"
  stroke-linejoin="round"
  aria-hidden="true"
  focusable="false"
>
  {@html glyphs[name] ?? ''}
</svg>
