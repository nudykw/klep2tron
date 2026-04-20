use bevy::prelude::*;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use crate::{GameState, GraphicsSettings, QualityLevel, Project, Room, DirtyTiles};

pub struct BenchmarkPlugin;

impl Plugin for BenchmarkPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BenchmarkData>()
            .add_systems(OnEnter(GameState::Benchmark), setup_benchmark)
            .add_systems(Update, (
                benchmark_logic_system,
                benchmark_camera_system,
                benchmark_ui_system,
            ).run_if(in_state(GameState::Benchmark)))
            .add_systems(OnExit(GameState::Benchmark), cleanup_benchmark);
    }
}

#[derive(Resource, Default)]
pub struct BenchmarkData {
    pub levels: Vec<QualityLevel>,
    pub current_level_idx: usize,
    pub timer: Timer,
    pub fps_samples: Vec<f32>,
    pub results: Vec<(QualityLevel, f32)>,
    pub original_settings: Option<GraphicsSettings>,
    pub finished: bool,
    pub aborted: bool,
}

#[derive(Component)]
pub struct BenchmarkEntity;

#[derive(Component)]
pub struct BenchmarkCamera;

#[derive(Component)]
pub struct BenchmarkUi;

fn setup_benchmark(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut project: ResMut<Project>,
    mut settings: ResMut<GraphicsSettings>,
    mut benchmark_data: ResMut<BenchmarkData>,
    mut dirty: ResMut<DirtyTiles>,
) {
    // Snap and modify settings for test
    benchmark_data.original_settings = Some(settings.clone());
    let mut test_settings = settings.clone();
    test_settings.vsync = false; // Disable VSync to get uncapped FPS
    test_settings.quality_level = QualityLevel::Low;
    *settings = test_settings;

    // Load map data if missing (identical to setup_game_world logic)
    #[cfg(not(target_arch = "wasm32"))]
    if project.rooms.is_empty() {
        if let Ok(content) = std::fs::read_to_string("assets/map.json") {
            if let Ok(loaded) = serde_json::from_str::<Project>(&content) {
                *project = loaded;
            }
        }
    }
    if project.rooms.is_empty() { project.rooms.push(Room::default()); }
    dirty.full_rebuild = true;

    *benchmark_data = BenchmarkData {
        levels: vec![QualityLevel::Low, QualityLevel::Medium, QualityLevel::High, QualityLevel::Ultra],
        current_level_idx: 0,
        timer: Timer::from_seconds(2.5, TimerMode::Once),
        fps_samples: Vec::new(),
        results: Vec::new(),
        original_settings: benchmark_data.original_settings.clone(),
        finished: false,
        aborted: false,
    };

    // Spawn Camera
    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                order: 5, // Ensure it renders over the menu skybox
                ..default()
            },
            transform: Transform::from_xyz(15.0, 10.0, 15.0).looking_at(Vec3::new(7.5, 0.0, 7.5), Vec3::Y),
            ..default()
        },
        BenchmarkEntity,
        BenchmarkCamera,
        FogSettings {
            color: Color::srgb(0.05, 0.05, 0.1),
            falloff: FogFalloff::Linear { start: 10.0, end: 30.0 },
            ..default()
        }
    ));

    // Spawn Lighting
    commands.spawn((
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                shadows_enabled: true,
                illuminance: 12000.0,
                ..default()
            },
            transform: Transform::from_xyz(15.0, 30.0, 30.0).looking_at(Vec3::new(7.5, 0.0, 7.5), Vec3::Y),
            ..default()
        },
        BenchmarkEntity,
    ));

    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 200.0,
    });

    // We no longer spawn blocks manually here.
    // map_rendering_system will handle this because it now runs in Benchmark state.
    
    // Trigger map loading if not already loaded (normally done in setup_game_world)
    if project.rooms.is_empty() {
        info!("Benchmark: Map empty, map_rendering_system will wait for data");
    }

    // UI Setup
    let font = asset_server.load("fonts/Roboto-Regular.ttf");
    commands.spawn((
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            background_color: Color::srgba(0.0, 0.0, 0.0, 0.5).into(),
            ..default()
        },
        BenchmarkEntity,
        BenchmarkUi,
    )).with_children(|p| {
        p.spawn(TextBundle::from_section(
            "BENCHMARK IN PROGRESS",
            TextStyle { font: font.clone(), font_size: 40.0, color: Color::WHITE },
        ).with_style(Style { margin: UiRect::bottom(Val::Px(60.0)), ..default() }));
        
        p.spawn(TextBundle::from_section(
            "Testing Quality: Low",
            TextStyle { font: font.clone(), font_size: 30.0, color: Color::srgb(1.0, 0.8, 0.0) },
        ).with_style(Style { margin: UiRect::bottom(Val::Px(20.0)), ..default() }));
        p.spawn(TextBundle::from_section(
            "Current FPS: --",
            TextStyle { font: font.clone(), font_size: 24.0, color: Color::WHITE },
        ));
    });
}

