use bevy::prelude::*;
use crate::actor_editor::ActorPart;

#[derive(Component)]
pub struct InspectorPanel;

#[derive(Component)]
pub struct SocketSearchInput;

#[derive(Component)]
pub struct SocketPartFilterButton(pub Option<ActorPart>);

#[derive(Resource, Default)]
pub struct SocketFilterState {
    pub search_text: String,
    pub part_filter: Option<ActorPart>,
}

#[derive(Component)]
pub struct SocketListItem(pub Entity);

#[derive(Component)]
pub struct SocketListItemLabel;

#[derive(Component)]
pub struct SocketListContainer;

#[derive(Component)]
pub struct SocketAddModeButton;

#[derive(Component)]
pub struct MaterialColorPreview;
 
#[derive(Component)]
pub struct SocketNameInput;
 
#[derive(Component)]
pub struct SocketCommentInput;
 
#[derive(Component)]
pub struct SocketDetailsContainer;

#[derive(Resource, Default)]
pub struct SelectedSocket(pub Option<Entity>);

#[derive(Component)]
pub enum TransformAxis {
    X, Y, Z
}

#[derive(Component)]
pub enum RotationAxis {
    Roll, Pitch, Yaw
}

#[derive(Component)]
pub struct SocketResetRotationButton;

#[derive(Component)]
pub struct PartFocusButton(pub ActorPart);

#[derive(Component)]
pub struct PartSoloButton(pub ActorPart);

#[derive(Component)]
pub struct InspectionMasterToggle;

#[derive(Component)]
pub struct PartsSectionMarker;

#[derive(Component)]
pub struct InspectionToggle(pub InspectionToggleType);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InspectionToggleType {
    Ghost,
    Wireframe,
    Normals,
}
