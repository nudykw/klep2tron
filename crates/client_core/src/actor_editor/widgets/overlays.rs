use bevy::prelude::*;
use super::super::{ActorEditorEntity, ToastType, EditorAction};

#[derive(Component)]
pub struct ToastContainer;

#[derive(Component)]
pub struct ToastTimer(pub Timer);

pub fn spawn_toast_container(commands: &mut Commands, target_camera: Option<Entity>) -> Entity {
    let mut cmd = commands.spawn((NodeBundle { style: Style { position_type: PositionType::Absolute, bottom: Val::Px(40.0), right: Val::Px(20.0), flex_direction: FlexDirection::ColumnReverse, align_items: AlignItems::End, ..default() }, z_index: ZIndex::Global(110), ..default() }, ToastContainer, ActorEditorEntity, ));
    if let Some(camera) = target_camera { cmd.insert(bevy::ui::TargetCamera(camera)); }
    cmd.id()
}

pub fn spawn_toast_item(parent: &mut ChildBuilder, font: &Handle<Font>, icon_font: &Handle<Font>, message: &str, toast_type: ToastType) {
    let (icon, color) = match toast_type { ToastType::Info => ("\u{f05a}", Color::srgb(0.3, 0.6, 1.0)), ToastType::Success => ("\u{f058}", Color::srgb(0.3, 0.8, 0.3)), ToastType::Error => ("\u{f071}", Color::srgb(0.8, 0.3, 0.3)), };
    parent.spawn((NodeBundle { style: Style { width: Val::Px(250.0), margin: UiRect::bottom(Val::Px(10.0)), padding: UiRect::all(Val::Px(12.0)), flex_direction: FlexDirection::Row, align_items: AlignItems::Center, ..default() }, background_color: Color::srgba(0.1, 0.1, 0.1, 0.95).into(), border_radius: BorderRadius::all(Val::Px(8.0)), border_color: color.with_alpha(0.3).into(), ..default() }, ToastTimer(Timer::from_seconds(4.0, TimerMode::Once)), )).with_children(|p| {
        p.spawn(TextBundle::from_section(format!("{} ", icon), TextStyle { font: icon_font.clone(), font_size: 18.0, color }));
        p.spawn(TextBundle::from_section(message, TextStyle { font: font.clone(), font_size: 14.0, color: Color::WHITE }));
    });
}

#[derive(Component)]
pub struct ModalOverlay;

#[derive(Component)]
pub struct ConfirmModalButton(pub EditorAction);

#[derive(Component)]
pub struct CancelModalButton;

pub fn spawn_confirmation_modal(commands: &mut Commands, font: &Handle<Font>, _icon_font: &Handle<Font>, title: &str, message: &str, action: EditorAction, target_camera: Option<Entity>) {
    let mut cmd = commands.spawn((NodeBundle { style: Style { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), align_items: AlignItems::Center, justify_content: JustifyContent::Center, ..default() }, background_color: Color::srgba(0.0, 0.0, 0.0, 0.6).into(), z_index: ZIndex::Global(200), ..default() }, ModalOverlay, ActorEditorEntity, ));
    if let Some(camera) = target_camera { cmd.insert(bevy::ui::TargetCamera(camera)); }
    cmd.with_children(|p| {
        p.spawn(NodeBundle { style: Style { width: Val::Px(400.0), padding: UiRect::all(Val::Px(25.0)), flex_direction: FlexDirection::Column, ..default() }, background_color: Color::srgba(0.15, 0.15, 0.15, 1.0).into(), border_radius: BorderRadius::all(Val::Px(12.0)), ..default() }).with_children(|modal| {
            modal.spawn(TextBundle::from_section(title.to_uppercase(), TextStyle { font: font.clone(), font_size: 20.0, color: Color::WHITE }));
            modal.spawn(NodeBundle { style: Style { margin: UiRect::vertical(Val::Px(20.0)), ..default() }, ..default() }).with_children(|m| { m.spawn(TextBundle::from_section(message, TextStyle { font: font.clone(), font_size: 15.0, color: Color::srgb(0.8, 0.8, 0.8) })); });
            modal.spawn(NodeBundle { style: Style { flex_direction: FlexDirection::Row, justify_content: JustifyContent::End, ..default() }, ..default() }).with_children(|btns| {
                btns.spawn((ButtonBundle { style: Style { padding: UiRect::horizontal(Val::Px(20.0)), height: Val::Px(35.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, margin: UiRect::right(Val::Px(10.0)), ..default() }, background_color: Color::srgba(1.0, 1.0, 1.0, 0.05).into(), border_radius: BorderRadius::all(Val::Px(6.0)), ..default() }, CancelModalButton, )).with_children(|btn| { btn.spawn(TextBundle::from_section("CANCEL", TextStyle { font: font.clone(), font_size: 14.0, color: Color::WHITE })); });
                btns.spawn((ButtonBundle { style: Style { padding: UiRect::horizontal(Val::Px(20.0)), height: Val::Px(35.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, background_color: Color::srgba(0.8, 0.2, 0.2, 0.8).into(), border_radius: BorderRadius::all(Val::Px(6.0)), ..default() }, ConfirmModalButton(action), )).with_children(|btn| { btn.spawn(TextBundle::from_section("CONFIRM", TextStyle { font: font.clone(), font_size: 14.0, color: Color::WHITE })); });
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

pub fn spawn_progress_bar(parent: &mut ChildBuilder, font: &Handle<Font>) {
    parent.spawn(NodeBundle { style: Style { width: Val::Px(300.0), height: Val::Px(40.0), flex_direction: FlexDirection::Column, justify_content: JustifyContent::Center, ..default() }, ..default() }).with_children(|p| {
        p.spawn(NodeBundle { style: Style { width: Val::Percent(100.0), height: Val::Px(8.0), ..default() }, background_color: Color::srgba(1.0, 1.0, 1.0, 0.1).into(), border_radius: BorderRadius::all(Val::Px(4.0)), ..default() }).with_children(|bg| {
            bg.spawn((NodeBundle { style: Style { width: Val::Percent(0.0), height: Val::Percent(100.0), ..default() }, background_color: Color::srgb(0.3, 0.6, 1.0).into(), border_radius: BorderRadius::all(Val::Px(4.0)), ..default() }, ProgressBarFill, ));
        });
        p.spawn((TextBundle::from_section("0%", TextStyle { font: font.clone(), font_size: 14.0, color: Color::srgb(0.7, 0.7, 0.7) }).with_style(Style { margin: UiRect::top(Val::Px(8.0)), align_self: AlignSelf::Center, ..default() }), ProgressBarText, ));
    });
}

pub fn spawn_loading_overlay(commands: &mut Commands, font: &Handle<Font>, target_camera: Option<Entity>) {
    let mut cmd = commands.spawn((NodeBundle { style: Style { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), display: Display::None, align_items: AlignItems::Center, justify_content: JustifyContent::Center, flex_direction: FlexDirection::Column, ..default() }, background_color: Color::srgba(0.0, 0.0, 0.0, 0.85).into(), z_index: ZIndex::Global(300), ..default() }, LoadingOverlay, ActorEditorEntity, ));
    if let Some(camera) = target_camera { cmd.insert(bevy::ui::TargetCamera(camera)); }
    cmd.with_children(|p| {
        p.spawn(TextBundle::from_section("IMPORTING MODEL", TextStyle { font: font.clone(), font_size: 24.0, color: Color::WHITE }).with_style(Style { margin: UiRect::bottom(Val::Px(20.0)), ..default() }));
        spawn_progress_bar(p, font);
    });
}
