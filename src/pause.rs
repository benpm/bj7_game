use crate::GameState;
use crate::environment::{Environment, RunTimer};
use crate::health::Health;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

pub struct PausePlugin;

impl Plugin for PausePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), init_paused)
            .add_systems(
                Update,
                (toggle_pause, manage_pause_menu, handle_pause_buttons)
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(OnExit(GameState::Playing), cleanup_pause);
    }
}

#[derive(Resource, Default)]
pub struct Paused(pub bool);

pub fn game_not_paused(paused: Option<Res<Paused>>) -> bool {
    paused.map_or(true, |p| !p.0)
}

#[derive(Component)]
struct PauseMenu;

#[derive(Component)]
struct PauseContinue;

#[derive(Component)]
struct PauseExit;

#[derive(Component)]
struct PauseStatText;

fn init_paused(mut commands: Commands) {
    commands.insert_resource(Paused(false));
}

fn toggle_pause(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut paused: ResMut<Paused>,
    mut cursor_q: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        paused.0 = !paused.0;
        if let Ok(mut cursor) = cursor_q.single_mut() {
            if paused.0 {
                cursor.grab_mode = CursorGrabMode::None;
                cursor.visible = true;
            } else {
                cursor.grab_mode = CursorGrabMode::Locked;
                cursor.visible = false;
            }
        }
    }
}

fn manage_pause_menu(
    paused: Res<Paused>,
    mut commands: Commands,
    menu_query: Query<Entity, With<PauseMenu>>,
    health: Option<Res<Health>>,
    run_timer: Option<Res<RunTimer>>,
    environment: Option<Res<State<Environment>>>,
) {
    if !paused.is_changed() {
        // Update stats text if paused
        return;
    }

    if paused.0 && menu_query.is_empty() {
        let health_pct = health.map_or(100.0, |h| h.fraction() * 100.0);
        let env_name = environment.map_or("---", |e| e.get().label());
        let elapsed = run_timer.map_or(0.0, |t| t.elapsed);
        let minutes = (elapsed / 60.0) as u32;
        let seconds = (elapsed % 60.0) as u32;

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
                GlobalZIndex(100),
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
                PauseMenu,
            ))
            .with_children(|parent| {
                // Title
                parent.spawn((
                    Text::new("PAUSED"),
                    TextFont {
                        font_size: 48.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    Node {
                        margin: UiRect::bottom(Val::Px(30.0)),
                        ..default()
                    },
                ));

                // Stats
                let stats = format!(
                    "Sanity: {:.0}%\nEnvironment: {}\nTime: {}:{:02}",
                    health_pct, env_name, minutes, seconds
                );
                parent.spawn((
                    Text::new(stats),
                    TextFont {
                        font_size: 24.0,
                        ..default()
                    },
                    TextColor(Color::srgba(0.8, 0.8, 0.8, 1.0)),
                    Node {
                        margin: UiRect::bottom(Val::Px(40.0)),
                        ..default()
                    },
                    PauseStatText,
                ));

                // Continue button
                parent
                    .spawn((
                        Button,
                        Node {
                            width: Val::Px(200.0),
                            height: Val::Px(50.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            margin: UiRect::bottom(Val::Px(10.0)),
                            ..default()
                        },
                        BackgroundColor(Color::linear_rgb(0.15, 0.15, 0.15)),
                        PauseContinue,
                    ))
                    .with_child((
                        Text::new("Continue"),
                        TextFont {
                            font_size: 32.0,
                            ..default()
                        },
                        TextColor(Color::linear_rgb(0.9, 0.9, 0.9)),
                    ));

                // Exit to Menu button
                parent
                    .spawn((
                        Button,
                        Node {
                            width: Val::Px(200.0),
                            height: Val::Px(50.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::linear_rgb(0.15, 0.15, 0.15)),
                        PauseExit,
                    ))
                    .with_child((
                        Text::new("Exit to Menu"),
                        TextFont {
                            font_size: 32.0,
                            ..default()
                        },
                        TextColor(Color::linear_rgb(0.9, 0.9, 0.9)),
                    ));
            });
    } else if !paused.0 {
        for entity in &menu_query {
            commands.entity(entity).despawn();
        }
    }
}

fn handle_pause_buttons(
    mut paused: ResMut<Paused>,
    mut next_state: ResMut<NextState<GameState>>,
    continue_q: Query<&Interaction, (Changed<Interaction>, With<PauseContinue>)>,
    exit_q: Query<&Interaction, (Changed<Interaction>, With<PauseExit>)>,
    mut cursor_q: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    for interaction in &continue_q {
        if *interaction == Interaction::Pressed {
            paused.0 = false;
            if let Ok(mut cursor) = cursor_q.single_mut() {
                cursor.grab_mode = CursorGrabMode::Locked;
                cursor.visible = false;
            }
        }
    }
    for interaction in &exit_q {
        if *interaction == Interaction::Pressed {
            paused.0 = false;
            next_state.set(GameState::Menu);
        }
    }
}

fn cleanup_pause(
    mut commands: Commands,
    menu_query: Query<Entity, With<PauseMenu>>,
) {
    commands.remove_resource::<Paused>();
    for entity in &menu_query {
        commands.entity(entity).despawn();
    }
}
