use bevy::prelude::*;
pub use client_core::*;
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
include!(concat!(env!("OUT_DIR"), "/version.rs"));

pub const TILE_SIZE: f32 = 1.0;
pub const TILE_H:    f32 = 0.5; 

#[derive(GizmoConfigGroup, Default, Reflect)]
pub struct HiddenGizmos;

#[derive(GizmoConfigGroup, Default, Reflect)]
pub struct BoxGizmos;

#[derive(Resource, Default)]
pub struct Selection {
    pub x: usize,
    pub z: usize,
}

#[derive(Resource)]
pub struct EditorState {
    pub current_type: TileType,
}

impl Default for EditorState {
    fn default() -> Self {
        Self { current_type: TileType::Cube }
    }
}

#[derive(Component)] pub struct OrbitCamera { pub center: Vec3, pub radius: f32, pub angle: f32, pub height: f32 }
#[derive(Component)] pub struct CameraDebugText;
#[derive(Component)] pub struct HudText;
#[derive(Component)] pub struct PersistentCamera2d;
#[derive(Component)] pub struct TileTypeButton(pub TileType);

#[derive(Resource)]
pub struct LoadingTimer(pub Timer);

pub fn run_game() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window { 
                    title: format!("Klep2Tron_{}_MAP_EDITOR", VERSION), 
                    ..default() 
                }),
                ..default()
            })
            .set(AssetPlugin {
                meta_check: AssetMetaCheck::Never,
                ..default()
            })
        )
        .add_plugins(FrameTimeDiagnosticsPlugin)
        .add_plugins(bevy_obj::ObjPlugin)
        .add_plugins(SystemInformationDiagnosticsPlugin)
        .add_plugins(ClientCorePlugin)
        .init_gizmo_group::<HiddenGizmos>()
        .init_gizmo_group::<BoxGizmos>()
        .init_resource::<Selection>()
        .init_resource::<EditorState>()
        .add_systems(OnEnter(GameState::InGame), setup_editor)
        .add_systems(Update, (
            camera_control_system, 
            sync_overlay_camera_system,
            selection_system,
            mouse_selection_system,
            editor_ui_system,
            selection_highlight_system,
            hud_update_system,
            room_switching_system,
            auto_save_system,
        ).run_if(in_state(GameState::InGame)))
        // System for forced window title update (in case of Bevy lags in Wasm)
        .add_systems(Update, update_window_title)
        .run();
}


pub fn update_window_title(mut windows: Query<&mut Window>) {
    for mut window in windows.iter_mut() {
        let expected = format!("Klep2Tron_{}_MAP_EDITOR", VERSION);
        if window.title != expected {
            window.title = expected;
        }
    }
}

pub fn handle_menu_input(keyboard: Res<ButtonInput<KeyCode>>, mut state: ResMut<NextState<GameState>>) {
    if keyboard.just_pressed(KeyCode::Enter) { state.set(GameState::Loading); }
}

