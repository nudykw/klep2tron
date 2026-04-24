use bevy::prelude::*;
use super::widgets::Tooltip;

#[derive(Component)]
pub struct ProjectPanel;

#[derive(Component)]
pub enum ProjectAction {
    Import,
    Open,
    Save,
}

pub fn setup_project_panel(
    parent: &mut ChildBuilder,
    font: &Handle<Font>,
    icon_font: &Handle<Font>,
) {
    parent.spawn((
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                margin: UiRect::top(Val::Px(20.0)),
                flex_shrink: 0.0,
                ..default()
            },
            ..default()
        },
        ProjectPanel,
    )).with_children(|p| {
        spawn_button(p, font, icon_font, "IMPORT", "\u{f093}", "[Ctrl+I]", "Import external model", ProjectAction::Import);
        spawn_button(p, font, icon_font, "OPEN", "\u{f07c}", "[Ctrl+O]", "Open existing actor project", ProjectAction::Open);
        spawn_button(p, font, icon_font, "SAVE", "\u{f0c7}", "[Ctrl+S]", "Save current actor", ProjectAction::Save);

        // Mode Switch
        p.spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Px(40.0),
                margin: UiRect::top(Val::Px(30.0)),
                flex_direction: FlexDirection::Row,
                ..default()
            },
            ..default()
        }).with_children(|row| {
            spawn_mode_tab(row, font, "SLICING", true);
            spawn_mode_tab(row, font, "SOCKETS", false);
        });
    });
}

fn spawn_button(
    parent: &mut ChildBuilder,
    font: &Handle<Font>,
    icon_font: &Handle<Font>,
    text: &str,
    icon: &str,
    hint: &str,
    tooltip: &str,
    action: ProjectAction,
) {
    parent.spawn((
        ButtonBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Px(45.0),
                margin: UiRect::bottom(Val::Px(10.0)),
                padding: UiRect::horizontal(Val::Px(15.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            background_color: Color::srgba(1.0, 1.0, 1.0, 0.05).into(),
            border_radius: BorderRadius::all(Val::Px(8.0)),
            ..default()
        },
        action,
        Tooltip(tooltip.to_string()),
    )).with_children(|p| {
        p.spawn(NodeBundle {
            style: Style {
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        }).with_children(|left| {
            left.spawn(TextBundle::from_section(
                icon,
                TextStyle { font: icon_font.clone(), font_size: 18.0, color: Color::srgb(0.3, 0.6, 1.0) },
            ));
            left.spawn(NodeBundle {
                style: Style { width: Val::Px(10.0), ..default() },
                ..default()
            });
            left.spawn(TextBundle::from_section(
                text,
                TextStyle { font: font.clone(), font_size: 16.0, color: Color::WHITE },
            ));
        });
        
        p.spawn(TextBundle::from_section(
            hint,
            TextStyle { font: font.clone(), font_size: 12.0, color: Color::srgb(0.5, 0.5, 0.5) },
        ));
    });
}

fn spawn_mode_tab(
    parent: &mut ChildBuilder,
    font: &Handle<Font>,
    text: &str,
    is_active: bool,
) {
    parent.spawn(ButtonBundle {
        style: Style {
            flex_grow: 1.0,
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        background_color: if is_active { Color::srgba(0.3, 0.5, 1.0, 0.3).into() } else { Color::srgba(1.0, 1.0, 1.0, 0.05).into() },
        ..default()
    }).with_children(|p| {
        p.spawn(TextBundle::from_section(
            text,
            TextStyle { font: font.clone(), font_size: 14.0, color: Color::WHITE },
        ));
    });
}
