# ForgeBrainStorm

[![CI](https://github.com/ForgeLabHub/ForgeBrainStorm/actions/workflows/ci.yml/badge.svg)](https://github.com/ForgeLabHub/ForgeBrainStorm/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

AI-powered multiplayer quiz platform with dynamic question generation and multiple game modes.

## Features

### 🎮 6 Game Modes

- **Classic** - Synchronous team-based gameplay
- **Free For All (FFA)** - Individual speed competition
- **Team Battle** - Strategic team competition
- **Survival (Lives)** - Last player standing wins
- **Cooperative** - Collaborative gameplay against the system
- **Svojaigra** - Jeopardy-style point bidding

### 🤖 AI-Powered Question Generation

- **GPT-4o-mini Integration** - High-quality, context-aware questions
- **Ollama Support** - Local AI provider option
- **Dynamic Difficulty Adjustment (DDA)** - Automatically adapts question complexity based on player performance
- **Retrieval-Augmented Generation (RAG)** - Generate questions from your own documents and knowledge base

### 📊 Player Progression

- XP and leveling system
- Rank titles (Bronze → Silver → Gold → Platinum → Diamond → Master)
- Win rate tracking
- Coin rewards
- Persistent player profiles

### 🎨 Modern UI/UX

- **Mobile-First Design** - Optimized for all devices
- **Dark Theme** - Eye-friendly dark interface
- **Glassmorphism** - Modern glass-effect cards
- **Smooth Animations** - 60fps transitions
- **Accessibility** - Respects reduced motion preferences

### ⚡ Performance

- **Rust Backend** - Zero-cost abstractions, no garbage collector
- **WebSocket Communication** - Real-time updates with <20ms latency
- **Connection Pooling** - Efficient HTTP client management
- **Lazy Loading** - Components load on demand
- **Optimized for Mobile** - Reduced animations and effects on mobile devices

## Tech Stack

### Backend
- **Rust** with Axum web framework
- **Tokio** async runtime
- **WebSocket** for real-time communication
- **OpenAI API** / **Ollama** for AI integration

### Frontend
- **Svelte 5** with runes (`$state`, `$derived`, `$props`)
- **TypeScript** for type safety
- **Tailwind CSS** for styling
- **Vite** for fast builds

### Infrastructure
- **Docker** containerization
- **GitHub Actions** CI/CD
- **Multi-stage builds** for optimized images

## Quick Start

### Prerequisites

- Docker and Docker Compose
- OR Rust 1.75+ and Node.js 20+

### Using Docker (Recommended)

```bash
# Clone the repository
git clone https://github.com/ForgeLabHub/ForgeBrainStorm.git
cd ForgeBrainStorm

# Start all services
docker-compose up -d

# Access the app
open http://localhost:5173
```

### Manual Setup

#### Backend

```bash
cd rust-server

# Install dependencies
cargo build

# Set environment variables
export OPENAI_API_KEY=your_key_here

# Run the server
cargo run
```

#### Frontend

```bash
cd frontend

# Install dependencies
npm install

# Start dev server
npm run dev
```

## Configuration

### Environment Variables

```bash
# OpenAI API
OPENAI_API_KEY=sk-...

# Server
RUST_LOG=info
PORT=8080

# Ollama (optional)
OLLAMA_BASE_URL=http://localhost:11434
```

### AI Settings

Configure AI provider in the app settings page:

- **OpenAI**: Cloud-based, highest quality
- **Ollama**: Local, privacy-focused, free

## Game Modes

### Classic
All players answer the same questions simultaneously. Perfect for large groups.

```json
{
  "mode": "classic",
  "topic": "World History",
  "num_questions": 10,
  "difficulty": "medium"
}
```

### Free For All
Individual competition based on speed and accuracy.

### Team Battle
Players form teams and compete collaboratively.

### Survival
Each player has 3 lives. Wrong answers cost a life. Last one standing wins.

### Cooperative
Players work together to maximize the group score.

### Svoyaigra
Jeopardy-style gameplay with strategic question selection.

## API Documentation

### REST Endpoints

```http
POST /api/room/create
GET  /api/rooms
GET  /api/room/{code}
POST /api/auth/register
POST /api/auth/login
GET  /api/auth/me
```

### WebSocket

```javascript
const ws = new WebSocket('ws://localhost:8080/ws');

ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  // Handle game updates
};
```

## Architecture

```
ForgeBrainStorm/
├── rust-server/          # Backend
│   ├── src/
│   │   ├── domain/       # Core models
│   │   ├── services/     # Business logic
│   │   ├── routes/       # HTTP endpoints
│   │   └── ws/           # WebSocket handlers
│   └── tests/            # Integration tests
├── frontend/             # Frontend
│   ├── src/
│   │   ├── routes/       # Svelte pages
│   │   ├── lib/          # Shared utilities
│   │   └── app.css       # Global styles
│   └── static/           # Static assets
└── docs/                 # Documentation
    ├── features/         # Feature guides
    └── architecture/     # Technical docs
```

## Documentation

- [Game Modes](docs/features/game_modes.md)
- [Dynamic Difficulty Adjustment](docs/features/dynamic_difficulty_adjustment.md)
- [Retrieval-Augmented Generation](docs/features/retrieval_augmented_generation.md)
- [Mobile Optimization](docs/features/mobile_optimization.md)
- [Performance Guide](docs/architecture/performance.md)

## Development

### Running Tests

```bash
# Backend tests
cd rust-server
cargo test

# Frontend tests
cd frontend
npm test
```

### Code Quality

```bash
# Rust formatting
cargo fmt

# Rust linting
cargo clippy

# Svelte linting
cd frontend
npm run lint
```

## Performance Targets

| Metric | Target | Actual |
|--------|--------|--------|
| Room creation | < 100ms | ~50ms |
| Question generation | < 3s | ~2s |
| WebSocket update | < 20ms | ~10ms |
| Mobile FPS | 60fps | 60fps |

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Inspired by Kahoot and Quizizz
- Built with modern web technologies
- AI-powered question generation

## Roadmap

- [ ] Voice chat integration
- [ ] Custom question templates
- [ ] Tournament mode
- [ ] Player achievements
- [ ] Leaderboards
- [ ] Mobile apps (iOS/Android)
- [ ] Question categories marketplace

## Support

- 📧 Email: support@forgelabhub.com
- 💬 Discord: [Join our community](https://discord.gg/forgebrainstorm)
- 📚 Documentation: [docs/](docs/)

---

Built with ❤️ by ForgeLabHub
