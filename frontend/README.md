# BrainStorm Frontend (SvelteKit)

Modern frontend for the BrainStorm quiz platform.

## Status

**Phase 1**: Basic scaffolding
- ⏳ SvelteKit setup
- ⏳ Tailwind CSS + shadcn-svelte
- ⏳ TypeScript + typed WebSocket client
- ⏳ Auth pages (login/register)
- ⏳ Lobby (room list, create room)
- ⏳ Game screen (questions, answers, chat)
- ⏳ Leaderboard
- ⏳ Profile page

## Stack

- **Framework**: SvelteKit (SPA mode)
- **Styling**: Tailwind CSS + shadcn-svelte
- **State**: Svelte stores + typed WS events
- **Validation**: zod
- **WebSocket**: Native API with reconnect/backoff

## Development

```bash
npm install
npm run dev
```

Open http://localhost:5173

## Build

```bash
npm run build
npm run preview
```

## Architecture

```
src/
├── routes/              # SvelteKit routes
│   ├── +layout.svelte   # Root layout
│   ├── +page.svelte     # Landing page
│   ├── auth/            # Login, register
│   ├── lobby/           # Room list, create
│   ├── room/            # Game screen
│   └── profile/         # User profile
├── lib/
│   ├── api.ts           # HTTP client (fetch wrapper)
│   ├── ws.ts            # WebSocket client with reconnect
│   ├── stores.ts        # Svelte stores (auth, room, game)
│   ├── types.ts         # TypeScript types
│   └── components/      # Reusable UI components
└── app.html             # HTML template
```

## Features

### P1 — Engagement
- Profile with avatars, stats, game history
- Friends/invitations (share link)
- Training mode (solo, no leaderboard)
- Tournaments (series of rooms)

### P2 — UX
- Lobby redesign with ready-check
- Game results with breakdown
- PWA (offline, installable)
- Mobile-first responsive design
- Accessibility (WCAG 2.1 AA)

### P3 — AI & Content
- Question cache (reduce API calls)
- User-generated questions (moderation)
- AI judge for open-ended

### P4 — Social
- Spectator mode
- Team mode with team chat
- Game replay

## Design System

- **Colors**: Tailwind default palette + custom brand colors
- **Typography**: Inter (body), JetBrains Mono (code)
- **Components**: shadcn-svelte (Button, Card, Dialog, etc.)
- **Animations**: svelte/transition + motion
- **Dark mode**: Automatic + toggle

## Backend API

Connects to Rust backend at `http://localhost:5000` (configurable via `VITE_API_URL`).

### Endpoints
- `POST /api/auth/register` — create account
- `POST /api/auth/login` — authenticate
- `GET /api/rooms` — list rooms
- `POST /api/rooms` — create room
- `GET /api/leaderboard/top` — top players
- `WS /ws` — real-time gameplay

See Rust backend README for full API documentation.

## License

MIT
