use crate::GameState;
use crate::loading::{FontAssets, TextureAssets};
use crate::pause::game_not_paused;
use crate::player::Player;
use bevy::prelude::*;
use bevy::text::FontSmoothing;
use bevy_text_animation::{TextAnimationFinished, TextAnimatorPlugin, TextSimpleAnimator};
use serde::Deserialize;
use std::collections::HashMap;

pub struct DialogPlugin;

impl Plugin for DialogPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TextAnimatorPlugin)
            .add_systems(OnEnter(GameState::Playing), init_dialog)
            .add_systems(
                Update,
                (
                    check_npc_proximity,
                    handle_dialog_input,
                    track_animation_finished,
                    manage_dialog_ui,
                    manage_prompt_ui,
                )
                    .chain()
                    .run_if(in_state(GameState::Playing).and(game_not_paused)),
            )
            .add_systems(OnExit(GameState::Playing), cleanup_dialog);
    }
}

const DIALOG_TREE_RON: &str = include_str!("../assets/dialogs/dialog_tree.ron");
const TEXT_SPEED: f32 = 20.0;

#[derive(Debug, Deserialize)]
struct DialogTreeRon {
    dialogs: HashMap<String, Vec<String>>,
}

#[derive(Resource)]
struct DialogTree {
    dialogs: HashMap<String, Vec<String>>,
}

/// Mark an entity as an interactable NPC with a dialog.
#[derive(Component)]
pub struct Npc {
    pub dialog_id: String,
    pub range: f32,
}

#[derive(Resource)]
#[derive(Default)]
pub struct DialogState {
    pub active: bool,
    lines: Vec<String>,
    line_index: usize,
    anim_done: bool,
    /// Signals the UI system to spawn, update, or despawn the dialog box.
    dirty: bool,
}


/// Run condition: returns true when no dialog is active.
pub fn dialog_not_active(state: Option<Res<DialogState>>) -> bool {
    state.is_none_or(|s| !s.active)
}

/// Tracks the dialog_id of the nearest NPC within interaction range.
#[derive(Resource, Default)]
struct NearbyNpc(Option<String>);

#[derive(Component)]
struct DialogUi;

#[derive(Component)]
struct DialogText;

#[derive(Component)]
struct PromptUi;

fn init_dialog(mut commands: Commands) {
    let data: DialogTreeRon =
        ron::from_str(DIALOG_TREE_RON).expect("Failed to parse dialog_tree.ron");
    commands.insert_resource(DialogTree {
        dialogs: data.dialogs,
    });
    commands.insert_resource(DialogState::default());
    commands.insert_resource(NearbyNpc::default());
}

fn check_npc_proximity(
    player_q: Query<&GlobalTransform, With<Player>>,
    npc_q: Query<(&GlobalTransform, &Npc)>,
    mut nearby: ResMut<NearbyNpc>,
) {
    let Ok(player_tf) = player_q.single() else {
        nearby.0 = None;
        return;
    };

    let player_pos = player_tf.translation();
    let mut best: Option<(f32, &str)> = None;

    for (tf, npc) in &npc_q {
        let dist = player_pos.distance(tf.translation());
        if dist <= npc.range && best.as_ref().is_none_or(|(d, _)| dist < *d) {
            best = Some((dist, &npc.dialog_id));
        }
    }

    let new_id = best.map(|(_, id)| id.to_string());
    if nearby.0 != new_id {
        nearby.0 = new_id;
    }
}

fn handle_dialog_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<DialogState>,
    nearby: Res<NearbyNpc>,
    tree: Option<Res<DialogTree>>,
    mut text_q: Query<&mut Text, With<DialogText>>,
) {
    let e_pressed = keyboard.just_pressed(KeyCode::KeyE);
    let space_pressed = keyboard.just_pressed(KeyCode::Space);

    if !state.active {
        if e_pressed
            && let Some(ref id) = nearby.0
            && let Some(tree) = tree
            && let Some(lines) = tree.dialogs.get(id)
            && !lines.is_empty()
        {
            state.active = true;
            state.lines = lines.clone();
            state.line_index = 0;
            state.anim_done = false;
            state.dirty = true;
        }
        return;
    }

    if !e_pressed && !space_pressed {
        return;
    }

    if !state.anim_done {
        // Skip animation — show full text immediately
        if let Some(line) = state.lines.get(state.line_index) {
            for mut text in &mut text_q {
                **text = line.clone();
            }
        }
        state.anim_done = true;
    } else {
        // Advance to next line
        state.line_index += 1;
        if state.line_index >= state.lines.len() {
            state.active = false;
            state.lines.clear();
            state.line_index = 0;
        }
        state.anim_done = false;
        state.dirty = true;
    }
}

