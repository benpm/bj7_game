use crate::GameState;
use crate::actor::{Actor, ActorIntent, GROUND_Y};
use crate::dialog::Npc;
use crate::pause::game_not_paused;
use crate::player::Player;
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, MeshVertexBufferLayoutRef, PrimitiveTopology};
use bevy::pbr::{MaterialPipeline, MaterialPipelineKey};
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup, RenderPipelineDescriptor, SpecializedMeshPipelineError,
};
use bevy::shader::ShaderRef;
use rand::Rng;
use serde::Deserialize;

const MAX_ABERRATIONS: usize = 5;
const SPAWN_MIN_SECS: f32 = 5.0;
const SPAWN_MAX_SECS: f32 = 10.0;
const SPAWN_MIN_DIST: f32 = 8.0;
const SPAWN_MAX_DIST: f32 = 18.0;
const SPAWN_HALF_ANGLE: f32 = std::f32::consts::FRAC_PI_4; // ±45° from look dir
const SPAWN_ANIM_SECS: f32 = 0.5;
const SENSITIVITY_DURING_SPAWN: f32 = 0.15; // multiplied onto normal sensitivity
const DISTANCE_SCALE_NEAR: f32 = 5.0;
const DISTANCE_SCALE_FAR: f32 = 50.0;
const KILL_COUNTDOWN_SECS: f32 = 5.0;
const KILL_PROXIMITY: f32 = 3.0;
const MAX_SHAKE_INTENSITY: f32 = 0.3;

const ABERRATION_TYPES_RON: &str = include_str!("../assets/defs/types.ron");

#[derive(Asset, TypePath, AsBindGroup, Clone)]
struct AberrationMaterial {
    #[texture(0)]
    #[sampler(1)]
    base_texture: Option<Handle<Image>>,
}

impl Material for AberrationMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/aberration_distort.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Mask(0.5)
    }

    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = None;
        Ok(())
    }
}

pub struct AberrationPlugin;

impl Plugin for AberrationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<AberrationMaterial>::default())
            .add_systems(OnEnter(GameState::Playing), init_aberrations)
            .add_systems(
                Update,
                (
                    spawn_aberration_periodic,
                    aberration_face_player,
                    aberration_distance_scale,
                    animate_spawn,
                    kill_countdown_proximity,
                    kill_countdown_tick,
                )
                    .run_if(in_state(GameState::Playing).and(game_not_paused)),
            )
            .add_systems(OnExit(GameState::Playing), cleanup_aberrations);
    }
}

// --- RON data structures ---

#[derive(Deserialize)]
struct AberrationTypesRon {
    types: Vec<AberrationTypeRon>,
}

#[derive(Deserialize)]
struct AberrationTypeRon {
    layers: Vec<LayerRon>,
    #[serde(default = "default_size")]
    size: f32,
    #[serde(default)]
    npc: bool,
}

fn default_size() -> f32 {
    2.0
}

#[derive(Deserialize)]
struct LayerRon {
    texture: String,
    columns: u32,
}

// --- Runtime resources ---

#[derive(Resource)]
struct AberrationTypes(Vec<AberrationTypeDef>);

struct AberrationTypeDef {
    layers: Vec<LayerDef>,
    size: f32,
    npc: bool,
}

struct LayerDef {
    texture: Handle<Image>,
    columns: u32,
}

/// Marker for aberration enemies.
#[derive(Component)]
pub struct Aberration;

/// Tracks the spawn-in scale animation.
#[derive(Component)]
pub struct SpawnAnimation {
    timer: Timer,
}

/// Kill countdown: when a non-NPC aberration is approached, counts down to game over.
#[derive(Component)]
struct KillCountdown {
    timer: Timer,
    base_pos: Vec3,
}

/// Returns a sensitivity multiplier (< 1.0 during spawn animations, 1.0 otherwise).
pub fn spawn_sensitivity_factor(active: usize) -> f32 {
    if active > 0 {
        SENSITIVITY_DURING_SPAWN
    } else {
        1.0
    }
}

#[derive(Resource)]
struct AberrationSpawnTimer {
    timer: Timer,
}

fn random_spawn_delay() -> f32 {
    rand::rng().random_range(SPAWN_MIN_SECS..SPAWN_MAX_SECS)
}

