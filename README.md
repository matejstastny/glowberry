# Lantern

A simple, fast Minecraft launcher for managing modpacks with a streamlined feature set and minimalist UI. Created as an alternative launcher for users who prefer a lighter approach to modpack management.

## Features

- **Simple UI** - clean dark interface, zero jargon
- **Modrinth integration** - search and install modpacks directly
- **mrpack support** - full Modrinth modpack format support
- **Smart updates** - one-click modpack updates that actually work
- **File locks** - protect your keybinds, settings, and configs from being overwritten during pack updates
- **Cross-platform** - macOS and Windows
- **Lightweight** - ~5MB binary, instant startup

## Tech Stack

| Layer    | Technology         |
| -------- | ------------------ |
| Backend  | Rust (Tauri v2)    |
| Frontend | React + TypeScript |
| Bundler  | Vite               |
| API      | Modrinth v2        |

<details>
<summary><h2>Development</h2></summary>

### Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Node.js](https://nodejs.org/) (v20+)
- System dependencies for Tauri - see [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/)

### Setup

```bash
git clone https://github.com/matejstastny/lantern.git
cd lantern
npm install
```

### Commands

```bash
npm run tauri dev        # Run in development mode (hot reload)
npm run tauri build      # Build for production
npm run lint             # ESLint (frontend) + Clippy (Rust)
npm run format           # Prettier (frontend) + rustfmt (Rust)
npm run format:check     # Check formatting without changing files
npm run check            # TypeScript type checking
```

### Project Structure

```
lantern/
├── src/                    # React frontend
│   ├── api/                # Tauri IPC wrappers
│   ├── components/         # Reusable UI components
│   ├── hooks/              # React hooks (shared state)
│   ├── pages/              # Page components
│   ├── types.ts            # TypeScript types
│   ├── App.tsx             # Root component + routing
│   └── main.tsx            # Entry point
├── src-tauri/              # Rust backend
│   └── src/
│       ├── auth/           # Microsoft authentication
│       ├── commands/       # Tauri IPC command handlers
│       ├── download/       # Parallel download manager
│       ├── instance/       # Instance management + file locks
│       ├── minecraft/      # Java detection, game launching
│       ├── modrinth/       # Modrinth API client + mrpack parser
│       ├── error.rs        # Error types
│       ├── state.rs        # App state
│       └── lib.rs          # Entry point
└── static/                 # Static assets
```

</details>
