use crate::GameState;
use crate::loading::{FontAssets, TextureAssets};
use crate::pause::game_not_paused;
use crate::player::Player;
use bevy::prelude::*;
use bevy::text::FontSmoothing;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};
use bevy_text_animation::{TextAnimationFinished, TextAnimatorPlugin, TextSimpleAnimator};
use rand::Rng;
use serde::Deserialize;

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
                    handle_response_click,
                    track_animation_finished,
                    manage_dialog_ui,
                    manage_prompt_ui,
                    response_button_hover,
                    animate_response_buttons,
                )
                    .chain()
                    .run_if(in_state(GameState::Playing).and(game_not_paused)),
            )
            .add_systems(OnExit(GameState::Playing), cleanup_dialog);
    }
}

const DIALOG_RON: &str = include_str!("../assets/defs/dialog.ron");
const TEXT_SPEED: f32 = 20.0;

// --- RON data ---

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
enum Role {
    Npc,
    Player,
}

#[derive(Debug, Clone, Deserialize)]
struct DialogueNode {
    role: Role,
    text: String,
    responses: Vec<DialogueNode>,
    #[serde(default)]
    win: bool,
}

#[derive(Debug, Deserialize)]
struct DialogueTrees(Vec<DialogueNode>);

// --- Resources ---

#[derive(Resource)]
struct DialogTrees(Vec<DialogueNode>);

/// Mark an entity as an interactable NPC.
#[derive(Component)]
pub struct Npc {
    pub range: f32,
}

#[derive(Resource, Default)]
pub struct DialogState {
    pub active: bool,
    /// The current NPC node being displayed.
    current_node: Option<DialogueNode>,
    anim_done: bool,
    dirty: bool,
    /// Whether response buttons are currently visible.
    responses_shown: bool,
    /// The NPC entity this dialog is with.
    npc_entity: Option<Entity>,
}

/// Run condition: returns true when no dialog is active.
pub fn dialog_not_active(state: Option<Res<DialogState>>) -> bool {
    state.is_none_or(|s| !s.active)
}

/// Tracks the nearest NPC within interaction range.
#[derive(Resource, Default)]
struct NearbyNpc(Option<Entity>);

#[derive(Component)]
struct DialogUi;

#[derive(Component)]
struct DialogText;

#[derive(Component)]
struct ResponseContainer;

/// Index into the current node's `responses` array.
#[derive(Component)]
struct ResponseButton(usize);

/// Animates a response button from 0 to full height over RESPONSE_ANIM_SECS.
#[derive(Component)]
struct ResponseButtonAnim {
    timer: Timer,
}

const RESPONSE_ANIM_SECS: f32 = 1.0;

#[derive(Component)]
struct PromptUi;

fn init_dialog(mut commands: Commands) {
    let data: DialogueTrees = ron::from_str(DIALOG_RON).expect("Failed to parse dialog.ron");
    commands.insert_resource(DialogTrees(data.0));
    commands.insert_resource(DialogState::default());
    commands.insert_resource(NearbyNpc::default());
}

fn check_npc_proximity(
    player_q: Query<&GlobalTransform, With<Player>>,
    npc_q: Query<(Entity, &GlobalTransform, &Npc)>,
    mut nearby: ResMut<NearbyNpc>,
) {
    let Ok(player_tf) = player_q.single() else {
        nearby.0 = None;
        return;
    };

    let player_pos = player_tf.translation();
    let found = npc_q
        .iter()
        .filter(|(_, tf, npc)| player_pos.distance(tf.translation()) <= npc.range)
        .min_by(|(_, a, _), (_, b, _)| {
            let da = player_pos.distance(a.translation());
            let db = player_pos.distance(b.translation());
            da.partial_cmp(&db).unwrap()
        })
        .map(|(entity, _, _)| entity);

    nearby.0 = found;
}

