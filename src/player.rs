use crate::GameState;
use crate::actions::Actions;
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), (spawn_player, grab_cursor))
            .add_systems(
                Update,
                (fps_mouse_look, fps_movement, fps_gravity)
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(OnExit(GameState::Playing), (cleanup_player, release_cursor));
    }
}

#[derive(Component)]
pub struct Player;

#[derive(Component)]
struct FpsCamera;

#[derive(Component)]
struct FpsController {
    speed: f32,
    sensitivity: f32,
    yaw: f32,
    pitch: f32,
    vertical_velocity: f32,
    grounded: bool,
}

impl Default for FpsController {
    fn default() -> Self {
        Self {
            speed: 7.0,
            sensitivity: 0.003,
            yaw: 0.0,
            pitch: 0.0,
            vertical_velocity: 0.0,
            grounded: true,
        }
    }
}

const GROUND_Y: f32 = 0.0;
const PLAYER_HEIGHT: f32 = 1.7;
const GRAVITY: f32 = 9.8;
const MAX_PITCH: f32 = std::f32::consts::FRAC_PI_2 - 0.01;

fn spawn_player(mut commands: Commands) {
    commands
        .spawn((
            Transform::from_xyz(0.0, GROUND_Y + PLAYER_HEIGHT, 5.0),
            Visibility::default(),
            Player,
            FpsController::default(),
        ))
        .with_children(|parent| {
            parent.spawn((Camera3d::default(), Transform::default(), FpsCamera));
        });
}

fn fps_mouse_look(
    mouse_motion: Res<AccumulatedMouseMotion>,
    mut player_query: Query<(&mut FpsController, &mut Transform), With<Player>>,
    mut camera_query: Query<&mut Transform, (With<FpsCamera>, Without<Player>)>,
) {
    let Ok((mut controller, mut player_transform)) = player_query.single_mut() else {
        return;
    };

    if mouse_motion.delta == Vec2::ZERO {
        return;
    }

    controller.yaw -= mouse_motion.delta.x * controller.sensitivity;
    controller.pitch -= mouse_motion.delta.y * controller.sensitivity;
    controller.pitch = controller.pitch.clamp(-MAX_PITCH, MAX_PITCH);

    player_transform.rotation = Quat::from_rotation_y(controller.yaw);

    if let Ok(mut camera_transform) = camera_query.single_mut() {
        camera_transform.rotation = Quat::from_rotation_x(controller.pitch);
    }
}

fn fps_movement(
    time: Res<Time>,
    actions: Res<Actions>,
    mut query: Query<(&FpsController, &mut Transform), With<Player>>,
) {
    let Some(movement) = actions.player_movement else {
        return;
    };

    let Ok((controller, mut transform)) = query.single_mut() else {
        return;
    };

    let forward = transform.forward();
    let right = transform.right();

    // Project onto horizontal plane (ignore y component)
    let forward_flat = Vec3::new(forward.x, 0.0, forward.z).normalize_or_zero();
    let right_flat = Vec3::new(right.x, 0.0, right.z).normalize_or_zero();

    // movement.y = forward/back (W/S maps to Up/Down in Actions)
    // movement.x = strafe left/right (A/D maps to Left/Right in Actions)
    let velocity = (forward_flat * movement.y + right_flat * movement.x) * controller.speed;

    transform.translation += velocity * time.delta_secs();
}

fn fps_gravity(time: Res<Time>, mut query: Query<(&mut FpsController, &mut Transform)>) {
    for (mut controller, mut transform) in &mut query {
        if !controller.grounded {
            controller.vertical_velocity -= GRAVITY * time.delta_secs();
        }

        transform.translation.y += controller.vertical_velocity * time.delta_secs();

        let ground_level = GROUND_Y + PLAYER_HEIGHT;
        if transform.translation.y <= ground_level {
            transform.translation.y = ground_level;
            controller.vertical_velocity = 0.0;
            controller.grounded = true;
        } else {
            controller.grounded = false;
        }
    }
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