pub fn setup_editor(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut project: ResMut<Project>,
    mut client_assets: ResMut<ClientAssets>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut config_store: ResMut<GizmoConfigStore>,
) {
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
    if let Ok(content) = std::fs::read_to_string("map.json") {
        if let Ok(loaded) = serde_json::from_str::<Project>(&content) {
            *project = loaded;
            println!("Loaded project with {} rooms", project.rooms.len());
        }
    }
    if project.rooms.is_empty() { project.rooms.push(Room::default()); }

    // Lighting
    commands.insert_resource(AmbientLight { color: Color::WHITE, brightness: 500.0 });
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight { illuminance: 5000.0, shadows_enabled: true, ..default() },
        transform: Transform::from_xyz(10.0, 20.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight { illuminance: 3000.0, shadows_enabled: false, ..default() },
        transform: Transform::from_xyz(-10.0, 15.0, -5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // HUD
    commands.spawn((NodeBundle {
        style: Style { position_type: PositionType::Absolute, top: Val::Px(10.0), left: Val::Px(10.0), padding: UiRect::all(Val::Px(8.0)), flex_direction: FlexDirection::Column, ..default() },
        background_color: Color::srgba(0.0, 0.0, 0.0, 0.8).into(), ..default()
    }, MapEntity)).with_children(|p| {
        p.spawn((TextBundle::from_section("CAM:", TextStyle { font: font.clone(), font_size: 18.0, color: Color::WHITE }), CameraDebugText));
        p.spawn((TextBundle::from_section("ROOM:", TextStyle { font: font.clone(), font_size: 18.0, color: Color::srgb(1.0, 1.0, 0.0) }), HudText));
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
            p.spawn((ButtonBundle {
                style: Style { width: Val::Px(70.0), height: Val::Px(70.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() },
                background_color: Color::srgb(0.2, 0.2, 0.2).into(),
                border_color: Color::srgb(0.4, 0.4, 0.4).into(),
                ..default()
            }, TileTypeButton(*tt))).with_children(|p| {
                p.spawn(ImageBundle {
                    image: UiImage::new(preview_handles[idx].clone()),
                    style: Style { width: Val::Px(60.0), height: Val::Px(60.0), ..default() },
                    ..default()
                });
            });
        }
    });

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight { shadows_enabled: true, illuminance: 10_000.0, ..default() },
        transform: Transform::from_rotation(Quat::from_rotation_x(-0.8) * Quat::from_rotation_y(0.6)), ..default()
    });
    
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
    editor_state: Res<EditorState>,
) {
    if keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight) { return; }
    if keyboard.just_pressed(KeyCode::ArrowUp)    { selection.z = selection.z.saturating_sub(1); }
    if keyboard.just_pressed(KeyCode::ArrowDown)  { if selection.z < 15 { selection.z += 1; } }
    if keyboard.just_pressed(KeyCode::ArrowLeft)  { selection.x = selection.x.saturating_sub(1); }
    if keyboard.just_pressed(KeyCode::ArrowRight) { if selection.x < 15 { selection.x += 1; } }
    
    let room_idx = project.current_room_idx;
    
    // Feature: Clone from neighbor with 'F'
    if keyboard.just_pressed(KeyCode::KeyF) {
        let x = selection.x;
        let z = selection.z;
        let neighbors = [
            (x as i32, z as i32 + 1),     // Behind
            (x as i32 + 1, z as i32 + 1), // Behind-Right
            (x as i32 - 1, z as i32 + 1), // Behind-Left
            (x as i32 + 1, z as i32),     // Right
        ];
        for (nx, nz) in neighbors {
            if nx >= 0 && nx < 16 && nz >= 0 && nz < 16 {
                let neighbor_cell = project.rooms[room_idx].cells[nx as usize][nz as usize];
                if neighbor_cell.h >= 0 {
                    project.rooms[room_idx].cells[x][z] = neighbor_cell;
                    info!("Cloned block from {} {}", nx, nz);
                    break;
                }
            }
        }
    }

    if keyboard.just_pressed(KeyCode::KeyQ) || keyboard.just_pressed(KeyCode::KeyA) {
        let cell = &mut project.rooms[room_idx].cells[selection.x][selection.z];
        if keyboard.just_pressed(KeyCode::KeyQ) { 
            cell.h += 1; 
            if cell.h >= 0 { cell.tt = editor_state.current_type; }
        }
        if keyboard.just_pressed(KeyCode::KeyA) { 
            cell.h = (cell.h - 1).max(-1); 
            if cell.h < 0 { cell.tt = TileType::Empty; }
            else { cell.tt = editor_state.current_type; }
        }
    }
}

pub fn selection_highlight_system(
    selection: Res<Selection>, 
    project: Res<Project>,
    editor_state: Res<EditorState>,
    time: Res<Time>,
    mut gizmos: Gizmos<BoxGizmos>,
) {
    let room_idx = project.current_room_idx;
    let cell = project.rooms[room_idx].cells[selection.x][selection.z];
    
    // Blink logic: 2.5 Hz (5 changes per second)
    let is_on = (time.elapsed_seconds() * 5.0).sin() > 0.0;
    if !is_on { return; }

    // Position follows the cell height directly
    let top_pos = Vec3::new(selection.x as f32, (cell.h as f32 - 0.5) * TILE_H, selection.z as f32);
    let transform = Transform::from_translation(top_pos).with_scale(Vec3::new(TILE_SIZE * 1.01, TILE_H * 1.01, TILE_SIZE * 1.01));
    
    // Use the type from EditorState for PREVIEW
    let preview_type = editor_state.current_type;
    let color = Color::srgba(1.0, 1.0, 1.0, 0.25);

    match preview_type {
        TileType::Cube => {
            gizmos.cuboid(transform, color);
        },
        _ => {
            let rot = match preview_type {
                TileType::WedgeN => 0.0,
                TileType::WedgeE => -std::f32::consts::FRAC_PI_2,
                TileType::WedgeS => std::f32::consts::PI,
                TileType::WedgeW => std::f32::consts::FRAC_PI_2,
                _ => 0.0,
            };
            draw_wedge_generic(&mut gizmos, transform, rot, color);
        }
    }
}

