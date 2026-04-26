use bevy::prelude::*;
use super::widgets::{Tooltip, ScrollingList};
use super::ActorPart;
use super::ui::inspector::types::*;

#[derive(Component)]
pub struct ProjectPanel;

#[derive(Component)]
pub enum ProjectAction {
    Import,
    Open,
    Save,
}

#[derive(Component)]
pub struct ModeTab(pub super::EditorMode);

#[derive(Component)]
pub struct ProjectModeContent(pub super::EditorMode);

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
            spawn_mode_tab(row, font, "SLICING", super::EditorMode::Slicing);
            spawn_mode_tab(row, font, "SOCKETS", super::EditorMode::Sockets);
        });

        // Content Area
        p.spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                margin: UiRect::top(Val::Px(15.0)),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ..default()
        }).with_children(|content| {
            // SLICING CONTENT
            content.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        display: Display::Flex, // Default
                        ..default()
                    },
                    ..default()
                },
                ProjectModeContent(super::EditorMode::Slicing),
            )).with_children(|slicing| {
                spawn_slicing_precision_ui(slicing, font, icon_font);
            });

            // SOCKETS CONTENT
            content.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        display: Display::None, // Hidden by default
                        ..default()
                    },
                    ..default()
                },
                ProjectModeContent(super::EditorMode::Sockets),
            )).with_children(|sockets_content| {
                // --- SEARCH & ADD ---
                sockets_content.spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(5.0),
                        margin: UiRect::bottom(Val::Px(10.0)),
                        ..default()
                    },
                    ..default()
                }).with_children(|row| {
                    row.spawn((
                        ButtonBundle {
                            style: Style {
                                width: Val::Px(30.0),
                                height: Val::Px(30.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            background_color: Color::srgba(1.0, 1.0, 1.0, 0.05).into(),
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                            ..default()
                        },
                        SocketAddModeButton,
                        Tooltip("Toggle Socket Placement Mode".to_string()),
                    )).with_children(|b| {
                        b.spawn(TextBundle::from_section(
                            "\u{f067}", // plus icon
                            TextStyle { font: icon_font.clone(), font_size: 14.0, color: Color::WHITE },
                        ));
                    });

                    row.spawn((
                        crate::actor_editor::widgets::TextInputBundle {
                            button: ButtonBundle {
                                style: Style {
                                    flex_grow: 1.0,
                                    height: Val::Px(30.0),
                                    padding: UiRect::horizontal(Val::Px(10.0)),
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                background_color: Color::srgba(1.0, 1.0, 1.0, 0.05).into(),
                                border_radius: BorderRadius::all(Val::Px(4.0)),
                                z_index: ZIndex::Global(100),
                                ..default()
                            },
                            input: crate::actor_editor::widgets::TextInput {
                                placeholder: "Search sockets...".to_string(),
                                ..default()
                            },
                        },
                        SocketSearchInput,
                        bevy_mod_picking::prelude::PickableBundle::default(),
                    )).with_children(|search| {
                        search.spawn((
                            TextBundle {
                                text: Text::from_section(
                                    "\u{f002} ", // fa-search
                                    TextStyle { font: icon_font.clone(), font_size: 14.0, color: Color::srgb(0.5, 0.5, 0.5) },
                                ),
                                focus_policy: bevy::ui::FocusPolicy::Pass,
                                ..default()
                            },
                        ));
                        search.spawn((
                            TextBundle {
                                text: Text::from_section(
                                    "Search sockets...",
                                    TextStyle { font: font.clone(), font_size: 14.0, color: Color::srgb(0.5, 0.5, 0.5) },
                                ),
                                focus_policy: bevy::ui::FocusPolicy::Pass,
                                ..default()
                            },
                            crate::actor_editor::widgets::TextInputContent,
                        ));
                    });
                });

                // --- FILTERS ---
                sockets_content.spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        margin: UiRect::vertical(Val::Px(5.0)),
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(4.0),
                        ..default()
                    },
                    ..default()
                }).with_children(|btns| {
                    for (part, label) in [
                        (None, "ALL"),
                        (Some(ActorPart::Head), "HEAD"),
                        (Some(ActorPart::Body), "BODY"),
                        (Some(ActorPart::Engine), "ENG"),
                    ] {
                        btns.spawn((
                            ButtonBundle {
                                style: Style {
                                    flex_grow: 1.0,
                                    height: Val::Px(20.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                background_color: Color::srgba(1.0, 1.0, 1.0, 0.05).into(),
                                border_radius: BorderRadius::all(Val::Px(2.0)),
                                ..default()
                            },
                            SocketPartFilterButton(part),
                        )).with_children(|b| {
                            b.spawn(TextBundle::from_section(
                                label,
                                TextStyle { font: font.clone(), font_size: 9.0, color: Color::srgb(0.7, 0.7, 0.7) },
                            ));
                        });
                    }
                });

                // --- LIST ---
                sockets_content.spawn((
                    NodeBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            flex_direction: FlexDirection::Column,
                            max_height: Val::Px(500.0), // Increased for hierarchy panel
                            ..default()
                        },
                        ..default()
                    },
                    SocketListContainer,
                    ScrollingList::default(),
                ));
            });
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
    mode: super::EditorMode,
) {
    parent.spawn((
        ButtonBundle {
            style: Style {
                flex_grow: 1.0,
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            background_color: Color::srgba(1.0, 1.0, 1.0, 0.05).into(),
            ..default()
        },
        ModeTab(mode),
    )).with_children(|p| {
        p.spawn(TextBundle::from_section(
            text,
            TextStyle { font: font.clone(), font_size: 14.0, color: Color::WHITE },
        ));
    });
}

fn spawn_slicing_precision_ui(
    parent: &mut ChildBuilder,
    font: &Handle<Font>,
    _icon_font: &Handle<Font>,
) {
    // Mode Toggle: AUTO / MANUAL
    parent.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Px(30.0),
            margin: UiRect::vertical(Val::Px(10.0)),
            flex_direction: FlexDirection::Row,
            ..default()
        },
        ..default()
    }).with_children(|row| {
        for (label, is_manual) in [("AUTO", false), ("MANUAL", true)] {
            row.spawn((
                ButtonBundle {
                    style: Style {
                        flex_grow: 1.0,
                        height: Val::Percent(100.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: Color::srgba(1.0, 1.0, 1.0, 0.05).into(),
                    border_radius: if is_manual { BorderRadius::right(Val::Px(4.0)) } else { BorderRadius::left(Val::Px(4.0)) },
                    ..default()
                },
                super::SlicingAutoManualToggle,
                bevy_mod_picking::prelude::PickableBundle::default(),
            )).insert(InteractionState { is_active: false }) // We'll use this for visual state
            .with_children(|b| {
                b.spawn(TextBundle::from_section(
                    label,
                    TextStyle { font: font.clone(), font_size: 11.0, color: Color::WHITE },
                ));
            });
        }
    });

    // Auto Mode Container
    parent.spawn((
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            background_color: Color::srgba(0.0, 0.0, 0.0, 0.2).into(),
            border_radius: BorderRadius::all(Val::Px(6.0)),
            ..default()
        },
        super::SlicingAutoModeContainer,
    )).with_children(|container| {
        // TOP CUT
        container.spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        }).with_children(|row| {
            row.spawn(TextBundle::from_section(
                "Top Cut:",
                TextStyle { font: font.clone(), font_size: 13.0, color: Color::srgb(0.7, 0.7, 0.7) },
            ));
            
            row.spawn((
                super::widgets::TextInputBundle {
                    button: ButtonBundle {
                        style: Style {
                            width: Val::Px(60.0),
                            height: Val::Px(28.0),
                            padding: UiRect::horizontal(Val::Px(8.0)),
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        background_color: Color::srgba(0.0, 0.0, 0.0, 0.3).into(),
                        border_radius: BorderRadius::all(Val::Px(4.0)),
                        ..default()
                    },
                    input: super::widgets::TextInput {
                        value: "1.000".to_string(),
                        placeholder: "1.000".to_string(),
                        ..default()
                    },
                },
                super::SlicingTopCutInput,
                super::widgets::Tooltip("Top slicing plane position. Use UP/DOWN arrows to nudge by 0.001".to_string()),
            )).with_children(|p| {
                p.spawn((
                    TextBundle::from_section(
                        "1.000",
                        TextStyle {
                            font: font.clone(),
                            font_size: 13.0,
                            color: Color::WHITE,
                        },
                    ),
                    super::widgets::TextInputContent,
                ));
            });
        });

        // BOTTOM CUT
        container.spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        }).with_children(|row| {
            row.spawn(TextBundle::from_section(
                "Bottom Cut:",
                TextStyle { font: font.clone(), font_size: 13.0, color: Color::srgb(0.7, 0.7, 0.7) },
            ));
            
            row.spawn((
                super::widgets::TextInputBundle {
                    button: ButtonBundle {
                        style: Style {
                            width: Val::Px(60.0),
                            height: Val::Px(28.0),
                            padding: UiRect::horizontal(Val::Px(8.0)),
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        background_color: Color::srgba(0.0, 0.0, 0.0, 0.3).into(),
                        border_radius: BorderRadius::all(Val::Px(4.0)),
                        ..default()
                    },
                    input: super::widgets::TextInput {
                        value: "0.000".to_string(),
                        placeholder: "0.000".to_string(),
                        ..default()
                    },
                },
                super::SlicingBottomCutInput,
                super::widgets::Tooltip("Bottom slicing plane position. Use UP/DOWN arrows to nudge by 0.001".to_string()),
            )).with_children(|p| {
                p.spawn((
                    TextBundle::from_section(
                        "0.000",
                        TextStyle {
                            font: font.clone(),
                            font_size: 13.0,
                            color: Color::WHITE,
                        },
                    ),
                    super::widgets::TextInputContent,
                ));
            });
        });

        // RIM THICKNESS
        container.spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                margin: UiRect::top(Val::Px(5.0)),
                ..default()
            },
            ..default()
        }).with_children(|row| {
            row.spawn(TextBundle::from_section(
                "Rim:",
                TextStyle { font: font.clone(), font_size: 13.0, color: Color::srgb(0.7, 0.7, 0.7) },
            ));
            
            row.spawn(NodeBundle {
                style: Style { width: Val::Px(100.0), ..default() },
                ..default()
            }).with_children(|slider_p| {
                super::widgets::spawn_slider_ext(
                    slider_p,
                    0.0,
                    1.0,
                    0.0,
                    (
                        super::SlicingRimThicknessSlider,
                        super::widgets::Tooltip("Left: Solid, Right: Hollow, Center: Rim Thickness".to_string()),
                    ),
                );
            });
        });
    });
}