fn track_animation_finished(
    mut events: MessageReader<TextAnimationFinished>,
    mut state: ResMut<DialogState>,
) {
    for _event in events.read() {
        if state.active {
            state.anim_done = true;
        }
    }
}

fn manage_dialog_ui(
    mut commands: Commands,
    mut state: ResMut<DialogState>,
    dialog_q: Query<Entity, With<DialogUi>>,
    text_q: Query<Entity, With<DialogText>>,
    fonts: Res<FontAssets>,
    textures: Res<TextureAssets>,
) {
    if !state.dirty {
        return;
    }
    state.dirty = false;

    if state.active {
        let line = state
            .lines
            .get(state.line_index)
            .cloned()
            .unwrap_or_default();

        if dialog_q.is_empty() {
            // Spawn dialog box
            let font = fonts.main.clone();
            commands
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        position_type: PositionType::Absolute,
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::FlexEnd,
                        padding: UiRect::bottom(Val::Px(20.0)),
                        ..default()
                    },
                    GlobalZIndex(80),
                    DialogUi,
                ))
                .with_children(|parent| {
                    parent
                        .spawn((
                            Node {
                                width: Val::Percent(80.0),
                                min_height: Val::Px(80.0),
                                padding: UiRect::all(Val::Px(16.0)),
                                ..default()
                            },
                            ImageNode {
                                image: textures.textbox.clone(),
                                image_mode: NodeImageMode::Sliced(TextureSlicer {
                                    border: BorderRect::all(16.0),
                                    center_scale_mode: SliceScaleMode::Stretch,
                                    sides_scale_mode: SliceScaleMode::Stretch,
                                    max_corner_scale: 1.0,
                                }),
                                ..default()
                            },
                        ))
                        .with_child((
                            Text::new(""),
                            TextFont {
                                font,
                                font_size: 16.0,
                                font_smoothing: FontSmoothing::None,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                            TextSimpleAnimator::new(&line, TEXT_SPEED),
                            DialogText,
                        ));
                });
        } else {
            // Update text for next line
            for entity in &text_q {
                commands
                    .entity(entity)
                    .insert(TextSimpleAnimator::new(&line, TEXT_SPEED));
            }
        }
    } else {
        // Despawn dialog box
        for entity in &dialog_q {
            commands.entity(entity).despawn();
        }
    }
}

fn manage_prompt_ui(
    mut commands: Commands,
    nearby: Res<NearbyNpc>,
    state: Res<DialogState>,
    prompt_q: Query<Entity, With<PromptUi>>,
    fonts: Res<FontAssets>,
) {
    let should_show = nearby.0.is_some() && !state.active;

    if should_show && prompt_q.is_empty() {
        commands.spawn((
            Text::new("[E] Talk"),
            TextFont {
                font: fonts.main.clone(),
                font_size: 16.0,
                font_smoothing: FontSmoothing::None,
                ..default()
            },
            TextColor(Color::WHITE),
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(120.0),
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            GlobalZIndex(80),
            PromptUi,
        ));
    } else if !should_show {
        for entity in &prompt_q {
            commands.entity(entity).despawn();
        }
    }
}

fn cleanup_dialog(
    mut commands: Commands,
    dialog_q: Query<Entity, With<DialogUi>>,
    prompt_q: Query<Entity, With<PromptUi>>,
) {
    for entity in &dialog_q {
        commands.entity(entity).despawn();
    }
    for entity in &prompt_q {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<DialogState>();
    commands.remove_resource::<DialogTree>();
    commands.remove_resource::<NearbyNpc>();
}