/// Build a quad mesh showing a single frame from a horizontal sprite sheet.
fn sprite_frame_quad(width: f32, height: f32, columns: u32, frame: u32) -> Mesh {
    let u_min = frame as f32 / columns as f32;
    let u_max = (frame + 1) as f32 / columns as f32;

    let hw = width / 2.0;
    let hh = height / 2.0;

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![
            [-hw, -hh, 0.0],
            [hw, -hh, 0.0],
            [hw, hh, 0.0],
            [-hw, hh, 0.0],
        ],
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 0.0, 1.0]; 4])
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_UV_0,
        vec![[u_min, 1.0], [u_max, 1.0], [u_max, 0.0], [u_min, 0.0]],
    )
    .with_inserted_indices(Indices::U32(vec![0, 1, 2, 0, 2, 3]))
}

fn init_aberrations(mut commands: Commands, asset_server: Res<AssetServer>) {
    let data: AberrationTypesRon =
        ron::from_str(ABERRATION_TYPES_RON).expect("Failed to parse aberration types.ron");

    let types = data
        .types
        .into_iter()
        .map(|t| AberrationTypeDef {
            size: t.size,
            npc: t.npc,
            layers: t
                .layers
                .into_iter()
                .map(|l| LayerDef {
                    texture: asset_server.load(&l.texture),
                    columns: l.columns,
                })
                .collect(),
        })
        .collect();

    commands.insert_resource(AberrationTypes(types));
    commands.insert_resource(AberrationSpawnTimer {
        timer: Timer::from_seconds(random_spawn_delay(), TimerMode::Once),
    });
}

fn spawn_aberration_periodic(
    mut commands: Commands,
    time: Res<Time>,
    mut spawn_timer: ResMut<AberrationSpawnTimer>,
    types: Res<AberrationTypes>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<AberrationMaterial>>,
    aberration_query: Query<(), With<Aberration>>,
    player_query: Query<(&Transform, &Actor), With<Player>>,
) {
    spawn_timer.timer.tick(time.delta());
    if spawn_timer.timer.fraction() < 1.0 {
        return;
    }

    if aberration_query.iter().count() >= MAX_ABERRATIONS {
        spawn_timer.timer = Timer::from_seconds(1.0, TimerMode::Once);
        return;
    }

    let Ok((player_tf, player_actor)) = player_query.single() else {
        return;
    };

    if types.0.is_empty() {
        return;
    }

    let mut rng = rand::rng();
    let angle_offset = rng.random_range(-SPAWN_HALF_ANGLE..SPAWN_HALF_ANGLE);
    let spawn_yaw = player_actor.yaw + angle_offset;
    let dist = rng.random_range(SPAWN_MIN_DIST..SPAWN_MAX_DIST);

    let (sin_yaw, cos_yaw) = spawn_yaw.sin_cos();
    let spawn_pos = Vec3::new(
        player_tf.translation.x - sin_yaw * dist,
        GROUND_Y + 1.0,
        player_tf.translation.z - cos_yaw * dist,
    );

    let type_def = &types.0[rng.random_range(0..types.0.len())];

    let mut entity_cmd = commands.spawn((
        Transform::from_translation(spawn_pos).with_scale(Vec3::new(0.0, 1.0, 1.0)),
        Visibility::default(),
        Aberration,
        SpawnAnimation {
            timer: Timer::from_seconds(SPAWN_ANIM_SECS, TimerMode::Once),
        },
        Actor {
            speed: 0.0,
            height: 1.0,
            yaw: 0.0,
            vertical_velocity: 0.0,
            grounded: true,
        },
        ActorIntent::default(),
    ));

    if type_def.npc {
        entity_cmd.insert(Npc { range: KILL_PROXIMITY });
    }

    entity_cmd.with_children(|parent| {
            for (i, layer) in type_def.layers.iter().enumerate() {
                let frame = rng.random_range(0..layer.columns);
                let quad = meshes.add(sprite_frame_quad(type_def.size, type_def.size, layer.columns, frame));
                parent.spawn((
                    Mesh3d(quad),
                    MeshMaterial3d(materials.add(AberrationMaterial {
                        base_texture: Some(layer.texture.clone()),
                    })),
                    Transform::from_xyz(0.0, 0.0, i as f32 * 0.01),
                ));
            }
        });

    spawn_timer.timer = Timer::from_seconds(random_spawn_delay(), TimerMode::Once);
}