pub fn mouse_selection_system(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), (With<OrbitCamera>, Without<SelectionHighlight>)>,
    project: Res<Project>,
    mut selection: ResMut<Selection>,
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
                selection.x = x;
                selection.z = z;
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
    mut interaction_query: Query<(&Interaction, &TileTypeButton, &mut BackgroundColor, &mut BorderColor)>,
    mut editor_state: ResMut<EditorState>,
    mut project: ResMut<Project>,
    selection: Res<Selection>,
) {
    for (interaction, tt_btn, mut color, mut border) in interaction_query.iter_mut() {
        let is_selected = editor_state.current_type == tt_btn.0;
        match *interaction {
            Interaction::Pressed => {
                let mut changed = false;
                if !is_selected { editor_state.current_type = tt_btn.0; changed = true; }
                let room_idx = project.current_room_idx;
                let cell = &mut project.rooms[room_idx].cells[selection.x][selection.z];
                if cell.h >= 0 && cell.tt != tt_btn.0 { cell.tt = tt_btn.0; changed = true; }
                if changed { *color = Color::srgb(0.0, 1.0, 1.0).into(); }
            }
            Interaction::Hovered => {
                if is_selected { *color = Color::srgb(0.0, 0.8, 0.8).into(); }
                else { *color = Color::srgb(0.4, 0.4, 0.4).into(); }
            }
            Interaction::None => {
                if is_selected { 
                    *color = Color::srgb(0.0, 0.6, 0.6).into(); 
                    *border = Color::srgb(1.0, 1.0, 1.0).into();
                } else { 
                    *color = Color::srgb(0.2, 0.2, 0.2).into(); 
                    *border = Color::srgb(0.4, 0.4, 0.4).into();
                }
            }
        }
    }
}

pub fn room_switching_system(keyboard: Res<ButtonInput<KeyCode>>, mut project: ResMut<Project>) {
    if keyboard.just_pressed(KeyCode::BracketRight) {
        project.current_room_idx += 1;
        if project.current_room_idx >= project.rooms.len() { project.rooms.push(Room::default()); }
    }
    if keyboard.just_pressed(KeyCode::BracketLeft) {
        if project.current_room_idx > 0 { project.current_room_idx -= 1; }
    }
}

pub fn auto_save_system(project: Res<Project>) {
    if project.is_changed() {
        #[cfg(not(target_arch = "wasm32"))]
        if let Ok(json) = serde_json::to_string_pretty(&*project) {
            let _ = std::fs::write("map.json", json);
        }
    }
}

#[derive(Resource, Default)]
pub struct MaterialCache {
    pub map: std::collections::HashMap<(i32, bool), (Handle<StandardMaterial>, Handle<StandardMaterial>)>,
}

