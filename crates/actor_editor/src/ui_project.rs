use bevy::prelude::*;
use client_core::ui::widgets::*;
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
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    icon_font: &Handle<Font>,
) {
    parent.spawn((
        UiNode { node: Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                margin: UiRect::top(Val::Px(20.0)),
                flex_shrink: 0.0,
                ..default()
            }, ..default() },
        ProjectPanel,
    )).with_children(|p| {
        spawn_button(p, font, icon_font, "IMPORT", "\u{f093}", "[Ctrl+I]", "Import external model", ProjectAction::Import);
        spawn_button(p, font, icon_font, "OPEN", "\u{f07c}", "[Ctrl+O]", "Open existing actor project", ProjectAction::Open);
        spawn_button(p, font, icon_font, "SAVE", "\u{f0c7}", "[Ctrl+S]", "Save current actor", ProjectAction::Save);

        // Mode Switch
        p.spawn(UiNode { node: Node {
                width: Val::Percent(100.0),
                height: Val::Px(40.0),
                margin: UiRect::top(Val::Px(30.0)),
                flex_direction: FlexDirection::Row,
                ..default()
            }, ..default() }).with_children(|row| {
            spawn_mode_tab(row, font, "SLICING", super::EditorMode::Slicing);
            spawn_mode_tab(row, font, "SOCKETS", super::EditorMode::Sockets);
        });

        // Content Area
        p.spawn(UiNode { node: Node {
                width: Val::Percent(100.0),
                margin: UiRect::top(Val::Px(15.0)),
                flex_direction: FlexDirection::Column,
                ..default()
            }, ..default() }).with_children(|content| {
            // SLICING CONTENT
            content.spawn((
                UiNode { node: Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        display: Display::Flex, // Default
                        ..default()
                    }, ..default() },
                ProjectModeContent(super::EditorMode::Slicing),
            )).with_children(|slicing| {
                spawn_slicing_precision_ui(slicing, font, icon_font);
            });

            // SOCKETS CONTENT
            content.spawn((
                UiNode { node: Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        display: Display::None, // Hidden by default
                        ..default()
                    }, ..default() },
                ProjectModeContent(super::EditorMode::Sockets),
            )).with_children(|sockets_content| {
                // --- SEARCH & ADD ---
                sockets_content.spawn(UiNode { node: Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(5.0),
                        margin: UiRect::bottom(Val::Px(10.0)),
                        ..default()
                    }, ..default() }).with_children(|row| {
                    row.spawn((
                        (Button, UiNode { node: Node {
                                width: Val::Px(30.0),
                                height: Val::Px(30.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border_radius: BorderRadius::all(Val::Px(4.0)), ..default()
                            }, background_color: BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.05)), ..default() }),
                        SocketAddModeButton,
                        Tooltip("Toggle Socket Placement Mode".to_string()),
                    )).with_children(|b| {
                        b.spawn(ui_text("\u{f067}", &icon_font.clone(), 14.0, Color::WHITE));
                    });

                    row.spawn((
                        crate::widgets::TextInputBundle {
                            button: (Button, UiNode { node: Node {
                                    flex_grow: 1.0,
                                    height: Val::Px(30.0),
                                    padding: UiRect::horizontal(Val::Px(10.0)),
                                    align_items: AlignItems::Center,
                                    border_radius: BorderRadius::all(Val::Px(4.0)), ..default()
                                }, background_color: BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.05)), global_z_index: GlobalZIndex(100), ..default() }),
                            input: crate::widgets::TextInput {
                                placeholder: "Search sockets...".to_string(),
                                ..default()
                            },
                        },
                        SocketSearchInput,
                        Pickable::default(),
                    )).with_children(|search| {
                        search.spawn((
                            ui_text("\u{f002} ", &icon_font.clone(), 14.0, Color::srgb(0.5, 0.5, 0.5)),
                        ));
                        search.spawn((
                            ui_text("Search sockets...", &font.clone(), 14.0, Color::srgb(0.5, 0.5, 0.5)),
                            crate::widgets::TextInputContent,
                        ));
                    });
                });

                // --- FILTERS ---
                sockets_content.spawn(UiNode { node: Node {
                        width: Val::Percent(100.0),
                        margin: UiRect::vertical(Val::Px(5.0)),
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(4.0),
                        ..default()
                    }, ..default() }).with_children(|btns| {
                    for (part, label) in [
                        (None, "ALL"),
                        (Some(ActorPart::Head), "HEAD"),
                        (Some(ActorPart::Body), "BODY"),
                        (Some(ActorPart::Engine), "ENG"),
                    ] {
                        btns.spawn((
                            (Button, UiNode { node: Node {
                                    flex_grow: 1.0,
                                    height: Val::Px(20.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    border_radius: BorderRadius::all(Val::Px(2.0)), ..default()
                                }, background_color: BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.05)), ..default() }),
                            SocketPartFilterButton(part),
                        )).with_children(|b| {
                            b.spawn(ui_text(label, &font.clone(), 9.0, Color::srgb(0.7, 0.7, 0.7)));
                        });
                    }
                });

                // --- LIST ---
                sockets_content.spawn((
                    UiNode { node: Node {
                            width: Val::Percent(100.0),
                            flex_direction: FlexDirection::Column,
                            max_height: Val::Px(500.0), // Increased for hierarchy panel
                            ..default()
                        }, ..default() },
                    SocketListContainer,
                    ScrollingList::default(),
                ));
            });
        });
    });
}

