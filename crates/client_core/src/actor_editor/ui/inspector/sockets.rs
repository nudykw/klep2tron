use bevy::prelude::*;
use crate::actor_editor::{
    widgets::{spawn_collapsible_section, Tooltip},
    SocketColorPicker, SocketColorPickerContainer, SocketColorHueSlider, SocketColorPreset
};
use super::types::*;

pub fn spawn_sockets_section(
    p: &mut ChildBuilder,
    font: &Handle<Font>,
    icon_font: &Handle<Font>,
    vfx_presets: &crate::actor_editor::vfx_assets::VfxPresets,
    vfx_registry: &crate::actor_editor::vfx_assets::VfxRegistry,
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
            content.spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    margin: UiRect::top(Val::Px(10.0)),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                },
                ..default()
            }).with_children(|row| {
                for (axis, label) in [(TransformAxis::X, "X"), (TransformAxis::Y, "Y"), (TransformAxis::Z, "Z")] {
                    row.spawn((
                        NodeBundle {
                            style: Style {
                                width: Val::Px(75.0),
                                height: Val::Px(25.0),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                            background_color: Color::srgba(0.0, 0.0, 0.0, 0.3).into(),
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                            ..default()
                        },
                        axis,
                    )).with_children(|box_| {
                        box_.spawn(TextBundle::from_section(
                            format!("{}: {:.2}", label, 0.0),
                            TextStyle { font: font.clone(), font_size: 11.0, color: Color::WHITE },
                        ));
                    });
                }
            });

            // --- ROTATION DISPLAY ---
            content.spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    margin: UiRect::top(Val::Px(5.0)),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                },
                ..default()
            }).with_children(|row| {
                for (axis, label) in [(RotationAxis::Roll, "R"), (RotationAxis::Pitch, "P"), (RotationAxis::Yaw, "Y")] {
                    row.spawn((
                        NodeBundle {
                            style: Style {
                                width: Val::Px(75.0),
                                height: Val::Px(25.0),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                            background_color: Color::srgba(0.1, 0.1, 0.1, 0.4).into(),
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                            ..default()
                        },
                        axis,
                    )).with_children(|box_| {
                        box_.spawn(TextBundle::from_section(
                            format!("{}: {:.1}°", label, 0.0),
                            TextStyle { font: font.clone(), font_size: 11.0, color: Color::srgb(0.8, 0.8, 1.0) },
                        ));
                    });
                }
            });

            // --- RESET BUTTON ---
            content.spawn((
                ButtonBundle {
                    style: Style {
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
                b.spawn(TextBundle::from_section(
                    "Reset Rotation",
                    TextStyle { font: font.clone(), font_size: 12.0, color: Color::srgb(0.7, 0.7, 0.7) },
                ));
            });

            // --- SOCKET DETAILS (Name & Comment) ---
            content.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        margin: UiRect::top(Val::Px(15.0)),
                        row_gap: Val::Px(8.0),
                        display: Display::None, // Hidden by default
                        ..default()
                    },
                    ..default()
                },
                SocketDetailsContainer,
            )).with_children(|details| {
                // ... (details content remains same, but we will add logic to hide it in systems)
                details.spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Px(1.0),
                        margin: UiRect::vertical(Val::Px(5.0)),
                        ..default()
                    },
                    background_color: Color::srgba(1.0, 1.0, 1.0, 0.1).into(),
                    ..default()
                });

                details.spawn((
                    NodeBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(8.0),
                            ..default()
                        },
                        ..default()
                    },
                    SocketMetadataSection,
                )).with_children(|meta| {
                    meta.spawn(TextBundle::from_section(
                        "Socket Name",
                        TextStyle { font: font.clone(), font_size: 12.0, color: Color::srgb(0.6, 0.6, 0.6) },
                    ));
                    
                    meta.spawn((
                        crate::actor_editor::widgets::TextInputBundle {
                            button: ButtonBundle {
                                style: Style {
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
                            input: crate::actor_editor::widgets::TextInput {
                                placeholder: "Socket name...".to_string(),
                                ..default()
                            },
                        },
                        SocketNameInput,
                    ))
                    .with_children(|p| {
                        p.spawn((
                            TextBundle::from_section(
                                "Socket name...",
                                TextStyle { font: font.clone(), font_size: 13.0, color: Color::srgb(0.5, 0.5, 0.5) },
                            ),
                            crate::actor_editor::widgets::TextInputContent,
                        ));
                    });

                    meta.spawn(TextBundle::from_section(
                        "Comment",
                        TextStyle { font: font.clone(), font_size: 12.0, color: Color::srgb(0.6, 0.6, 0.6) },
                    ));

                    meta.spawn((
                        crate::actor_editor::widgets::TextInputBundle {
                            button: ButtonBundle {
                                style: Style {
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
                            input: crate::actor_editor::widgets::TextInput {
                                placeholder: "Add a comment...".to_string(),
                                ..default()
                            },
                        },
                        SocketCommentInput,
                    ))
                    .with_children(|p| {
                        p.spawn((
                            TextBundle::from_section(
                                "Add a comment...",
                                TextStyle { font: font.clone(), font_size: 13.0, color: Color::srgb(0.5, 0.5, 0.5) },
                            ),
                            crate::actor_editor::widgets::TextInputContent,
                        ));
                    });

                    meta.spawn(TextBundle::from_section(
                        "Visual Color",
                        TextStyle { font: font.clone(), font_size: 12.0, color: Color::srgb(0.6, 0.6, 0.6) },
                    ));
                    
                    crate::actor_editor::widgets::spawn_color_picker_ext::<
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
                        vfx.spawn(NodeBundle {
                            style: Style {
                                width: Val::Percent(100.0),
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::Center,
                                margin: UiRect::bottom(Val::Px(10.0)),
                                column_gap: Val::Px(10.0),
                                ..default()
                            },
                            ..default()
                        }).with_children(|row| {
                            row.spawn((
                                ButtonBundle {
                                    style: Style {
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
                            row.spawn(TextBundle::from_section(
                                "Enable VFX",
                                TextStyle { font: font.clone(), font_size: 13.0, color: Color::srgb(0.8, 0.8, 0.8) },
                            ));
                        });

                        // --- EMISSION ---
                        spawn_collapsible_section(vfx, font, icon_font, "EMISSION", true, SocketVfxEmissionSection, |sub| {
                            sub.spawn(TextBundle::from_section("Rate", TextStyle { font: font.clone(), font_size: 11.0, color: Color::srgb(0.6, 0.6, 0.6) }));
                            crate::actor_editor::widgets::spawn_slider_ext(sub, 0.0, 10.0, 1.0, (SocketVfxSlider::EmissionRate, Tooltip("Particles per second multiplier".to_string())));

                            sub.spawn(TextBundle::from_section("Lifetime", TextStyle { font: font.clone(), font_size: 11.0, color: Color::srgb(0.6, 0.6, 0.6) }));
                            crate::actor_editor::widgets::spawn_slider_ext(sub, 0.1, 5.0, 1.0, (SocketVfxSlider::EmissionLifetime, Tooltip("How long each particle lives".to_string())));

                            sub.spawn(TextBundle::from_section("Jitter", TextStyle { font: font.clone(), font_size: 11.0, color: Color::srgb(0.6, 0.6, 0.6) }));
                            crate::actor_editor::widgets::spawn_slider_ext(sub, 0.0, 1.0, 0.1, (SocketVfxSlider::EmissionJitter, Tooltip("Randomness in emission timing".to_string())));
                        });

                        // --- MOTION ---
                        spawn_collapsible_section(vfx, font, icon_font, "MOTION & PHYSICS", false, SocketVfxMotionSection, |sub| {
                            sub.spawn(TextBundle::from_section("Speed", TextStyle { font: font.clone(), font_size: 11.0, color: Color::srgb(0.6, 0.6, 0.6) }));
                            crate::actor_editor::widgets::spawn_slider_ext(sub, 0.0, 10.0, 1.0, (SocketVfxSlider::MotionSpeed, Tooltip("Initial particle speed".to_string())));

                            sub.spawn(TextBundle::from_section("Spread", TextStyle { font: font.clone(), font_size: 11.0, color: Color::srgb(0.6, 0.6, 0.6) }));
                            crate::actor_editor::widgets::spawn_slider_ext(sub, 0.0, 1.0, 0.2, (SocketVfxSlider::MotionSpread, Tooltip("Cone angle of emission".to_string())));

                            sub.spawn(TextBundle::from_section("Gravity", TextStyle { font: font.clone(), font_size: 11.0, color: Color::srgb(0.6, 0.6, 0.6) }));
                            crate::actor_editor::widgets::spawn_slider_ext(sub, -5.0, 5.0, 0.0, (SocketVfxSlider::MotionGravity, Tooltip("Vertical acceleration (negative is down)".to_string())));

                            sub.spawn(TextBundle::from_section("Drag", TextStyle { font: font.clone(), font_size: 11.0, color: Color::srgb(0.6, 0.6, 0.6) }));
                            crate::actor_editor::widgets::spawn_slider_ext(sub, 0.0, 2.0, 0.0, (SocketVfxSlider::MotionDrag, Tooltip("Air resistance (slows down particles)".to_string())));
                        });

                        // --- VISUALS ---
                        spawn_collapsible_section(vfx, font, icon_font, "VISUALS", false, SocketVfxVisualsSection, |sub| {
                            sub.spawn(TextBundle::from_section("Global Scale", TextStyle { font: font.clone(), font_size: 11.0, color: Color::srgb(0.6, 0.6, 0.6) }));
                            crate::actor_editor::widgets::spawn_slider_ext(sub, 0.1, 5.0, 1.0, (SocketVfxSlider::VisualsScale, Tooltip("Overall size multiplier".to_string())));

                            sub.spawn(TextBundle::from_section("Start Size", TextStyle { font: font.clone(), font_size: 11.0, color: Color::srgb(0.6, 0.6, 0.6) }));
                            crate::actor_editor::widgets::spawn_slider_ext(sub, 0.0, 5.0, 1.0, (SocketVfxSlider::VisualsSizeStart, Tooltip("Particle size at birth".to_string())));

                            sub.spawn(TextBundle::from_section("End Size", TextStyle { font: font.clone(), font_size: 11.0, color: Color::srgb(0.6, 0.6, 0.6) }));
                            crate::actor_editor::widgets::spawn_slider_ext(sub, 0.0, 5.0, 0.5, (SocketVfxSlider::VisualsSizeEnd, Tooltip("Particle size at the end of its life".to_string())));
                        });
                        
                        // Preset Buttons (Simple list for now)
                        vfx.spawn(TextBundle::from_section(
                            "Presets",
                            TextStyle { font: font.clone(), font_size: 11.0, color: Color::srgb(0.6, 0.6, 0.6), ..default() },
                        ));
                        vfx.spawn(NodeBundle {
                            style: Style {
                                width: Val::Percent(100.0),
                                flex_direction: FlexDirection::Row,
                                flex_wrap: FlexWrap::Wrap,
                                column_gap: Val::Px(4.0),
                                row_gap: Val::Px(4.0),
                                margin: UiRect::top(Val::Px(5.0)),
                                ..default()
                            },
                            ..default()
                        }).with_children(|grid| {
                            let mut names: Vec<_> = vfx_presets.library.presets.keys().collect();
                            names.sort();
                            for preset in names {
                                grid.spawn((
                                    ButtonBundle {
                                        style: Style {
                                            padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                                            ..default()
                                        },
                                        background_color: Color::srgba(1.0, 1.0, 1.0, 0.05).into(),
                                        border_radius: BorderRadius::all(Val::Px(4.0)),
                                        ..default()
                                    },
                                    SocketVfxPresetItem(preset.to_string()),
                                )).with_children(|b| {
                                    b.spawn(TextBundle::from_section(
                                        preset,
                                        TextStyle { font: font.clone(), font_size: 11.0, color: Color::srgb(0.8, 0.8, 0.8) },
                                    ));
                                });
                            }
                        });

                        // Texture Groups
                        vfx.spawn(TextBundle::from_section(
                            "Texture Groups (Random Variation)",
                            TextStyle { font: font.clone(), font_size: 11.0, color: Color::srgb(0.0, 1.0, 0.8), ..default() },
                        ).with_style(Style { margin: UiRect::top(Val::Px(10.0)), ..default() }));

                        vfx.spawn(NodeBundle {
                            style: Style {
                                width: Val::Percent(100.0),
                                flex_direction: FlexDirection::Row,
                                flex_wrap: FlexWrap::Wrap,
                                column_gap: Val::Px(4.0),
                                row_gap: Val::Px(4.0),
                                margin: UiRect::top(Val::Px(5.0)),
                                ..default()
                            },
                            ..default()
                        }).with_children(|grid| {
                            let mut group_names: Vec<_> = vfx_registry.groups.keys().collect();
                            group_names.sort();
                            
                            for name in group_names {
                                let handles = &vfx_registry.groups[name];
                                grid.spawn((
                                    ButtonBundle {
                                        style: Style {
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
                                        style: Style {
                                            width: Val::Percent(90.0),
                                            height: Val::Percent(90.0),
                                            ..default()
                                        },
                                        image: UiImage::new(handles[0].clone()),
                                        ..default()
                                    });
                                    // Add a small indicator for group
                                    b.spawn(NodeBundle {
                                        style: Style {
                                            position_type: PositionType::Absolute,
                                            right: Val::Px(2.0),
                                            bottom: Val::Px(2.0),
                                            width: Val::Px(8.0),
                                            height: Val::Px(8.0),
                                            ..default()
                                        },
                                        background_color: Color::srgba(0.0, 1.0, 0.8, 0.8).into(),
                                        border_radius: BorderRadius::all(Val::Px(4.0)),
                                        ..default()
                                    });
                                });
                            }
                        });

                        // Kenney Textures
                        vfx.spawn(TextBundle::from_section(
                            "Kenney Textures",
                            TextStyle { font: font.clone(), font_size: 11.0, color: Color::srgb(0.6, 0.6, 0.6), ..default() },
                        ).with_style(Style { margin: UiRect::top(Val::Px(10.0)), ..default() }));
                        
                        vfx.spawn(NodeBundle {
                            style: Style {
                                width: Val::Percent(100.0),
                                flex_direction: FlexDirection::Row,
                                flex_wrap: FlexWrap::Wrap,
                                column_gap: Val::Px(4.0),
                                row_gap: Val::Px(4.0),
                                margin: UiRect::top(Val::Px(5.0)),
                                ..default()
                            },
                            ..default()
                        }).with_children(|grid| {
                            for (name, handle) in &vfx_registry.textures {
                                grid.spawn((
                                    ButtonBundle {
                                        style: Style {
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
                                        style: Style {
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
