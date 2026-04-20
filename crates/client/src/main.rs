use bevy::prelude::*;
use client_core::ClientCorePlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Klep2tron Client".into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(bevy_obj::ObjPlugin)
        .add_plugins(ClientCorePlugin)
        .run();
}
