use bevy::prelude::*;
pub use client_core::{ClientCorePlugin, ClientCoreOptions, Project, Room, TileType, GameState, MapEntity, ExtraMenuButtons, MenuAction, HudText, Selection, ClientAssets, DirtyTiles, CommandHistory, HelpState, RoomTransition, EditorMode};
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin, SystemInformationDiagnosticsPlugin};
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::asset::AssetMetaCheck;
use bevy::render::render_resource::{Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages};
use bevy::render::camera::RenderTarget;
use bevy::render::view::RenderLayers;

#[derive(Component)]
pub struct SelectionHighlight;

#[derive(Component)]
pub struct UiPreview;

#[derive(Component)]
pub struct OverlayCamera;

// Include auto-generated version
mod build_info { include!(concat!(env!("OUT_DIR"), "/version.rs")); }

pub const TILE_SIZE: f32 = 1.0;
pub const TILE_H:    f32 = 0.5; 

#[derive(GizmoConfigGroup, Default, Reflect)]
pub struct HiddenGizmos;

#[derive(GizmoConfigGroup, Default, Reflect)]
pub struct BoxGizmos;


#[derive(Resource)]
pub struct EditorState {
    pub current_type: TileType,
    pub last_selected_cell: client_core::TileCell,
}

impl Default for EditorState {
    fn default() -> Self {
        Self { 
            current_type: TileType::Cube,
            last_selected_cell: client_core::TileCell::default(),
        }
    }
}

#[derive(Component)] pub struct OrbitCamera { pub center: Vec3, pub radius: f32, pub angle: f32, pub height: f32 }
#[derive(Component)] pub struct CameraDebugText;
#[derive(Component)] pub struct PersistentCamera2d;
#[derive(Component)] pub struct TileTypeButton(pub TileType);
#[derive(Component)] pub struct TooltipText(pub String);
#[derive(Component)] pub struct TooltipUi;
#[derive(Component)] pub struct HelpButton;

#[derive(Resource)]
pub struct LoadingTimer(pub Timer);

#[derive(Component)]
pub struct RttCamera;

#[derive(Component)]
pub struct RttCameraTarget(pub Vec3);

pub fn run_game() {
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
        )
        .add_plugins(bevy_obj::ObjPlugin)
        .add_plugins(ClientCorePlugin {
            options: ClientCoreOptions { skip_default_setup: true }
        })
        .insert_resource(ExtraMenuButtons {
            buttons: vec![("LEVEL EDITOR".to_string(), MenuAction::StartEditor)]
        })
        .init_resource::<EditorState>()
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
        ).run_if(in_state(GameState::InGame).and_then(is_editor_active)))
        // System for forced window title update (in case of Bevy lags in Wasm)
        .add_systems(Update, update_window_title)
        .run();
}

pub fn is_editor_active(mode: Res<EditorMode>) -> bool {
    mode.is_active
}


pub fn update_window_title(mut windows: Query<&mut Window>) {
    for mut window in windows.iter_mut() {
        let expected = format!("Klep2Tron_{}_MAP_EDITOR", build_info::VERSION);
        if window.title != expected {
            window.title = expected;
        }
    }
}