fn kill_countdown_proximity(
    mut commands: Commands,
    player_q: Query<&GlobalTransform, With<Player>>,
    aberration_q: Query<
        (Entity, &GlobalTransform),
        (With<Aberration>, Without<Npc>, Without<KillCountdown>),
    >,
) {
    let Ok(player_tf) = player_q.single() else {
        return;
    };
    let player_pos = player_tf.translation();

    for (entity, ab_tf) in &aberration_q {
        let dist = player_pos.distance(ab_tf.translation());
        if dist <= KILL_PROXIMITY {
            commands.entity(entity).insert(KillCountdown {
                timer: Timer::from_seconds(KILL_COUNTDOWN_SECS, TimerMode::Once),
                base_pos: ab_tf.translation(),
            });
        }
    }
}

fn kill_countdown_tick(
    mut commands: Commands,
    time: Res<Time>,
    mut next_state: ResMut<NextState<GameState>>,
    player_q: Query<&GlobalTransform, With<Player>>,
    mut query: Query<(Entity, &GlobalTransform, &mut Transform, &mut KillCountdown), With<Aberration>>,
) {
    let Ok(player_tf) = player_q.single() else {
        return;
    };
    let player_pos = player_tf.translation();

    for (entity, global_tf, mut transform, mut countdown) in &mut query {
        let dist = player_pos.distance(global_tf.translation());

        // Cancel countdown if player moves away
        if dist > KILL_PROXIMITY * 2.0 {
            transform.translation = countdown.base_pos;
            commands.entity(entity).remove::<KillCountdown>();
            continue;
        }

        countdown.timer.tick(time.delta());
        let progress = countdown.timer.fraction(); // 0.0 → 1.0

        // Shake intensity increases as countdown progresses
        let intensity = progress * progress * MAX_SHAKE_INTENSITY;
        let mut rng = rand::rng();
        let shake_x = rng.random_range(-intensity..intensity);
        let shake_z = rng.random_range(-intensity..intensity);
        transform.translation.x = countdown.base_pos.x + shake_x;
        transform.translation.z = countdown.base_pos.z + shake_z;

        if progress >= 1.0 {
            // Game over — return to menu
            next_state.set(GameState::Menu);
            return;
        }
    }
}

fn aberration_distance_scale(
    player_q: Query<&GlobalTransform, With<Player>>,
    mut aberration_q: Query<(&GlobalTransform, &mut Transform), (With<Aberration>, Without<SpawnAnimation>)>,
) {
    let Ok(player_tf) = player_q.single() else {
        return;
    };
    let player_pos = player_tf.translation();

    for (global_tf, mut transform) in &mut aberration_q {
        let dist = player_pos.distance(global_tf.translation());
        let t = 1.0 - ((dist - DISTANCE_SCALE_NEAR) / (DISTANCE_SCALE_FAR - DISTANCE_SCALE_NEAR)).clamp(0.0, 1.0);
        transform.scale = Vec3::splat(t);
    }
}

fn aberration_face_player(
    player_query: Query<&GlobalTransform, With<Player>>,
    mut aberration_query: Query<(&mut Actor, &Transform), With<Aberration>>,
) {
    let Ok(player_global) = player_query.single() else {
        return;
    };
    let player_pos = player_global.translation();

    for (mut actor, transform) in &mut aberration_query {
        let dir = player_pos - transform.translation;
        actor.yaw = dir.x.atan2(dir.z);
    }
}

fn animate_spawn(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut SpawnAnimation)>,
) {
    for (entity, mut transform, mut anim) in &mut query {
        anim.timer.tick(time.delta());
        let t = anim.timer.fraction();
        transform.scale.x = t;
        if t >= 1.0 {
            transform.scale.x = 1.0;
            commands.entity(entity).remove::<SpawnAnimation>();
        }
    }
}

fn cleanup_aberrations(mut commands: Commands, query: Query<Entity, With<Aberration>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<AberrationSpawnTimer>();
    commands.remove_resource::<AberrationTypes>();
}
