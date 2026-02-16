// disable console on windows for release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::DefaultPlugins;
use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;
use bevy_embedded_assets::EmbeddedAssetPlugin;
use bevy_game::GamePlugin; // ToDo: Replace bevy_game with your new crate name.

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(EmbeddedAssetPlugin::default())
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Bevy game".to_string(), // ToDo
                        // Tells wasm not to override default event handling, like F5 and Ctrl+R
                        prevent_default_event_handling: false,
                        present_mode: bevy::window::PresentMode::AutoNoVsync,
                        fit_canvas_to_parent: true,
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(GamePlugin)
        .run();
}
