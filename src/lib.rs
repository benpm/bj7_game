#![allow(clippy::type_complexity)]

mod aberration;
mod actions;
pub mod actor;
pub mod audio;
mod death;
mod dialog;
mod dispel;
mod environment;
mod health;
mod loading;
mod menu;
mod palette;
mod pause;
mod player;
pub mod scaling;
mod transition;
mod world;

use crate::aberration::AberrationPlugin;
use crate::actions::ActionsPlugin;
use crate::actor::ActorPlugin;
use crate::audio::GameAudioPlugin;
use crate::death::DeathPlugin;
use crate::dialog::DialogPlugin;
use crate::dispel::DispelPlugin;
use crate::environment::EnvironmentPlugin;
use crate::health::HealthPlugin;
use crate::loading::LoadingPlugin;
use crate::menu::MenuPlugin;
use crate::palette::PalettePlugin;
use crate::pause::PausePlugin;
use crate::player::PlayerPlugin;
use crate::scaling::ScalingPlugin;
use crate::transition::TransitionPlugin;
use crate::world::WorldPlugin;

use bevy::app::App;
use bevy::prelude::*;

#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
pub enum GameState {
    #[default]
    Loading,
    Playing,
    Menu,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(UiScale(2.0))
            .init_state::<GameState>()
            .add_plugins((
                ScalingPlugin,
                GameAudioPlugin,
                LoadingPlugin,
                MenuPlugin,
                ActionsPlugin,
                ActorPlugin,
                AberrationPlugin,
                DeathPlugin,
                DialogPlugin,
            ))
            .add_plugins((
                DispelPlugin,
                HealthPlugin,
                EnvironmentPlugin,
                PalettePlugin,
                PausePlugin,
                PlayerPlugin,
                TransitionPlugin,
                WorldPlugin,
            ));
    }
}
