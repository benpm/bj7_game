use crate::GameState;
use crate::actor::GROUND_Y;
use crate::dialog::Npc;
use crate::pause::game_not_paused;
use crate::player::Player;
use bevy::prelude::*;
use serde::Deserialize;

const NPCS_RON: &str = include_str!("../assets/defs/npcs.ron");
const NPC_CUBE_HEIGHT: f32 = 1.8;
const SPRITE_HOVER_HEIGHT: f32 = 0.4; // gap above the cube

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), setup_world)
            .add_systems(
                Update,
                npc_sprite_face_player
                    .run_if(in_state(GameState::Playing).and(game_not_paused)),
            )
            .add_systems(OnExit(GameState::Playing), cleanup_world);
    }
}

// --- RON data ---

#[derive(Deserialize)]
struct NpcsRon {
    npcs: Vec<NpcRon>,
}

#[derive(Deserialize)]
struct NpcRon {
    position: (f32, f32, f32),
    sprite: String,
    range: f32,
}

// --- Components ---

#[derive(Component)]
struct WorldEntity;

/// Billboard sprite floating above an NPC.
#[derive(Component)]
struct NpcSprite;

fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
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

    // Load NPCs from .ron
    let npc_data: NpcsRon = ron::from_str(NPCS_RON).expect("Failed to parse npcs.ron");
    let cube_mesh = meshes.add(Cuboid::new(0.6, NPC_CUBE_HEIGHT, 0.6));
    let cube_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.8, 0.8, 0.8),
        emissive: bevy::color::LinearRgba::new(0.3, 0.3, 0.3, 1.0),
        ..default()
    });
    let sprite_quad = meshes.add(Rectangle::new(1.0, 1.0));

    for npc in &npc_data.npcs {
        let sprite_texture: Handle<Image> = asset_server.load(&npc.sprite);
        let sprite_y = NPC_CUBE_HEIGHT / 2.0 + SPRITE_HOVER_HEIGHT + 0.5;

        commands
            .spawn((
                Mesh3d(cube_mesh.clone()),
                MeshMaterial3d(cube_material.clone()),
                Transform::from_xyz(
                    npc.position.0,
                    GROUND_Y + NPC_CUBE_HEIGHT / 2.0 + npc.position.1,
                    npc.position.2,
                ),
                Npc { range: npc.range },
                WorldEntity,
            ))
            .with_children(|parent| {
                parent.spawn((
                    Mesh3d(sprite_quad.clone()),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color_texture: Some(sprite_texture),
                        alpha_mode: AlphaMode::Mask(0.5),
                        unlit: true,
                        cull_mode: None,
                        ..default()
                    })),
                    Transform::from_xyz(0.0, sprite_y, 0.0),
                    NpcSprite,
                ));
            });
    }

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

/// Rotate NPC billboard sprites to face the player (Y-axis only).
fn npc_sprite_face_player(
    player_q: Query<&GlobalTransform, With<Player>>,
    mut sprite_q: Query<(&mut Transform, &GlobalTransform), With<NpcSprite>>,
) {
    let Ok(player_tf) = player_q.single() else {
        return;
    };
    let player_pos = player_tf.translation();

    for (mut transform, global_tf) in &mut sprite_q {
        let pos = global_tf.translation();
        let dir = player_pos - pos;
        transform.rotation = Quat::from_rotation_y(dir.x.atan2(dir.z));
    }
}

fn cleanup_world(mut commands: Commands, query: Query<Entity, With<WorldEntity>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
