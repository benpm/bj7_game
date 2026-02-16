use crate::GameState;
use crate::loading::{FontAssets, TextureAssets};
use crate::palette::PaletteSqueeze;
use crate::scaling::CanvasImage;
use bevy::prelude::*;
use bevy::text::FontSmoothing;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Menu), setup_menu)
            .add_systems(Update, click_play_button.run_if(in_state(GameState::Menu)))
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
) {
    for (interaction, image_node, change_state, open_link, exit_app) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
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

fn cleanup_menu(mut commands: Commands, menu: Query<Entity, With<Menu>>) {
    for entity in menu.iter() {
        commands.entity(entity).despawn();
    }
}