pub fn handle_menu_input(
    keyboard: Res<ButtonInput<KeyCode>>, 
    mut state: ResMut<NextState<GameState>>,
    help_state: Res<HelpState>,
) {
    // HOTKEY_SYNC: Esc to return to menu
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
    // Load map.json
    #[cfg(not(target_arch = "wasm32"))]
    if let Ok(content) = std::fs::read_to_string("assets/map.json") {
        if let Ok(loaded) = serde_json::from_str::<Project>(&content) {
            *project = loaded;
        }
    }
    if project.rooms.is_empty() { project.rooms.push(Room::default()); }
    
    // Fix non-empty tiles with h < 0 (they should not be holes)
    for room in project.rooms.iter_mut() {
        for row in room.cells.iter_mut() {
            for cell in row.iter_mut() {
                if cell.h < 0 && cell.tt != TileType::Empty { cell.h = 0; }
            }
        }
    }

    // Initial last_selected_cell from starting position (0,0)
    let room_idx = project.current_room_idx;
    editor_state.last_selected_cell = project.rooms[room_idx].cells[0][0];

    // HUD for the editor
    let font = asset_server.load("fonts/Roboto-Regular.ttf");
    
    client_assets.highlight_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.0, 1.0, 1.0, 0.1),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    // Configure gizmos
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
    
    #[cfg(not(target_arch = "wasm32"))]
    if let Ok(content) = std::fs::read_to_string("assets/map.json") {
        if let Ok(loaded) = serde_json::from_str::<Project>(&content) {
            *project = loaded;
            println!("Loaded project with {} rooms", project.rooms.len());
        }
    }
    if project.rooms.is_empty() { project.rooms.push(Room::default()); }

    // Lighting
    commands.insert_resource(AmbientLight { color: Color::WHITE, brightness: 500.0 });
    commands.spawn((DirectionalLightBundle {
        directional_light: DirectionalLight { illuminance: 5000.0, shadows_enabled: false, ..default() },
        transform: Transform::from_xyz(10.0, 20.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    }, MapEntity));
    commands.spawn((DirectionalLightBundle {
        directional_light: DirectionalLight { illuminance: 3000.0, shadows_enabled: false, ..default() },
        transform: Transform::from_xyz(-10.0, 15.0, -5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    }, MapEntity));

    // HUD
    commands.spawn((NodeBundle {
        style: Style { 
            position_type: PositionType::Absolute, 
            top: Val::Px(10.0), left: Val::Px(10.0), 
            padding: UiRect::all(Val::Px(10.0)), 
            flex_direction: FlexDirection::Column, 
            row_gap: Val::Px(5.0),
            ..default() 
        },
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

        // Preview Camera
        commands.spawn((
            Camera3dBundle {
                camera: Camera { target: RenderTarget::Image(handle), clear_color: Color::srgba(0.1, 0.1, 0.1, 1.0).into(), ..default() },
                transform: Transform::from_xyz(pos.x + 1.2, pos.y + 0.8, pos.z + 1.2).looking_at(pos, Vec3::Y),
                ..default()
            },
            layer.clone(),
            RttCamera,
            RttCameraTarget(pos),
        ));

        // Preview Light
        commands.spawn((
            DirectionalLightBundle {
                directional_light: DirectionalLight { illuminance: 10000.0, shadows_enabled: false, ..default() },
                transform: Transform::from_xyz(pos.x + 1.0, pos.y + 2.0, pos.z + 1.0).looking_at(pos, Vec3::Y),
                ..default()
            },
            layer.clone(),
        ));

        // Preview Mesh
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
                mesh, material: top_mat.clone(),
                transform: Transform::from_translation(pos)
                    .with_scale(Vec3::new(1.0, 0.5, 1.0))
                    .with_rotation(Quat::from_rotation_y(rot)),
                ..default()
            },
            layer,
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
        
        // Help Button
        p.spawn((ButtonBundle {
            style: Style { width: Val::Px(70.0), height: Val::Px(70.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() },
            background_color: Color::srgb(0.1, 0.3, 0.3).into(),
            border_color: Color::srgb(0.0, 0.8, 0.8).into(),
            ..default()
        }, HelpButton, TooltipText("Help (F1)".to_string()))).with_children(|p| {
            p.spawn(TextBundle::from_section("?", TextStyle { font: font.clone(), font_size: 40.0, color: Color::WHITE }));
        });
    });

    // Tooltip Node (Global for editor)
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

    commands.spawn((DirectionalLightBundle {
        directional_light: DirectionalLight { shadows_enabled: true, illuminance: 10_000.0, ..default() },
        transform: Transform::from_rotation(Quat::from_rotation_x(-0.8) * Quat::from_rotation_y(0.6)), ..default()
    }, MapEntity));
    
    commands.insert_resource(AmbientLight { color: Color::WHITE, brightness: 200.0 });

    let center = Vec3::new(7.5, 0.0, 7.5);
    commands.spawn((
        Camera3dBundle {
            camera: Camera { order: 1, ..default() },
            transform: Transform::from_xyz(21.8, 8.0, 17.6).looking_at(center, Vec3::Y),
            ..default()
        },
        OrbitCamera { center, radius: 17.5, angle: 6.9, height: 8.0 },
        FogSettings { color: Color::BLACK, falloff: FogFalloff::Linear { start: 10.0, end: 40.0 }, ..default() },
        MapEntity,
    ));

    // Overlay Camera for selection (always on top)
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

pub fn sync_overlay_camera_system(
    main_query: Query<&Transform, (With<OrbitCamera>, Without<OverlayCamera>)>,
    mut overlay_query: Query<&mut Transform, With<OverlayCamera>>,
) {
    if let Ok(main_trans) = main_query.get_single() {
        if let Ok(mut over_trans) = overlay_query.get_single_mut() {
            *over_trans = *main_trans;
        }
    }
}

pub fn camera_control_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut OrbitCamera)>,
    mut text_query: Query<&mut Text, With<CameraDebugText>>
) {
    let dt = time.delta_seconds();
    let speed = 10.0;
    let rot_speed = 2.0;
    for (mut transform, mut orbit) in query.iter_mut() {
        if keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight) {
            if keyboard.pressed(KeyCode::ArrowUp)   { orbit.radius -= speed * dt; }
            if keyboard.pressed(KeyCode::ArrowDown) { orbit.radius += speed * dt; }
            if keyboard.pressed(KeyCode::ArrowLeft) { orbit.angle += rot_speed * dt; }
            if keyboard.pressed(KeyCode::ArrowRight) { orbit.angle -= rot_speed * dt; }
            if keyboard.pressed(KeyCode::KeyQ)       { orbit.height += speed * dt; }
            if keyboard.pressed(KeyCode::KeyA)       { orbit.height -= speed * dt; }
        }
        orbit.radius = orbit.radius.max(1.0);
        let x = orbit.center.x + orbit.radius * orbit.angle.cos();
        let z = orbit.center.z + orbit.radius * orbit.angle.sin();
        *transform = Transform::from_xyz(x, orbit.height, z).looking_at(orbit.center, Vec3::Y);
        if let Ok(mut text) = text_query.get_single_mut() {
            text.sections[0].value = format!("CAM: X:{:.1} Y:{:.1} Z:{:.1} R:{:.1} A:{:.1}", x, orbit.height, z, orbit.radius, orbit.angle);
        }
    }
}

