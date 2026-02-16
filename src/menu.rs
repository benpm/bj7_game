use crate::GameState;
use crate::audio::GameVolume;
use crate::loading::{AudioAssets, FontAssets, TextureAssets};
use crate::palette::PaletteSqueeze;
use crate::scaling::CanvasImage;
use bevy::prelude::*;
use bevy::text::FontSmoothing;
use bevy_kira_audio::{Audio, AudioControl};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Menu), setup_menu)
            .add_systems(
                Update,
                (click_play_button, handle_volume_buttons)
                    .run_if(in_state(GameState::Menu)),
            )
            .add_systems(OnExit(GameState::Menu), cleanup_menu);
    }
}

#[derive(Component)]
struct Menu;

/// Helper to create the 9-slice slicer for the textbox texture (48x48, 16px border).
fn textbox_slicer() -> TextureSlicer {
    TextureSlicer {
        border: BorderRect::all(16.0),
        center_scale_mode: SliceScaleMode::Stretch,
        sides_scale_mode: SliceScaleMode::Tile { stretch_value: 1.0 },
        max_corner_scale: 1.0,
    }
}

fn setup_menu(
    mut commands: Commands,
    textures: Res<TextureAssets>,
    fonts: Res<FontAssets>,
    canvas: Res<CanvasImage>,
    vol: Res<GameVolume>,
) {
    let font = fonts.main.clone();
    info!("menu");
    commands.spawn((
        Camera2d,
        Camera {
            order: -1,
            clear_color: ClearColorConfig::Custom(Color::linear_rgba(0.1, 0.1, 0.1, 1.0)),
            ..default()
        },
        bevy::camera::RenderTarget::from(canvas.0.clone()),
        Msaa::Off,
        Menu,
        PaletteSqueeze::default(),
    ));

    // Tiled splash background
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            ..default()
        },
        ImageNode {
            image: textures.splash.clone(),
            image_mode: NodeImageMode::Tiled {
                tile_x: true,
                tile_y: true,
                stretch_value: 1.0,
            },
            ..default()
        },
        GlobalZIndex(-1),
        Menu,
    ));

    let textfont = TextFont {
        font: font.clone(),
        font_size: 32.0,
        font_smoothing: FontSmoothing::None,
        ..default()
    };

    let textbox_image = textures.textbox.clone();

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            Menu,
        ))
        .with_children(|children| {
            // Play button
            children
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(140.0),
                        height: Val::Px(50.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    ImageNode {
                        image: textbox_image.clone(),
                        image_mode: NodeImageMode::Sliced(textbox_slicer()),
                        ..default()
                    },
                    ChangeState(GameState::Playing),
                ))
                .with_child((
                    Text::new("Play"),
                    textfont.clone(),
                    TextColor(Color::linear_rgb(0.9, 0.9, 0.9)),
                ));
            // Volume row
            children
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    margin: UiRect::top(Val::Px(10.0)),
                    column_gap: Val::Px(4.0),
                    ..default()
                })
                .with_children(|row| {
                    // Minus button
                    row.spawn((
                        Button,
                        Node {
                            width: Val::Px(40.0),
                            height: Val::Px(40.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        ImageNode {
                            image: textbox_image.clone(),
                            image_mode: NodeImageMode::Sliced(textbox_slicer()),
                            ..default()
                        },
                        VolumeDown,
                    ))
                    .with_child((
                        Text::new("-"),
                        textfont.clone(),
                        TextColor(Color::linear_rgb(0.9, 0.9, 0.9)),
                    ));
                    // Volume display
                    row.spawn((
                        Node {
                            width: Val::Px(80.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        ImageNode {
                            image: textbox_image.clone(),
                            image_mode: NodeImageMode::Sliced(textbox_slicer()),
                            ..default()
                        },
                    ))
                    .with_child((
                        Text::new(format!("{:.0}%", vol.0 * 100.0)),
                        textfont.clone(),
                        TextColor(Color::linear_rgb(0.9, 0.9, 0.9)),
                        VolumeDisplay,
                    ));
                    // Plus button
                    row.spawn((
                        Button,
                        Node {
                            width: Val::Px(40.0),
                            height: Val::Px(40.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        ImageNode {
                            image: textbox_image.clone(),
                            image_mode: NodeImageMode::Sliced(textbox_slicer()),
                            ..default()
                        },
                        VolumeUp,
                    ))
                    .with_child((
                        Text::new("+"),
                        textfont.clone(),
                        TextColor(Color::linear_rgb(0.9, 0.9, 0.9)),
                    ));
                });
            // Exit button
            children
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(140.0),
                        height: Val::Px(50.0),
                        margin: UiRect::top(Val::Px(10.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    ImageNode {
                        image: textbox_image.clone(),
                        image_mode: NodeImageMode::Sliced(textbox_slicer()),
                        ..default()
                    },
                    ExitApp,
                ))
                .with_child((
                    Text::new("Exit"),
                    textfont.clone(),
                    TextColor(Color::linear_rgb(0.9, 0.9, 0.9)),
                ));
        });
    commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceAround,
                bottom: Val::Px(5.),
                width: Val::Percent(100.),
                position_type: PositionType::Absolute,
                ..default()
            },
            Menu,
        ))
        .with_children(|children| {
            children
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(170.0),
                        height: Val::Px(50.0),
                        justify_content: JustifyContent::SpaceAround,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(5.)),
                        ..default()
                    },
                    OpenLink("https://bevyengine.org"),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Made with Bevy"),
                        textfont.clone(),
                        TextColor(Color::linear_rgb(0.9, 0.9, 0.9)),
                    ));
                    parent.spawn((
                        ImageNode {
                            image: textures.bevy.clone(),
                            ..default()
                        },
                        Node {
                            width: Val::Px(32.),
                            ..default()
                        },
                    ));
                });
            children
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(170.0),
                        height: Val::Px(50.0),
                        justify_content: JustifyContent::SpaceAround,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(5.)),
                        ..default()
                    },
                    OpenLink("https://github.com/NiklasEi/bevy_game_template"),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Open source"),
                        textfont.clone(),
                        TextColor(Color::linear_rgb(0.9, 0.9, 0.9)),
                    ));
                    parent.spawn((
                        ImageNode::new(textures.github.clone()),
                        Node {
                            width: Val::Px(32.),
                            ..default()
                        },
                    ));
                });
        });
}