fn spawn_button(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    icon_font: &Handle<Font>,
    text: &str,
    icon: &str,
    hint: &str,
    tooltip: &str,
    action: ProjectAction,
) {
    parent.spawn((
        (Button, UiNode { node: Node {
                width: Val::Percent(100.0),
                height: Val::Px(45.0),
                margin: UiRect::bottom(Val::Px(10.0)),
                padding: UiRect::horizontal(Val::Px(15.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                border_radius: BorderRadius::all(Val::Px(8.0)), ..default()
            }, background_color: BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.05)), ..default() }),
        action,
        Tooltip(tooltip.to_string()),
    )).with_children(|p| {
        p.spawn(UiNode { node: Node {
                align_items: AlignItems::Center,
                ..default()
            }, ..default() }).with_children(|left| {
            left.spawn(ui_text(icon, &icon_font.clone(), 18.0, Color::srgb(0.3, 0.6, 1.0)));
            left.spawn(UiNode { node: Node { width: Val::Px(10.0), ..default() }, ..default() });
            left.spawn(ui_text(text, &font.clone(), 16.0, Color::WHITE));
        });
        
        p.spawn(ui_text(hint, &font.clone(), 12.0, Color::srgb(0.5, 0.5, 0.5)));
    });
}

fn spawn_mode_tab(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    text: &str,
    mode: super::EditorMode,
) {
    parent.spawn((
        (Button, UiNode { node: Node {
                flex_grow: 1.0,
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            }, background_color: BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.05)), ..default() }),
        ModeTab(mode),
    )).with_children(|p| {
        p.spawn(ui_text(text, &font.clone(), 14.0, Color::WHITE));
    });
}

