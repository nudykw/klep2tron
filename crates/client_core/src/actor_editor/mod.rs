use bevy::prelude::*;
use crate::GameState;
pub use shared::npc::{ActorPart, SocketDefinition};

pub mod ui_root;
pub mod ui_inspector;
pub mod ui_project;
pub mod systems;
pub mod geometry;
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
           .init_resource::<SlicingSettings>()
           .init_resource::<EditorStatus>()
           .init_resource::<EditorMaterialColor>()
           .init_resource::<PendingImport>()
           .init_resource::<ImportProgress>()
           .init_resource::<systems::SlicingTask>()
           .init_resource::<InspectionSettings>()
           .add_event::<ResetCameraEvent>()
           .add_event::<ActorSaveEvent>()
           .add_event::<ActorImportEvent>()
           .add_event::<ToastEvent>()
           .add_event::<ConfirmationRequestEvent>()
           .add_event::<InspectionFocusEvent>()
           .add_systems(OnEnter(GameState::ActorEditor), (ui_root::setup_actor_editor, navigation::setup_navigation).chain())
           .add_systems(Update, (
                systems::actor_editor_input_system,
                widgets::collapsible_system,
                widgets::scroll_system,
                widgets::slider_system,
                widgets::panel_resize_system,
                widgets::update_panel_style_system,
                widgets::panel_toggle_system,
                widgets::tooltip_system,
                ui_inspector::socket_filter_system,
                ui_inspector::socket_transform_update_system,
                systems::gizmo_sync_system,
                navigation::camera_reset_handler,
                navigation::grid_system,
                navigation::camera_control_blocking_system,
           ).run_if(in_state(GameState::ActorEditor)))
           .add_systems(Update, (
                systems::gizmo_viewport_system,
                widgets::viewport_button_system,
                widgets::range_slider_system,
                systems::status_update_system,
                systems::polycount_update_system,
                systems::toast_manager_system,
                systems::modal_manager_system,
                systems::color_picker_system,
                systems::material_sync_system,
                systems::actor_import_button_system,
                systems::actor_import_event_system,
                systems::actor_import_processing_system,
                systems::progress_bar_update_system,
                systems::import_loading_overlay_system,
                systems::normalization_system,
           ).run_if(in_state(GameState::ActorEditor)))
           .add_systems(Update, (
                    systems::slicing_ui_sync_system,
                    systems::slicing_ui_visibility_system,
                    systems::slicing_gizmo_manager_system,
                    systems::slicing_gizmo_sync_system,
                    systems::slicer_lock_system,
                    systems::inspection_input_system,
                    systems::inspection_visibility_system,
                    systems::inspection_camera_focus_system,
                    systems::inspection_highlight_system,
                    systems::inspection_debug_draw_system,
                    systems::inspection_ui_logic_system,
                    systems::inspection_ui_sync_system,
                ).chain().run_if(in_state(GameState::ActorEditor)))

           .add_systems(PostUpdate, (
                systems::mesh_slicing_system,
                systems::draw_slicing_contours_system,
                systems::draw_actor_bounds_debug_system,
            ).after(bevy::transform::TransformSystem::TransformPropagate)
             .run_if(in_state(GameState::ActorEditor)))
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

#[derive(Resource, Default)]
pub struct SocketSettings {
    pub is_adding: bool,
    pub show_visuals: bool,
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

#[derive(Resource)]
pub struct SlicingSettings {
    pub top_cut: f32,
    pub bottom_cut: f32,
    pub preview: bool,
    pub locked: bool,
    pub hovered_gizmo: Option<SlicingGizmoType>,
    // Confirmation mechanic
    pub dragging_gizmo: Option<SlicingGizmoType>,
    pub needs_confirm: bool,
    pub confirm_pos: Vec3,
    pub trigger_slice: bool,
    // Internal state to track changes

    pub last_top: f32,
    pub last_bottom: f32,
}

impl Default for SlicingSettings {
    fn default() -> Self {
        Self {
            top_cut: 1.0,
            bottom_cut: 0.0,
            preview: true,
            locked: false,
            hovered_gizmo: None,
            dragging_gizmo: None,
            needs_confirm: false,
            confirm_pos: Vec3::ZERO,
            trigger_slice: false,
            last_top: -1.0,

            last_bottom: -1.0,
        }
    }
}


#[derive(Component, Default)]
pub struct SlicingContours {
    pub segments: Vec<[Vec3; 2]>,
}

#[derive(Component)]
pub struct ActorBounds {
    pub min: Vec3,
    pub max: Vec3,
    pub original_polys: usize,
}

#[derive(Component)]
pub struct SlicingGizmo;

#[derive(Component)]
pub struct ConfirmationCircle;


#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelResizer {
    Left,
    Right,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlicingGizmoType {
    Top,
    Bottom,
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct ActorSocket {
    pub definition: SocketDefinition,
}

#[derive(Resource, Default)]
pub struct InspectionSettings {
    pub is_active: bool,
    pub isolated_part: Option<ActorPart>,
    pub ghost_mode: bool,
    pub wireframe: bool,
    pub show_normals: bool,
    pub hovered_part: Option<ActorPart>,
}

#[derive(Event)]
pub struct InspectionFocusEvent(pub ActorPart);

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
    pub total_original_polys: usize,
}

#[derive(Resource, Default)]
pub struct ImportProgress(pub f32);

pub const GIZMO_LAYER: bevy::render::view::RenderLayers = bevy::render::view::RenderLayers::layer(1);