pub fn selection_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut selection: ResMut<Selection>,
    mut project: ResMut<Project>,
    mut history: ResMut<CommandHistory>,
    mut dirty: ResMut<DirtyTiles>,
    // HOTKEY_SYNC: selection_system uses Arrows, Q/A, F
    mut editor_state: ResMut<EditorState>,
    camera_query: Query<&Transform, With<OrbitCamera>>,
) {
    if keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight) { return; }
    
    // Determine movement vectors relative to camera
    let mut move_dx = 0i32;
    let mut move_dz = 0i32;

    if let Ok(cam_transform) = camera_query.get_single() {
        let forward = cam_transform.forward();
        let forward_h = Vec2::new(forward.x, forward.z).normalize_or_zero();
        
        // Find dominant world direction (quantized to 4 axes)
        let (f_dx, f_dz) = if forward_h.x.abs() > forward_h.y.abs() {
            if forward_h.x > 0.0 { (1, 0) } else { (-1, 0) }
        } else {
            if forward_h.y > 0.0 { (0, 1) } else { (0, -1) }
        };

        // Relative directions:
        // ArrowUp: (f_dx, f_dz)
        // ArrowDown: (-f_dx, -f_dz)
        // ArrowRight: (-f_dz, f_dx)
        // ArrowLeft: (f_dz, -f_dx)

        if keyboard.just_pressed(KeyCode::ArrowUp)    { move_dx = f_dx; move_dz = f_dz; }
        if keyboard.just_pressed(KeyCode::ArrowDown)  { move_dx = -f_dx; move_dz = -f_dz; }
        if keyboard.just_pressed(KeyCode::ArrowRight) { move_dx = -f_dz; move_dz = f_dx; }
        if keyboard.just_pressed(KeyCode::ArrowLeft)  { move_dx = f_dz; move_dz = -f_dx; }
    } else {
        // Fallback to absolute if no camera found
        if keyboard.just_pressed(KeyCode::ArrowUp)    { move_dz = -1; }
        if keyboard.just_pressed(KeyCode::ArrowDown)  { move_dz = 1; }
        if keyboard.just_pressed(KeyCode::ArrowLeft)  { move_dx = -1; }
        if keyboard.just_pressed(KeyCode::ArrowRight) { move_dx = 1; }
    }

    if move_dx != 0 || move_dz != 0 {
        // Save current cell before moving
        let room_idx = project.current_room_idx;
        editor_state.last_selected_cell = project.rooms[room_idx].cells[selection.x][selection.z];
        
        let new_x = (selection.x as i32 + move_dx).clamp(0, 15);
        let new_z = (selection.z as i32 + move_dz).clamp(0, 15);
        selection.x = new_x as usize;
        selection.z = new_z as usize;
    }
    
    let room_idx = project.current_room_idx;
    
    // Feature: Clone from previous selection with 'F'
    if keyboard.just_pressed(KeyCode::KeyF) {
        let room_idx = project.current_room_idx;
        let cell = editor_state.last_selected_cell;
        
        history.push_undo(&project);
        project.rooms[room_idx].cells[selection.x][selection.z] = cell;
        dirty.tiles.push((selection.x, selection.z));
        info!("Cloned previous cell with h={}", cell.h);
    }

    if keyboard.just_pressed(KeyCode::KeyQ) || keyboard.just_pressed(KeyCode::KeyA) {
        history.push_undo(&project);
        let cell = &mut project.rooms[room_idx].cells[selection.x][selection.z];
        if keyboard.just_pressed(KeyCode::KeyQ) { 
            cell.h += 1; 
            // If rising from hole, it becomes floor (Empty) at first
            if cell.h == 0 { cell.tt = TileType::Empty; }
            else if cell.h > 0 { cell.tt = editor_state.current_type; }
            dirty.tiles.push((selection.x, selection.z));
        }
        if keyboard.just_pressed(KeyCode::KeyA) { 
            cell.h = (cell.h - 1).max(-1); 
            if cell.h < 0 { cell.tt = TileType::Empty; }
            dirty.tiles.push((selection.x, selection.z));
        }
    }
}