fn spawn_slicing_precision_ui(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    _icon_font: &Handle<Font>,
) {
    // Mode Toggle: AUTO / MANUAL
    parent.spawn(UiNode { node: Node {
            width: Val::Percent(100.0),
            height: Val::Px(30.0),
            margin: UiRect::vertical(Val::Px(10.0)),
            flex_direction: FlexDirection::Row,
            ..default()
        }, ..default() }).with_children(|row| {
        for (label, is_manual) in [("AUTO", false), ("MANUAL", true)] {
            row.spawn((
                (Button, UiNode { node: Node {
                        flex_grow: 1.0,
                        height: Val::Percent(100.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border_radius: if is_manual { BorderRadius::right(Val::Px(4.0)) } else { BorderRadius::left(Val::Px(4.0)) }, ..default()
                    }, background_color: BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.05)), ..default() }),
                super::SlicingAutoManualToggle,
                Pickable::default(),
            )).insert(InteractionState { is_active: false }) // We'll use this for visual state
            .with_children(|b| {
                b.spawn(ui_text(label, &font.clone(), 11.0, Color::WHITE));
            });
        }
    });

    // Auto Mode Container
    parent.spawn((
        UiNode { node: Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            }, background_color: BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.2)), border_radius: BorderRadius::all(Val::Px(6.0)), ..default() },
        super::SlicingAutoModeContainer,
    )).with_children(|container| {
        // TOP CUT
        container.spawn(UiNode { node: Node {
                width: Val::Percent(100.0),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                ..default()
            }, ..default() }).with_children(|row| {
            row.spawn(ui_text("Top Cut:", &font.clone(), 13.0, Color::srgb(0.7, 0.7, 0.7)));
            
            row.spawn((
                super::widgets::TextInputBundle {
                    button: (Button, UiNode { node: Node {
                            width: Val::Px(60.0),
                            height: Val::Px(28.0),
                            padding: UiRect::horizontal(Val::Px(8.0)),
                            align_items: AlignItems::Center,
                            border_radius: BorderRadius::all(Val::Px(4.0)), ..default()
                        }, background_color: BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.3)), ..default() }),
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
                    ui_text("1.000", &font.clone(), 13.0, Color::WHITE),
                    super::widgets::TextInputContent,
                ));
            });
        });

        // BOTTOM CUT
        container.spawn(UiNode { node: Node {
                width: Val::Percent(100.0),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                ..default()
            }, ..default() }).with_children(|row| {
            row.spawn(ui_text("Bottom Cut:", &font.clone(), 13.0, Color::srgb(0.7, 0.7, 0.7)));
            
            row.spawn((
                super::widgets::TextInputBundle {
                    button: (Button, UiNode { node: Node {
                            width: Val::Px(60.0),
                            height: Val::Px(28.0),
                            padding: UiRect::horizontal(Val::Px(8.0)),
                            align_items: AlignItems::Center,
                            border_radius: BorderRadius::all(Val::Px(4.0)), ..default()
                        }, background_color: BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.3)), ..default() }),
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
                    ui_text("0.000", &font.clone(), 13.0, Color::WHITE),
                    super::widgets::TextInputContent,
                ));
            });
        });

        // RIM THICKNESS
        container.spawn(UiNode { node: Node {
                width: Val::Percent(100.0),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                margin: UiRect::top(Val::Px(5.0)),
                ..default()
            }, ..default() }).with_children(|row| {
            row.spawn(ui_text("Rim:", &font.clone(), 13.0, Color::srgb(0.7, 0.7, 0.7)));
            
            row.spawn(UiNode { node: Node { width: Val::Px(100.0), ..default() }, ..default() }).with_children(|slider_p| {
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

    // Manual Mode Container
    parent.spawn((
        UiNode { node: Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                padding: UiRect::all(Val::Px(10.0)),
                display: Display::None, // Hidden by default
                ..default()
            }, background_color: BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.2)), border_radius: BorderRadius::all(Val::Px(6.0)), ..default() },
        super::SlicingManualModeContainer,
    )).with_children(|container| {
        // Selection Counter
        container.spawn(UiNode { node: Node {
                width: Val::Percent(100.0),
                height: Val::Px(25.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            }, background_color: BackgroundColor(Color::srgba(0.0, 1.0, 1.0, 0.1)), border_radius: BorderRadius::all(Val::Px(4.0)), ..default() }).with_children(|row| {
            row.spawn((
                ui_text("Selected: 0 triangles", &font.clone(), 12.0, Color::srgb(0.0, 1.0, 1.0)),
                crate::TriangleSelectionCounter,
            ));
        });
        
        container.spawn(ui_text("Use Left Mouse Button to lasso triangles.\nHold Alt to subtract from selection.", &font.clone(), 11.0, Color::srgb(0.6, 0.6, 0.6)));

        // ── Target Part Selector ───────────────────────────────────────────
        container.spawn(UiNode { node: Node {
                width: Val::Percent(100.0),
                margin: UiRect::top(Val::Px(10.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                ..default()
            }, ..default() }).with_children(|col| {
            col.spawn(ui_text("Move to Part:", &font.clone(), 11.0, Color::srgb(0.6, 0.6, 0.6)));

            col.spawn(UiNode { node: Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(28.0),
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(3.0),
                    ..default()
                }, ..default() }).with_children(|row| {
                for (part, label, tooltip) in [
                    (ActorPart::Head,   "HEAD", "Move selected triangles to Head part"),
                    (ActorPart::Body,   "BODY", "Move selected triangles to Body part"),
                    (ActorPart::Engine, "LEGS", "Move selected triangles to Legs part"),
                ] {
                    row.spawn((
                        (Button, UiNode { node: Node {
                                flex_grow: 1.0,
                                height: Val::Percent(100.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border: UiRect::all(Val::Px(1.0)),
                                border_radius: BorderRadius::all(Val::Px(3.0)), ..default()
                            }, background_color: BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.05)), border_color: BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.1)), ..default() }),
                        super::TargetPartButton(part),
                        super::widgets::Tooltip(tooltip.to_string()),
                        Pickable::default(),
                    )).with_children(|b| {
                        b.spawn(ui_text(label, &font.clone(), 10.0, Color::WHITE));
                    });
                }
            });
        });
    });

}
