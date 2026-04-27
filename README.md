<img src="./assets/glowberry.png" alt="Project icon" width="20%" align="right">

# Glowberry

A simple, fast Minecraft launcher for managing modpacks with a streamlined feature set and minimalist UI. Created as an alternative launcher for users who prefer a lighter approach to modpack management. Currently hard locked to a singular Modrinth modpack (my own, [Starlight](https://modrinth.com/modpack/starlightmodpack)), but future updates will add support for more packs from Modrinth or local `mrpack` files.

## Features

- **Simple UI** - clean dark interface, zero jargon
- **Modrinth integration** - search and install modpacks directly
- **mrpack support** - full Modrinth modpack format support
- **Self-updates** - updates Glowberry from the latest GitHub release
- **Smart updates** - one-click modpack updates that actually work
- **File locks** - protect your keybinds, settings, and configs from being overwritten during pack updates
- **Cross-platform** - macOS and Windows
- **Lightweight** - ~10MB binary, instant startup

> Glowberry self-updates use Tauri's updater plugin, so release builds must be signed and the updater public key in [src-tauri/tauri.conf.json](src-tauri/tauri.conf.json) must match the signing key used in GitHub Actions.

## Releasing updates

1. Generate a signing key pair with `npm run tauri signer generate -- -w ~/.tauri/glowberry.key`.
2. Copy the public key into [src-tauri/tauri.conf.json](src-tauri/tauri.conf.json).
3. Add the private key contents to GitHub Secrets as `TAURI_SIGNING_PRIVATE_KEY`.
4. If the key was created with a password, also add `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`.

Glowberry's release workflow uploads `latest.json` and updater signatures automatically.
