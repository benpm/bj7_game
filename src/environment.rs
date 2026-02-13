use crate::GameState;
use crate::palette::PaletteDarken;
use bevy::prelude::*;

pub struct EnvironmentPlugin;

impl Plugin for EnvironmentPlugin {
    fn build(&self, app: &mut App) {
        app.add_sub_state::<Environment>()
            .add_systems(OnEnter(GameState::Playing), init_timers)
            .add_systems(
                Update,
                (tick_run_timer, tick_cycle_and_darken, update_label)
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

/// Tracks the post-switch recovery fade (darken 1→0).
#[derive(Resource)]
struct RecoveryTimer {
    timer: Timer,
}

/// HUD text showing current environment name.
#[derive(Component)]
struct EnvironmentLabel;

const RUN_DURATION: f32 = 300.0;
const CYCLE_INTERVAL: f32 = 60.0;
const TRANSITION_LEAD_SECS: f32 = 5.0;

fn init_timers(mut commands: Commands) {
    commands.insert_resource(RunTimer { elapsed: 0.0 });
    commands.insert_resource(CycleTimer {
        timer: Timer::from_seconds(CYCLE_INTERVAL, TimerMode::Repeating),
    });
    commands.insert_resource(PaletteDarken::default());

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

fn tick_cycle_and_darken(
    time: Res<Time>,
    mut cycle_timer: ResMut<CycleTimer>,
    mut darken: ResMut<PaletteDarken>,
    recovery: Option<ResMut<RecoveryTimer>>,
    mut commands: Commands,
    environment: Res<State<Environment>>,
    mut next_env: ResMut<NextState<Environment>>,
) {
    cycle_timer.timer.tick(time.delta());

    // Pre-switch darkening: last TRANSITION_LEAD_SECS of the cycle
    let remaining_frac = 1.0 - cycle_timer.timer.fraction();
    let transition_threshold = TRANSITION_LEAD_SECS / CYCLE_INTERVAL;

    if remaining_frac <= transition_threshold && recovery.is_none() {
        // Map remaining fraction to darken: 0→1 as we approach the switch
        darken.value = 1.0 - (remaining_frac / transition_threshold);
    }

    // Cycle fired: switch environment, start recovery
    if cycle_timer.timer.just_finished() {
        let next = environment.get().next();
        next_env.set(next);
        darken.value = 1.0;
        commands.insert_resource(RecoveryTimer {
            timer: Timer::from_seconds(TRANSITION_LEAD_SECS, TimerMode::Once),
        });
    }

    // Post-switch recovery: darken 1→0
    if let Some(mut recovery) = recovery {
        recovery.timer.tick(time.delta());
        darken.value = 1.0 - recovery.timer.fraction();
        if recovery.timer.fraction() >= 1.0 {
            darken.value = 0.0;
            commands.remove_resource::<RecoveryTimer>();
        }
    }
}

fn update_label(
    environment: Res<State<Environment>>,
    mut label_query: Query<&mut Text, With<EnvironmentLabel>>,
) {
    if environment.is_changed() {
        for mut text in &mut label_query {
            **text = environment.get().label().to_string();
        }
    }
}

fn cleanup_environment(
    mut commands: Commands,
    label_query: Query<Entity, With<EnvironmentLabel>>,
) {
    commands.remove_resource::<RunTimer>();
    commands.remove_resource::<CycleTimer>();
    commands.remove_resource::<PaletteDarken>();
    commands.remove_resource::<RecoveryTimer>();
    for entity in &label_query {
        commands.entity(entity).despawn();
    }
}
