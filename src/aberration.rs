use crate::GameState;
use crate::loading::TextureAssets;
use crate::player::Player;
use bevy::prelude::*;
use rand::Rng;

pub struct AberrationPlugin;

impl Plugin for AberrationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_aberrations)
            .add_systems(
                Update,
                billboard_face_camera.run_if(in_state(GameState::Playing)),
            )
            .add_systems(OnExit(GameState::Playing), cleanup_aberrations);
    }
}

/// Marker for aberration enemies.
#[derive(Component)]
pub struct Aberration;

/// Marker for entities that should always face the camera (Y-axis only).
#[derive(Component)]
struct Billboard;

fn spawn_aberrations(
    mut commands: Commands,
    textures: Res<TextureAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut rng = rand::rng();
    let quad = meshes.add(Rectangle::new(2.0, 2.0));

    // Spawn several aberrations at scattered positions
    let positions = [
        Vec3::new(4.0, 1.0, -6.0),
        Vec3::new(-6.0, 1.0, -3.0),
        Vec3::new(2.0, 1.0, -12.0),
        Vec3::new(-3.0, 1.0, -9.0),
        Vec3::new(8.0, 1.0, -8.0),
        Vec3::new(-8.0, 1.0, -14.0),
    ];

    for pos in &positions {
        let texture_index = rng.random_range(0..textures.aberrations.len());
        let texture = textures.aberrations[texture_index].clone();

        commands.spawn((
            Mesh3d(quad.clone()),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color_texture: Some(texture),
                alpha_mode: AlphaMode::Mask(0.5),
                unlit: true,
                cull_mode: None,
                ..default()
            })),
            Transform::from_translation(*pos),
            Aberration,
            Billboard,
        ));
    }
}

fn billboard_face_camera(
    camera_query: Query<&GlobalTransform, With<Player>>,
    mut billboard_query: Query<&mut Transform, With<Billboard>>,
) {
    let Ok(camera_global) = camera_query.single() else {
        return;
    };
    let camera_pos = camera_global.translation();

    for mut transform in &mut billboard_query {
        // Only rotate around Y axis to stay upright
        let dir = camera_pos - transform.translation;
        let angle = dir.x.atan2(dir.z);
        transform.rotation = Quat::from_rotation_y(angle);
    }
}

fn cleanup_aberrations(mut commands: Commands, query: Query<Entity, With<Aberration>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