pub fn selection_highlight_system(
    selection: Res<Selection>, 
    project: Res<Project>,
    editor_state: Res<EditorState>,
    camera_query: Query<&Transform, With<OrbitCamera>>,
    time: Res<Time>,
    mut gizmos: Gizmos<BoxGizmos>,
) {
    let room_idx = project.current_room_idx;
    let cell = project.rooms[room_idx].cells[selection.x][selection.z];
    let cam_pos = camera_query.get_single().map(|t| t.translation).unwrap_or(Vec3::ZERO);
    
    // 0. Synchronized Blink
    let is_on = (time.elapsed_seconds() * 5.0).sin() > 0.0;
    if !is_on { return; }

    // 1. Vertical Column Highlight (Dashed) - only if above floor
    if cell.h > 0 {
        let h_s = TILE_SIZE * 0.5;
        let base_y = 0.0;
        let top_y = (cell.h as f32 - 1.0) * TILE_H;
        
        // Draw vertical lines only
        for dx in [-h_s, h_s] {
            for dz in [-h_s, h_s] {
                let start = Vec3::new(selection.x as f32 + dx, base_y, selection.z as f32 + dz);
                let end = Vec3::new(selection.x as f32 + dx, top_y, selection.z as f32 + dz);
                draw_dashed_line(&mut gizmos, start, end, Color::srgba(1.0, 1.0, 1.0, 0.15));
            }
        }
    }

    // 2. Selection Box Highlight (Refined Edges)
    let top_pos = Vec3::new(selection.x as f32, (cell.h as f32 - 0.5) * TILE_H, selection.z as f32);
    let transform = Transform::from_translation(top_pos).with_scale(Vec3::new(TILE_SIZE * 1.01, TILE_H * 1.01, TILE_SIZE * 1.01));
    
    let preview_type = editor_state.current_type;
    let color = Color::srgba(1.0, 1.0, 1.0, 0.4);

    match preview_type {
        TileType::Cube => {
            draw_refined_cuboid(&mut gizmos, transform, cam_pos, color);
        },
        _ => {
            let rot = match preview_type {
                TileType::WedgeN => 0.0,
                TileType::WedgeE => -std::f32::consts::FRAC_PI_2,
                TileType::WedgeS => std::f32::consts::PI,
                TileType::WedgeW => std::f32::consts::FRAC_PI_2,
                _ => 0.0,
            };
            draw_refined_wedge(&mut gizmos, transform, rot, cam_pos, color);
        }
    }
}

