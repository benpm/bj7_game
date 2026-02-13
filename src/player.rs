use crate::GameState;
use crate::aberration::{SpawnAnimation, spawn_sensitivity_factor};
use crate::actions::Actions;
use crate::actor::{Actor, ActorIntent, GROUND_Y};
use crate::palette::PaletteSqueeze;
use crate::pause::game_not_paused;
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), (spawn_player, grab_cursor))
            .add_systems(
                Update,
                (player_mouse_look, player_movement_input)
                    .chain()
                    .run_if(in_state(GameState::Playing).and(game_not_paused)),
            )
            .add_systems(OnExit(GameState::Playing), (cleanup_player, release_cursor));
    }
}

#[derive(Component)]
pub struct Player;

#[derive(Component)]
struct FpsCamera;

const PLAYER_HEIGHT: f32 = 1.7;
const MOUSE_SENSITIVITY: f32 = 0.001;
const MAX_PITCH: f32 = std::f32::consts::FRAC_PI_2 - 0.01;

fn spawn_player(mut commands: Commands) {
    commands
        .spawn((
            Transform::from_xyz(0.0, GROUND_Y + PLAYER_HEIGHT, 5.0),
            Visibility::default(),
            Player,
            Actor {
                speed: 7.0,
                height: PLAYER_HEIGHT,
                yaw: 0.0,
                vertical_velocity: 0.0,
                grounded: true,
            },
            ActorIntent::default(),
        ))
        .with_children(|parent| {
            parent.spawn((
                Camera3d::default(),
                Transform::default(),
                FpsCamera,
                PaletteSqueeze::default(),
            ));
        });
}

/// Pitch is stored on the camera child, not on Actor (only player cameras pitch).
fn player_mouse_look(
    mouse_motion: Res<AccumulatedMouseMotion>,
    mut player_query: Query<&mut Actor, With<Player>>,
    mut camera_query: Query<&mut Transform, (With<FpsCamera>, Without<Player>)>,
    spawn_anim_query: Query<(), With<SpawnAnimation>>,
) {
    let Ok(mut actor) = player_query.single_mut() else {
        return;
    };

    if mouse_motion.delta == Vec2::ZERO {
        return;
    }

    let sensitivity = MOUSE_SENSITIVITY * spawn_sensitivity_factor(spawn_anim_query.iter().count());

    actor.yaw -= mouse_motion.delta.x * sensitivity;

    let pitch = if let Ok(cam) = camera_query.single() {
        let (pitch, _, _) = cam.rotation.to_euler(EulerRot::XYZ);
        pitch
    } else {
        0.0
    };
    let new_pitch = (pitch - mouse_motion.delta.y * sensitivity).clamp(-MAX_PITCH, MAX_PITCH);

    if let Ok(mut camera_transform) = camera_query.single_mut() {
        camera_transform.rotation = Quat::from_rotation_x(new_pitch);
    }
}

fn player_movement_input(
    actions: Res<Actions>,
    mut query: Query<&mut ActorIntent, With<Player>>,
) {
    let Ok(mut intent) = query.single_mut() else {
        return;
    };

    intent.move_direction = actions.player_movement.unwrap_or(Vec2::ZERO);
}

fn grab_cursor(mut cursor_query: Query<&mut CursorOptions, With<PrimaryWindow>>) {
    if let Ok(mut cursor) = cursor_query.single_mut() {
        cursor.grab_mode = CursorGrabMode::Locked;
        cursor.visible = false;
    }
}

fn release_cursor(mut cursor_query: Query<&mut CursorOptions, With<PrimaryWindow>>) {
    if let Ok(mut cursor) = cursor_query.single_mut() {
        cursor.grab_mode = CursorGrabMode::None;
        cursor.visible = true;
    }
}

fn cleanup_player(mut commands: Commands, query: Query<Entity, With<Player>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
