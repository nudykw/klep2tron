use bevy::prelude::*;
pub use client_core::{ClientCorePlugin, ClientCoreOptions, Project, Room, TileType, GameState, MapEntity, ExtraMenuButtons, MenuAction, MenuItemType, HudText, Selection, ClientAssets, DirtyTiles, CommandHistory, HelpState, RoomTransition, EditorMode};
use bevy::asset::AssetMetaCheck;
use bevy::render::render_resource::{Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages};
use bevy::render::camera::RenderTarget;
use bevy::render::view::RenderLayers;

pub mod camera;
pub mod ui;
pub mod logic;

pub use crate::camera::*;
pub use crate::ui::*;
pub use crate::logic::*;

#[derive(Component)] pub struct SelectionHighlight;
#[derive(Component)] pub struct UiPreview;
#[derive(Component)] pub struct OverlayCamera;
#[derive(Component)] pub struct RttCamera;
#[derive(Component)] pub struct RttCameraTarget(pub Vec3);
#[derive(Component)] pub struct CameraDebugText;
#[derive(Component)] pub struct PersistentCamera2d;
#[derive(Component)] pub struct TileTypeButton(pub TileType);
#[derive(Component)] pub struct TooltipText(pub String);
#[derive(Component)] pub struct TooltipUi;
#[derive(Component)] pub struct HelpButton;

#[derive(GizmoConfigGroup, Default, Reflect)] pub struct HiddenGizmos;
#[derive(GizmoConfigGroup, Default, Reflect)] pub struct BoxGizmos;

mod build_info { include!(concat!(env!("OUT_DIR"), "/version.rs")); }

pub const TILE_SIZE: f32 = 1.0;
pub const TILE_H:    f32 = 0.5; 

#[derive(Resource)]
pub struct EditorState {
    pub current_type: TileType,
    pub last_selected_cell: client_core::Cell,
}

impl Default for EditorState {
    fn default() -> Self {
        Self { 
            current_type: TileType::Cube,
            last_selected_cell: client_core::Cell::default(),
        }
    }
}

pub fn run_game() {
    client_core::pre_init_gpu_settings();
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window { 
                    title: format!("Klep2Tron_{}_MAP_EDITOR", build_info::VERSION), 
                    ..default() 
                }),
                ..default()
            })
            .set(AssetPlugin {
                meta_check: AssetMetaCheck::Never,
                ..default()
            })
            .set(bevy::render::RenderPlugin {
                render_creation: bevy::render::settings::RenderCreation::Automatic(client_core::get_wgpu_settings()),
                ..default()
            })
        )
        .add_plugins(bevy_obj::ObjPlugin)
        .add_plugins(ClientCorePlugin {
            options: ClientCoreOptions { title: "Klep2Tron Editor".to_string() }
        })
        .insert_resource(ExtraMenuButtons {
            buttons: vec![("LEVEL EDITOR".to_string(), MenuAction::StartEditor)]
        })
        .init_resource::<EditorState>()
        .init_resource::<EditorMode>()
        .init_resource::<Selection>()
        .init_gizmo_group::<HiddenGizmos>()
        .init_gizmo_group::<BoxGizmos>()
        .add_systems(OnEnter(GameState::InGame), setup_editor)
        .add_systems(Update, (
            update_window_title, 
            handle_menu_input, 
            sync_rtt_cameras_system,
            camera_control_system,
            sync_overlay_camera_system,
            selection_system,
            mouse_selection_system,
            editor_ui_system,
            selection_highlight_system,
            room_switching_system,
            auto_save_system,
            undo_redo_system,
            editor_tooltip_system,
            attach_editor_camera,
        ).run_if(in_state(GameState::InGame).and_then(is_editor_active)))
        .add_systems(Update, update_window_title)
        .run();
}

pub fn is_editor_active(mode: Res<EditorMode>) -> bool {
    mode.is_active
}

pub fn update_window_title(mut windows: Query<&mut Window>) {
    for mut window in windows.iter_mut() {
        let expected = format!("Klep2Tron_{}_MAP_EDITOR", build_info::VERSION);
        if window.title != expected { window.title = expected; }
    }
}

pub fn handle_menu_input(
    keyboard: Res<ButtonInput<KeyCode>>, 
    mut state: ResMut<NextState<GameState>>,
    help_state: Res<HelpState>,
) {
    if keyboard.just_pressed(KeyCode::Escape) && !help_state.is_open { 
        state.set(GameState::Menu); 
    }
}

