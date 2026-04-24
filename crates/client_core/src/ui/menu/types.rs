use bevy::prelude::*;
use crate::GraphicsSettings;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MenuItemType {
    Action,
    Submenu,
    Toggle,
    Slider,
}

#[derive(Clone, Debug, PartialEq, Component)]
pub enum MenuAction {
    StartGame,
    StartEditor,
    Exit,
    Navigate(String), 
    Back,
    OpenSettings,
    ApplySettings,
    ToggleVSync,
    NextWindowMode, PrevWindowMode,
    NextUpscaling, PrevUpscaling,
    NextQuality, PrevQuality,
    SetResolution(u32, u32),
    ConfirmYes,
    ConfirmNo,
    ConfirmCancel,
    OpenAdvanced,
    NextGpu, PrevGpu,
    NextSsao, PrevSsao,
    ToggleBloom,
    NextFog, PrevFog,
    NextShadowRes, PrevShadowRes,
    NextShadowQuality, PrevShadowQuality,
    ToggleFpsLimit,
    NextFpsLimit, PrevFpsLimit,
    RunBenchmark,
    OpenActorEditor,
    None,
}

#[derive(Resource, Default, PartialEq, Eq, Debug, Clone, Copy)]
pub enum InputDevice {
    #[default]
    Keyboard,
    Gamepad,
    Mouse,
    Touch,
}

#[derive(States, Default, PartialEq, Eq, Debug, Clone, Copy, Hash)]
pub enum MenuSubState {
    #[default]
    Main,
    Settings,
    Confirmation,
    Advanced,
}

#[derive(Component)]
pub struct MenuContainer {
    pub current_selection: usize,
    pub items_count: usize,
}

#[derive(Component)]
pub struct MenuScrollContainer;

#[derive(Component)]
pub struct MenuViewport;

#[derive(Component)]
pub struct MenuTitleDisplay;

#[derive(Component)]
pub struct MenuItem {
    pub index: usize,
    pub item_type: MenuItemType,
    pub action: MenuAction,
    pub tooltip: Option<String>,
    pub is_disabled: bool,
}

#[derive(Component)]
pub struct MenuFocus; 

#[derive(Component)]
pub struct MenuTooltip {
    pub timer: Timer,
}

#[derive(Component)]
pub struct InputHintFooter;

#[derive(Component)]
pub struct TooltipDisplay;

#[derive(Resource, Default)]
pub struct ExtraMenuButtons {
    pub buttons: Vec<(String, MenuAction)>,
}

#[derive(Resource, Default)]
pub struct MenuSelectionMemory {
    pub selections: std::collections::HashMap<MenuSubState, usize>,
}

#[derive(Resource, Default)]
pub struct ExitConfirmationActive(pub bool);

#[derive(Component)]
pub struct MenuEntity;

#[derive(Component)]
pub struct ConfirmationOverlay;

#[derive(Resource, Default, Deref, DerefMut)]
pub struct PendingGraphicsSettings(pub GraphicsSettings);

#[derive(Resource, Default)]
pub struct ConfirmationData {
    pub message: String,
    pub has_cancel: bool,
}

#[derive(Resource)]
pub struct MenuNavigationTimer {
    pub timer: Timer,
}

impl Default for MenuNavigationTimer {
    fn default() -> Self {
        let mut timer = Timer::from_seconds(0.4, TimerMode::Once);
        timer.set_elapsed(std::time::Duration::from_secs_f32(0.4));
        Self { timer }
    }
}

#[derive(Component)]
pub struct MenuItemRoot;
