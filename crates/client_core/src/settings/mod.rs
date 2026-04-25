use bevy::prelude::*;
use bevy::render::renderer::RenderAdapterInfo;
use serde::{Deserialize, Serialize};

pub mod auto_detect;
use auto_detect::auto_detect_graphics;

#[derive(Resource, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GraphicsSettings {
    pub version: u32,
    pub is_loading: bool, // Safe mode flag
    
    // Display
    pub window_mode: MyWindowMode,
    pub resolution_scale: f32, 
    pub vsync: bool,
    pub fps_limit_enabled: bool,
    pub fps_limit: u32, 
    
    // Quality
    pub quality_level: QualityLevel,
    pub shadow_quality: QualityLevel,
    pub fog_quality: QualityLevel,
    
    // Performance
    pub upscaling: UpscalingMode,

    // Advanced
    pub selected_gpu: Option<String>,
    pub ssao: QualityLevel,
    pub bloom: bool,
    pub shadow_resolution: u32,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum QualityLevel {
    Off,
    Low,
    Medium,
    High,
    Ultra,
    Custom,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum UpscalingMode {
    None,
    FSR,
    TAA,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum MyWindowMode {
    Windowed,
    BorderlessFullscreen,
    Fullscreen,
}

impl MyWindowMode {
    pub fn is_supported(&self) -> bool {
        match self {
            MyWindowMode::Fullscreen => {
                #[cfg(target_arch = "wasm32")] return false;
                #[cfg(not(target_arch = "wasm32"))] {
                    // Wayland doesn't support exclusive fullscreen
                    let is_wayland = std::env::var("XDG_SESSION_TYPE").map(|v| v == "wayland").unwrap_or(false);
                    !is_wayland
                }
            },
            _ => true,
        }
    }
}

impl Default for GraphicsSettings {
    fn default() -> Self {
        Self {
            version: 1,
            is_loading: false,
            window_mode: MyWindowMode::Windowed,
            resolution_scale: 1.0,
            vsync: true,
            fps_limit_enabled: false,
            fps_limit: 60,
            quality_level: QualityLevel::Medium,
            shadow_quality: QualityLevel::Medium,
            fog_quality: QualityLevel::Medium,
            upscaling: UpscalingMode::None,
            selected_gpu: None,
            ssao: QualityLevel::Off,
            bloom: true,
            shadow_resolution: 1024,
        }
    }
}

#[derive(Resource, Default, Debug, Clone)]
pub struct GpuList {
    pub names: Vec<String>,
}

pub struct SettingsPlugin;

#[derive(Resource)]
struct NeedsAutoDetect(bool);

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        let (settings, needs_auto) = load_settings_or_default();
        app.insert_resource(settings)
           .insert_resource(NeedsAutoDetect(needs_auto))
           .insert_resource(bevy::pbr::DirectionalLightShadowMap { size: 1024 })
           .init_resource::<GpuList>()
           .add_plugins(bevy_framepace::FramepacePlugin)
           .add_systems(Update, (
               apply_settings_system,
               init_settings_system,
           ));
    }
}

pub fn pre_init_gpu_settings() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Ok(content) = std::fs::read_to_string(SETTINGS_FILE) {
            if let Ok(settings) = serde_json::from_str::<GraphicsSettings>(&content) {
                if let Some(gpu_name) = settings.selected_gpu {
                    println!("--- Forcing GPU Adapter: {} ---", gpu_name);
                    std::env::set_var("WGPU_ADAPTER_NAME", gpu_name);
                }
            }
        }
    }
}

pub fn get_wgpu_settings() -> bevy::render::settings::WgpuSettings {
    let mut wgpu_settings = bevy::render::settings::WgpuSettings::default();
    
    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Ok(content) = std::fs::read_to_string(SETTINGS_FILE) {
            if let Ok(settings) = serde_json::from_str::<GraphicsSettings>(&content) {
                if let Some(gpu_name) = settings.selected_gpu {
                    let name = gpu_name.to_lowercase();
                    if name.contains("rtx") || name.contains("gtx") || name.contains("discrete") || name.contains("radeon rx") {
                        wgpu_settings.power_preference = bevy::render::settings::PowerPreference::HighPerformance;
                    } else if name.contains("integrated") || name.contains("intel") || name.contains("uhd") {
                        wgpu_settings.power_preference = bevy::render::settings::PowerPreference::LowPower;
                    }
                }
            }
        }
    }
    
    wgpu_settings
}

