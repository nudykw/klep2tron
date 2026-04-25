use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Reflect, Default)]
pub struct VfxPresetLibrary {
    pub presets: HashMap<String, EffectConfig>,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component)]
pub enum ActorPart {
    Head,
    #[default]
    Body,
    Engine,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Reflect, PartialEq, Default)]
pub enum EffectType {
    #[default]
    Plasma,
    MuzzleFlash,
    Smoke,
    Hanabi,
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect, PartialEq)]
pub struct EmissionConfig {
    pub rate: f32,
    pub lifetime: f32,
    pub jitter: f32,
}

impl Default for EmissionConfig {
    fn default() -> Self {
        Self {
            rate: 1.0,
            lifetime: 1.0,
            jitter: 0.1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect, PartialEq)]
pub struct MotionConfig {
    pub speed: f32,
    pub spread: f32,
    pub gravity: f32,
    pub drag: f32,
}

impl Default for MotionConfig {
    fn default() -> Self {
        Self {
            speed: 1.0,
            spread: 0.2,
            gravity: 0.0,
            drag: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect, PartialEq)]
pub struct VisualsConfig {
    pub scale: f32,
    pub color_start: Color,
    pub color_end: Color,
    pub size_start: f32,
    pub size_end: f32,
}

impl Default for VisualsConfig {
    fn default() -> Self {
        Self {
            scale: 1.0,
            color_start: Color::srgb(1.0, 1.0, 1.0),
            color_end: Color::srgba(1.0, 1.0, 1.0, 0.0),
            size_start: 1.0,
            size_end: 0.5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect, PartialEq, Default)]
pub struct EffectConfig {
    pub effect_type: EffectType,
    pub emission: EmissionConfig,
    pub motion: MotionConfig,
    pub visuals: VisualsConfig,
    pub asset_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect, Default)]
pub struct SocketDefinition {
    pub name: String,
    pub part: ActorPart,
    pub position: Vec3,
    pub rotation: Quat,
    pub comment: String,
    pub color: Color,
    pub effect: Option<EffectConfig>,
    pub effect_preset: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect, Default)]
pub struct ActorConfig {
    pub sockets: Vec<SocketDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect, Default)]
pub struct ActorProject {
    pub name: String,
    pub source_path: String,
    pub cut_top: f32,
    pub cut_bottom: f32,
    pub config: ActorConfig,
}
