use bevy::prelude::*;
use super::super::{EditorMaterialColor, SlicingSettings};

pub trait Command: Send + Sync + 'static {
    fn name(&self) -> String;
    fn execute(&self, world: &mut World);
    fn undo(&self, world: &mut World);
}

#[derive(Resource)]
pub struct ActionStack {
    pub undo: Vec<Box<dyn Command>>,
    pub redo: Vec<Box<dyn Command>>,
    pub max_size: usize,
}

impl Default for ActionStack {
    fn default() -> Self {
        Self {
            undo: Vec::new(),
            redo: Vec::new(),
            max_size: 50,
        }
    }
}

impl ActionStack {
    pub fn push(&mut self, command: Box<dyn Command>) {
        info!("ActionStack: Pushing new command '{}'", command.name());
        self.undo.push(command);
        self.redo.clear();
        if self.undo.len() > self.max_size {
            self.undo.remove(0);
        }
    }
}

#[derive(Event)]
pub struct UndoEvent;

#[derive(Event)]
pub struct RedoEvent;

// --- Commands ---

pub struct TransformSocketGroupCommand {
    pub transforms: Vec<(Entity, Transform, Transform)>,
}

impl Command for TransformSocketGroupCommand {
    fn name(&self) -> String { "Transform Sockets".to_string() }
    
    fn execute(&self, world: &mut World) {
        for (entity, _, new_transform) in &self.transforms {
            if let Some(mut transform) = world.get_mut::<Transform>(*entity) {
                *transform = *new_transform;
            }
        }
    }
    
    fn undo(&self, world: &mut World) {
        for (entity, old_transform, _) in &self.transforms {
            if let Some(mut transform) = world.get_mut::<Transform>(*entity) {
                *transform = *old_transform;
            }
        }
    }
}

pub struct ChangeMaterialColorCommand {
    pub old_color: Color,
    pub new_color: Color,
}

impl Command for ChangeMaterialColorCommand {
    fn name(&self) -> String { "Change Material Color".to_string() }
    
    fn execute(&self, world: &mut World) {
        if let Some(mut color_res) = world.get_resource_mut::<EditorMaterialColor>() {
            color_res.color = self.new_color;
        }
    }
    
    fn undo(&self, world: &mut World) {
        if let Some(mut color_res) = world.get_resource_mut::<EditorMaterialColor>() {
            color_res.color = self.old_color;
        }
    }
}

pub struct UpdateSlicingCommand {
    pub old_top: f32,
    pub old_bottom: f32,
    pub new_top: f32,
    pub new_bottom: f32,
}

impl Command for UpdateSlicingCommand {
    fn name(&self) -> String { "Update Slicing".to_string() }
    
    fn execute(&self, world: &mut World) {
        if let Some(mut slicing) = world.get_resource_mut::<SlicingSettings>() {
            slicing.top_cut = self.new_top;
            slicing.bottom_cut = self.new_bottom;
            slicing.trigger_slice = true;
            slicing.suppress_undo = true;
        }
    }
    
    fn undo(&self, world: &mut World) {
        if let Some(mut slicing) = world.get_resource_mut::<SlicingSettings>() {
            slicing.top_cut = self.old_top;
            slicing.bottom_cut = self.old_bottom;
            slicing.trigger_slice = true;
            slicing.suppress_undo = true;
        }
    }
}

pub struct AddSocketCommand {
    pub entity: Entity,
    pub definition: crate::actor_editor::SocketDefinition,
}

impl Command for AddSocketCommand {
    fn name(&self) -> String { "Add Socket".to_string() }
    
    fn execute(&self, world: &mut World) {
        if let Some(mut entity) = world.get_entity_mut(self.entity) {
            if let Some(mut visibility) = entity.get_mut::<Visibility>() {
                *visibility = Visibility::Inherited;
            }
            entity.insert(crate::actor_editor::ActorSocket {
                definition: self.definition.clone(),
            });
        }
    }
    
    fn undo(&self, world: &mut World) {
        if let Some(mut entity) = world.get_entity_mut(self.entity) {
            if let Some(mut visibility) = entity.get_mut::<Visibility>() {
                *visibility = Visibility::Hidden;
            }
            entity.remove::<crate::actor_editor::ActorSocket>();
        }
    }
}

pub struct DeleteSocketCommand {
    pub entity: Entity,
    pub definition: crate::actor_editor::SocketDefinition,
}

impl Command for DeleteSocketCommand {
    fn name(&self) -> String { "Delete Socket".to_string() }
    
    fn execute(&self, world: &mut World) {
        if let Some(mut entity) = world.get_entity_mut(self.entity) {
            if let Some(mut visibility) = entity.get_mut::<Visibility>() {
                *visibility = Visibility::Hidden;
            }
            entity.remove::<crate::actor_editor::ActorSocket>();
        }
    }
    
    fn undo(&self, world: &mut World) {
        if let Some(mut entity) = world.get_entity_mut(self.entity) {
            if let Some(mut visibility) = entity.get_mut::<Visibility>() {
                *visibility = Visibility::Inherited;
            }
            entity.insert(crate::actor_editor::ActorSocket {
                definition: self.definition.clone(),
            });
        }
    }
}
pub struct ScaleModelCommand {
    pub entity: Entity,
    pub old_scale: Vec3,
    pub new_scale: Vec3,
}