fn draw_refined_cuboid(gizmos: &mut Gizmos<BoxGizmos>, transform: Transform, cam_pos: Vec3, color: Color) {
    let center = transform.translation;
    let half_scale = transform.scale * 0.5;
    
    // 8 vertices: (dx, dy, dz)
    let mut v = [Vec3::ZERO; 8];
    let mut i = 0;
    for dx in [-1.0, 1.0] {
        for dy in [-1.0, 1.0] {
            for dz in [-1.0, 1.0] {
                v[i] = center + Vec3::new(dx * half_scale.x, dy * half_scale.y, dz * half_scale.z);
                i += 1;
            }
        }
    }

    // 12 edges: (idx1, idx2, face_idx1, face_idx2)
    // Face Normals: 0:+X, 1:-X, 2:+Y, 3:-Y, 4:+Z, 5:-Z
    let edges = [
        (0, 1, 1, 3), (2, 3, 1, 2), (4, 5, 0, 3), (6, 7, 0, 2), // Z-aligned (X and Y are fixed)
        (0, 2, 1, 5), (1, 3, 1, 4), (4, 6, 0, 5), (5, 7, 0, 4), // Y-aligned (X and Z are fixed)
        (0, 4, 3, 5), (1, 5, 3, 4), (2, 6, 2, 5), (3, 7, 2, 4), // X-aligned (Y and Z are fixed)
    ];

    let face_normals = [
        Vec3::X, Vec3::NEG_X, Vec3::Y, Vec3::NEG_Y, Vec3::Z, Vec3::NEG_Z
    ];

    for (i1, i2, n1, n2) in edges {
        let is_back1 = face_normals[n1].dot(cam_pos - center) < 0.0;
        let is_back2 = face_normals[n2].dot(cam_pos - center) < 0.0;
        
        if is_back1 && is_back2 {
            draw_dashed_line(gizmos, v[i1], v[i2], color.with_alpha(0.15));
        } else {
            gizmos.line(v[i1], v[i2], color);
        }
    }
}

fn draw_refined_wedge(gizmos: &mut Gizmos<BoxGizmos>, transform: Transform, rot: f32, cam_pos: Vec3, color: Color) {
    // For simplicity, we just draw the refined wedge with dashed lines for back edges too
    // But since it's a custom shape, we'll manually define its 6 vertices and 9 edges
    let center = transform.translation;
    let half_scale = transform.scale * 0.5;
    let rotation = Quat::from_rotation_y(rot);

    // Local vertices for a wedge (N orientation)
    // Bottom 4, Top 2 (on the back side)
    let local_v = [
        Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, -1.0, -1.0), // Bottom Back
        Vec3::new(-1.0, -1.0, 1.0), Vec3::new(1.0, -1.0, 1.0),   // Bottom Front
        Vec3::new(-1.0, 1.0, -1.0), Vec3::new(1.0, 1.0, -1.0),  // Top Back
    ];

    let mut v = [Vec3::ZERO; 6];
    for i in 0..6 {
        v[i] = center + rotation * (local_v[i] * half_scale);
    }

    // Edges
    let edges = [
        (0, 1), (1, 3), (3, 2), (2, 0), // Bottom
        (4, 5), (0, 4), (1, 5),         // Back & Top
        (2, 4), (3, 5),                 // Slopes
    ];

    // For wedge, we'll just check if the edge center is further from camera than box center
    for (i1, i2) in edges {
        let edge_center = (v[i1] + v[i2]) * 0.5;
        let is_back = (edge_center - center).dot(cam_pos - center) < -0.2; // Heuristic
        
        if is_back {
            draw_dashed_line(gizmos, v[i1], v[i2], color.with_alpha(0.15));
        } else {
            gizmos.line(v[i1], v[i2], color);
        }
    }
}