fn benchmark_camera_system(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<BenchmarkCamera>>,
    mut angle: Local<f32>,
) {
    *angle += time.delta_seconds() * 0.4;
    let radius = 18.0;
    let center = Vec3::new(7.5, 0.0, 7.5);
    let height = 10.0 + (angle.sin() * 0.2).abs() * 5.0;

    for mut transform in query.iter_mut() {
        let x = center.x + radius * angle.cos();
        let z = center.z + radius * angle.sin();
        *transform = Transform::from_xyz(x, height, z).looking_at(center, Vec3::Y);
    }
}

fn benchmark_logic_system(
    time: Res<Time>,
    diagnostics: Res<DiagnosticsStore>,
    mut benchmark_data: ResMut<BenchmarkData>,
    mut settings: ResMut<GraphicsSettings>,
    mut next_state: ResMut<NextState<GameState>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if benchmark_data.finished || benchmark_data.aborted {
        if keyboard.just_pressed(KeyCode::Enter) || keyboard.just_pressed(KeyCode::Escape) {
            // Recommendation logic
            if !benchmark_data.aborted && !benchmark_data.results.is_empty() {
                // Find highest quality with >= 25 FPS, or closest to it
                let mut best_fit = benchmark_data.results[0].0;
                for (level, fps) in benchmark_data.results.iter() {
                    if *fps >= 25.0 {
                        best_fit = *level;
                    } else {
                        break;
                    }
                }
                settings.quality_level = best_fit;
                // Re-apply sub-settings based on quality level (copying logic from menu.rs)
                match best_fit {
                    QualityLevel::Low => { settings.shadow_quality = QualityLevel::Low; settings.fog_quality = QualityLevel::Low; },
                    QualityLevel::Medium => { settings.shadow_quality = QualityLevel::Medium; settings.fog_quality = QualityLevel::Low; },
                    QualityLevel::High => { settings.shadow_quality = QualityLevel::High; settings.fog_quality = QualityLevel::Medium; },
                    QualityLevel::Ultra => { settings.shadow_quality = QualityLevel::Ultra; settings.fog_quality = QualityLevel::High; },
                    _ => {},
                }
            }
            next_state.set(GameState::Menu);
        }
        return;
    }

    // Apply current quality level
    let current_level = benchmark_data.levels[benchmark_data.current_level_idx];
    if settings.quality_level != current_level {
        settings.quality_level = current_level;
        match current_level {
            QualityLevel::Low => { settings.shadow_quality = QualityLevel::Low; settings.fog_quality = QualityLevel::Low; },
            QualityLevel::Medium => { settings.shadow_quality = QualityLevel::Medium; settings.fog_quality = QualityLevel::Low; },
            QualityLevel::High => { settings.shadow_quality = QualityLevel::High; settings.fog_quality = QualityLevel::Medium; },
            QualityLevel::Ultra => { settings.shadow_quality = QualityLevel::Ultra; settings.fog_quality = QualityLevel::High; },
            _ => {},
        }
    }

    benchmark_data.timer.tick(time.delta());

    // Sample FPS after 0.5s warmup
    if benchmark_data.timer.elapsed_secs() > 0.5 {
        if let Some(fps_diag) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(fps_val) = fps_diag.smoothed() {
                benchmark_data.fps_samples.push(fps_val as f32);
                
                // Abort condition
                if fps_val < 15.0 && benchmark_data.timer.elapsed_secs() > 1.0 {
                    benchmark_data.aborted = true;
                }
            }
        }
    }

    if benchmark_data.timer.finished() || benchmark_data.aborted {
        // Calculate average
        let avg_fps = if benchmark_data.fps_samples.is_empty() {
            0.0
        } else {
            benchmark_data.fps_samples.iter().sum::<f32>() / benchmark_data.fps_samples.len() as f32
        };

        benchmark_data.results.push((current_level, avg_fps));
        benchmark_data.fps_samples.clear();

        if benchmark_data.aborted || benchmark_data.current_level_idx >= benchmark_data.levels.len() - 1 {
            benchmark_data.finished = true;
        } else {
            benchmark_data.current_level_idx += 1;
            benchmark_data.timer.reset();
        }
    }
}

