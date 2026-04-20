use bevy::render::renderer::RenderAdapterInfo;
use crate::settings::{GraphicsSettings, QualityLevel, UpscalingMode};

pub fn auto_detect_graphics(adapter: &RenderAdapterInfo) -> GraphicsSettings {
    let mut settings = GraphicsSettings::default();
    
    let device_name = adapter.name.to_lowercase();
    let is_mobile = format!("{:?}", adapter.device_type).contains("IntegratedGpu") || 
                    device_name.contains("android") || 
                    device_name.contains("ios") ||
                    device_name.contains("apple gpu");

    if is_mobile {
        settings.quality_level = QualityLevel::Low;
        settings.shadow_quality = QualityLevel::Low;
        settings.fog_quality = QualityLevel::Low;
        settings.resolution_scale = 0.75;
        settings.upscaling = UpscalingMode::FSR;
    } else if device_name.contains("rtx") || device_name.contains("radeon rx") {
        settings.quality_level = QualityLevel::Ultra;
        settings.shadow_quality = QualityLevel::Ultra;
        settings.fog_quality = QualityLevel::High;
        settings.upscaling = UpscalingMode::TAA;
    } else if device_name.contains("gtx") || device_name.contains("radeon") {
        settings.quality_level = QualityLevel::High;
        settings.shadow_quality = QualityLevel::High;
    } else {
        settings.quality_level = QualityLevel::Medium;
    }

    settings.is_loading = true; // Still loading
    settings
}
