use crate::GameState;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_kira_audio::AudioSource;
pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(GameState::Loading)
                .continue_to_state(GameState::Menu)
                .load_collection::<FontAssets>()
                .load_collection::<TextureAssets>()
                .load_collection::<AudioAssets>(),
        );
    }
}

#[derive(AssetCollection, Resource)]
pub struct FontAssets {
    #[asset(path = "fonts/sd_auto_pilot.ttf")]
    pub main: Handle<Font>,
}

#[derive(AssetCollection, Resource)]
pub struct TextureAssets {
    #[asset(path = "textures/bevy.png")]
    pub bevy: Handle<Image>,
    #[asset(path = "textures/github.png")]
    pub github: Handle<Image>,
    #[asset(path = "textures/textbox9x9.png")]
    pub textbox: Handle<Image>,
    #[asset(path = "textures/unknown.png")]
    pub unknown: Handle<Image>,
    #[asset(path = "textures/WhiteFeatherCursor.png")]
    pub feather_cursor: Handle<Image>,
    #[asset(path = "textures/splash.png")]
    pub splash: Handle<Image>,
    #[asset(path = "textures/death.png")]
    pub death: Handle<Image>,
}

#[derive(AssetCollection, Resource)]
pub struct AudioAssets {
    #[asset(path = "audio/death.mp3")]
    pub death: Handle<AudioSource>,
    #[asset(path = "audio/dispel.mp3")]
    pub dispel: Handle<AudioSource>,
    #[asset(path = "audio/footsteps.ogg")]
    pub footsteps: Handle<AudioSource>,
    #[asset(path = "audio/fx1.wav")]
    pub fx1: Handle<AudioSource>,
    #[asset(path = "audio/talk.mp3")]
    pub talk: Handle<AudioSource>,
}
