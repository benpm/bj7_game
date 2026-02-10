use crate::GameState;
use bevy::prelude::*;

pub struct EnvironmentPlugin;

impl Plugin for EnvironmentPlugin {
    fn build(&self, app: &mut App) {
        app.add_sub_state::<Environment>()
            .add_systems(
                OnEnter(GameState::Playing),
                (init_timers, spawn_faint_overlay),
            )
            .add_systems(
                Update,
                (tick_run_timer, tick_cycle_timer, update_faint_transition)
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(OnExit(GameState::Playing), cleanup_environment);
    }
}

/// The three fever-dream environments the player cycles through.
#[derive(SubStates, Clone, Eq, PartialEq, Debug, Hash, Default)]
#[source(GameState = GameState::Playing)]
pub enum Environment {
    #[default]
    Delirium,
    Dissociation,
    Hypervigilance,
}

impl Environment {
    fn next(&self) -> Self {
        match self {
            Environment::Delirium => Environment::Dissociation,
            Environment::Dissociation => Environment::Hypervigilance,
            Environment::Hypervigilance => Environment::Delirium,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Environment::Delirium => "DELIRIUM",
            Environment::Dissociation => "DISSOCIATION",
            Environment::Hypervigilance => "HYPERVIGILANCE",
        }
    }
}

/// Total elapsed time for the run. Game ends at RUN_DURATION.
#[derive(Resource)]
struct RunTimer {
    elapsed: f32,
}

/// Cycles between environments on a fixed interval.
#[derive(Resource)]
struct CycleTimer {
    timer: Timer,
}

const RUN_DURATION: f32 = 300.0; // 5 minutes
const CYCLE_INTERVAL: f32 = 60.0; // Switch environment every 60 seconds
const FAINT_FADE_SECS: f32 = 0.5;

/// Tracks the white-out faint transition between environments.
#[derive(Resource)]
struct FaintTransition {
    phase: FaintPhase,
    next_environment: Option<Environment>,
}

enum FaintPhase {
    None,
    FadingOut(Timer),
    FadingIn(Timer),
}

impl Default for FaintTransition {
    fn default() -> Self {
        Self {
            phase: FaintPhase::None,
            next_environment: None,
        }
    }
}

#[derive(Component)]
struct FaintOverlay;

/// HUD text showing current environment name.
#[derive(Component)]
struct EnvironmentLabel;

fn init_timers(mut commands: Commands) {
    commands.insert_resource(RunTimer { elapsed: 0.0 });
    commands.insert_resource(CycleTimer {
        timer: Timer::from_seconds(CYCLE_INTERVAL, TimerMode::Repeating),
    });
    commands.insert_resource(FaintTransition::default());

    // Environment label in top-center
    commands.spawn((
        Text::new("DELIRIUM"),
        TextFont {
            font_size: 28.0,
            ..default()
        },
        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.7)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(20.0),
            width: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            ..default()
        },
        GlobalZIndex(40),
        EnvironmentLabel,
    ));
}

fn spawn_faint_overlay(mut commands: Commands) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            ..default()
        },
        GlobalZIndex(60),
        BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.0)),
        FaintOverlay,
    ));
}

fn tick_run_timer(
    time: Res<Time>,
    mut run_timer: ResMut<RunTimer>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    run_timer.elapsed += time.delta_secs();
    if run_timer.elapsed >= RUN_DURATION {
        next_game_state.set(GameState::Menu);
    }
}

fn tick_cycle_timer(
    time: Res<Time>,
    mut cycle_timer: ResMut<CycleTimer>,
    mut faint: ResMut<FaintTransition>,
    environment: Res<State<Environment>>,
) {
    cycle_timer.timer.tick(time.delta());

    if cycle_timer.timer.just_finished() && matches!(faint.phase, FaintPhase::None) {
        faint.phase = FaintPhase::FadingOut(Timer::from_seconds(FAINT_FADE_SECS, TimerMode::Once));
        faint.next_environment = Some(environment.get().next());
    }
}

fn update_faint_transition(
    time: Res<Time>,
    mut faint: ResMut<FaintTransition>,
    mut overlay_query: Query<&mut BackgroundColor, With<FaintOverlay>>,
    mut next_env: ResMut<NextState<Environment>>,
    mut label_query: Query<&mut Text, With<EnvironmentLabel>>,
) {
    let mut new_phase = None;

    match &mut faint.phase {
        FaintPhase::None => {}
        FaintPhase::FadingOut(timer) => {
            timer.tick(time.delta());
            let alpha = timer.fraction();
            for mut bg in &mut overlay_query {
                bg.0 = Color::srgba(1.0, 1.0, 1.0, alpha);
            }
            if timer.is_finished() {
                // At peak white-out: switch environment
                if let Some(env) = faint.next_environment.take() {
                    for mut text in &mut label_query {
                        **text = env.label().to_string();
                    }
                    next_env.set(env);
                }
                new_phase = Some(FaintPhase::FadingIn(Timer::from_seconds(
                    FAINT_FADE_SECS,
                    TimerMode::Once,
                )));
            }
        }
        FaintPhase::FadingIn(timer) => {
            timer.tick(time.delta());
            let alpha = 1.0 - timer.fraction();
            for mut bg in &mut overlay_query {
                bg.0 = Color::srgba(1.0, 1.0, 1.0, alpha);
            }
            if timer.is_finished() {
                new_phase = Some(FaintPhase::None);
            }
        }
    }

    if let Some(phase) = new_phase {
        faint.phase = phase;
    }
}

fn cleanup_environment(
    mut commands: Commands,
    faint_query: Query<Entity, With<FaintOverlay>>,
    label_query: Query<Entity, With<EnvironmentLabel>>,
) {
    commands.remove_resource::<RunTimer>();
    commands.remove_resource::<CycleTimer>();
    commands.remove_resource::<FaintTransition>();
    for entity in faint_query.iter().chain(label_query.iter()) {
        commands.entity(entity).despawn();
    }
}
