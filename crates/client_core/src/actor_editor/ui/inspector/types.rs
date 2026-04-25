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
 
#[derive(Component)]
pub struct SocketMetadataSection;

#[derive(Resource, Default)]
pub struct SelectedSocket(pub Vec<Entity>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
pub enum SelectionMode {
    #[default]
    Replace,
    Add,
    Range,
}

#[derive(Resource, Default)]
pub struct MultiSelectionState {
    pub mode: SelectionMode,
    pub last_selected: Option<Entity>,
}

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
pub struct SocketsSectionMarker;

#[derive(Component)]
pub struct PartsSectionMarker;

#[derive(Component)]
pub struct OptimizationCapsToggle;

#[derive(Component)]
pub struct OptimizationRimSlider;

#[derive(Component)]
pub struct OptimizationSectionMarker;

#[derive(Component)]
pub struct OptimizationTargetInput;

#[derive(Component)]
pub struct OptimizeMeshButton;

#[derive(Component)]
pub struct OptimizationOriginalToggle;

#[derive(Component)]
pub struct OptimizationTargetInputMarker;

#[derive(Component)]
pub struct OptimizationWireframeToggle;

#[derive(Component)]
pub struct InspectionToggle(pub InspectionToggleType);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InspectionToggleType {
    Ghost,
    Wireframe,
    Normals,
}

#[derive(Component)]
pub struct SocketVfxSection;
#[derive(Component)]
pub struct SocketVfxEmissionSection;
#[derive(Component)]
pub struct SocketVfxMotionSection;
#[derive(Component)]
pub struct SocketVfxVisualsSection;

#[derive(Component)]
pub struct SocketVfxToggle;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
pub enum SocketVfxSlider {
    EmissionRate,
    EmissionLifetime,
    EmissionJitter,
    MotionSpeed,
    MotionSpread,
    MotionGravity,
    MotionDrag,
    VisualsScale,
    VisualsSizeStart,
    VisualsSizeEnd,
}
#[derive(Component)]
pub struct SocketVfxPresetItem(pub String);
#[derive(Component)]
pub struct SocketVfxTextureItem(pub String);
#[derive(Component)]
pub struct SocketVfxGroupItem(pub String);

#[derive(Component)]
pub struct SocketVfxColorPicker;

#[derive(Component)]
pub struct SocketVfxSavePresetButton;

#[derive(Component)]
pub struct SocketVfxPresetNameInput;

#[derive(Component)]
pub struct SocketVfxDetachPresetButton;

#[derive(Component)]
pub struct SocketVfxPresetStatusLabel;
