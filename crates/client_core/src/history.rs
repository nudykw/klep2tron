use bevy::prelude::*;
use crate::Project;

#[derive(Resource, Default)]
pub struct CommandHistory {
    pub undo_stack: Vec<Project>,
    pub redo_stack: Vec<Project>,
}

impl CommandHistory {
    pub fn push_undo(&mut self, project: &Project) {
        self.undo_stack.push(project.clone());
        self.redo_stack.clear();
        if self.undo_stack.len() > 100 {
            self.undo_stack.remove(0);
        }
    }

    pub fn undo(&mut self, current: &Project) -> Option<Project> {
        if let Some(prev) = self.undo_stack.pop() {
            self.redo_stack.push(current.clone());
            Some(prev)
        } else {
            None
        }
    }

    pub fn redo(&mut self, current: &Project) -> Option<Project> {
        if let Some(next) = self.redo_stack.pop() {
            self.undo_stack.push(current.clone());
            Some(next)
        } else {
            None
        }
    }
}