pub fn setup_editor(
    mut commands: Commands, 
    mut project: ResMut<Project>, 
    mut client_assets: ResMut<ClientAssets>,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut config_store: ResMut<GizmoConfigStore>,
    mut history: ResMut<CommandHistory>,
    mut editor_state: ResMut<EditorState>,
    editor_mode: Res<EditorMode>,
) {
    if !editor_mode.is_active { return; }
    history.undo_stack.clear();
    history.redo_stack.clear();
    
    #[cfg(not(target_arch = "wasm32"))]
    if let Ok(content) = std::fs::read_to_string("assets/map.json") {
        if let Ok(loaded) = serde_json::from_str::<Project>(&content) { *project = loaded; }
    }
    if project.rooms.is_empty() { project.rooms.push(Room::default()); }
    
    for room in project.rooms.iter_mut() {
        for row in room.cells.iter_mut() {
            for cell in row.iter_mut() {
                if cell.h < 0 && cell.tt != TileType::Empty { cell.h = 0; }
            }
        }
    }

    let room_idx = project.current_room_idx;
    editor_state.last_selected_cell = project.rooms[room_idx].cells[0][0];

    let font = asset_server.load("fonts/Roboto-Regular.ttf");
    
    client_assets.highlight_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.0, 1.0, 1.0, 0.1),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    let config = config_store.config_mut::<DefaultGizmoConfigGroup>().0;
    config.line_width = 6.0;
    config.depth_bias = -0.01; 

    let hidden_config = config_store.config_mut::<HiddenGizmos>().0;
    hidden_config.line_width = 2.5;
    hidden_config.depth_bias = -0.01;

    let box_config = config_store.config_mut::<BoxGizmos>().0;
    box_config.line_width = 3.0;
    box_config.depth_bias = -0.01; 
    box_config.render_layers = RenderLayers::layer(1);
    
    // HUD
    commands.spawn((NodeBundle {
        style: Style { position_type: PositionType::Absolute, top: Val::Px(10.0), left: Val::Px(10.0), padding: UiRect::all(Val::Px(10.0)), flex_direction: FlexDirection::Column, row_gap: Val::Px(5.0), ..default() },
        background_color: Color::srgba(0.0, 0.0, 0.0, 0.8).into(), ..default()
    }, MapEntity)).with_children(|p| {
        p.spawn((TextBundle::from_section("CAM:", TextStyle { font: font.clone(), font_size: 16.0, color: Color::WHITE }), CameraDebugText));
        p.spawn((TextBundle::from_section("FPS: 0", TextStyle { font: font.clone(), font_size: 16.0, color: Color::srgb(1.0, 1.0, 0.0) }), HudText));
    });

    // RTT Previews
    let types = [
        (TileType::Cube, 0), (TileType::WedgeS, 1), 
        (TileType::WedgeW, 2), (TileType::WedgeN, 3), (TileType::WedgeE, 4)
    ];
    let mut preview_handles = Vec::new();
    let top_mat = materials.add(StandardMaterial { base_color: Color::srgb(0.5, 0.8, 0.2), ..default() });

    for (tt, idx) in types.iter() {
        let size = Extent3d { width: 256, height: 256, ..default() };
        let mut image = Image {
            texture_descriptor: TextureDescriptor {
                label: None, size, dimension: TextureDimension::D2,
                format: TextureFormat::Bgra8UnormSrgb,
                mip_level_count: 1, sample_count: 1,
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_DST,
                view_formats: &[],
            },
            ..default()
        };
        image.resize(size);
        let handle = images.add(image);
        preview_handles.push(handle.clone());

        let layer = RenderLayers::layer(10 + *idx as usize);
        let pos = Vec3::new(100.0 + (*idx as f32 * 10.0), 1000.0, 0.0);

        commands.spawn((
            Camera3dBundle {
                camera: Camera { target: RenderTarget::Image(handle), clear_color: Color::srgba(0.1, 0.1, 0.1, 1.0).into(), ..default() },
                transform: Transform::from_xyz(pos.x + 1.2, pos.y + 0.8, pos.z + 1.2).looking_at(pos, Vec3::Y),
                ..default()
            },
            layer.clone(),
            RttCamera,
            RttCameraTarget(pos),
            MapEntity,
        ));

        commands.spawn((
            DirectionalLightBundle {
                directional_light: DirectionalLight { illuminance: 10000.0, shadows_enabled: false, ..default() },
                transform: Transform::from_xyz(pos.x + 1.0, pos.y + 2.0, pos.z + 1.0).looking_at(pos, Vec3::Y),
                ..default()
            },
            layer.clone(),
            MapEntity,
        ));

        let (mesh, rot) = match tt {
            TileType::Cube => (client_assets.cube_mesh.clone(), 0.0),
            TileType::WedgeN => (client_assets.wedge_mesh.clone(), 0.0),
            TileType::WedgeE => (client_assets.wedge_mesh.clone(), -std::f32::consts::FRAC_PI_2),
            TileType::WedgeS => (client_assets.wedge_mesh.clone(), std::f32::consts::PI),
            TileType::WedgeW => (client_assets.wedge_mesh.clone(), std::f32::consts::FRAC_PI_2),
            _ => (client_assets.cube_mesh.clone(), 0.0),
        };
        commands.spawn((
            PbrBundle {
                mesh: mesh.clone(), material: top_mat.clone(),
                transform: Transform::from_translation(pos)
                    .with_scale(Vec3::new(1.0, 0.5, 1.0))
                    .with_rotation(Quat::from_rotation_y(rot)),
                ..default()
            },
            layer.clone(),
            MapEntity,
        ));
    }

    // Top Panel
    commands.spawn((NodeBundle {
        style: Style { position_type: PositionType::Absolute, top: Val::Px(0.0), left: Val::Percent(30.0), width: Val::Percent(40.0), height: Val::Px(85.0), justify_content: JustifyContent::SpaceEvenly, align_items: AlignItems::Center, ..default() },
        background_color: Color::srgba(0.05, 0.05, 0.05, 0.95).into(), ..default()
    }, MapEntity)).with_children(|p| {
        for (idx, (tt, _)) in types.iter().enumerate() {
            let label = match tt {
                TileType::Cube => "Cube",
                TileType::WedgeN => "Wedge N",
                TileType::WedgeE => "Wedge E",
                TileType::WedgeS => "Wedge S",
                TileType::WedgeW => "Wedge W",
                _ => "Tile",
            };
            p.spawn((ButtonBundle {
                style: Style { width: Val::Px(70.0), height: Val::Px(70.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() },
                background_color: Color::srgb(0.2, 0.2, 0.2).into(),
                border_color: Color::srgb(0.4, 0.4, 0.4).into(),
                ..default()
            }, TileTypeButton(*tt), TooltipText(label.to_string()))).with_children(|p| {
                p.spawn(ImageBundle {
                    image: UiImage::new(preview_handles[idx].clone()),
                    style: Style { width: Val::Px(60.0), height: Val::Px(60.0), ..default() },
                    ..default()
                });
            });
        }
        
        p.spawn((ButtonBundle {
            style: Style { width: Val::Px(70.0), height: Val::Px(70.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() },
            background_color: Color::srgb(0.1, 0.3, 0.3).into(),
            border_color: Color::srgb(0.0, 0.8, 0.8).into(),
            ..default()
        }, HelpButton, TooltipText("Help (F1)".to_string()))).with_children(|p| {
            p.spawn(TextBundle::from_section("?", TextStyle { font: font.clone(), font_size: 40.0, color: Color::WHITE }));
        });
    });

    commands.spawn((NodeBundle {
        style: Style {
            position_type: PositionType::Absolute,
            display: Display::None,
            padding: UiRect::all(Val::Px(5.0)),
            border: UiRect::all(Val::Px(1.0)),
            ..default()
        },
        background_color: Color::srgba(0.0, 0.0, 0.0, 0.9).into(),
        border_color: Color::WHITE.into(),
        z_index: ZIndex::Global(200),
        ..default()
    }, TooltipUi)).with_children(|p| {
        p.spawn(TextBundle::from_section("", TextStyle { font: font.clone(), font_size: 16.0, color: Color::WHITE }));
    });

    commands.spawn((
        Camera3dBundle {
            camera: Camera { 
                order: 2, 
                clear_color: ClearColorConfig::None,
                ..default() 
            },
            camera_3d: Camera3d {
                depth_load_op: bevy::core_pipeline::core_3d::Camera3dDepthLoadOp::Clear(0.0),
                ..default()
            },
            ..default()
        },
        OverlayCamera,
        RenderLayers::layer(1),
        MapEntity,
    ));
}