fn benchmark_ui_system(
    benchmark_data: Res<BenchmarkData>,
    mut ui_query: Query<&Children, With<BenchmarkUi>>,
    mut text_query: Query<&mut Text>,
    diagnostics: Res<DiagnosticsStore>,
) {
    let Ok(children) = ui_query.get_single_mut() else { return; };
    
    if benchmark_data.finished || benchmark_data.aborted {
        // Results Screen
        if let Ok(mut text) = text_query.get_mut(children[0]) {
            text.sections[0].value = if benchmark_data.aborted { "BENCHMARK ABORTED" } else { "BENCHMARK FINISHED" }.to_string();
        }
        
        let mut results_text = "Results:\n".to_string();
        let mut best_fit = benchmark_data.levels[0];
        for (level, fps) in &benchmark_data.results {
            results_text += &format!("{:?}: {:.1} FPS\n", level, fps);
            if *fps >= 25.0 { best_fit = *level; }
        }
        
        if let Ok(mut text) = text_query.get_mut(children[1]) {
            text.sections[0].value = results_text;
            text.sections[0].style.color = Color::WHITE;
        }
        
        if let Ok(mut text) = text_query.get_mut(children[2]) {
            text.sections[0].value = format!("Recommended: {:?}\n[ENTER] Apply & Back  [ESC] Cancel", best_fit);
        }
    } else {
        // Progress Screen
        if let Ok(mut text) = text_query.get_mut(children[1]) {
            text.sections[0].value = format!("Testing Quality: {:?}", benchmark_data.levels[benchmark_data.current_level_idx]);
        }
        
        if let Ok(mut text) = text_query.get_mut(children[2]) {
            if let Some(fps_diag) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
                if let Some(fps_val) = fps_diag.smoothed() {
                    text.sections[0].value = format!("Current FPS: {:.1}", fps_val);
                }
            }
        }
    }
}

fn cleanup_benchmark(
    mut commands: Commands,
    query: Query<Entity, Or<(With<BenchmarkEntity>, With<BenchmarkUi>)>>,
    benchmark_data: Res<BenchmarkData>,
    mut settings: ResMut<GraphicsSettings>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    
    if !benchmark_data.finished && !benchmark_data.aborted {
        if let Some(original) = &benchmark_data.original_settings {
            *settings = original.clone();
        }
    }
}

pub fn get_rainbow_color(h: i32, is_even: bool) -> Color {
    let rainbow = [
        Color::srgb(1.0, 0.2, 0.2), 
        Color::srgb(1.0, 0.5, 0.0), 
        Color::srgb(1.0, 0.9, 0.0), 
        Color::srgb(0.2, 0.8, 0.2), 
        Color::srgb(0.0, 0.6, 1.0), 
        Color::srgb(0.3, 0.3, 0.9), 
        Color::srgb(0.6, 0.2, 0.8), 
    ];
    let idx = (h.max(0) as usize) % rainbow.len();
    let mut color = LinearRgba::from(rainbow[idx]);
    if !is_even {
        color.red *= 0.9;
        color.green *= 0.9;
        color.blue *= 0.9;
    }
    Color::from(color)
}
