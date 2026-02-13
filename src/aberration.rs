use crate::GameState;
use crate::actor::{Actor, ActorIntent, GROUND_Y};
use crate::loading::TextureAssets;
use crate::player::Player;
use bevy::prelude::*;
use rand::Rng;

const MAX_ABERRATIONS: usize = 5;
const SPAWN_MIN_SECS: f32 = 5.0;
const SPAWN_MAX_SECS: f32 = 10.0;
const SPAWN_MIN_DIST: f32 = 8.0;
const SPAWN_MAX_DIST: f32 = 18.0;
const SPAWN_HALF_ANGLE: f32 = std::f32::consts::FRAC_PI_4; // ±45° from look dir
const SPAWN_ANIM_SECS: f32 = 0.5;
const SENSITIVITY_DURING_SPAWN: f32 = 0.15; // multiplied onto normal sensitivity

pub struct AberrationPlugin;

impl Plugin for AberrationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), init_spawn_timer)
            .add_systems(
                Update,
                (spawn_aberration_periodic, aberration_face_player, animate_spawn)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(OnExit(GameState::Playing), cleanup_aberrations);
    }
}

/// Marker for aberration enemies.
#[derive(Component)]
pub struct Aberration;

/// Tracks the spawn-in scale animation.
#[derive(Component)]
pub struct SpawnAnimation {
    timer: Timer,
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

fn init_spawn_timer(mut commands: Commands) {
    commands.insert_resource(AberrationSpawnTimer {
        timer: Timer::from_seconds(random_spawn_delay(), TimerMode::Once),
    });
}

fn spawn_aberration_periodic(
    mut commands: Commands,
    time: Res<Time>,
    mut spawn_timer: ResMut<AberrationSpawnTimer>,
    textures: Res<TextureAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    aberration_query: Query<(), With<Aberration>>,
    player_query: Query<(&Transform, &Actor), With<Player>>,
) {
    spawn_timer.timer.tick(time.delta());
    if spawn_timer.timer.fraction() < 1.0 {
        return;
    }

    if aberration_query.iter().count() >= MAX_ABERRATIONS {
        // Reset timer and try again next tick
        spawn_timer.timer = Timer::from_seconds(1.0, TimerMode::Once);
        return;
    }

    let Ok((player_tf, player_actor)) = player_query.single() else {
        return;
    };

    // Pick a random position inside the player's forward cone
    let mut rng = rand::rng();
    let angle_offset = rng.random_range(-SPAWN_HALF_ANGLE..SPAWN_HALF_ANGLE);
    let spawn_yaw = player_actor.yaw + angle_offset;
    let dist = rng.random_range(SPAWN_MIN_DIST..SPAWN_MAX_DIST);

    let (sin_yaw, cos_yaw) = spawn_yaw.sin_cos();
    let spawn_pos = Vec3::new(
        player_tf.translation.x - sin_yaw * dist,
        GROUND_Y + 1.0, // aberration height / 2
        player_tf.translation.z - cos_yaw * dist,
    );

    let texture_index = rng.random_range(0..textures.aberrations.len());
    let texture = textures.aberrations[texture_index].clone();
    let quad = meshes.add(Rectangle::new(2.0, 2.0));

    commands.spawn((
        Mesh3d(quad),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(texture),
            alpha_mode: AlphaMode::Mask(0.5),
            unlit: true,
            cull_mode: None,
            ..default()
        })),
        Transform::from_translation(spawn_pos).with_scale(Vec3::new(0.0, 1.0, 1.0)),
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

    // Reset timer for next spawn
    spawn_timer.timer = Timer::from_seconds(random_spawn_delay(), TimerMode::Once);
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

fn cleanup_aberrations(
    mut commands: Commands,
    query: Query<Entity, With<Aberration>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<AberrationSpawnTimer>();
}
