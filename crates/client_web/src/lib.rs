use bevy::prelude::*;
use client_core::ClientCorePlugin;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn run() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Klep2tron Web Client".into(),
                fit_canvas_to_parent: true,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(bevy_obj::ObjPlugin)
        .add_plugins(ClientCorePlugin)
        .run();
}
