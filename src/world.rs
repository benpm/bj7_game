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
            base_color: Color::srgb(0.35, 0.55, 0.3),
            perceptual_roughness: 0.9,
            ..default()
        })),
        WorldEntity,
    ));

    // Directional light (sun)
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.8, 0.4, 0.0)),
        WorldEntity,
    ));

    // Scattered objects
    let objects: &[(Mesh, Vec3, Color)] = &[
        (Cuboid::new(1.0, 1.0, 1.0).into(), Vec3::new(3.0, 0.5, -2.0), Color::srgb(0.8, 0.2, 0.2)),
        (Cuboid::new(2.0, 2.0, 2.0).into(), Vec3::new(-5.0, 1.0, -8.0), Color::srgb(0.2, 0.3, 0.8)),
        (Cuboid::new(1.5, 0.5, 1.5).into(), Vec3::new(7.0, 0.25, -5.0), Color::srgb(0.9, 0.6, 0.1)),
        (Sphere::new(0.75).into(), Vec3::new(-3.0, 0.75, -4.0), Color::srgb(0.9, 0.9, 0.2)),
        (Sphere::new(1.2).into(), Vec3::new(5.0, 1.2, -10.0), Color::srgb(0.3, 0.8, 0.3)),
        (Cylinder::new(0.5, 2.0).into(), Vec3::new(-7.0, 1.0, -3.0), Color::srgb(0.7, 0.2, 0.7)),
        (Cylinder::new(0.8, 3.0).into(), Vec3::new(0.0, 1.5, -12.0), Color::srgb(0.2, 0.7, 0.7)),
        (Cuboid::new(0.8, 3.0, 0.8).into(), Vec3::new(-2.0, 1.5, -15.0), Color::srgb(0.6, 0.4, 0.2)),
    ];

    for (mesh, position, color) in objects {
        commands.spawn((
            Mesh3d(meshes.add(mesh.clone())),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: *color,
                ..default()
            })),
            Transform::from_translation(*position),
            WorldEntity,
        ));
    }
}

fn cleanup_world(mut commands: Commands, query: Query<Entity, With<WorldEntity>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
