# Bevy 0.18 Quick Reference

## Architecture

Bevy is a data-driven game engine built in Rust using an **Entity Component System (ECS)**. Everything is composed of:

- **Entities** - Unique IDs (lightweight handles, not objects)
- **Components** - Data attached to entities (plain structs with `#[derive(Component)]`)
- **Systems** - Functions that operate on components via queries
- **Resources** - Global singleton data (not tied to any entity)
- **Plugins** - Modular groups of systems, resources, and configuration

The app is structured as a pipeline of **Schedules** containing **Systems**, orchestrated by the `App` builder.

## App Setup

```rust
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)          // Window, renderer, input, assets, etc.
        .add_plugins(MyGamePlugin)            // Custom plugin
        .init_state::<GameState>()            // Register a state machine
        .insert_resource(MyResource::default())
        .add_systems(Startup, setup)
        .add_systems(Update, (movement, scoring))
        .add_systems(FixedUpdate, physics)
        .add_systems(OnEnter(GameState::Playing), spawn_level)
        .add_systems(OnExit(GameState::Playing), cleanup_level)
        .run();
}
```

## Components

```rust
#[derive(Component)]
struct Position { x: f32, y: f32 }

#[derive(Component)]
struct Velocity { x: f32, y: f32 }

#[derive(Component)]
struct Player;       // Marker component (no data, used for filtering)

#[derive(Component, Default)]
struct Health(f32);
```

## Bundles

Group components that are always spawned together.

```rust
#[derive(Bundle)]
struct PlayerBundle {
    player: Player,
    position: Position,
    velocity: Velocity,
}

// Spawn with a bundle
commands.spawn(PlayerBundle {
    player: Player,
    position: Position { x: 0.0, y: 0.0 },
    velocity: Velocity { x: 1.0, y: 0.0 },
});

// Or spawn with a tuple of components (no bundle needed)
commands.spawn((Player, Position { x: 0.0, y: 0.0 }, Velocity { x: 1.0, y: 0.0 }));
```

## Resources

Global singleton state, not tied to entities.

```rust
#[derive(Resource)]
struct Score(u32);

#[derive(Resource, Default)]
struct GameSettings { difficulty: u8 }

// Insert at app build time
app.insert_resource(Score(0));
app.init_resource::<GameSettings>();  // Uses Default::default()

// Access in systems
fn update_score(mut score: ResMut<Score>) {
    score.0 += 1;
}
fn read_settings(settings: Res<GameSettings>) {
    println!("Difficulty: {}", settings.difficulty);
}
```

## Systems

Functions that run game logic. Parameters are automatically injected by the ECS scheduler.

```rust
fn movement(mut query: Query<(&mut Position, &Velocity)>, time: Res<Time>) {
    for (mut pos, vel) in &mut query {
        pos.x += vel.x * time.delta_secs();
        pos.y += vel.y * time.delta_secs();
    }
}
```

### System Parameter Types

| Parameter | Description |
|---|---|
| `Query<D, F>` | Fetch component data `D` with optional filter `F` |
| `Res<T>` | Shared (read-only) resource borrow. Panics if missing. |
| `ResMut<T>` | Exclusive (mutable) resource borrow. Panics if missing. |
| `Option<Res<T>>` | Resource that might not exist |
| `Commands` | Queue spawn/despawn/insert/remove (deferred until sync point) |
| `Local<T>` | System-private persistent state (survives across calls) |
| `MessageReader<M>` | Read buffered messages (formerly `EventReader` before 0.17) |
| `MessageWriter<M>` | Write buffered messages (formerly `EventWriter` before 0.17) |
| `ParamSet<(P1, P2)>` | Resolve conflicting access (e.g. two mutable queries on same component) |
| `RemovedComponents<T>` | Track entities that had component `T` removed |
| `Single<D, F>` | Like `Query` but expects exactly one matching entity |

### Query Filters

```rust
// With/Without marker filtering
fn player_system(query: Query<&mut Transform, With<Player>>) { ... }
fn non_enemy(query: Query<&Health, Without<Enemy>>) { ... }

// Changed/Added detection
fn on_health_change(query: Query<&Health, Changed<Health>>) { ... }
fn on_spawn(query: Query<&Player, Added<Player>>) { ... }

// Combining filters
fn complex(query: Query<&Transform, (With<Player>, Without<Dead>)>) { ... }

// Optional components
fn maybe_has_shield(query: Query<(&Player, Option<&Shield>)>) { ... }
```

### Entity Access

```rust
// Get a specific entity's components
if let Ok(transform) = query.get(entity) { ... }
if let Ok(mut transform) = query.get_mut(entity) { ... }

// Single entity (panics if not exactly one)
let player_transform = query.single();

// Iterate
for (entity, transform, velocity) in &query { ... }
for mut transform in &mut query { ... }
```

