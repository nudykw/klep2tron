use bevy::prelude::*;
use crate::GameState;

pub mod ui_root;
pub mod ui_inspector;
pub mod ui_project;
pub mod systems_logic;
pub mod navigation;
pub mod widgets;

#[cfg(not(target_arch = "wasm32"))]
pub struct ActorEditorPlugin;

#[cfg(not(target_arch = "wasm32"))]
impl Plugin for ActorEditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(bevy_panorbit_camera::PanOrbitCameraPlugin)
           .init_resource::<ui_inspector::SocketFilter>()
           .init_resource::<ui_inspector::SelectedSocket>()
           .init_resource::<widgets::PanelSettings>()
           .init_resource::<ViewportSettings>()
           .init_resource::<EditorStatus>()
           .init_resource::<EditorMaterialColor>()
           .init_resource::<PendingImport>()
           .init_resource::<ImportProgress>()
           .add_event::<ResetCameraEvent>()
           .add_event::<ActorSaveEvent>()
           .add_event::<ActorImportEvent>()
           .add_event::<ToastEvent>()
           .add_event::<ConfirmationRequestEvent>()
           .add_systems(OnEnter(GameState::ActorEditor), (ui_root::setup_actor_editor, navigation::setup_navigation).chain())
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
                systems_logic::gizmo_sync_system,
                navigation::camera_reset_handler,
                navigation::grid_system,
           ).run_if(in_state(GameState::ActorEditor)))
           .add_systems(Update, (
                systems_logic::gizmo_viewport_system,
                widgets::viewport_button_system,
                systems_logic::status_update_system,
                systems_logic::polycount_update_system,
                systems_logic::toast_manager_system,
                systems_logic::modal_manager_system,
                systems_logic::color_picker_system,
                systems_logic::material_sync_system,
                systems_logic::actor_import_button_system,
                systems_logic::actor_import_event_system,
                systems_logic::actor_import_processing_system,
                systems_logic::progress_bar_update_system,
                systems_logic::import_loading_overlay_system,
                systems_logic::normalization_system,
           ).run_if(in_state(GameState::ActorEditor)))
           .add_systems(OnExit(GameState::ActorEditor), (ui_root::cleanup_actor_editor, crate::reset_ambient_light));
    }
}

// Events for decoupling (SOLID)
#[derive(Event)]
pub struct ActorSaveEvent;
#[derive(Event)]
pub struct ActorImportEvent(pub std::path::PathBuf);

#[derive(Event)]
pub struct ResetCameraEvent;

#[derive(Resource, Default, PartialEq, Eq)]
pub enum EditorStatus {
    #[default]
    Ready,
    Saving,
    Loading,
    Processing,
}

#[derive(Resource)]
pub struct EditorMaterialColor {
    pub color: Color,
    pub hue: f32,
    pub is_open: bool,
}

impl Default for EditorMaterialColor {
    fn default() -> Self {
        Self {
            color: Color::srgb(0.7, 0.7, 0.7),
            hue: 0.0,
            is_open: false,
        }
    }
}

#[derive(Clone, Copy)]
pub enum ToastType {
    Info,
    Success,
    Error,
}

#[derive(Event)]
pub struct ToastEvent {
    pub message: String,
    pub toast_type: ToastType,
}

#[derive(Clone, Copy)]
pub enum EditorAction {
    BackToMenu,
}

#[derive(Event)]
pub struct ConfirmationRequestEvent {
    pub title: String,
    pub message: String,
    pub action: EditorAction,
}

#[derive(Resource)]
pub struct ViewportSettings {
    pub grid: bool,
    pub slices: bool,
    pub sockets: bool,
    pub gizmos: bool,
}

impl Default for ViewportSettings {
    fn default() -> Self {
        Self {
            grid: true,
            slices: true,
            sockets: true,
            gizmos: true,
        }
    }
}

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
pub struct MainEditorCamera;

#[derive(Component)]
pub struct GizmoCamera;

#[derive(Component)]
pub struct GizmoEntity;

#[derive(Component)]
pub struct EditorHelper;

#[derive(Component)]
pub struct ActorEditorBackButton;

#[derive(Component)]
pub struct OriginalMeshComponent(pub Handle<Mesh>);

#[derive(Resource, Default)]
pub struct PendingImport {
    pub handle: Option<Handle<Scene>>,
    pub mesh_handle: Option<Handle<Mesh>>,
}

#[derive(Component)]
pub struct AwaitingNormalization;

#[derive(Component)]
pub struct NormalizationState {
    pub entities_to_process: Vec<Entity>,
    pub processed_count: usize,
    pub min: Vec3,
    pub max: Vec3,
    pub found_meshes: Vec<(Entity, Handle<Mesh>)>,
}

#[derive(Resource, Default)]
pub struct ImportProgress(pub f32);

pub const GIZMO_LAYER: bevy::render::view::RenderLayers = bevy::render::view::RenderLayers::layer(1);