pub fn mouse_selection_system(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), (With<OrbitCamera>, Without<SelectionHighlight>)>,
    project: Res<Project>,
    mut selection: ResMut<Selection>,
    mut editor_state: ResMut<EditorState>,
    interaction_query: Query<&Interaction>,
) {
    if !buttons.just_pressed(MouseButton::Left) { return; }
    
    // Ignore click if over UI
    for interaction in interaction_query.iter() {
        if *interaction != Interaction::None { return; }
    }
    
    let window = windows.single();
    let (camera, camera_transform) = camera_query.single();

    if let Some(cursor_pos) = window.cursor_position() {
        if let Some(ray) = camera.viewport_to_world(camera_transform, cursor_pos) {
            let mut best_hit: Option<(usize, usize, f32)> = None;
            let room = &project.rooms[project.current_room_idx];

            for x in 0..16 {
                for z in 0..16 {
                    let cell = room.cells[x][z];

                    // Check blocks
                    if cell.h >= 0 {
                        let (y_min, y_max) = if cell.h == 0 { (-0.5, 0.0) } else { (0.0, cell.h as f32 * TILE_H) };
                        let min = Vec3::new(x as f32 - 0.5, y_min, z as f32 - 0.5);
                        let max = Vec3::new(x as f32 + 0.5, y_max, z as f32 + 0.5);
                        if let Some(hit_t) = ray_aabb(ray, min, max) {
                            if best_hit.is_none() || hit_t < best_hit.unwrap().2 { best_hit = Some((x, z, hit_t)); }
                        }
                    }

                    // Always check floor plane for holes
                    let f_min = Vec3::new(x as f32 - 0.5, -0.5, z as f32 - 0.5);
                    let f_max = Vec3::new(x as f32 + 0.5, 0.0, z as f32 + 0.5);
                    if let Some(hit_t) = ray_aabb(ray, f_min, f_max) {
                        if best_hit.is_none() || hit_t < best_hit.unwrap().2 { best_hit = Some((x, z, hit_t)); }
                    }
                }
            }

            if let Some((x, z, _)) = best_hit {
                if selection.x != x || selection.z != z {
                    // Save current cell before jumping
                    let room_idx = project.current_room_idx;
                    editor_state.last_selected_cell = project.rooms[room_idx].cells[selection.x][selection.z];
                    
                    selection.x = x;
                    selection.z = z;
                }
            }
        }
    }
}

fn ray_aabb(ray: Ray3d, min: Vec3, max: Vec3) -> Option<f32> {
    let mut t_min = f32::NEG_INFINITY;
    let mut t_max = f32::INFINITY;
    for i in 0..3 {
        let inv_dir = 1.0 / ray.direction[i];
        let mut t1 = (min[i] - ray.origin[i]) * inv_dir;
        let mut t2 = (max[i] - ray.origin[i]) * inv_dir;
        if inv_dir < 0.0 { std::mem::swap(&mut t1, &mut t2); }
        t_min = t_min.max(t1);
        t_max = t_max.min(t2);
    }
    if t_max >= t_min && t_max > 0.0 {
        Some(if t_min > 0.0 { t_min } else { t_max })
    } else {
        None
    }
}

fn get_wedge_v(transform: Transform, rot: f32) -> [Vec3; 6] {
    let size = transform.scale;
    let pos = transform.translation;
    let hx = size.x / 2.0; let hy = size.y / 2.0; let hz = size.z / 2.0;
    let v_base = [
        Vec3::new(-hx, -hy,  hz), // 0 S-L-B
        Vec3::new( hx, -hy,  hz), // 1 S-R-B
        Vec3::new( hx,  hy, -hz), // 2 N-R-T
        Vec3::new(-hx,  hy, -hz), // 3 N-L-T
        Vec3::new(-hx, -hy, -hz), // 4 N-L-B
        Vec3::new( hx, -hy, -hz), // 5 N-R-B
    ];
    let rotation = Quat::from_rotation_y(rot);
    let mut v = [Vec3::ZERO; 6];
    for i in 0..6 { v[i] = pos + rotation * v_base[i]; }
    v
}

fn draw_wedge_generic<T: GizmoConfigGroup>(gizmos: &mut Gizmos<T>, transform: Transform, rot: f32, color: Color) {
    let v = get_wedge_v(transform, rot);
    let edges = [(0,1), (1,5), (5,4), (4,0), (4,3), (5,2), (3,2), (0,3), (1,2)];
    for (s, e) in edges { gizmos.line(v[s], v[e], color); }
}

