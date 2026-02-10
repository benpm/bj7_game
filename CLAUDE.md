# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Run Commands

```bash
# Native development (fast compile with dynamic linking)
cargo run --features dev

# Native release build
cargo run --release

# Run tests (all platforms in CI: Windows, Linux, macOS)
cargo test

# Run a single test
cargo test test_name

# Lint
cargo clippy --all-targets --all-features -- -Dwarnings

# Format check
cargo fmt --all -- --check

# Web development (requires: cargo install --locked trunk && rustup target add wasm32-unknown-unknown)
trunk serve              # Dev server on localhost:8080
trunk build --release    # Production Wasm build

# Mobile
cargo apk run -p mobile  # Android
cd mobile && make run     # iOS (requires Xcode)
```

## Architecture

This is a Bevy 0.18 game built on the [bevy_game_template](https://github.com/NiklasEi/bevy_game_template). It targets desktop, web (Wasm), Android, and iOS.

### Game State Machine

`GameState::Loading` → `GameState::Menu` ↔ `GameState::Playing`

State transitions gate which systems run. `OnEnter`/`OnExit` schedules handle setup/teardown per state. Escape key returns from Playing to Menu.

### Plugin Structure (src/lib.rs)

`GamePlugin` composes all subsystems as sub-plugins:

- **LoadingPlugin** (`loading.rs`) — Declarative asset loading via `bevy_asset_loader`. Defines `AudioAssets` and `TextureAssets` as `AssetCollection` resources. Transitions to Menu when done.
- **MenuPlugin** (`menu.rs`) — Main menu UI with Play and Exit buttons. Uses `webbrowser` crate for external links.
- **ActionsPlugin** (`actions/`) — Input abstraction layer. `Actions` resource holds `player_movement: Option<Vec2>` from WASD/arrows. Game systems read `Actions` instead of polling input directly.
- **InternalAudioPlugin** (`audio.rs`) — BGM via `bevy_kira_audio` (not Bevy's built-in audio). Loops `flying.ogg`, pauses when player idle.
- **PlayerPlugin** (`player.rs`) — First-person controller: `FpsController` component with mouse look, WASD movement relative to facing, basic gravity with ground collision. Camera3d spawned as child of player entity. Cursor locked during gameplay, released on exit.
- **WorldPlugin** (`world.rs`) — 3D scene: ground plane, directional light with shadows, scattered primitive objects. Setup/cleanup tied to Playing state.

### Key Dependencies

- **bevy_kira_audio** — Used instead of Bevy's built-in audio to avoid web/mobile conflicts. Bevy's audio features are explicitly excluded in Cargo.toml.
- **bevy_asset_loader** — Declarative asset collections with automatic loading states.

### Project Layout

- `src/` — Rust game code
- `mobile/` — Mobile platform crate (`staticlib`/`cdylib`), Android manifest, iOS Xcode project
- `godot_sketch/` — Godot 4.5 subproject for art/prototyping (Aseprite import via AsepriteWizard plugin)
  - Please ignore this directory
- `assets/audio/`, `assets/textures/` — Game assets
- `build/` — Platform-specific resources (icons, web styling)
- `.github/workflows/` — CI (test/lint/fmt) and release pipelines for all platforms

### Build Profiles

- **dev**: `opt-level = 1` for code, `opt-level = 3` for dependencies
- **release**: `opt-level = s`, LTO enabled, stripped
- **dist**: `opt-level = 3`, LTO, stripped (for distribution builds)

### Release Process

Tag with `v*.*.*` to trigger release builds. Outputs: Windows `.exe`+`.msi`, macOS universal `.dmg`, Linux binary, plus separate workflows for Android AAB and iOS TestFlight.
