mod types;
mod ui;
mod systems;

pub use self::types::*;
pub use self::ui::*;
pub use self::systems::input::*;
pub use self::systems::actions::*;
pub use self::systems::visuals::*;

use bevy::prelude::*;
use crate::GameState;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputDevice>()
            .init_resource::<MenuNavigationTimer>()
            .init_resource::<MenuSelectionMemory>()
            .init_resource::<PendingGraphicsSettings>()
            .init_resource::<ConfirmationData>();
        
        let menu_cond = menu_visible;

        app.add_systems(Update, (
                device_detection_system,
                menu_item_system,
                apply_deferred,
                menu_input_system,
                menu_navigation_system,
                menu_scrolling_system,
                menu_visual_system,
                tooltip_system,
                input_hint_system,
                menu_tooltip_system,
           ).chain().run_if(menu_cond.clone()))

           .add_systems(Update, sync_pending_settings.run_if(in_state(GameState::Menu)));
    }
}

fn menu_visible(
    state: Res<State<GameState>>,
    exit_confirm: Res<ExitConfirmationActive>,
) -> bool {
    *state.get() == GameState::Menu || exit_confirm.0
}