fn draw_dashed_wedge_generic<T: GizmoConfigGroup>(gizmos: &mut Gizmos<T>, transform: Transform, rot: f32, color: Color) {
    let v = get_wedge_v(transform, rot);
    let edges = [(0,1), (1,5), (5,4), (4,0), (4,3), (5,2), (3,2), (0,3), (1,2)];
    let segments = 20;
    for (s_idx, e_idx) in edges {
        let start = v[s_idx]; let end = v[e_idx];
        for i in 0..segments {
            if i % 2 == 0 {
                let s = start + (end - start) * (i as f32 / segments as f32);
                let e = start + (end - start) * ((i + 1) as f32 / segments as f32);
                gizmos.line(s, e, color);
            }
        }
    }
}

fn draw_dashed_cuboid_generic<T: GizmoConfigGroup>(gizmos: &mut Gizmos<T>, transform: Transform, color: Color) {
    let size = transform.scale;
    let pos = transform.translation;
    let half = size / 2.0;
    
    let v = [
        pos + Vec3::new(-half.x, -half.y, -half.z),
        pos + Vec3::new( half.x, -half.y, -half.z),
        pos + Vec3::new( half.x,  half.y, -half.z),
        pos + Vec3::new(-half.x,  half.y, -half.z),
        pos + Vec3::new(-half.x, -half.y,  half.z),
        pos + Vec3::new( half.x, -half.y,  half.z),
        pos + Vec3::new( half.x,  half.y,  half.z),
        pos + Vec3::new(-half.x,  half.y,  half.z),
    ];
    
    let edges = [
        (0,1), (1,2), (2,3), (3,0),
        (4,5), (5,6), (6,7), (7,4),
        (0,4), (1,5), (2,6), (3,7),
    ];
    
    let segments = 20;
    for (s_idx, e_idx) in edges {
        let start = v[s_idx];
        let end = v[e_idx];
        for i in 0..segments {
            if i % 2 == 0 {
                let s = start + (end - start) * (i as f32 / segments as f32);
                let e = start + (end - start) * ((i + 1) as f32 / segments as f32);
                gizmos.line(s, e, color);
            }
        }
    }
}

pub fn editor_ui_system(
    mut interaction_query: Query<(&Interaction, Option<&TileTypeButton>, Option<&HelpButton>, &mut BackgroundColor, &mut BorderColor)>,
    mut editor_state: ResMut<EditorState>,
    mut project: ResMut<Project>,
    mut history: ResMut<CommandHistory>,
    mut help_state: ResMut<HelpState>,
    selection: Res<Selection>,
    mut dirty: ResMut<DirtyTiles>,
) {
    for (interaction, tt_btn, help_btn, mut color, mut border) in interaction_query.iter_mut() {
        let is_selected = tt_btn.map_or(false, |b| editor_state.current_type == b.0);
        match *interaction {
            Interaction::Pressed => {
                if let Some(tt_btn) = tt_btn {
                    let mut changed = false;
                    if !is_selected { editor_state.current_type = tt_btn.0; changed = true; }
                    let room_idx = project.current_room_idx;
                    let cell = project.rooms[room_idx].cells[selection.x][selection.z];
                    if cell.h >= 0 && cell.tt != tt_btn.0 { 
                        history.push_undo(&project);
                        let cell = &mut project.rooms[room_idx].cells[selection.x][selection.z];
                        cell.tt = tt_btn.0; 
                        changed = true; 
                        dirty.tiles.push((selection.x, selection.z));
                    }
                    if changed { *color = Color::srgb(0.0, 1.0, 1.0).into(); }
                }
                if help_btn.is_some() {
                    help_state.is_open = !help_state.is_open;
                }
            }
            Interaction::Hovered => {
                if is_selected { *color = Color::srgb(0.0, 0.8, 0.8).into(); }
                else if help_btn.is_some() { *color = Color::srgb(0.0, 0.5, 0.5).into(); }
                else { *color = Color::srgb(0.4, 0.4, 0.4).into(); }
            }
            Interaction::None => {
                if is_selected { 
                    *color = Color::srgb(0.0, 0.6, 0.6).into(); 
                    *border = Color::srgb(1.0, 1.0, 1.0).into();
                } else if help_btn.is_some() {
                    *color = Color::srgb(0.1, 0.3, 0.3).into();
                    *border = Color::srgb(0.0, 0.8, 0.8).into();
                } else { 
                    *color = Color::srgb(0.2, 0.2, 0.2).into(); 
                    *border = Color::srgb(0.4, 0.4, 0.4).into();
                }
            }
        }
    }
}

