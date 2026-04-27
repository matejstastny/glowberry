# Glowberry

A simple, fast Minecraft launcher built with Tauri v2 (Rust) + React + TypeScript.

## Project Structure

- `src/` — React frontend (pages, components, hooks, API wrappers)
- `src-tauri/src/` — Rust backend (commands, modrinth API, instance management, downloads)
- Frontend talks to backend via Tauri IPC (`invoke()` calls)

## Commands

```bash
npm run tauri dev        # Run in dev mode (hot reload frontend, recompile Rust on save)
npm run tauri build      # Production build
npm run lint             # ESLint (frontend) + Clippy (Rust)
npm run format           # Prettier (frontend) + rustfmt (Rust)
npm run format:check     # Check formatting without changes
npm run check            # TypeScript type checking
```

## Code Conventions

Please format code before committing (Prettier for frontend, rustfmt for Rust).

### Rust (src-tauri/)

- `commands/` is a thin adapter layer — business logic goes in core modules (`modrinth/`, `instance/`, `download/`, etc.)
- All fallible functions return `Result<T, GlowberryError>`
- Structs that cross the IPC boundary derive `Serialize` (and `Deserialize` if received from frontend)
- Use `State<'_, AppState>` to access shared state in commands
- Formatted with rustfmt (config in `src-tauri/rustfmt.toml`)
- Linted with Clippy (dead_code and unused_imports are allowed during development)

### Frontend (src/)

- React with hooks — state via `useState`, shared state via custom hooks in `src/hooks/`
- CSS Modules for scoped styles (`*.module.css` — Vite supports natively, no extra deps)
- IPC wrappers in `src/api/` — components never call `invoke()` directly
- Types in `src/types.ts` mirror Rust structs
- Page routing is state-based in `App.tsx` (`useState<Page>`)
- CSS variables for theming defined in `src/styles.css` `:root`
- Formatted with Prettier (config in `.prettierrc`)
- Linted with ESLint (config in `eslint.config.js`)
- `@/` path alias maps to `src/` (configured in vite.config.ts + tsconfig.json)

### Adding a new Tauri command

1. Write logic in the core module (e.g., `modrinth/api.rs`)
2. Create command handler in `commands/` using `#[tauri::command]`
3. Register in `lib.rs` `generate_handler![]`
4. Add TS wrapper in `src/api/`
5. Add types to `src/types.ts` if needed

### Branches and Git

- The `main` branch is protected — all changes must go through PRs
- Use feature branches named `feature/your-feature-name`
- Commit when you are done with a logical chunk of work
- Commit using `feat:`, `fix:`, `refactor:`, `chore:`, (Github supported) etc. for better commit history
