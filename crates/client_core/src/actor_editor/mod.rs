use bevy::prelude::*;
use crate::GameState;
pub use shared::npc::{ActorPart, SocketDefinition};

pub mod ui;
pub mod ui_project;
pub mod systems;
pub mod geometry;
pub mod navigation;
pub mod widgets;
pub mod vfx_assets;

#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorMode {
    #[default]
    Slicing,
    Sockets,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Resource)]
pub struct EditorFonts {
    pub regular: Handle<Font>,
    pub icon: Handle<Font>,
}

pub struct ActorEditorPlugin;

#[cfg(not(target_arch = "wasm32"))]
impl Plugin for ActorEditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(bevy_panorbit_camera::PanOrbitCameraPlugin)
           .add_plugins(bevy_mod_picking::DefaultPickingPlugins)
           .add_plugins(bevy_hanabi::HanabiPlugin)
           .add_plugins(bevy::pbr::wireframe::WireframePlugin)
           .init_resource::<EditorMode>()
           .init_resource::<ui::inspector::SelectedSocket>()
           .init_resource::<ui::inspector::MultiSelectionState>()
           .init_resource::<ui::inspector::SocketFilterState>()
           .init_resource::<widgets::PanelSettings>()
           .init_resource::<ViewportSettings>()
           .init_resource::<SlicingSettings>()
           .init_resource::<EditorStatus>()
           .init_resource::<EditorMaterialColor>()
           .init_resource::<PendingImport>()
           .init_resource::<ImportProgress>()
           .init_resource::<CurrentProject>()
           .init_resource::<systems::SlicingTask>()
           .init_resource::<InspectionSettings>()
           .init_resource::<SocketSettings>()
           .init_resource::<SocketColorPickerState>()
           .init_resource::<GizmoBusy>()
           .init_resource::<vfx_assets::VfxPresets>()
           .init_resource::<vfx_assets::VfxRegistry>()
           .init_resource::<systems::undo_redo::ActionStack>()
           .init_resource::<systems::optimization::OptimizationSettings>()
           .init_resource::<systems::optimization::OptimizationTask>()
           .init_resource::<PendingSockets>()
            .init_resource::<LastUsedDirectory>()
           .add_event::<ResetCameraEvent>()
           .add_event::<ActorSaveEvent>()
           .add_event::<ActorImportEvent>()
           .add_event::<ActorLoadEvent>()
           .add_event::<ToastEvent>()
           .add_event::<ConfirmationRequestEvent>()
           .add_event::<InspectionFocusEvent>()
           .add_event::<systems::undo_redo::UndoEvent>()
           .add_event::<systems::undo_redo::RedoEvent>()
           .add_systems(OnEnter(GameState::ActorEditor), (
               vfx_assets::load_vfx_presets,
               vfx_assets::register_kenney_textures,
                ui::layout::setup_actor_editor, 
                navigation::setup_navigation,
                systems::init_gizmos_system,
            ).chain())
           .add_systems(Update, (
                systems::actor_editor_input_system,
                widgets::collapsible_system,
                widgets::scroll_system,
                widgets::scrolling_list_sync_system,
                widgets::scrollbar_sync_system,
                widgets::scrollbar_sync_visibility_system,
                widgets::scrollbar_drag_system,
                widgets::slider_system,
                widgets::panel_resize_system,
                widgets::update_panel_style_system,
                widgets::panel_toggle_system,
                widgets::tooltip_system,
           ).run_if(in_state(GameState::ActorEditor)))
           .add_systems(Update, (
                systems::mode_tab_interaction_system,
                systems::mode_visual_sync_system,
                systems::mode_content_visibility_system,
                systems::inspector_section_sync_system,
                ui::inspector::socket_transform_update_system,
                systems::gizmo_sync_system,
                navigation::camera_reset_handler,
                navigation::grid_system,
                navigation::camera_control_blocking_system, 
                widgets::text_input_system,
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
                systems::project_action_system,
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
                    systems::wireframe_sync_system,
            ).run_if(in_state(GameState::ActorEditor)))
            .add_systems(Update, (
                    systems::socket_picking_system,
                    systems::socket_spawn_system,
                    systems::socket_deletion_system,
                    systems::picking::socket_restoration_system,
                    systems::draw_socket_previews_system,
                    systems::socket_ui_interaction_system,
                    systems::socket_3d_selection_system,
                    systems::socket_button_visuals_system,
                    systems::update_socket_gizmos_system,
                    systems::manual_gizmo_picking_system,
                    systems::gizmo_highlight_system,
                    systems::manual_gizmo_dragging_system,
                    systems::gizmo_position_sync_system,
                    systems::actor_part_picking_priority_system,
                    systems::socket_gizmo_sync_system,
                    systems::xray_material_system,
                    systems::socket_visibility_system,
                ).run_if(in_state(GameState::ActorEditor)))
            .add_systems(Update, (
                    ui::inspector::socket_ui_list_sync_system,
                    ui::inspector::socket_ui_list_label_sync_system,
                    ui::inspector::socket_list_click_system,
                    ui::inspector::socket_list_highlight_system,
                    ui::inspector::socket_reset_rotation_system,
                    ui::inspector::socket_filter_update_system,
                    ui::inspector::socket_filter_ui_system,
                    ui::inspector::vfx::socket_vfx_ui_sync_system,
                    ui::inspector::vfx::socket_vfx_interaction_system,
                    ui::inspector::optimization::mesh_optimization_system,
                    ui::inspector::optimization::mesh_optimization_visuals_system,
                ).run_if(in_state(GameState::ActorEditor)))
            .add_systems(Update, (
                    systems::socket_color_picker_system,
                    systems::socket_material_sync_system,
                    systems::socket_metadata_sync_system,
                    systems::socket_validation_feedback_system,
                    systems::socket_vfx_preview_system,
                    systems::vfx_spawner::socket_vfx_spawner_system,
                    systems::vfx_spawner::socket_vfx_sync_system,
                    systems::undo_redo::undo_redo_shortcuts_system,
                    systems::undo_redo::undo_redo_ui_system,
                    systems::undo_redo::undo_redo_button_visual_system,
                ).run_if(in_state(GameState::ActorEditor)))
            .add_systems(Update, (
                    systems::undo_redo::handle_undo_redo,
                    systems::actor_save_system,
                    systems::load::actor_load_system,
                ).run_if(in_state(GameState::ActorEditor)))

           .add_systems(PostUpdate, (
                systems::mesh_slicing_system,
                systems::draw_slicing_contours_system,
                systems::draw_actor_bounds_debug_system,
            ).after(bevy::transform::TransformSystem::TransformPropagate)
             .run_if(in_state(GameState::ActorEditor)))
           .add_systems(OnExit(GameState::ActorEditor), (ui::layout::cleanup_actor_editor, crate::reset_ambient_light));
    }
}

