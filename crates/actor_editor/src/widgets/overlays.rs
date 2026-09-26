use bevy::prelude::*;
use client_core::ui::widgets::*;
use super::super::{ActorEditorEntity, ToastType, EditorAction};

#[derive(Component)]
pub struct ToastContainer;

#[derive(Component)]
pub struct ToastTimer(pub Timer);

pub fn spawn_toast_container(commands: &mut Commands, target_camera: Option<Entity>) -> Entity {
    let mut cmd = commands.spawn((UiNode { node: Node { position_type: PositionType::Absolute, bottom: Val::Px(40.0), right: Val::Px(20.0), flex_direction: FlexDirection::ColumnReverse, align_items: AlignItems::End, ..default() }, ..default() }, GlobalZIndex(110), ToastContainer, ActorEditorEntity, ));
    if let Some(camera) = target_camera { cmd.insert(bevy::ui::UiTargetCamera(camera)); }
    cmd.id()
}

pub fn spawn_toast_item(parent: &mut ChildSpawnerCommands, font: &Handle<Font>, icon_font: &Handle<Font>, message: &str, toast_type: ToastType) {
    let (icon, color) = match toast_type { ToastType::Info => ("\u{f05a}", Color::srgb(0.3, 0.6, 1.0)), ToastType::Success => ("\u{f058}", Color::srgb(0.3, 0.8, 0.3)), ToastType::Error => ("\u{f071}", Color::srgb(0.8, 0.3, 0.3)), };
    parent.spawn((UiNode { node: Node { width: Val::Px(250.0), margin: UiRect::bottom(Val::Px(10.0)), padding: UiRect::all(Val::Px(12.0)), flex_direction: FlexDirection::Row, align_items: AlignItems::Center, border_radius: BorderRadius::all(Val::Px(8.0)), ..default() }, background_color: BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.95)), border_color: BorderColor::all(color.with_alpha(0.3)), ..default() }, ToastTimer(Timer::from_seconds(4.0, TimerMode::Once)), )).with_children(|p| {
        p.spawn(ui_text(format!("{} ", icon), &icon_font.clone(), 18.0, Color::WHITE));
        p.spawn(ui_text(message, &font.clone(), 14.0, Color::WHITE));
    });
}

#[derive(Component)]
pub struct ModalOverlay;

#[derive(Component)]
pub struct ConfirmModalButton(pub EditorAction);

#[derive(Component)]
pub struct CancelModalButton;

pub fn spawn_confirmation_modal(commands: &mut Commands, font: &Handle<Font>, _icon_font: &Handle<Font>, title: &str, message: &str, action: EditorAction, target_camera: Option<Entity>) {
    let mut cmd = commands.spawn((UiNode { node: Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), align_items: AlignItems::Center, justify_content: JustifyContent::Center, ..default() }, background_color: BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)), ..default() }, GlobalZIndex(200), ModalOverlay, ActorEditorEntity, ));
    if let Some(camera) = target_camera { cmd.insert(bevy::ui::UiTargetCamera(camera)); }
    cmd.with_children(|p| {
        p.spawn(UiNode { node: Node { width: Val::Px(400.0), padding: UiRect::all(Val::Px(25.0)), flex_direction: FlexDirection::Column, border_radius: BorderRadius::all(Val::Px(12.0)), ..default() }, background_color: BackgroundColor(Color::srgba(0.15, 0.15, 0.15, 1.0)), ..default() }).with_children(|modal| {
            modal.spawn(ui_text(title.to_uppercase(), &font.clone(), 20.0, Color::WHITE));
            modal.spawn(UiNode { node: Node { margin: UiRect::vertical(Val::Px(20.0)), ..default() }, ..default() }).with_children(|m| { m.spawn(ui_text(message, &font.clone(), 15.0, Color::srgb(0.8, 0.8, 0.8))); });
            modal.spawn(UiNode { node: Node { flex_direction: FlexDirection::Row, justify_content: JustifyContent::End, ..default() }, ..default() }).with_children(|btns| {
                btns.spawn(((Button, UiNode { node: Node { padding: UiRect::horizontal(Val::Px(20.0)), height: Val::Px(35.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, margin: UiRect::right(Val::Px(10.0)), border_radius: BorderRadius::all(Val::Px(6.0)), ..default() }, background_color: BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.05)), ..default() }), CancelModalButton, )).with_children(|btn| { btn.spawn(ui_text("CANCEL", &font.clone(), 14.0, Color::WHITE)); });
                btns.spawn(((Button, UiNode { node: Node { padding: UiRect::horizontal(Val::Px(20.0)), height: Val::Px(35.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border_radius: BorderRadius::all(Val::Px(6.0)), ..default() }, background_color: BackgroundColor(Color::srgba(0.8, 0.2, 0.2, 0.8)), ..default() }), ConfirmModalButton(action), )).with_children(|btn| { btn.spawn(ui_text("CONFIRM", &font.clone(), 14.0, Color::WHITE)); });
            });
        });
    });
}

