# AGENTS.md

This file provides guidance to Claude Code, Copilot, Gemini CLI, etc.

## General Instructions
- Use context7 MCP to search for Bevy API or rust things
- Update and revise this file as needed, be concise though
- Other important files:
  - BEVY.md - update as needed, bevy info
  - TODO.md - if asked to work on TODOs, read this file. otherwise, you don't need to check it.
  - README.md - game design doc, general info, update as needed

## Commands
- Build: Use `bevy build` or `cargo build` if bevy CLI isn't available.
- Run: `bevy run`, or `cargo run`
- Check: When working on tasks, it's typically sufficient to just run `bevy lint` or `cargo check`

## Architecture

This is a Bevy 0.18 game built on the [bevy_game_template](https://github.com/NiklasEi/bevy_game_template). It targets desktop, web (Wasm), Android, and iOS.

### Game State Machine

`GameState::Loading` → `GameState::Menu` ↔ `GameState::Playing`

State transitions gate which systems run. `OnEnter`/`OnExit` schedules handle setup/teardown per state. Escape key returns from Playing to Menu.

### Plugin Structure (src/lib.rs)

`GamePlugin` composes all subsystems as sub-plugins:

- **MenuPlugin** (`menu.rs`) — Main menu UI with Play and Exit buttons. Uses `webbrowser` crate for external links.
- **ActionsPlugin** (`actions/`) — Input abstraction layer. `Actions` resource holds `player_movement: Option<Vec2>` from WASD/arrows. Game systems read `Actions` instead of polling input directly.
- **AberrationPlugin** (`aberration.rs`) — Billboard sprite enemies. Spawns quad meshes with aberration textures at scattered positions. Billboard system rotates sprites to face player (Y-axis only). Uses `AlphaMode::Mask`, unlit, double-sided.
- **DispelPlugin** (`dispel.rs`) — Draw-to-banish mechanic. Left click enters dispel mode (camera locks, cursor shown). Click+hold draws segmented line. Closing the loop dispels aberrations whose screen-space centroid is inside the polygon. Right click cancels. Uses Camera2d overlay for gizmo rendering.
- **HealthPlugin** (`health.rs`) — Health/sanity resource (0.0–1.0) with passive drain. White vignette overlay at `GlobalZIndex(50)` scales with health loss.
- **EnvironmentPlugin** (`environment.rs`) — `Environment` SubState under `GameState::Playing` (Delirium/Dissociation/Hypervigilance). 60s cycle timer with transition overlay at `GlobalZIndex(60)`. 5-minute run timer.
- **PalettePlugin** (`palette.rs`) — Post-processing shader via `FullscreenMaterial`. Quantizes rendered output to 3-color palette (black, dark grey, white) based on luminance. Shader at `assets/shaders/palette_quantize.wgsl`.
- **PlayerPlugin** (`player.rs`) — First-person controller: `FpsController` component with mouse look, WASD movement relative to facing, basic gravity with ground collision. Camera3d spawned as child of player entity with `PaletteQuantize` component. Cursor locked during gameplay, released on exit.
- **WorldPlugin** (`world.rs`) — 3D scene: ground plane, directional light with shadows, scattered primitive objects. Setup/cleanup tied to Playing state.

### Key Dependencies

Check `Cargo.toml` for more

- **bevy_kira_audio** — Used instead of Bevy's built-in audio to avoid web/mobile conflicts. Bevy's audio features are explicitly excluded in Cargo.toml.
- **bevy_asset_loader** — Declarative asset collections with automatic loading states.

### Project Layout

- `src/` — Rust game code
- `assets/audio/`, `assets/textures/`, `assets/vector_sprites` — Game assets
- `assets/shaders/` — WGSL shaders (palette quantize post-process)
- `build/` — Platform-specific resources (icons, web styling)
- `.github/workflows/` — CI (test/lint/fmt) and release pipelines for all platforms