// Events for decoupling (SOLID)
#[derive(Event)]
pub struct ActorSaveEvent {
    pub name: Option<String>,
    pub force: bool,
}
#[derive(Event)]
pub struct ActorImportEvent(pub std::path::PathBuf, pub bool);

#[derive(Event)]
pub struct ActorLoadEvent(pub std::path::PathBuf);

#[derive(Event)]
pub struct ResetCameraEvent;

#[derive(Resource, Default)]
pub struct LastUsedDirectory(pub Option<std::path::PathBuf>);

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EditorAction {
    BackToMenu,
    SaveProject(String),
    OverwriteProject(String),
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
    pub xray: bool,
    pub show_vfx: bool,
}

#[derive(Debug, Clone)]
pub struct HoveredSocketData {
    pub part_entity: Entity,
    pub part_type: ActorPart,
    pub point: Vec3,
    pub normal: Vec3,
}

#[derive(Resource, Default)]
pub struct SocketSettings {
    pub is_adding: bool,
    pub show_visuals: bool,
    pub hovered_data: Option<HoveredSocketData>,
}

impl Default for ViewportSettings {
    fn default() -> Self {
        Self {
            grid: true,
            slices: true,
            sockets: true,
            gizmos: true,
            xray: false,
            show_vfx: true,
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
    pub suppress_undo: bool,
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
            suppress_undo: false,
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

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ActorSocket {
    pub definition: SocketDefinition,
}

impl Default for ActorSocket {
    fn default() -> Self {
        Self {
            definition: SocketDefinition {
                color: Color::srgb(0.2, 0.8, 0.2), // Default green
                ..default()
            }
        }
    }
}

#[derive(Component, Default)]
pub struct SocketColorPicker;

#[derive(Component, Default)]
pub struct SocketColorHueSlider;

#[derive(Component)]
pub struct SocketColorPreset(pub Color);

impl From<Color> for SocketColorPreset {
    fn from(color: Color) -> Self {
        Self(color)
    }
}

#[derive(Component, Default)]
pub struct SocketColorPickerContainer;

#[derive(Resource)]
pub struct SocketColorPickerState {
    pub color: Color,
    pub hue: f32,
    pub is_open: bool,
}

impl Default for SocketColorPickerState {
    fn default() -> Self {
        Self {
            color: Color::srgb(0.2, 0.8, 0.2),
            hue: 120.0,
            is_open: false,
        }
    }
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
pub struct Actor3DRoot;

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
pub struct UndoButton;

#[derive(Component)]
pub struct RedoButton;

#[derive(Component)]
pub struct OriginalMeshComponent(pub Handle<Mesh>);

#[derive(Resource, Default)]
pub struct PendingImport {
    pub handle: Option<Handle<Scene>>,
    pub mesh_handle: Option<Handle<Mesh>>,
}
#[derive(Resource, Default)]
pub struct CurrentProject {
    pub name: String,
    pub source_path: String,
    pub is_saved: bool,
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
pub struct PendingSockets(pub Vec<SocketDefinition>);

#[derive(Resource, Default)]
pub struct ImportProgress(pub f32);

pub const GIZMO_LAYER: bevy::render::view::RenderLayers = bevy::render::view::RenderLayers::layer(1);
#[derive(Resource, Default)]
pub struct GizmoBusy(pub bool);

#[derive(Component)]
pub struct SaveModalInput;