pub fn spawn_save_modal(commands: &mut Commands, font: &Handle<Font>, initial_name: &str, target_camera: Option<Entity>) {
    let mut cmd = commands.spawn((UiNode { node: Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), align_items: AlignItems::Center, justify_content: JustifyContent::Center, ..default() }, background_color: BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)), ..default() }, GlobalZIndex(200), ModalOverlay, ActorEditorEntity, ));
    if let Some(camera) = target_camera { cmd.insert(bevy::ui::UiTargetCamera(camera)); }
    cmd.with_children(|p| {
        p.spawn(UiNode { node: Node { width: Val::Px(400.0), padding: UiRect::all(Val::Px(25.0)), flex_direction: FlexDirection::Column, border_radius: BorderRadius::all(Val::Px(12.0)), ..default() }, background_color: BackgroundColor(Color::srgba(0.15, 0.15, 0.15, 1.0)), ..default() }).with_children(|modal| {
            modal.spawn(ui_text("SAVE PROJECT", &font.clone(), 20.0, Color::WHITE));
            
            modal.spawn(UiNode { node: Node { margin: UiRect::vertical(Val::Px(15.0)), flex_direction: FlexDirection::Column, ..default() }, ..default() }).with_children(|m| { 
                m.spawn(ui_text("Model Name:", &font.clone(), 14.0, Color::srgb(0.6, 0.6, 0.6))); 
                
                m.spawn((
                    ((Button, UiNode { node: Node {
                                width: Val::Percent(100.0),
                                height: Val::Px(28.0),
                                padding: UiRect::horizontal(Val::Px(8.0)),
                                align_items: AlignItems::Center,
                                margin: UiRect::top(Val::Px(5.0)),
                                border_radius: BorderRadius::all(Val::Px(4.0)), ..default()
                            }, background_color: BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.3)), ..default() }), super::text_input::TextInput {
                            value: initial_name.to_string(),
                            placeholder: "Enter model name...".to_string(),
                            ..default()
                        }),
                    super::super::SaveModalInput,
                )).with_children(|p| {
                    p.spawn((
                        ui_text(if initial_name.is_empty() { "Enter model name..." } else { initial_name }, &font.clone(), 13.0, if initial_name.is_empty() { Color::srgb(0.5, 0.5, 0.5) } else { Color::WHITE }),
                        super::text_input::TextInputContent,
                    ));
                });
            });

            // Mark the text input for identification
            // Note: spawn_text_input returns the entity, but we want to mark the TextInput component.
            // Actually, we can just look for the entity with SaveModalInput.
            // I'll manually spawn it to have more control if needed, but let's try to wrap it.
            
            modal.spawn(UiNode { node: Node { flex_direction: FlexDirection::Row, justify_content: JustifyContent::End, margin: UiRect::top(Val::Px(10.0)), ..default() }, ..default() }).with_children(|btns| {
                btns.spawn(((Button, UiNode { node: Node { padding: UiRect::horizontal(Val::Px(20.0)), height: Val::Px(35.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, margin: UiRect::right(Val::Px(10.0)), border_radius: BorderRadius::all(Val::Px(6.0)), ..default() }, background_color: BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.05)), ..default() }), CancelModalButton, )).with_children(|btn| { btn.spawn(ui_text("CANCEL", &font.clone(), 14.0, Color::WHITE)); });
                btns.spawn(((Button, UiNode { node: Node { padding: UiRect::horizontal(Val::Px(20.0)), height: Val::Px(35.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border_radius: BorderRadius::all(Val::Px(6.0)), ..default() }, background_color: BackgroundColor(Color::srgba(0.3, 0.6, 1.0, 0.8)), ..default() }), ConfirmModalButton(EditorAction::SaveProject(initial_name.to_string())), )).with_children(|btn| { btn.spawn(ui_text("SAVE", &font.clone(), 14.0, Color::WHITE)); });
            });
        });
    });
}

#[derive(Component)]
pub struct ProgressBarFill;

#[derive(Component)]
pub struct LoadingOverlay;

#[derive(Component)]
pub struct ProgressBarText;

pub fn spawn_progress_bar(parent: &mut ChildSpawnerCommands, font: &Handle<Font>) {
    parent.spawn(UiNode { node: Node { width: Val::Px(300.0), height: Val::Px(40.0), flex_direction: FlexDirection::Column, justify_content: JustifyContent::Center, ..default() }, ..default() }).with_children(|p| {
        p.spawn(UiNode { node: Node { width: Val::Percent(100.0), height: Val::Px(8.0), border_radius: BorderRadius::all(Val::Px(4.0)), ..default() }, background_color: BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.1)), ..default() }).with_children(|bg| {
            bg.spawn((UiNode { node: Node { width: Val::Percent(0.0), height: Val::Percent(100.0), border_radius: BorderRadius::all(Val::Px(4.0)), ..default() }, background_color: BackgroundColor(Color::srgb(0.3, 0.6, 1.0)), ..default() }, ProgressBarFill, ));
        });
        p.spawn(((ui_text("0%", &font.clone(), 14.0, Color::srgb(0.7, 0.7, 0.7)), Node { margin: UiRect::top(Val::Px(8.0)), align_self: AlignSelf::Center, ..default() }), ProgressBarText, ));
    });
}

pub fn spawn_loading_overlay(commands: &mut Commands, font: &Handle<Font>, target_camera: Option<Entity>) {
    let mut cmd = commands.spawn((UiNode { node: Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), display: Display::None, align_items: AlignItems::Center, justify_content: JustifyContent::Center, flex_direction: FlexDirection::Column, ..default() }, background_color: BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.85)), ..default() }, GlobalZIndex(300), LoadingOverlay, ActorEditorEntity, ));
    if let Some(camera) = target_camera { cmd.insert(bevy::ui::UiTargetCamera(camera)); }
    cmd.with_children(|p| {
        p.spawn((ui_text("IMPORTING MODEL", &font.clone(), 24.0, Color::WHITE), Node { margin: UiRect::bottom(Val::Px(20.0)), ..default() }));
        spawn_progress_bar(p, font);
    });
}