impl Command for ScaleModelCommand {
    fn name(&self) -> String { "Scale Model".to_string() }
    
    fn execute(&self, world: &mut World) {
        if let Some(mut transform) = world.get_mut::<Transform>(self.entity) {
            transform.scale = self.new_scale;
        }
        if let Some(mut slicing) = world.get_resource_mut::<SlicingSettings>() {
            slicing.trigger_slice = true;
        }
    }
    
    fn undo(&self, world: &mut World) {
        if let Some(mut transform) = world.get_mut::<Transform>(self.entity) {
            transform.scale = self.old_scale;
        }
        if let Some(mut slicing) = world.get_resource_mut::<SlicingSettings>() {
            slicing.trigger_slice = true;
        }
    }
}

// --- Systems ---

pub fn undo_redo_shortcuts_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut undo_events: EventWriter<UndoEvent>,
    mut redo_events: EventWriter<RedoEvent>,
) {
    let ctrl = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);
    let shift = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);
    
    if ctrl && keyboard.just_pressed(KeyCode::KeyZ) {
        if shift {
            redo_events.send(RedoEvent);
        } else {
            undo_events.send(UndoEvent);
        }
    }
    
    if ctrl && keyboard.just_pressed(KeyCode::KeyY) {
        redo_events.send(RedoEvent);
    }
}


// Let's refine undo_redo_system to avoid borrow issues with World
pub fn handle_undo_redo(world: &mut World) {
    let undo_triggered = world.resource_mut::<Events<UndoEvent>>().drain().next().is_some();
    let redo_triggered = world.resource_mut::<Events<RedoEvent>>().drain().next().is_some();
    
    if undo_triggered {
        let cmd_opt = world.resource_mut::<ActionStack>().undo.pop();
        if let Some(cmd) = cmd_opt {
            info!("ActionStack: UNDO '{}'", cmd.name());
            cmd.undo(world);
            let mut stack = world.resource_mut::<ActionStack>();
            stack.redo.push(cmd);
            if stack.redo.len() > stack.max_size {
                stack.redo.remove(0);
            }
        }
    }
    
    if redo_triggered {
        let cmd_opt = world.resource_mut::<ActionStack>().redo.pop();
        if let Some(cmd) = cmd_opt {
            info!("ActionStack: REDO '{}'", cmd.name());
            cmd.execute(world);
            let mut stack = world.resource_mut::<ActionStack>();
            stack.undo.push(cmd);
            if stack.undo.len() > stack.max_size {
                stack.undo.remove(0);
            }
        }
    }
}

pub fn undo_redo_ui_system(
    action_stack: Res<ActionStack>,
    undo_btn_query: Query<&Interaction, (With<super::super::UndoButton>, Changed<Interaction>)>,
    redo_btn_query: Query<&Interaction, (With<super::super::RedoButton>, Changed<Interaction>)>,
    mut undo_events: EventWriter<UndoEvent>,
    mut redo_events: EventWriter<RedoEvent>,
) {
    for interaction in undo_btn_query.iter() {
        if *interaction == Interaction::Pressed && !action_stack.undo.is_empty() {
            undo_events.send(UndoEvent);
        }
    }
    for interaction in redo_btn_query.iter() {
        if *interaction == Interaction::Pressed && !action_stack.redo.is_empty() {
            redo_events.send(RedoEvent);
        }
    }
}

pub fn undo_redo_button_visual_system(
    action_stack: Res<ActionStack>,
    mut undo_query: Query<(&mut BackgroundColor, &Interaction), (With<super::super::UndoButton>, Without<super::super::RedoButton>)>,
    mut redo_query: Query<(&mut BackgroundColor, &Interaction), (With<super::super::RedoButton>, Without<super::super::UndoButton>)>,
) {
    let undo_available = !action_stack.undo.is_empty();
    let redo_available = !action_stack.redo.is_empty();

    for (mut bg, interaction) in undo_query.iter_mut() {
        if undo_available {
            *bg = match *interaction {
                Interaction::Pressed => Color::srgba(1.0, 1.0, 1.0, 0.4).into(),
                Interaction::Hovered => Color::srgba(1.0, 1.0, 1.0, 0.2).into(),
                Interaction::None => Color::srgba(1.0, 1.0, 1.0, 0.1).into(),
            };
        } else {
            *bg = Color::srgba(0.2, 0.2, 0.2, 0.1).into();
        }
    }

    for (mut bg, interaction) in redo_query.iter_mut() {
        if redo_available {
            *bg = match *interaction {
                Interaction::Pressed => Color::srgba(1.0, 1.0, 1.0, 0.4).into(),
                Interaction::Hovered => Color::srgba(1.0, 1.0, 1.0, 0.2).into(),
                Interaction::None => Color::srgba(1.0, 1.0, 1.0, 0.1).into(),
            };
        } else {
            *bg = Color::srgba(0.2, 0.2, 0.2, 0.1).into();
        }
    }
}