pub fn populate_gpu_list(
    gpu_list: &mut GpuList,
    instance_adapter_opt: Option<&RenderAdapterInfo>,
) {
    if !gpu_list.names.is_empty() { return; }

    // Try to get current adapter from Bevy first
    if let Some(adapter) = instance_adapter_opt {
        if !gpu_list.names.contains(&adapter.name) {
            gpu_list.names.push(adapter.name.clone());
        }
    }

    // Only enumerate other adapters if we are not on WASM
    #[cfg(not(target_arch = "wasm32"))]
    {
        // Use PRIMARY backends only (Vulkan/Metal/DX12) to avoid crashes with some drivers/OpenGL
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });
        for adapter in instance.enumerate_adapters(wgpu::Backends::PRIMARY) {
            let name = adapter.get_info().name;
            if !gpu_list.names.contains(&name) {
                gpu_list.names.push(name);
            }
        }
    }
}

fn init_settings_system(
    mut commands: Commands,
    needs_auto_opt: Option<Res<NeedsAutoDetect>>,
    adapter_opt: Option<Res<RenderAdapterInfo>>,
    mut settings: ResMut<GraphicsSettings>,
    mut gpu_list: ResMut<GpuList>,
) {
    if let Some(adapter) = adapter_opt {
        if settings.selected_gpu.is_none() {
            settings.selected_gpu = Some(adapter.name.clone());
        }
        
        if gpu_list.names.is_empty() {
             populate_gpu_list(&mut gpu_list, Some(&adapter));
        }

        if let Some(needs_auto) = needs_auto_opt {
            if needs_auto.0 {
                *settings = auto_detect_graphics(&adapter);
                save_settings(&settings);
            }
            commands.remove_resource::<NeedsAutoDetect>();
        }
    }
}

pub const SETTINGS_FILE: &str = "settings.json";

pub fn load_settings_or_default() -> (GraphicsSettings, bool) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Ok(content) = std::fs::read_to_string(SETTINGS_FILE) {
            if let Ok(mut settings) = serde_json::from_str::<GraphicsSettings>(&content) {
                if settings.is_loading {
                    println!("--- Detected crash on last run. Resetting to safe defaults. ---");
                    return (GraphicsSettings::default(), false);
                }
                settings.is_loading = true;
                // Fix unsupported modes on load
                if !settings.window_mode.is_supported() {
                    settings.window_mode = MyWindowMode::BorderlessFullscreen;
                }
                let _ = save_settings_to_disk(&settings);
                return (settings, false);
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                if let Ok(Some(content)) = storage.get_item("graphics_settings") {
                    if let Ok(mut settings) = serde_json::from_str::<GraphicsSettings>(&content) {
                        if settings.is_loading {
                            return (GraphicsSettings::default(), false);
                        }
                        settings.is_loading = true;
                        let _ = save_settings_to_web(&settings);
                        return (settings, false);
                    }
                }
            }
        }
    }

    (GraphicsSettings::default(), true)
}

pub fn save_settings(settings: &GraphicsSettings) {
    #[cfg(not(target_arch = "wasm32"))] let _ = save_settings_to_disk(settings);
    #[cfg(target_arch = "wasm32")] let _ = save_settings_to_web(settings);
}

fn save_settings_to_disk(settings: &GraphicsSettings) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string_pretty(settings)?;
    std::fs::write(SETTINGS_FILE, json)?;
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn save_settings_to_web(settings: &GraphicsSettings) -> Result<(), wasm_bindgen::JsValue> {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let json = serde_json::to_string(settings).map_err(|e| wasm_bindgen::JsValue::from_str(&e.to_string()))?;
            storage.set_item("graphics_settings", &json)?;
        }
    }
    Ok(())
}

fn apply_settings_system(
    settings: Res<GraphicsSettings>,
    mut windows: Query<&mut Window>,
    mut framepace: ResMut<bevy_framepace::FramepaceSettings>,
) {
    if !settings.is_changed() { return; }
    
    if let Ok(mut window) = windows.get_single_mut() {
        // Apply Window Mode
        window.mode = match settings.window_mode {
            MyWindowMode::Windowed => bevy::window::WindowMode::Windowed,
            MyWindowMode::BorderlessFullscreen => bevy::window::WindowMode::BorderlessFullscreen,
            MyWindowMode::Fullscreen => bevy::window::WindowMode::SizedFullscreen,
        };
        
        // Apply VSync
        window.present_mode = if settings.vsync {
            bevy::window::PresentMode::AutoVsync
        } else {
            bevy::window::PresentMode::AutoNoVsync
        };

        // Apply FPS Limit
        if settings.fps_limit_enabled && !settings.vsync {
            framepace.limiter = bevy_framepace::Limiter::from_framerate(settings.fps_limit as f64);
        } else {
            framepace.limiter = bevy_framepace::Limiter::Off;
        }
    }
}

pub fn finish_loading_settings(mut settings: ResMut<GraphicsSettings>) {
    settings.is_loading = false;
    save_settings(&settings);
}
