use crate::GameState;
use bevy::prelude::*;

pub struct HealthPlugin;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), (init_health, spawn_vignette))
            .add_systems(
                Update,
                (passive_drain, update_vignette)
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(OnExit(GameState::Playing), cleanup_health);
    }
}

#[derive(Resource)]
pub struct Health {
    pub current: f32,
    pub max: f32,
    /// Sanity drain per second (constant pressure)
    pub drain_rate: f32,
}

impl Default for Health {
    fn default() -> Self {
        Self {
            current: 1.0,
            max: 1.0,
            drain_rate: 0.02,
        }
    }
}

#[allow(dead_code)]
impl Health {
    pub fn fraction(&self) -> f32 {
        (self.current / self.max).clamp(0.0, 1.0)
    }

    pub fn damage(&mut self, amount: f32) {
        self.current = (self.current - amount).max(0.0);
    }

    pub fn heal(&mut self, amount: f32) {
        self.current = (self.current + amount).min(self.max);
    }

    pub fn is_dead(&self) -> bool {
        self.current <= 0.0
    }
}

#[derive(Component)]
struct HealthVignette;

fn init_health(mut commands: Commands) {
    commands.insert_resource(Health::default());
}

fn spawn_vignette(mut commands: Commands) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            ..default()
        },
        GlobalZIndex(50),
        BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.0)),
        HealthVignette,
    ));
}

fn passive_drain(time: Res<Time>, mut health: ResMut<Health>) {
    let drain = health.drain_rate * time.delta_secs();
    health.damage(drain);
}

fn update_vignette(
    health: Res<Health>,
    mut query: Query<&mut BackgroundColor, With<HealthVignette>>,
) {
    let alpha = 1.0 - health.fraction();
    for mut bg in &mut query {
        bg.0 = Color::srgba(1.0, 1.0, 1.0, alpha);
    }
}

fn cleanup_health(
    mut commands: Commands,
    query: Query<Entity, With<HealthVignette>>,
) {
    commands.remove_resource::<Health>();
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
