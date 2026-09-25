use bevy::prelude::*;
use client_core::ui::widgets::*;
use crate::{
    widgets::{spawn_collapsible_section, Tooltip},
    SocketColorPicker, SocketColorPickerContainer, SocketColorHueSlider, SocketColorPreset
};
use super::types::*;

pub fn spawn_sockets_section(
    p: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    icon_font: &Handle<Font>,
    vfx_presets: &crate::vfx_assets::VfxPresets,
    vfx_registry: &crate::vfx_assets::VfxRegistry,
) {
    spawn_collapsible_section(
        p,
        font,
        icon_font,
        "SOCKETS",
        false,
        SocketsSectionMarker,
        |content| {
            // --- POSITION DISPLAY ---
            content.spawn(UiNode { node: Node {
                    width: Val::Percent(100.0),
                    margin: UiRect::top(Val::Px(10.0)),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                }, ..default() }).with_children(|row| {
                for (axis, label) in [(TransformAxis::X, "X"), (TransformAxis::Y, "Y"), (TransformAxis::Z, "Z")] {
                    row.spawn((
                        UiNode { node: Node {
                                width: Val::Px(75.0),
                                height: Val::Px(25.0),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                ..default()
                            }, background_color: BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.3)), border_radius: BorderRadius::all(Val::Px(4.0)), ..default() },
                        axis,
                    )).with_children(|box_| {
                        box_.spawn(ui_text(format!("{}: {:.2}", label, 0.0), &font.clone(), 11.0, Color::WHITE));
                    });
                }
            });

            // --- ROTATION DISPLAY ---
            content.spawn(UiNode { node: Node {
                    width: Val::Percent(100.0),
                    margin: UiRect::top(Val::Px(5.0)),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                }, ..default() }).with_children(|row| {
                for (axis, label) in [(RotationAxis::Roll, "R"), (RotationAxis::Pitch, "P"), (RotationAxis::Yaw, "Y")] {
                    row.spawn((
                        UiNode { node: Node {
                                width: Val::Px(75.0),
                                height: Val::Px(25.0),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                ..default()
                            }, background_color: BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.4)), border_radius: BorderRadius::all(Val::Px(4.0)), ..default() },
                        axis,
                    )).with_children(|box_| {
                        box_.spawn(ui_text(format!("{}: {:.1}°", label, 0.0), &font.clone(), 11.0, Color::srgb(0.8, 0.8, 1.0)));
                    });
                }
            });

            // --- RESET BUTTON ---
            content.spawn((
                ButtonBundle {
                    style: Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(25.0),
                        margin: UiRect::top(Val::Px(10.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: Color::srgba(1.0, 1.0, 1.0, 0.05).into(),
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    ..default()
                },
                SocketResetRotationButton,
            )).with_children(|b| {
                b.spawn(ui_text("Reset Rotation", &font.clone(), 12.0, Color::srgb(0.7, 0.7, 0.7)));
            });

            // --- SOCKET DETAILS (Name & Comment) ---
            content.spawn((
                UiNode { node: Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        margin: UiRect::top(Val::Px(15.0)),
                        row_gap: Val::Px(8.0),
                        display: Display::None, // Hidden by default
                        ..default()
                    }, ..default() },
                SocketDetailsContainer,
            )).with_children(|details| {
                // ... (details content remains same, but we will add logic to hide it in systems)
                details.spawn(UiNode { node: Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(1.0),
                        margin: UiRect::vertical(Val::Px(5.0)),
                        ..default()
                    }, background_color: BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.1)), ..default() });

                details.spawn((
                    UiNode { node: Node {
                            width: Val::Percent(100.0),
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(8.0),
                            ..default()
                        }, ..default() },
                    SocketMetadataSection,
                )).with_children(|meta| {
                    meta.spawn(ui_text("Socket Name", &font.clone(), 12.0, Color::srgb(0.6, 0.6, 0.6)));
                    
                    meta.spawn((
                        crate::widgets::TextInputBundle {
                            button: ButtonBundle {
                                style: Node {
                                    width: Val::Percent(100.0),
                                    height: Val::Px(28.0),
                                    padding: UiRect::horizontal(Val::Px(8.0)),
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                background_color: Color::srgba(0.0, 0.0, 0.0, 0.3).into(),
                                border_radius: BorderRadius::all(Val::Px(4.0)),
                                ..default()
                            },
                            input: crate::widgets::TextInput {
                                placeholder: "Socket name...".to_string(),
                                ..default()
                            },
                        },
                        SocketNameInput,
                    ))
                    .with_children(|p| {
                        p.spawn((
                            ui_text("Socket name...", &font.clone(), 13.0, Color::srgb(0.5, 0.5, 0.5)),
                            crate::widgets::TextInputContent,
                        ));
                    });

                    meta.spawn(ui_text("Comment", &font.clone(), 12.0, Color::srgb(0.6, 0.6, 0.6)));

                    meta.spawn((
                        crate::widgets::TextInputBundle {
                            button: ButtonBundle {
                                style: Node {
                                    width: Val::Percent(100.0),
                                    height: Val::Px(28.0),
                                    padding: UiRect::horizontal(Val::Px(8.0)),
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                background_color: Color::srgba(0.0, 0.0, 0.0, 0.3).into(),
                                border_radius: BorderRadius::all(Val::Px(4.0)),
                                ..default()
                            },
                            input: crate::widgets::TextInput {
                                placeholder: "Add a comment...".to_string(),
                                ..default()
                            },
                        },
                        SocketCommentInput,
                    ))
                    .with_children(|p| {
                        p.spawn((
                            ui_text("Add a comment...", &font.clone(), 13.0, Color::srgb(0.5, 0.5, 0.5)),
                            crate::widgets::TextInputContent,
                        ));
                    });

                    meta.spawn(ui_text("Visual Color", &font.clone(), 12.0, Color::srgb(0.6, 0.6, 0.6)));
                    
                    crate::widgets::spawn_color_picker_ext::<
                        SocketColorPicker, 
                        SocketColorPickerContainer, 
                        SocketColorHueSlider, 
                        SocketColorPreset
                    >(meta, Color::srgb(0.2, 0.8, 0.2), false);
                });

                // --- VFX SETTINGS ---
                spawn_collapsible_section(
                    details,
                    font,
                    icon_font,
                    "VFX SETTINGS",
                    false,
                    SocketVfxSection,
                    |vfx| {
                        // Toggle
                        vfx.spawn(UiNode { node: Node {
                                width: Val::Percent(100.0),
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::Center,
                                margin: UiRect::bottom(Val::Px(10.0)),
                                column_gap: Val::Px(10.0),
                                ..default()
                            }, ..default() }).with_children(|row| {
                            row.spawn((
                                ButtonBundle {
                                    style: Node {
                                        width: Val::Px(16.0),
                                        height: Val::Px(16.0),
                                        border: UiRect::all(Val::Px(1.0)),
                                        ..default()
                                    },
                                    background_color: Color::srgba(0.0, 0.0, 0.0, 0.5).into(),
                                    border_color: Color::srgba(1.0, 1.0, 1.0, 0.2).into(),
                                    border_radius: BorderRadius::all(Val::Px(2.0)),
                                    ..default()
                                },
                                SocketVfxToggle,
                                Tooltip("Toggle visual effects for this socket".to_string()),
                            ));
                            row.spawn(ui_text("Enable VFX", &font.clone(), 13.0, Color::srgb(0.8, 0.8, 0.8)));
                        });

                        // --- EMISSION ---
                        spawn_collapsible_section(vfx, font, icon_font, "EMISSION", true, SocketVfxEmissionSection, |sub| {
                            sub.spawn(ui_text("Rate", &font.clone(), 11.0, Color::srgb(0.6, 0.6, 0.6)));
                            crate::widgets::spawn_slider_ext(sub, 0.0, 10.0, 1.0, (SocketVfxSlider::EmissionRate, Tooltip("Particles per second multiplier".to_string())));

                            sub.spawn(ui_text("Lifetime", &font.clone(), 11.0, Color::srgb(0.6, 0.6, 0.6)));
                            crate::widgets::spawn_slider_ext(sub, 0.1, 5.0, 1.0, (SocketVfxSlider::EmissionLifetime, Tooltip("How long each particle lives".to_string())));

                            sub.spawn(ui_text("Jitter", &font.clone(), 11.0, Color::srgb(0.6, 0.6, 0.6)));
                            crate::widgets::spawn_slider_ext(sub, 0.0, 1.0, 0.1, (SocketVfxSlider::EmissionJitter, Tooltip("Randomness in emission timing".to_string())));
                        });

                        // --- MOTION ---
                        spawn_collapsible_section(vfx, font, icon_font, "MOTION & PHYSICS", false, SocketVfxMotionSection, |sub| {
                            sub.spawn(ui_text("Speed", &font.clone(), 11.0, Color::srgb(0.6, 0.6, 0.6)));
                            crate::widgets::spawn_slider_ext(sub, 0.0, 10.0, 1.0, (SocketVfxSlider::MotionSpeed, Tooltip("Initial particle speed".to_string())));

                            sub.spawn(ui_text("Spread", &font.clone(), 11.0, Color::srgb(0.6, 0.6, 0.6)));
                            crate::widgets::spawn_slider_ext(sub, 0.0, 1.0, 0.2, (SocketVfxSlider::MotionSpread, Tooltip("Cone angle of emission".to_string())));

                            sub.spawn(ui_text("Gravity", &font.clone(), 11.0, Color::srgb(0.6, 0.6, 0.6)));
                            crate::widgets::spawn_slider_ext(sub, -5.0, 5.0, 0.0, (SocketVfxSlider::MotionGravity, Tooltip("Vertical acceleration (negative is down)".to_string())));

                            sub.spawn(ui_text("Drag", &font.clone(), 11.0, Color::srgb(0.6, 0.6, 0.6)));
                            crate::widgets::spawn_slider_ext(sub, 0.0, 2.0, 0.0, (SocketVfxSlider::MotionDrag, Tooltip("Air resistance (slows down particles)".to_string())));
                        });

                        // --- VISUALS ---
                        spawn_collapsible_section(vfx, font, icon_font, "VISUALS", false, SocketVfxVisualsSection, |sub| {
                            sub.spawn(ui_text("Global Scale", &font.clone(), 11.0, Color::srgb(0.6, 0.6, 0.6)));
                            crate::widgets::spawn_slider_ext(sub, 0.1, 5.0, 1.0, (SocketVfxSlider::VisualsScale, Tooltip("Overall size multiplier".to_string())));

                            sub.spawn(ui_text("Start Size", &font.clone(), 11.0, Color::srgb(0.6, 0.6, 0.6)));
                            crate::widgets::spawn_slider_ext(sub, 0.0, 5.0, 1.0, (SocketVfxSlider::VisualsSizeStart, Tooltip("Particle size at birth".to_string())));

                            sub.spawn(ui_text("End Size", &font.clone(), 11.0, Color::srgb(0.6, 0.6, 0.6)));
                            crate::widgets::spawn_slider_ext(sub, 0.0, 5.0, 0.5, (SocketVfxSlider::VisualsSizeEnd, Tooltip("Particle size at the end of its life".to_string())));
                        });
                        
                        // --- PRESETS MANAGEMENT ---
                        vfx.spawn(UiNode { node: Node {
                                width: Val::Percent(100.0),
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(5.0),
                                margin: UiRect::vertical(Val::Px(10.0)),
                                ..default()
                            }, ..default() }).with_children(|manage| {
                            manage.spawn((
                                ui_text("No Linked Preset", &font.clone(), 11.0, Color::srgb(0.5, 0.5, 0.5)),
                                SocketVfxPresetStatusLabel,
                            ));

                            manage.spawn(UiNode { node: Node {
                                    width: Val::Percent(100.0),
                                    flex_direction: FlexDirection::Row,
                                    column_gap: Val::Px(5.0),
                                    ..default()
                                }, ..default() }).with_children(|row| {
                                row.spawn((
                                    crate::widgets::TextInputBundle {
                                        button: ButtonBundle {
                                            style: Node {
                                                flex_grow: 1.0,
                                                height: Val::Px(24.0),
                                                padding: UiRect::horizontal(Val::Px(6.0)),
                                                align_items: AlignItems::Center,
                                                ..default()
                                            },
                                            background_color: Color::srgba(0.0, 0.0, 0.0, 0.3).into(),
                                            border_radius: BorderRadius::all(Val::Px(4.0)),
                                            ..default()
                                        },
                                        input: crate::widgets::TextInput {
                                            placeholder: "Preset name...".to_string(),
                                            ..default()
                                        },
                                    },
                                    SocketVfxPresetNameInput,
                                ))
                                .with_children(|p| {
                                    p.spawn((
                                        ui_text("Preset name...", &font.clone(), 11.0, Color::srgb(0.4, 0.4, 0.4)),
                                        crate::widgets::TextInputContent,
                                    ));
                                });

                                row.spawn((
                                    ButtonBundle {
                                        style: Node {
                                            width: Val::Px(24.0),
                                            height: Val::Px(24.0),
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            ..default()
                                        },
                                        background_color: Color::srgba(0.2, 0.6, 1.0, 0.2).into(),
                                        border_radius: BorderRadius::all(Val::Px(4.0)),
                                        ..default()
                                    },
                                    SocketVfxSavePresetButton,
                                    Tooltip("Save current settings as preset".to_string()),
                                )).with_children(|b| {
                                    b.spawn(ui_text("\u{f0c7}", &icon_font.clone(), 12.0, Color::WHITE));
                                });

                                row.spawn((
                                    ButtonBundle {
                                        style: Node {
                                            width: Val::Px(24.0),
                                            height: Val::Px(24.0),
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            ..default()
                                        },
                                        background_color: Color::srgba(1.0, 0.3, 0.3, 0.2).into(),
                                        border_radius: BorderRadius::all(Val::Px(4.0)),
                                        ..default()
                                    },
                                    SocketVfxDetachPresetButton,
                                    Tooltip("Detach from preset (make unique)".to_string()),
                                )).with_children(|b| {
                                    b.spawn(ui_text("\u{f127}", &icon_font.clone(), 12.0, Color::WHITE));
                                });
                            });
                        });
                        
                        // Preset Buttons (Simple list for now)
                        vfx.spawn(ui_text("Library Presets", &font.clone(), 11.0, Color::srgb(0.6, 0.6, 0.6)));
                        vfx.spawn(UiNode { node: Node {
                                width: Val::Percent(100.0),
                                flex_direction: FlexDirection::Row,
                                flex_wrap: FlexWrap::Wrap,
                                column_gap: Val::Px(4.0),
                                row_gap: Val::Px(4.0),
                                margin: UiRect::top(Val::Px(5.0)),
                                ..default()
                            }, ..default() }).with_children(|grid| {
                            let mut names: Vec<_> = vfx_presets.library.presets.keys().collect();
                            names.sort();
                            for preset in names {
                                grid.spawn((
                                    ButtonBundle {
                                        style: Node {
                                            padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                                            ..default()
                                        },
                                        background_color: Color::srgba(1.0, 1.0, 1.0, 0.05).into(),
                                        border_radius: BorderRadius::all(Val::Px(4.0)),
                                        ..default()
                                    },
                                    SocketVfxPresetItem(preset.to_string()),
                                )).with_children(|b| {
                                    b.spawn(ui_text(preset, &font.clone(), 11.0, Color::srgb(0.8, 0.8, 0.8)));
                                });
                            }
                        });

                        // Texture Groups
                        vfx.spawn((ui_text("Texture Groups (Random Variation)", &font.clone(), 11.0, Color::srgb(0.0, 1.0, 0.8)), Node { margin: UiRect::top(Val::Px(10.0)), ..default() }));

                        vfx.spawn(UiNode { node: Node {
                                width: Val::Percent(100.0),
                                flex_direction: FlexDirection::Row,
                                flex_wrap: FlexWrap::Wrap,
                                column_gap: Val::Px(4.0),
                                row_gap: Val::Px(4.0),
                                margin: UiRect::top(Val::Px(5.0)),
                                ..default()
                            }, ..default() }).with_children(|grid| {
                            let mut group_names: Vec<_> = vfx_registry.groups.keys().collect();
                            group_names.sort();
                            
                            for name in group_names {
                                let handles = &vfx_registry.groups[name];
                                grid.spawn((
                                    ButtonBundle {
                                        style: Node {
                                            width: Val::Px(40.0),
                                            height: Val::Px(40.0),
                                            padding: UiRect::all(Val::Px(2.0)),
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            ..default()
                                        },
                                        background_color: Color::srgba(1.0, 1.0, 1.0, 0.05).into(),
                                        border_radius: BorderRadius::all(Val::Px(4.0)),
                                        ..default()
                                    },
                                    SocketVfxGroupItem(name.clone()),
                                    Tooltip(format!("Group: {} ({} variations)", name, handles.len())),
                                )).with_children(|b| {
                                    // Show first image of group as preview
                                    b.spawn(ImageBundle {
                                        style: Node {
                                            width: Val::Percent(90.0),
                                            height: Val::Percent(90.0),
                                            ..default()
                                        },
                                        image: UiImage::new(handles[0].clone()),
                                        ..default()
                                    });
                                    // Add a small indicator for group
                                    b.spawn(UiNode { node: Node {
                                            position_type: PositionType::Absolute,
                                            right: Val::Px(2.0),
                                            bottom: Val::Px(2.0),
                                            width: Val::Px(8.0),
                                            height: Val::Px(8.0),
                                            ..default()
                                        }, background_color: BackgroundColor(Color::srgba(0.0, 1.0, 0.8, 0.8)), border_radius: BorderRadius::all(Val::Px(4.0)), ..default() });
                                });
                            }
                        });

                        // Kenney Textures
                        vfx.spawn((ui_text("Kenney Textures", &font.clone(), 11.0, Color::srgb(0.6, 0.6, 0.6)), Node { margin: UiRect::top(Val::Px(10.0)), ..default() }));
                        
                        vfx.spawn(UiNode { node: Node {
                                width: Val::Percent(100.0),
                                flex_direction: FlexDirection::Row,
                                flex_wrap: FlexWrap::Wrap,
                                column_gap: Val::Px(4.0),
                                row_gap: Val::Px(4.0),
                                margin: UiRect::top(Val::Px(5.0)),
                                ..default()
                            }, ..default() }).with_children(|grid| {
                            for (name, handle) in &vfx_registry.textures {
                                grid.spawn((
                                    ButtonBundle {
                                        style: Node {
                                            width: Val::Px(30.0),
                                            height: Val::Px(30.0),
                                            padding: UiRect::all(Val::Px(2.0)),
                                            ..default()
                                        },
                                        background_color: Color::srgba(1.0, 1.0, 1.0, 0.05).into(),
                                        border_radius: BorderRadius::all(Val::Px(4.0)),
                                        ..default()
                                    },
                                    SocketVfxTextureItem(name.clone()),
                                    Tooltip(name.clone()),
                                )).with_children(|b| {
                                    b.spawn(ImageBundle {
                                        style: Node {
                                            width: Val::Percent(100.0),
                                            height: Val::Percent(100.0),
                                            ..default()
                                        },
                                        image: UiImage::new(handle.clone()),
                                        ..default()
                                    });
                                });
                            }
                        });
                    }
                );
            });
        }
    );
}
