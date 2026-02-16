use crate::GameState;
use bevy::prelude::*;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), setup_world)
            .add_systems(OnExit(GameState::Playing), cleanup_world);
    }
}

#[derive(Component)]
struct WorldEntity;

fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(50.0, 50.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.0, 0.0, 0.0),
            perceptual_roughness: 0.9,
            ..default()
        })),
        WorldEntity,
    ));

    // Spawn a 10x10 grid of tall, thin black cylinders
    for x in 0..10 {
        for z in 0..10 {
            let position = Vec3::new(x as f32 * 2.0 - 9.0, 5.0, z as f32 * 2.0 - 9.0);
            commands.spawn((
                Mesh3d(meshes.add(Cylinder::new(0.1, 10.0))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(0.0, 0.0, 0.0),
                    ..default()
                })),
                Transform::from_translation(position),
                WorldEntity,
            ));
        }
    }
}

fn cleanup_world(mut commands: Commands, query: Query<Entity, With<WorldEntity>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