fn handle_dialog_input(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut state: ResMut<DialogState>,
    nearby: Res<NearbyNpc>,
    trees: Option<Res<DialogTrees>>,
    mut text_q: Query<&mut Text, With<DialogText>>,
    mut cursor_q: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    let e_pressed = keyboard.just_pressed(KeyCode::KeyE);
    let esc_pressed = keyboard.just_pressed(KeyCode::Escape);
    let space_pressed = keyboard.just_pressed(KeyCode::Space);
    let left_click = mouse.just_pressed(MouseButton::Left);

    if !state.active {
        if e_pressed
            && nearby.0.is_some()
            && let Some(trees) = trees
            && !trees.0.is_empty()
        {
            let idx = rand::rng().random_range(0..trees.0.len());
            let tree = trees.0[idx].clone();
            state.active = true;
            state.current_node = Some(tree);
            state.anim_done = false;
            state.dirty = true;
            state.responses_shown = false;
            state.npc_entity = nearby.0;
            // Show cursor for dialog interaction
            if let Ok(mut cursor) = cursor_q.single_mut() {
                cursor.grab_mode = CursorGrabMode::None;
                cursor.visible = true;
            }
        }
        return;
    }

    // Escape or E always closes dialog
    if esc_pressed || e_pressed {
        close_dialog(&mut commands, &mut state, &mut cursor_q);
        return;
    }

    // Left click closes dialog when no responses are available
    let has_responses = state
        .current_node
        .as_ref()
        .is_some_and(|n| n.responses.iter().any(|r| r.role == Role::Player));

    if left_click && state.anim_done && !has_responses {
        close_dialog(&mut commands, &mut state, &mut cursor_q);
        return;
    }

    // Don't handle Space when response buttons are showing
    if state.responses_shown {
        return;
    }

    if !space_pressed {
        return;
    }

    if !state.anim_done {
        // Skip animation — show full text immediately
        if let Some(ref node) = state.current_node {
            for mut text in &mut text_q {
                **text = node.text.clone();
            }
        }
        state.anim_done = true;
        state.dirty = true;
    } else if !has_responses {
        close_dialog(&mut commands, &mut state, &mut cursor_q);
    }
}

fn handle_response_click(
    mut commands: Commands,
    mut state: ResMut<DialogState>,
    interaction_q: Query<(&Interaction, &ResponseButton), Changed<Interaction>>,
    mut cursor_q: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    if !state.active {
        return;
    }

    for (interaction, response_btn) in &interaction_q {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let Some(ref node) = state.current_node else {
            continue;
        };

        let Some(player_node) = node.responses.get(response_btn.0) else {
            continue;
        };

        // Navigate to the first NPC response of this player choice
        if let Some(next_npc) = player_node.responses.first() {
            state.current_node = Some(next_npc.clone());
            state.anim_done = false;
            state.dirty = true;
            state.responses_shown = false;
        } else {
            // No further dialog — end
            close_dialog(&mut commands, &mut state, &mut cursor_q);
        }
    }
}

fn close_dialog(
    commands: &mut Commands,
    state: &mut ResMut<DialogState>,
    cursor_q: &mut Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    // Check win condition before clearing state
    let win = state
        .current_node
        .as_ref()
        .is_some_and(|n| n.win);
    if win {
        if let Some(npc_entity) = state.npc_entity {
            commands.entity(npc_entity).despawn();
        }
    }

    state.active = false;
    state.current_node = None;
    state.dirty = true;
    state.responses_shown = false;
    state.npc_entity = None;
    // Re-lock cursor
    if let Ok(mut cursor) = cursor_q.single_mut() {
        cursor.grab_mode = CursorGrabMode::Locked;
        cursor.visible = false;
    }
}

fn track_animation_finished(
    mut events: MessageReader<TextAnimationFinished>,
    mut state: ResMut<DialogState>,
) {
    for _event in events.read() {
        if state.active && !state.anim_done {
            state.anim_done = true;
            state.dirty = true;
        }
    }
}