pub fn map_rendering_system(
    mut commands: Commands,
    project: Res<Project>,
    client_assets: Res<ClientAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    tile_query: Query<Entity, With<TileEntity>>,
    mut cache: Local<MaterialCache>,
) {
    if !project.is_changed() { return; }
    for entity in tile_query.iter() { commands.entity(entity).despawn_recursive(); }

    let room = &project.rooms[project.current_room_idx];
    let mut ramp_cache: std::collections::HashMap<(usize, usize), i32> = std::collections::HashMap::with_capacity(64);

    for x in 0..16 {
        for z in 0..16 {
            let cell = room.cells[x][z];
            if cell.h < 0 { continue; }
            
            let is_even = (x + z) % 2 == 0;

            // Determine the "target terrace height" by following the ramp
            // If the ramp eventually leads to a Cube, use that Cube's height.
            let target_h = *ramp_cache.entry((x, z)).or_insert_with(|| {
                let mut tx = x as i32;
                let mut tz = z as i32;
                let mut steps = 0;
                let mut found_cube_h = None;
                while steps < 16 {
                    let c = room.cells[tx as usize][tz as usize];
                    if matches!(c.tt, TileType::Cube) {
                        found_cube_h = Some(c.h);
                        break;
                    }
                    let (dx, dz) = match c.tt {
                        TileType::WedgeN => (0, -1),
                        TileType::WedgeE => (1, 0),
                        TileType::WedgeS => (0, 1),
                        TileType::WedgeW => (-1, 0),
                        _ => break,
                    };
                    tx += dx;
                    tz += dz;
                    if tx < 0 || tx >= 16 || tz < 0 || tz >= 16 { break; }
                    steps += 1;
                }
                found_cube_h.unwrap_or(cell.h)
            });

            // Parity is always local to keep the checkerboard consistent
            let target_is_even = is_even;

            let (mat_top, mat_side) = cache.map.entry((target_h, target_is_even)).or_insert_with(|| {
                let hue = (target_h as f32 * 30.0) % 360.0; 
                let base_light = 0.4 + (target_h as f32 * 0.02).min(0.2);
                let top_light = if target_is_even { base_light } else { base_light * 0.85 };
                
                let top = materials.add(StandardMaterial { 
                    base_color: Color::hsla(hue, 0.7, top_light, 1.0), 
                    perceptual_roughness: 0.6,
                    metallic: 0.1,
                    ..default() 
                });
                let side = materials.add(StandardMaterial { 
                    base_color: Color::hsla(hue, 0.6, top_light * 0.5, 1.0), 
                    perceptual_roughness: 0.9, 
                    ..default() 
                });
                (top, side)
            }).clone();

            let top_pos = Vec3::new(x as f32, (cell.h as f32 - 0.5) * TILE_H, z as f32);
            let (mesh, rot) = match cell.tt {
                TileType::WedgeN => (client_assets.wedge_mesh.clone(), 0.0),
                TileType::WedgeE => (client_assets.wedge_mesh.clone(), -std::f32::consts::FRAC_PI_2),
                TileType::WedgeS => (client_assets.wedge_mesh.clone(), std::f32::consts::PI),
                TileType::WedgeW => (client_assets.wedge_mesh.clone(), std::f32::consts::FRAC_PI_2),
                _ => (client_assets.cube_mesh.clone(), 0.0),
            };

            commands.spawn((PbrBundle {
                mesh, material: mat_top,
                transform: Transform::from_translation(top_pos)
                    .with_scale(Vec3::new(1.0, 0.5, 1.0))
                    .with_rotation(Quat::from_rotation_y(rot)),
                ..default()
            }, TileEntity));

            if cell.h > 0 {
                let p_h = cell.h as f32 * TILE_H;
                // For sides, we use the "local" is_even to keep consistent vertical lines
                let (_, mat_side_local) = cache.map.entry((cell.h, is_even)).or_insert_with(|| {
                    let hue = (cell.h as f32 * 30.0) % 360.0;
                    let base_light = 0.4 + (cell.h as f32 * 0.02).min(0.2);
                    let top_light = if is_even { base_light } else { base_light * 0.85 };
                    let top = materials.add(StandardMaterial { base_color: Color::hsla(hue, 0.7, top_light, 1.0), ..default() });
                    let side = materials.add(StandardMaterial { base_color: Color::hsla(hue, 0.6, top_light * 0.5, 1.0), ..default() });
                    (top, side)
                }).clone();

                commands.spawn((PbrBundle {
                    mesh: client_assets.cube_mesh.clone(), material: mat_side_local,
                    transform: Transform::from_translation(Vec3::new(x as f32, (p_h - TILE_H)/2.0, z as f32))
                        .with_scale(Vec3::new(1.0, p_h - TILE_H, 1.0)),
                    ..default()
                }, TileEntity));
            }
        }
    }
}

pub fn hud_update_system(diagnostics: Res<DiagnosticsStore>, project: Res<Project>, selection: Res<Selection>, mut query: Query<&mut Text, With<HudText>>) {
    let fps = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS).and_then(|d| d.smoothed()).unwrap_or(0.0);
    let room = &project.rooms[project.current_room_idx];
    let cell = room.cells[selection.x][selection.z];
    if let Ok(mut text) = query.get_single_mut() {
        text.sections[0].value = format!("FPS: {:.0} | ROOM: {} | POS: {}, {}, {}", fps, project.current_room_idx, selection.x, selection.z, cell.h);
    }
}


