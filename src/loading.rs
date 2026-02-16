use crate::GameState;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(GameState::Loading)
                .continue_to_state(GameState::Menu)
                .load_collection::<FontAssets>()
                .load_collection::<TextureAssets>(),
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
    #[asset(path = "textures/FeatherCursor.png")]
    pub feather_cursor: Handle<Image>,
}