## Commands

Deferred world mutations (applied at next sync point).

```rust
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Spawn entity
    let entity = commands.spawn((
        Player,
        Transform::default(),
        Sprite::from_image(asset_server.load("player.png")),
    )).id();

    // Despawn
    commands.entity(entity).despawn();

    // Add/remove components on existing entity
    commands.entity(entity).insert(Velocity { x: 1.0, y: 0.0 });
    commands.entity(entity).remove::<Velocity>();

    // Insert resource
    commands.insert_resource(Score(0));
}
```

## Schedules

Systems are organized into schedules that run in a defined order each frame.

### Startup (Run Once)
| Schedule | When |
|---|---|
| `PreStartup` | Before `Startup` |
| `Startup` | Once at app init. Setup/spawning goes here. |
| `PostStartup` | After `Startup` |

### Main Loop (Every Frame)
| Schedule | When |
|---|---|
| `First` | Before everything else |
| `PreUpdate` | Before game logic. Bevy internals (input processing) run here. |
| `StateTransition` | State machine transitions |
| `Update` | **Standard game logic goes here.** |
| `PostUpdate` | After game logic. Bevy internals (transform propagation, rendering) run here. |
| `Last` | After everything else |

### Fixed Timestep (Deterministic Rate)
| Schedule | When |
|---|---|
| `FixedUpdate` | **Physics, AI, networking.** Runs at fixed rate independent of framerate. |
| `FixedFirst` / `FixedPreUpdate` / `FixedPostUpdate` / `FixedLast` | Fixed-rate equivalents of the main loop schedules. |

### State Transitions
| Schedule | When |
|---|---|
| `OnEnter(state)` | When entering a state |
| `OnExit(state)` | When leaving a state |
| `OnTransition(state)` | During a state transition |

### System Ordering

```rust
app.add_systems(Update, (
    input_system,
    movement_system.after(input_system),
    collision_system.after(movement_system),
));

// Or use chain() for sequential ordering
app.add_systems(Update, (input_system, movement_system, collision_system).chain());

// Run conditions
app.add_systems(Update, pause_menu.run_if(in_state(GameState::Paused)));
```

## States

Finite state machine for game flow control.

```rust
#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    #[default]
    Loading,
    Menu,
    Playing,
    Paused,
}

// Register
app.init_state::<GameState>();

// Transition in a system
fn start_game(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::Playing);
}

// Systems that only run in a specific state
app.add_systems(Update, game_logic.run_if(in_state(GameState::Playing)));

// Setup/teardown on state transitions
app.add_systems(OnEnter(GameState::Playing), spawn_level);
app.add_systems(OnExit(GameState::Playing), despawn_level);
```

## Messages (Buffered Events)

As of Bevy 0.17+, buffered events are called **Messages** (`MessageReader`/`MessageWriter`). The term "Event" now refers exclusively to observer-triggered events.

```rust
#[derive(Message)]  // was #[derive(Event)] before 0.17
struct Explosion { position: Vec2, damage: f32 }

// Send
fn create_explosion(mut writer: MessageWriter<Explosion>) {
    writer.write(Explosion { position: Vec2::ZERO, damage: 50.0 });
}

// Receive
fn handle_explosions(mut reader: MessageReader<Explosion>) {
    for explosion in reader.read() {
        println!("Boom at {:?}, damage: {}", explosion.position, explosion.damage);
    }
}

// Register
app.add_message::<Explosion>();  // was add_event before 0.17
```

## Observers (Triggered Events)

Observers respond to events targeted at specific entities (or global triggers). These run immediately, not buffered.

```rust
#[derive(Event)]  // Observers still use #[derive(Event)]
struct OnDeath;

// Attach observer to entity
commands.spawn((Player, Health(100.0)))
    .observe(|trigger: Trigger<OnDeath>, mut commands: Commands| {
        commands.entity(trigger.target()).despawn();
    });

// Trigger the event
commands.trigger_targets(OnDeath, player_entity);
```

## Input Handling

```rust
// Keyboard (polling)
fn keyboard(input: Res<ButtonInput<KeyCode>>) {
    if input.just_pressed(KeyCode::Space) { /* jump */ }
    if input.pressed(KeyCode::KeyW) { /* move forward */ }
    if input.just_released(KeyCode::Escape) { /* pause */ }
}

// Mouse button (polling)
fn mouse_button(input: Res<ButtonInput<MouseButton>>) {
    if input.just_pressed(MouseButton::Left) { /* shoot */ }
}

// Mouse motion (accumulated per frame)
fn mouse_look(motion: Res<AccumulatedMouseMotion>) {
    if motion.delta != Vec2::ZERO {
        let dx = motion.delta.x;
        let dy = motion.delta.y;
    }
}

// Mouse scroll
fn zoom(scroll: Res<AccumulatedMouseScroll>) {
    let zoom_delta = scroll.delta.y;
}

// Keyboard events (message-based)
fn text_input(mut reader: MessageReader<KeyboardInput>) {
    for event in reader.read() {
        if event.state.is_pressed() {
            match (&event.logical_key, &event.text) {
                (Key::Enter, _) => { /* submit */ }
                (_, Some(text)) => { /* typed text */ }
                _ => {}
            }
        }
    }
}

// Gamepad
fn gamepad(input: Res<ButtonInput<GamepadButton>>) {
    if input.just_pressed(GamepadButton::South) { /* A button */ }
}
```

