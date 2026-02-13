use crate::GameState;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(GameState::Loading)
                .continue_to_state(GameState::Menu)
                .load_collection::<TextureAssets>(),
        );
    }
}

#[derive(AssetCollection, Resource)]
pub struct TextureAssets {
    #[asset(path = "textures/bevy.png")]
    pub bevy: Handle<Image>,
    #[asset(path = "textures/github.png")]
    pub github: Handle<Image>,
    #[asset(
        paths(
            "textures/1_pink_aberration.png",
            "textures/2_pink_aberration.png",
            "textures/3_pink_aberration.png",
            "textures/4_pink_aberration.png",
        ),
        collection(typed)
    )]
    pub aberrations: Vec<Handle<Image>>,
}
