use bevy::prelude::*;
use client_core::ClientCorePlugin;

fn main() {
    client_core::pre_init_gpu_settings();
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Klep2tron Client".into(),
                ..default()
            }),
            ..default()
        }).set(bevy::render::RenderPlugin {
            render_creation: bevy::render::settings::RenderCreation::Automatic(client_core::get_wgpu_settings()),
            ..default()
        }))
        .add_plugins(bevy_obj::ObjPlugin)
        .add_plugins(ClientCorePlugin::default())
        .run();
}