## Assets

```rust
// Load assets via AssetServer
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let texture: Handle<Image> = asset_server.load("textures/player.png");
    let font: Handle<Font> = asset_server.load("fonts/main.ttf");
    let sound: Handle<AudioSource> = asset_server.load("audio/bgm.ogg");
}

// Access loaded assets
fn use_texture(images: Res<Assets<Image>>, handle: Res<MyTextureHandle>) {
    if let Some(image) = images.get(&handle.0) { ... }
}
```

### bevy_asset_loader (used in this project)

```rust
#[derive(AssetCollection, Resource)]
struct TextureAssets {
    #[asset(path = "textures/bevy.png")]
    bevy: Handle<Image>,
    #[asset(path = "textures/github.png")]
    github: Handle<Image>,
}

// Automatically loads during a loading state
app.add_loading_state(
    LoadingState::new(GameState::Loading)
        .continue_to_state(GameState::Menu)
        .load_collection::<TextureAssets>()
);
```

## 2D Rendering

```rust
fn setup_2d(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Camera
    commands.spawn(Camera2d);

    // Sprite
    commands.spawn(Sprite::from_image(asset_server.load("player.png")));

    // Sprite with transform
    commands.spawn((
        Sprite::from_image(asset_server.load("enemy.png")),
        Transform::from_xyz(100.0, 50.0, 0.0),
    ));

    // Colored sprite (no texture)
    commands.spawn((
        Sprite::from_color(Color::srgb(1.0, 0.0, 0.0), Vec2::new(50.0, 50.0)),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
}
```

## 3D Rendering

```rust
fn setup_3d(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Light
    commands.spawn((
        PointLight { shadows_enabled: true, ..default() },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));

    // Mesh
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.8, 0.2, 0.2))),
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));

    // Ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(5.0, 5.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
    ));
}
```

## Transforms

```rust
// Position, rotation, scale
Transform::from_xyz(10.0, 5.0, 0.0)
Transform::from_translation(Vec3::new(10.0, 5.0, 0.0))
Transform::from_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_4))
Transform::from_scale(Vec3::splat(2.0))

// Chaining
Transform::from_xyz(0.0, 0.0, 0.0)
    .with_rotation(Quat::from_rotation_y(1.0))
    .with_scale(Vec3::splat(0.5))

// Useful methods
transform.translation += Vec3::new(1.0, 0.0, 0.0);
transform.rotate_z(0.1);
transform.look_at(target_pos, Vec3::Y);
let forward: Dir3 = transform.forward();
let right: Dir3 = transform.right();
```

## Plugins

```rust
pub struct MyPlugin;

impl Plugin for MyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
           .add_systems(Update, update)
           .insert_resource(MyResource::default())
           .add_message::<MyMessage>();
    }
}

// Use it
app.add_plugins(MyPlugin);
```

## UI (bevy_ui)

Flexbox/Grid layout powered by Taffy. Uses ECS nodes.

```rust
fn setup_ui(mut commands: Commands) {
    commands.spawn(Camera2d);

    // Root node
    commands.spawn(Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        flex_direction: FlexDirection::Column,
        ..default()
    }).with_children(|parent| {
        // Text
        parent.spawn((
            Text::new("Hello Bevy!"),
            TextFont { font_size: 40.0, ..default() },
            TextColor(Color::WHITE),
        ));

        // Button
        parent.spawn((
            Button,
            Node {
                width: Val::Px(200.0),
                height: Val::Px(65.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
        )).with_children(|parent| {
            parent.spawn((
                Text::new("Play"),
                TextFont { font_size: 30.0, ..default() },
                TextColor(Color::WHITE),
            ));
        });
    });
}

// Button interaction
fn button_system(
    mut query: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<Button>)>,
) {
    for (interaction, mut bg) in &mut query {
        match *interaction {
            Interaction::Pressed => { bg.0 = Color::srgb(0.35, 0.75, 0.35); }
            Interaction::Hovered => { bg.0 = Color::srgb(0.25, 0.25, 0.25); }
            Interaction::None => { bg.0 = Color::srgb(0.15, 0.15, 0.15); }
        }
    }
}
```

