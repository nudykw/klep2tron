use bevy::prelude::*;
use crate::{GameState, EditorMode, GraphicsSettings, save_settings, MyWindowMode, UpscalingMode, QualityLevel};
use super::super::types::*;

pub fn handle_menu_action(
    action: MenuAction, 
    next_game_state: &mut ResMut<NextState<GameState>>, 
    editor_mode: &mut ResMut<EditorMode>,
    next_menu_state: &mut ResMut<NextState<MenuSubState>>,
    settings: &mut ResMut<GraphicsSettings>,
    pending: &mut ResMut<PendingGraphicsSettings>,
    confirmation: &mut ResMut<ConfirmationData>,
    _game_state: &Res<State<GameState>>,
    exit_confirm: &mut ResMut<ExitConfirmationActive>,
    gpu_list: &mut ResMut<crate::settings::GpuList>,
    instance_adapter: &Option<Res<bevy::render::renderer::RenderAdapterInfo>>,
    menu_state: &Res<State<MenuSubState>>,
) {
    match action {
        MenuAction::StartGame => { 
            next_game_state.set(GameState::Loading); 
            editor_mode.is_active = false; 
        },
        MenuAction::StartEditor => { 
            next_game_state.set(GameState::Loading); 
            editor_mode.is_active = true; 
        },
        MenuAction::Exit => { 
            #[cfg(not(target_arch = "wasm32"))]
            std::process::exit(0);
        },
        MenuAction::Back => { 
            let has_changes = ***pending != **settings;
            match *menu_state.get() {
                MenuSubState::Advanced => {
                    if has_changes {
                        if pending.selected_gpu != settings.selected_gpu {
                            confirmation.message = "GPU changed. Save and restart app?".to_string();
                        } else {
                            confirmation.message = "Advanced settings changed, apply them?".to_string();
                        }
                        confirmation.has_cancel = true;
                        next_menu_state.set(MenuSubState::Confirmation);
                    } else {
                        next_menu_state.set(MenuSubState::Settings);
                    }
                },
                MenuSubState::Settings => {
                    if has_changes {
                        if pending.selected_gpu != settings.selected_gpu {
                            confirmation.message = "GPU changed. Save and restart app?".to_string();
                        } else {
                            confirmation.message = "Settings have been changed, apply them?".to_string();
                        }
                        confirmation.has_cancel = true;
                        next_menu_state.set(MenuSubState::Confirmation);
                    } else {
                        next_menu_state.set(MenuSubState::Main); 
                    }
                },
                _ => {
                    next_menu_state.set(MenuSubState::Main);
                }
            }
        },
        MenuAction::OpenSettings => { next_menu_state.set(MenuSubState::Settings); },
        MenuAction::OpenAdvanced => { 
            next_menu_state.set(MenuSubState::Advanced); 
            crate::settings::populate_gpu_list(gpu_list, instance_adapter.as_ref().map(|v| &**v));
        },
        MenuAction::NextSsao => {
            pending.ssao = match pending.ssao {
                QualityLevel::Off => QualityLevel::Low,
                QualityLevel::Low => QualityLevel::Medium,
                QualityLevel::Medium => QualityLevel::High,
                QualityLevel::High => QualityLevel::Ultra,
                _ => QualityLevel::Off,
            };
        },
        MenuAction::PrevSsao => {
            pending.ssao = match pending.ssao {
                QualityLevel::Off => QualityLevel::Ultra,
                QualityLevel::Low => QualityLevel::Off,
                QualityLevel::Medium => QualityLevel::Low,
                QualityLevel::High => QualityLevel::Medium,
                _ => QualityLevel::High,
            };
        },
        MenuAction::ToggleBloom => {
            pending.bloom = !pending.bloom;
        },
        MenuAction::NextFog => {
            pending.fog_quality = match pending.fog_quality {
                QualityLevel::Off => QualityLevel::Low,
                QualityLevel::Low => QualityLevel::Medium,
                QualityLevel::Medium => QualityLevel::High,
                QualityLevel::High => QualityLevel::Ultra,
                _ => QualityLevel::Off,
            };
        },
        MenuAction::PrevFog => {
            pending.fog_quality = match pending.fog_quality {
                QualityLevel::Off => QualityLevel::Ultra,
                QualityLevel::Low => QualityLevel::Off,
                QualityLevel::Medium => QualityLevel::Low,
                QualityLevel::High => QualityLevel::Medium,
                _ => QualityLevel::High,
            };
        },
        MenuAction::NextShadowRes => {
            pending.shadow_resolution = match pending.shadow_resolution {
                512 => 1024,
                1024 => 2048,
                2048 => 4096,
                _ => 512,
            };
        },
        MenuAction::PrevShadowRes => {
            pending.shadow_resolution = match pending.shadow_resolution {
                512 => 4096,
                1024 => 512,
                2048 => 1024,
                _ => 2048,
            };
        },
        MenuAction::NextShadowQuality => {
            pending.shadow_quality = match pending.shadow_quality {
                QualityLevel::Off => QualityLevel::Low,
                QualityLevel::Low => QualityLevel::Medium,
                QualityLevel::Medium => QualityLevel::High,
                QualityLevel::High => QualityLevel::Ultra,
                _ => QualityLevel::Off,
            };
        },
        MenuAction::PrevShadowQuality => {
            pending.shadow_quality = match pending.shadow_quality {
                QualityLevel::Off => QualityLevel::Ultra,
                QualityLevel::Low => QualityLevel::Off,
                QualityLevel::Medium => QualityLevel::Low,
                QualityLevel::High => QualityLevel::Medium,
                _ => QualityLevel::High,
            };
        },
        MenuAction::ToggleFpsLimit => {
            pending.fps_limit_enabled = !pending.fps_limit_enabled;
        },
        MenuAction::NextFpsLimit => {
            pending.fps_limit = match pending.fps_limit {
                30 => 60,
                60 => 120,
                120 => 144,
                144 => 240,
                _ => 30,
            };
        },
        MenuAction::PrevFpsLimit => {
            pending.fps_limit = match pending.fps_limit {
                30 => 240,
                60 => 30,
                120 => 60,
                144 => 120,
                _ => 144,
            };
        },
        MenuAction::NextGpu => {
            if !gpu_list.names.is_empty() {
                let current = settings.selected_gpu.clone().unwrap_or_else(|| gpu_list.names[0].clone());
                if let Some(idx) = gpu_list.names.iter().position(|n| n == &current) {
                    let next_idx = (idx + 1) % gpu_list.names.len();
                    pending.selected_gpu = Some(gpu_list.names[next_idx].clone());
                }
            }
        },
        MenuAction::PrevGpu => {
            if !gpu_list.names.is_empty() {
                let current = settings.selected_gpu.clone().unwrap_or_else(|| gpu_list.names[0].clone());
                if let Some(idx) = gpu_list.names.iter().position(|n| n == &current) {
                    let next_idx = (idx + gpu_list.names.len() - 1) % gpu_list.names.len();
                    pending.selected_gpu = Some(gpu_list.names[next_idx].clone());
                }
            }
        },
        MenuAction::ApplySettings => {
            if ***pending != **settings {
                **settings = (***pending).clone();
                save_settings(&**settings);
            }
        },
        MenuAction::ConfirmYes => {
            if exit_confirm.0 {
                exit_confirm.0 = false;
                editor_mode.is_active = false;
                next_game_state.set(GameState::Menu);
            } else {
                let gpu_changed = pending.selected_gpu != settings.selected_gpu;
                **settings = (***pending).clone();
                save_settings(&**settings);
                
                if gpu_changed {
                    #[cfg(not(target_arch = "wasm32"))]
                    std::process::exit(0);
                }
                
                next_menu_state.set(MenuSubState::Main);
            }
        },
        MenuAction::ConfirmNo => {
            if exit_confirm.0 {
                exit_confirm.0 = false;
            } else {
                next_menu_state.set(MenuSubState::Settings);
            }
        },
        MenuAction::ConfirmCancel => {
            if exit_confirm.0 {
                exit_confirm.0 = false;
            } else {
                next_menu_state.set(MenuSubState::Settings);
            }
        },
        MenuAction::RunBenchmark => {
            next_game_state.set(GameState::Benchmark);
        },
        MenuAction::OpenActorEditor => {
            next_game_state.set(GameState::ActorEditor);
        },
        MenuAction::ToggleVSync => {
            pending.vsync = !pending.vsync;
        },
        MenuAction::NextWindowMode => {
            pending.window_mode = match pending.window_mode {
                MyWindowMode::Windowed => MyWindowMode::BorderlessFullscreen,
                MyWindowMode::BorderlessFullscreen => {
                    if MyWindowMode::Fullscreen.is_supported() {
                        MyWindowMode::Fullscreen
                    } else {
                        MyWindowMode::Windowed
                    }
                },
                MyWindowMode::Fullscreen => MyWindowMode::Windowed,
            };
        },
        MenuAction::PrevWindowMode => {
            pending.window_mode = match pending.window_mode {
                MyWindowMode::Windowed => {
                    if MyWindowMode::Fullscreen.is_supported() {
                        MyWindowMode::Fullscreen
                    } else {
                        MyWindowMode::BorderlessFullscreen
                    }
                },
                MyWindowMode::BorderlessFullscreen => MyWindowMode::Windowed,
                MyWindowMode::Fullscreen => MyWindowMode::BorderlessFullscreen,
            };
        },
        MenuAction::NextUpscaling => {
            pending.upscaling = match pending.upscaling {
                UpscalingMode::None => UpscalingMode::FSR,
                UpscalingMode::FSR => UpscalingMode::TAA,
                UpscalingMode::TAA => UpscalingMode::None,
            };
        },
        MenuAction::PrevUpscaling => {
            pending.upscaling = match pending.upscaling {
                UpscalingMode::None => UpscalingMode::TAA,
                UpscalingMode::FSR => UpscalingMode::None,
                UpscalingMode::TAA => UpscalingMode::FSR,
            };
        },
        MenuAction::NextQuality => {
            pending.quality_level = match pending.quality_level {
                QualityLevel::Low => QualityLevel::Medium,
                QualityLevel::Medium => QualityLevel::High,
                QualityLevel::High => QualityLevel::Ultra,
                QualityLevel::Ultra => QualityLevel::Low,
                _ => QualityLevel::Medium,
            };
            match pending.quality_level {
                QualityLevel::Low => { pending.shadow_quality = QualityLevel::Low; pending.fog_quality = QualityLevel::Low; },
                QualityLevel::Medium => { pending.shadow_quality = QualityLevel::Medium; pending.fog_quality = QualityLevel::Low; },
                QualityLevel::High => { pending.shadow_quality = QualityLevel::High; pending.fog_quality = QualityLevel::Medium; },
                QualityLevel::Ultra => { pending.shadow_quality = QualityLevel::Ultra; pending.fog_quality = QualityLevel::High; },
                _ => {},
            }
        },
        MenuAction::PrevQuality => {
            pending.quality_level = match pending.quality_level {
                QualityLevel::Low => QualityLevel::Ultra,
                QualityLevel::Medium => QualityLevel::Low,
                QualityLevel::High => QualityLevel::Medium,
                QualityLevel::Ultra => QualityLevel::High,
                _ => QualityLevel::Medium,
            };
            match pending.quality_level {
                QualityLevel::Low => { pending.shadow_quality = QualityLevel::Low; pending.fog_quality = QualityLevel::Low; },
                QualityLevel::Medium => { pending.shadow_quality = QualityLevel::Medium; pending.fog_quality = QualityLevel::Low; },
                QualityLevel::High => { pending.shadow_quality = QualityLevel::High; pending.fog_quality = QualityLevel::Medium; },
                QualityLevel::Ultra => { pending.shadow_quality = QualityLevel::Ultra; pending.fog_quality = QualityLevel::High; },
                _ => {},
            }
        },
        _ => {}
    }
}
