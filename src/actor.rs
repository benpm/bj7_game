use crate::GameState;
use crate::pause::game_not_paused;
use bevy::prelude::*;

pub struct ActorPlugin;

impl Plugin for ActorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (actor_apply_yaw, actor_movement, actor_gravity)
                .chain()
                .run_if(in_state(GameState::Playing).and(game_not_paused)),
        );
    }
}

pub const GROUND_Y: f32 = 0.0;
const GRAVITY: f32 = 9.8;

/// Shared physical state for any controllable entity in the game world.
#[derive(Component)]
pub struct Actor {
    pub speed: f32,
    pub height: f32,
    pub yaw: f32,
    pub vertical_velocity: f32,
    pub grounded: bool,
}

/// Per-frame movement intent. Written by controller systems (player input or AI),
/// consumed by shared actor systems.
#[derive(Component, Default)]
pub struct ActorIntent {
    /// Local-space movement: x = strafe right, y = forward. Zero = no movement.
    pub move_direction: Vec2,
}

fn actor_apply_yaw(mut query: Query<(&Actor, &mut Transform)>) {
    for (actor, mut transform) in &mut query {
        transform.rotation = Quat::from_rotation_y(actor.yaw);
    }
}

fn actor_movement(
    time: Res<Time>,
    mut query: Query<(&Actor, &ActorIntent, &mut Transform)>,
) {
    for (actor, intent, mut transform) in &mut query {
        if intent.move_direction == Vec2::ZERO {
            continue;
        }

        let (sin_yaw, cos_yaw) = actor.yaw.sin_cos();
        let forward = Vec3::new(-sin_yaw, 0.0, -cos_yaw);
        let right = Vec3::new(cos_yaw, 0.0, -sin_yaw);

        let velocity = (forward * intent.move_direction.y + right * intent.move_direction.x)
            .normalize_or_zero()
            * actor.speed;

        transform.translation += velocity * time.delta_secs();
    }
}

fn actor_gravity(time: Res<Time>, mut query: Query<(&mut Actor, &mut Transform)>) {
    for (mut actor, mut transform) in &mut query {
        if !actor.grounded {
            actor.vertical_velocity -= GRAVITY * time.delta_secs();
        }

        transform.translation.y += actor.vertical_velocity * time.delta_secs();

        let ground_level = GROUND_Y + actor.height;
        if transform.translation.y <= ground_level {
            transform.translation.y = ground_level;
            actor.vertical_velocity = 0.0;
            actor.grounded = true;
        } else {
            actor.grounded = false;
        }
    }
}