## Timer

```rust
#[derive(Component)]
struct SpawnTimer(Timer);

fn setup(mut commands: Commands) {
    commands.spawn(SpawnTimer(Timer::from_seconds(2.0, TimerMode::Repeating)));
}

fn tick_timer(time: Res<Time>, mut query: Query<&mut SpawnTimer>) {
    for mut timer in &mut query {
        timer.0.tick(time.delta());
        if timer.0.just_finished() {
            // Spawn something every 2 seconds
        }
    }
}
```

## Audio (bevy_kira_audio)

This project uses `bevy_kira_audio` instead of Bevy's built-in audio (they clash).

```rust
use bevy_kira_audio::prelude::*;

fn play_bgm(audio: Res<Audio>, asset_server: Res<AssetServer>) {
    audio.play(asset_server.load("audio/bgm.ogg"))
        .looped()
        .with_volume(Volume::Amplitude(0.5));
}

fn play_sfx(audio: Res<Audio>, asset_server: Res<AssetServer>) {
    audio.play(asset_server.load("audio/explosion.ogg"));
}
```

## Bevy 0.18 Notable Features

- **BSN (`bsn!` macro)**: New scene/UI composition system with Rust-native syntax, IDE support (go-to-def, autocomplete), and template inheritance. File-based `.bsn` asset loader planned for a later release.
- **Solari Raytracer**: Specular reflections, physically-based soft shadows, ReSTIR DI/GI for near-offline-quality pathtraced lighting at real-time rates.
- **Atmosphere System**: Procedural atmosphere via `ScatteringMedium` asset (desert skies, alien planets, fog).
- **Built-in Camera Controllers**: `FreeCamera` (3D fly cam) and `PanCamera` (2D WASD + scroll zoom).
- **UI**: `Popover`, `MenuPopup`, `AutoDirectionalNavigation` (gamepad/keyboard nav), `IgnoreScroll`, pickable text sections.
- **Text**: Variable font weight (`FontWeight`), strikethrough/underline, OpenType features (ligatures, small caps).
- **FullscreenMaterial**: Simplified fullscreen post-processing shaders.
- **Safe mutable component access**: `get_components_mut()` with runtime aliasing checks, eliminating `unsafe` in most cases.
- **System removal**: `remove_systems_in_set()` for zero-overhead system disabling.
- **Bevy Feathers**: Opinionated widget toolkit for editors/tools (not for in-game UI).

## Common Third-Party Crates

| Crate | Purpose |
|---|---|
| `bevy_kira_audio` | Audio (ogg/mp3/flac/wav, channels, web support) |
| `bevy_asset_loader` | Declarative asset loading with `AssetCollection` |
| `avian2d` / `avian3d` | ECS-native physics (recommended over Rapier for Bevy) |
| `bevy_rapier2d` / `bevy_rapier3d` | Rapier physics integration (more mature) |
| `leafwing-input-manager` | Input-to-action mapping |
| `bevy_egui` | egui immediate-mode GUI integration |
| `bevy-inspector-egui` | Runtime entity/resource inspector |
| `bevy_hanabi` | GPU particle effects |
| `bevy_tweening` | Tween/interpolation animation |
| `bevy_common_assets` | Custom asset loaders (JSON, RON, TOML, YAML) |
| `bevy_embedded_assets` | Embed assets in binary |
| `bevy_prototype_lyon` | 2D shape drawing |
| `bevy-tnua` | Character controller (works with Avian or Rapier) |
| `lightyear` / `bevy_replicon` | Multiplayer networking |

## Performance Tips

- Use `--features dev` (enables `bevy/dynamic_linking`) for fast compile times during development.
- `opt-level = 3` on dependencies, `opt-level = 1` on your code (already configured in this project's `Cargo.toml`).
- Use `FixedUpdate` for deterministic simulation logic (physics, game rules).
- Marker components (`struct Player;`) are cheap and enable efficient query filtering.
- Prefer `Changed<T>` and `Added<T>` filters to avoid processing unchanged entities.
- Use `Commands` for structural changes (spawn/despawn); direct `Query` mutation for data changes.

## This Project's Structure

```
src/
  main.rs           - Entry point, DefaultPlugins + window config
  lib.rs            - GamePlugin, GameState enum (Loading/Menu/Playing)
  loading.rs        - Asset loading with bevy_asset_loader
  menu.rs           - Main menu UI
  actions/
    mod.rs          - Input actions system
    game_control.rs - Keyboard control definitions
  audio.rs          - Audio plugin (bevy_kira_audio)
  player.rs         - Player entity and movement
mobile/
  src/lib.rs        - Mobile platform support
```

**State flow**: `Loading` (assets) -> `Menu` (play button) -> `Playing` (gameplay)