fn manage_dialog_ui(
    mut commands: Commands,
    mut state: ResMut<DialogState>,
    dialog_q: Query<Entity, With<DialogUi>>,
    text_q: Query<Entity, With<DialogText>>,
    response_container_q: Query<(Entity, Option<&Children>), With<ResponseContainer>>,
    fonts: Res<FontAssets>,
    textures: Res<TextureAssets>,
) {
    if !state.dirty {
        return;
    }
    state.dirty = false;

    if !state.active {
        // Despawn dialog box
        for entity in &dialog_q {
            commands.entity(entity).despawn();
        }
        return;
    }

    // Collect data from current_node before mutating state
    let Some(node) = state.current_node.clone() else {
        return;
    };

    if dialog_q.is_empty() {
        // Spawn dialog box
        spawn_dialog_box(&mut commands, &node, &fonts, &textures);
    } else if state.anim_done && !state.responses_shown {
        // Show response buttons
        let player_responses: Vec<(usize, String)> = node
            .responses
            .iter()
            .enumerate()
            .filter(|(_, r)| r.role == Role::Player)
            .map(|(i, r)| (i, r.text.clone()))
            .collect();

        if player_responses.is_empty() {
            // No player responses — dialog will end on next E press
            return;
        }

        state.responses_shown = true;

        let font = fonts.main.clone();
        let textbox = textures.textbox.clone();

        for (container_entity, _) in &response_container_q {
            commands.entity(container_entity).with_children(|parent| {
                for (original_idx, text) in &player_responses {
                    parent
                        .spawn((
                            Button,
                            Node {
                                padding: UiRect::new(
                                    Val::Px(16.0),
                                    Val::Px(16.0),
                                    Val::Px(8.0),
                                    Val::Px(8.0),
                                ),
                                height: Val::Px(0.0),
                                overflow: Overflow::clip(),
                                ..default()
                            },
                            ImageNode {
                                image: textbox.clone(),
                                image_mode: NodeImageMode::Sliced(TextureSlicer {
                                    border: BorderRect::all(16.0),
                                    center_scale_mode: SliceScaleMode::Stretch,
                                    sides_scale_mode: SliceScaleMode::Stretch,
                                    max_corner_scale: 1.0,
                                }),
                                ..default()
                            },
                            ResponseButton(*original_idx),
                            ResponseButtonAnim {
                                timer: Timer::from_seconds(RESPONSE_ANIM_SECS, TimerMode::Once),
                            },
                        ))
                        .with_child((
                            Text::new(text),
                            TextFont {
                                font: font.clone(),
                                font_size: 32.0,
                                font_smoothing: FontSmoothing::None,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                }
            });
        }
    } else if !state.anim_done {
        // New NPC line — update text and clear response buttons
        for entity in &text_q {
            commands
                .entity(entity)
                .insert(TextSimpleAnimator::new(&node.text, TEXT_SPEED));
        }
        for (_, children) in &response_container_q {
            if let Some(children) = children {
                for child in children.iter() {
                    commands.entity(child).despawn();
                }
            }
        }
    }
}

fn spawn_dialog_box(
    commands: &mut Commands,
    node: &DialogueNode,
    fonts: &Res<FontAssets>,
    textures: &Res<TextureAssets>,
) {
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
            // Row: NPC portrait + text box
            parent
                .spawn(Node {
                    width: Val::Percent(80.0),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::FlexEnd,
                    column_gap: Val::Px(8.0),
                    ..default()
                })
                .with_children(|row| {
                    // NPC portrait
                    row.spawn((
                        Node {
                            width: Val::Px(64.0),
                            height: Val::Px(64.0),
                            ..default()
                        },
                        ImageNode {
                            image: textures.unknown.clone(),
                            ..default()
                        },
                    ));

                    // NPC text box
                    row.spawn((
                        Node {
                            flex_grow: 1.0,
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
                            font_size: 32.0,
                            font_smoothing: FontSmoothing::None,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        TextSimpleAnimator::new(&node.text, TEXT_SPEED),
                        DialogText,
                    ));
                });

            // Response button container (initially empty, indented)
            parent.spawn((
                Node {
                    width: Val::Percent(80.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(4.0),
                    margin: UiRect::top(Val::Px(8.0)),
                    padding: UiRect::left(Val::Px(72.0)),
                    ..default()
                },
                ResponseContainer,
            ));
        });
}

fn animate_response_buttons(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Node, &mut ResponseButtonAnim)>,
) {
    for (entity, mut node, mut anim) in &mut query {
        anim.timer.tick(time.delta());
        let t = anim.timer.fraction();
        // Animate from 0 to Auto height using scale factor on a max height
        node.height = Val::Px(t * 48.0);
        if t >= 1.0 {
            node.height = Val::Auto;
            commands.entity(entity).remove::<ResponseButtonAnim>();
        }
    }
}

fn response_button_hover(
    mut interaction_q: Query<
        (&Interaction, &mut ImageNode),
        (Changed<Interaction>, With<ResponseButton>),
    >,
) {
    for (interaction, mut image_node) in &mut interaction_q {
        match *interaction {
            Interaction::Hovered | Interaction::Pressed => {
                image_node.color = Color::linear_rgb(0.6, 0.6, 0.6);
            }
            Interaction::None => {
                image_node.color = Color::WHITE;
            }
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
                font_size: 32.0,
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
    mut cursor_q: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    for entity in &dialog_q {
        commands.entity(entity).despawn();
    }
    for entity in &prompt_q {
        commands.entity(entity).despawn();
    }
    // Ensure cursor is released (player.rs release_cursor handles re-lock on exit)
    if let Ok(mut cursor) = cursor_q.single_mut() {
        cursor.grab_mode = CursorGrabMode::None;
        cursor.visible = true;
    }
    commands.remove_resource::<DialogState>();
    commands.remove_resource::<DialogTrees>();
    commands.remove_resource::<NearbyNpc>();
}
