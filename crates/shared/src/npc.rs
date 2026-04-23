use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component)]
pub enum ActorPart {
    Head,
    #[default]
    Body,
    Engine,
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect, Default)]
pub struct SocketDefinition {
    pub name: String,
    pub part: ActorPart,
    pub position: Vec3,
    pub rotation: Quat,
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect, Default)]
pub struct ActorConfig {
    pub sockets: Vec<SocketDefinition>,
}
