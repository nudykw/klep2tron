use bevy::prelude::*;
use crate::{Project, DirtyTiles};

#[derive(Resource, Default)]
pub struct RoomTransition {
    pub phase: TransitionPhase,
    pub timer: f32,
    pub target_room_idx: usize,
    pub speed: f32,
}

impl RoomTransition {
    pub fn start(&mut self, target_idx: usize) {
        self.phase = TransitionPhase::Out;
        self.target_room_idx = target_idx;
        self.timer = 0.0;
        self.speed = 2.0;
    }
}

#[derive(PartialEq, Default, Debug, Clone, Copy)]
pub enum TransitionPhase {
    #[default]
    Idle,
    Out,
    In,
}

#[derive(Component)]
pub struct TransitionUi;

pub fn transition_logic_system(
    time: Res<Time>,
    mut transition: ResMut<RoomTransition>,
    mut project: ResMut<Project>,
    mut dirty: ResMut<DirtyTiles>,
) {
    if transition.phase == TransitionPhase::Idle { return; }

    transition.timer += time.delta_seconds() * transition.speed;

    if transition.phase == TransitionPhase::Out && transition.timer >= 1.0 {
        project.current_room_idx = transition.target_room_idx;
        dirty.full_rebuild = true;
        transition.phase = TransitionPhase::In;
        transition.timer = 0.0;
    } else if transition.phase == TransitionPhase::In && transition.timer >= 1.0 {
        transition.phase = TransitionPhase::Idle;
        transition.timer = 0.0;
    }
}

pub fn transition_ui_system(
    mut commands: Commands,
    transition: Res<RoomTransition>,
    query: Query<Entity, With<TransitionUi>>,
    mut overlay_query: Query<&mut BackgroundColor, With<TransitionUi>>,
) {
    if transition.phase == TransitionPhase::Idle {
        if let Ok(entity) = query.get_single() {
            commands.entity(entity).despawn_recursive();
        }
        return;
    }

    if query.is_empty() {
        commands.spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0), height: Val::Percent(100.0),
                    position_type: PositionType::Absolute,
                    ..default()
                },
                background_color: Color::NONE.into(),
                z_index: ZIndex::Global(1000), 
                ..default()
            },
            TransitionUi,
        ));
    } else if let Ok(mut color) = overlay_query.get_single_mut() {
        let alpha = match transition.phase {
            TransitionPhase::Out => transition.timer.clamp(0.0, 1.0),
            TransitionPhase::In => (1.0 - transition.timer).clamp(0.0, 1.0),
            _ => 0.0,
        };
        *color = Color::srgba(0.0, 0.0, 0.0, alpha).into();
    }
}
