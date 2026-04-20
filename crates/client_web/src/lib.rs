use bevy::prelude::*;
use client_core::ClientCorePlugin;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn run() {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Klep2tron Web Client".into(),
                fit_canvas_to_parent: true,
                ..default()
            }),
            ..default()
        }).set(AssetPlugin {
            meta_check: bevy::asset::AssetMetaCheck::Never,
            ..default()
        }))
        .add_plugins(bevy_obj::ObjPlugin)
        .add_plugins(ClientCorePlugin::default())
        .run();
}