#[derive(Component)]
struct ChangeState(GameState);

#[derive(Component)]
struct OpenLink(&'static str);

#[derive(Component)]
struct ExitApp;

#[derive(Component)]
struct VolumeUp;

#[derive(Component)]
struct VolumeDown;

#[derive(Component)]
struct VolumeDisplay;

fn click_play_button(
    mut next_state: ResMut<NextState<GameState>>,
    mut exit: MessageWriter<AppExit>,
    mut interaction_query: Query<
        (
            &Interaction,
            Option<&mut ImageNode>,
            Option<&ChangeState>,
            Option<&OpenLink>,
            Option<&ExitApp>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    audio: Res<Audio>,
    audio_assets: Res<AudioAssets>,
) {
    for (interaction, image_node, change_state, open_link, exit_app) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                audio.play(audio_assets.fx1.clone());
                if let Some(state) = change_state {
                    next_state.set(state.0.clone());
                } else if exit_app.is_some() {
                    exit.write(AppExit::Success);
                } else if let Some(link) = open_link
                    && let Err(error) = webbrowser::open(link.0)
                {
                    warn!("Failed to open link {error:?}");
                }
            }
            Interaction::Hovered => {
                if let Some(mut img) = image_node {
                    img.color = Color::linear_rgb(0.6, 0.6, 0.6);
                }
            }
            Interaction::None => {
                if let Some(mut img) = image_node {
                    img.color = Color::WHITE;
                }
            }
        }
    }
}

fn handle_volume_buttons(
    mut vol: ResMut<GameVolume>,
    up_q: Query<&Interaction, (Changed<Interaction>, With<VolumeUp>)>,
    down_q: Query<&Interaction, (Changed<Interaction>, With<VolumeDown>)>,
    mut display_q: Query<&mut Text, With<VolumeDisplay>>,
    audio: Res<Audio>,
    audio_assets: Res<AudioAssets>,
) {
    let mut changed = false;
    for interaction in &up_q {
        if *interaction == Interaction::Pressed {
            vol.0 = (vol.0 + 0.1).min(1.0);
            audio.play(audio_assets.fx1.clone());
            changed = true;
        }
    }
    for interaction in &down_q {
        if *interaction == Interaction::Pressed {
            vol.0 = (vol.0 - 0.1).max(0.0);
            audio.play(audio_assets.fx1.clone());
            changed = true;
        }
    }
    if changed {
        for mut text in &mut display_q {
            **text = format!("{:.0}%", vol.0 * 100.0);
        }
    }
}

fn cleanup_menu(mut commands: Commands, menu: Query<Entity, With<Menu>>) {
    for entity in menu.iter() {
        commands.entity(entity).despawn();
    }
}
