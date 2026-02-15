use crate::GameState;
use crate::palette::PaletteDarken;
use bevy::prelude::*;

pub struct TransitionPlugin;

impl Plugin for TransitionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Playing),
            start_fade_in_on_enter_playing,
        )
        .add_systems(OnEnter(GameState::Menu), start_fade_in_on_enter_menu)
        .add_systems(Update, tick_transition);
    }
}

/// Controls a fade-to-black / fade-from-black transition via PaletteDarken.
#[derive(Resource)]
pub struct SceneTransition {
    phase: TransitionPhase,
}

#[derive(Default)]
enum TransitionPhase {
    #[default]
    Idle,
    FadingOut {
        duration: f32,
        elapsed: f32,
    },
    FadingIn {
        duration: f32,
        elapsed: f32,
    },
}

#[allow(dead_code)]
impl SceneTransition {
    pub fn new() -> Self {
        Self {
            phase: TransitionPhase::Idle,
        }
    }

    /// Begin fading from current state to fully black over `duration` seconds.
    pub fn fade_out(&mut self, duration: f32) {
        self.phase = TransitionPhase::FadingOut {
            duration,
            elapsed: 0.0,
        };
    }

    /// Begin fading from fully black back to normal over `duration` seconds.
    pub fn fade_in(&mut self, duration: f32) {
        self.phase = TransitionPhase::FadingIn {
            duration,
            elapsed: 0.0,
        };
    }

    /// Returns true when the screen is fully black (darken >= 1.0) and idle or just finished fading out.
    pub fn is_fully_dark(&self) -> bool {
        matches!(self.phase, TransitionPhase::Idle) && self.darken_value() >= 1.0
    }

    /// Returns true when idle and not dark.
    pub fn is_idle(&self) -> bool {
        matches!(self.phase, TransitionPhase::Idle)
    }

    /// Current darken value: 0.0 = normal, 1.0 = fully black.
    pub fn darken_value(&self) -> f32 {
        match &self.phase {
            TransitionPhase::Idle => 0.0,
            TransitionPhase::FadingOut { duration, elapsed } => {
                (elapsed / duration).clamp(0.0, 1.0)
            }
            TransitionPhase::FadingIn { duration, elapsed } => {
                1.0 - (elapsed / duration).clamp(0.0, 1.0)
            }
        }
    }

    fn tick(&mut self, dt: f32) {
        match &mut self.phase {
            TransitionPhase::Idle => {}
            TransitionPhase::FadingOut { duration, elapsed } => {
                *elapsed += dt;
                if *elapsed >= *duration {
                    // Stay dark — caller decides what happens next
                    self.phase = TransitionPhase::Idle;
                }
            }
            TransitionPhase::FadingIn { duration, elapsed } => {
                *elapsed += dt;
                if *elapsed >= *duration {
                    self.phase = TransitionPhase::Idle;
                }
            }
        }
    }
}

fn tick_transition(
    time: Res<Time>,
    transition: Option<ResMut<SceneTransition>>,
    mut darken: Option<ResMut<PaletteDarken>>,
) {
    let Some(mut transition) = transition else {
        return;
    };
    transition.tick(time.delta_secs());
    if let Some(darken) = darken.as_mut() {
        darken.value = transition.darken_value();
    }
}

fn start_fade_in_on_enter_playing(mut commands: Commands) {
    commands.insert_resource(SceneTransition {
        phase: TransitionPhase::FadingIn {
            duration: 1.0,
            elapsed: 0.0,
        },
    });
}

fn start_fade_in_on_enter_menu(mut commands: Commands) {
    commands.insert_resource(PaletteDarken::default());
    commands.insert_resource(SceneTransition {
        phase: TransitionPhase::FadingIn {
            duration: 1.0,
            elapsed: 0.0,
        },
    });
}
