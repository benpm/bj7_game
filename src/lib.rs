#![allow(clippy::type_complexity)]

mod aberration;
pub mod actor;
mod actions;
mod audio;
mod environment;
mod health;
mod loading;
mod menu;
mod palette;
mod player;
mod world;

use crate::aberration::AberrationPlugin;
use crate::actor::ActorPlugin;
use crate::actions::ActionsPlugin;
use crate::audio::InternalAudioPlugin;
use crate::environment::EnvironmentPlugin;
use crate::health::HealthPlugin;
use crate::loading::LoadingPlugin;
use crate::menu::MenuPlugin;
use crate::palette::PalettePlugin;
use crate::player::PlayerPlugin;
use crate::world::WorldPlugin;

use bevy::app::App;
#[cfg(debug_assertions)]
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
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
        app.init_state::<GameState>().add_plugins((
            LoadingPlugin,
            MenuPlugin,
            ActionsPlugin,
            InternalAudioPlugin,
            ActorPlugin,
            AberrationPlugin,
            HealthPlugin,
            EnvironmentPlugin,
            PalettePlugin,
            PlayerPlugin,
            WorldPlugin,
        ));

        #[cfg(debug_assertions)]
        {
            app.add_plugins((
                FrameTimeDiagnosticsPlugin::default(),
                LogDiagnosticsPlugin::default(),
            ));
        }
    }
}
