use crate::GameState;
use crate::environment::RunTimer;
use crate::loading::{AudioAssets, FontAssets, TextureAssets};
use crate::pause::Paused;
use bevy::prelude::*;
use bevy::text::FontSmoothing;
use bevy_kira_audio::{Audio, AudioControl};

pub struct DeathPlugin;

impl Plugin for DeathPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (show_death_screen, handle_death_button)
                .chain()
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(OnExit(GameState::Playing), cleanup_death);
    }
}

/// Insert this resource to trigger the death screen.
#[derive(Resource)]
pub struct Dead;

#[derive(Component)]
struct DeathScreen;

#[derive(Component)]
struct DeathReturnButton;

fn textbox_slicer() -> TextureSlicer {
    TextureSlicer {
        border: BorderRect::all(16.0),
        center_scale_mode: SliceScaleMode::Stretch,
        sides_scale_mode: SliceScaleMode::Stretch,
        max_corner_scale: 1.0,
    }
}

fn show_death_screen(
    mut commands: Commands,
    dead: Option<Res<Dead>>,
    existing: Query<(), With<DeathScreen>>,
    fonts: Res<FontAssets>,
    textures: Res<TextureAssets>,
    run_timer: Option<Res<RunTimer>>,
    mut paused: ResMut<Paused>,
    audio: Res<Audio>,
    audio_assets: Res<AudioAssets>,
) {
    if dead.is_none() || !existing.is_empty() {
        return;
    }

    audio.play(audio_assets.death.clone());

    // Freeze gameplay
    paused.0 = true;

    let elapsed = run_timer.map_or(0.0, |t| t.elapsed);
    let minutes = (elapsed / 60.0) as u32;
    let seconds = (elapsed % 60.0) as u32;

    let font = fonts.main.clone();
    let textbox_image = textures.textbox.clone();

    // Fullscreen overlay
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            GlobalZIndex(200),
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.85)),
            DeathScreen,
        ))
        .with_children(|parent| {
            // Death image
            parent.spawn((
                ImageNode::new(textures.death.clone()),
                Node {
                    width: Val::Px(256.0),
                    height: Val::Px(256.0),
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },
            ));

            // Survival time
            parent.spawn((
                Text::new(format!("You survived {}:{:02}", minutes, seconds)),
                TextFont {
                    font: font.clone(),
                    font_size: 32.0,
                    font_smoothing: FontSmoothing::None,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::bottom(Val::Px(30.0)),
                    ..default()
                },
            ));

            // Return to Menu button
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(220.0),
                        height: Val::Px(50.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    ImageNode {
                        image: textbox_image,
                        image_mode: NodeImageMode::Sliced(textbox_slicer()),
                        ..default()
                    },
                    DeathReturnButton,
                ))
                .with_child((
                    Text::new("Return to Menu"),
                    TextFont {
                        font,
                        font_size: 32.0,
                        font_smoothing: FontSmoothing::None,
                        ..default()
                    },
                    TextColor(Color::linear_rgb(0.9, 0.9, 0.9)),
                ));
        });
}

fn handle_death_button(
    mut next_state: ResMut<NextState<GameState>>,
    query: Query<&Interaction, (Changed<Interaction>, With<DeathReturnButton>)>,
    audio: Res<Audio>,
    audio_assets: Res<AudioAssets>,
) {
    for interaction in &query {
        if *interaction == Interaction::Pressed {
            audio.play(audio_assets.fx1.clone());
            next_state.set(GameState::Menu);
        }
    }
}

fn cleanup_death(mut commands: Commands, query: Query<Entity, With<DeathScreen>>) {
    commands.remove_resource::<Dead>();
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
