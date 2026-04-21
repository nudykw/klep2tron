use bevy::prelude::*;
use crate::GameState;

pub mod ui_root;
pub mod ui_inspector;
pub mod ui_project;
pub mod systems_logic;
pub mod widgets;

#[cfg(not(target_arch = "wasm32"))]
pub struct ActorEditorPlugin;

#[cfg(not(target_arch = "wasm32"))]
impl Plugin for ActorEditorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ui_inspector::SocketFilter>()
           .init_resource::<ui_inspector::SelectedSocket>()
           .init_resource::<widgets::PanelSettings>()
           .add_systems(OnEnter(GameState::ActorEditor), ui_root::setup_actor_editor)
           .add_systems(Update, (
                systems_logic::actor_editor_input_system,
                widgets::collapsible_system,
                widgets::scroll_system,
                widgets::slider_system,
                widgets::panel_resize_system,
                widgets::update_panel_style_system,
                widgets::panel_toggle_system,
                widgets::tooltip_system,
                ui_inspector::socket_filter_system,
                ui_inspector::socket_transform_update_system,
           ).run_if(in_state(GameState::ActorEditor)))
           .add_systems(OnExit(GameState::ActorEditor), (ui_root::cleanup_actor_editor, crate::reset_ambient_light));
    }
}

// Events for decoupling (SOLID)
pub struct ActorSaveEvent;
pub struct ActorImportEvent;

#[derive(Event)]
pub struct SocketUpdateEvent {
    pub entity: Entity,
    pub transform: Transform,
}

#[derive(Event)]
pub struct MaterialUpdateEvent {
    pub color: Color,
    pub metallic: f32,
    pub roughness: f32,
}

#[derive(Component)]
pub struct ActorEditorEntity;

#[derive(Component)]
pub struct ActorEditorBackButton;
