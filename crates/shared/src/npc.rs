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
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct EffectConfig {
    pub effect_type: EffectType,
    pub color: Color,
    pub speed: f32,
    pub scale: f32,
    pub intensity: f32,
}

impl Default for EffectConfig {
    fn default() -> Self {
        Self {
            effect_type: EffectType::default(),
            color: Color::srgb(0.0, 1.0, 1.0), // Cyan default for Plasma
            speed: 1.0,
            scale: 1.0,
            intensity: 1.0,
        }
    }
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
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect, Default)]
pub struct ActorConfig {
    pub sockets: Vec<SocketDefinition>,
}
