use bevy::prelude::*;

pub mod spawning;
pub mod picking;
pub mod interaction;
pub mod visuals;

pub use spawning::*;
pub use picking::*;
pub use interaction::*;
pub use visuals::*;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum GizmoAxisType {
    X, Y, Z
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum GizmoAction {
    Translate,
    Rotate,
}

#[derive(Component)]
pub struct SocketGizmo;

#[derive(Component)]
pub struct GizmoAxis {
    pub axis: GizmoAxisType,
    pub action: GizmoAction,
}

#[derive(Component)]
pub struct SocketLink(pub Entity);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ManualGizmoInteraction {
    #[default]
    None,
    Hovered,
    Pressed,
}