pub fn editor_tooltip_system(
    windows: Query<&Window>,
    mut tooltip_query: Query<(&mut Style, &mut Visibility, &Children), With<TooltipUi>>,
    mut text_query: Query<&mut Text>,
    interaction_query: Query<(&Interaction, &TooltipText)>,
) {
    let window = windows.single();
    let mut tooltip_active = false;

    if let Ok((mut style, mut visibility, children)) = tooltip_query.get_single_mut() {
        for (interaction, tooltip) in interaction_query.iter() {
            if *interaction == Interaction::Hovered {
                tooltip_active = true;
                if let Some(mut text) = text_query.get_mut(children[0]).ok() {
                    text.sections[0].value = tooltip.0.clone();
                }
                
                if let Some(cursor_pos) = window.cursor_position() {
                    style.left = Val::Px(cursor_pos.x + 15.0);
                    style.top = Val::Px(cursor_pos.y + 15.0);
                    style.display = Display::Flex;
                    *visibility = Visibility::Visible;
                }
                break;
            }
        }

        if !tooltip_active {
            style.display = Display::None;
            *visibility = Visibility::Hidden;
        }
    }
}

pub fn room_switching_system(
    keyboard: Res<ButtonInput<KeyCode>>, 
    mut project: ResMut<Project>, 
    mut history: ResMut<CommandHistory>,
    mut transition: ResMut<RoomTransition>,
) {
    if keyboard.just_pressed(KeyCode::BracketRight) {
        history.push_undo(&project);
        let next_idx = project.current_room_idx + 1;
        if next_idx >= project.rooms.len() { project.rooms.push(Room::default()); }
        transition.start(next_idx);
    }
    if keyboard.just_pressed(KeyCode::BracketLeft) {
        if project.current_room_idx > 0 { 
            history.push_undo(&project);
            transition.start(project.current_room_idx - 1);
        }
    }
}

pub fn auto_save_system(project: Res<Project>) {
    if project.is_changed() {
        #[cfg(not(target_arch = "wasm32"))]
        if let Ok(json) = serde_json::to_string_pretty(&*project) {
            let _ = std::fs::write("assets/map.json", json);
        }
    }
}

pub fn undo_redo_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut project: ResMut<Project>,
    mut history: ResMut<CommandHistory>,
    mut dirty: ResMut<DirtyTiles>,
) {
    // HOTKEY_SYNC: undo_redo_system uses Ctrl+Z, Ctrl+U
    let ctrl = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);
    
    if ctrl && keyboard.just_pressed(KeyCode::KeyZ) {
        if let Some(prev) = history.undo(&project) {
            *project = prev;
            dirty.full_rebuild = true;
            info!("Undo successful");
        }
    }

    if ctrl && keyboard.just_pressed(KeyCode::KeyU) {
        if let Some(next) = history.redo(&project) {
            *project = next;
            dirty.full_rebuild = true;
            info!("Redo successful");
        }
    }
}




fn sync_rtt_cameras_system(
    main_cam_query: Query<&OrbitCamera>,
    mut rtt_query: Query<(&mut Transform, &RttCameraTarget), With<RttCamera>>,
) {
    let Ok(orbit) = main_cam_query.get_single() else { return };
    
    for (mut transform, target) in rtt_query.iter_mut() {
        let radius = 2.0; // Fixed radius for previews
        let x = target.0.x + radius * orbit.angle.cos();
        let z = target.0.z + radius * orbit.angle.sin();
        let y = target.0.y + (orbit.height / orbit.radius) * radius; // Scale height proportionally
        
        *transform = Transform::from_xyz(x, y, z).looking_at(target.0, Vec3::Y);
    }
}

fn draw_dashed_line(gizmos: &mut Gizmos<BoxGizmos>, start: Vec3, end: Vec3, color: Color) {
    let dir = end - start;
    let length = dir.length();
    if length < 0.01 { return; }
    let norm = dir / length;
    let dash_len = 0.05;
    let gap_len = 0.05;
    let mut current = 0.0;
    while current < length {
        let dash_end = (current + dash_len).min(length);
        gizmos.line(start + norm * current, start + norm * dash_end, color);
        current += dash_len + gap_len;
    }
}
