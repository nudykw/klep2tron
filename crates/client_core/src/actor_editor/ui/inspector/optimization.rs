use bevy::prelude::*;
use crate::actor_editor::{
    widgets::{spawn_collapsible_section_ext, spawn_text_input, TextInput},
    EditorStatus, ToastEvent, ToastType, SlicingSettings, OriginalMeshComponent,
};
use super::types::*;
use crate::actor_editor::systems::optimization::{OptimizationSettings, OptimizedMeshComponent, OptimizeMeshCommand, perform_mesh_optimization};
use crate::actor_editor::systems::undo_redo::ActionStack;

pub fn spawn_optimization_section(
    p: &mut ChildBuilder,
    font: &Handle<Font>,
    icon_font: &Handle<Font>,
) {
    spawn_collapsible_section_ext(
        p,
        font,
        icon_font,
        "POLYCOUNT BUDGET",
        false,
        OptimizationSectionMarker,
        |content| {
            // Target Triangles Input
            content.spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Px(30.0),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceBetween,
                    padding: UiRect::horizontal(Val::Px(5.0)),
                    ..default()
                },
                ..default()
            }).with_children(|row| {
                row.spawn(TextBundle::from_section(
                    "Target Tris:",
                    TextStyle { font: font.clone(), font_size: 13.0, color: Color::srgb(0.7, 0.7, 0.7) },
                ));
                
                spawn_text_input(row, font, "15000", "15000", Val::Px(80.0));
            });
            
            // Note: I need to handle the marker for the input. spawn_text_input returns the entity.
        },
        |_header| {}
    );
}

// I'll rewrite spawn_optimization_section to be more robust with markers
pub fn spawn_optimization_section_v2(
    p: &mut ChildBuilder,
    font: &Handle<Font>,
    icon_font: &Handle<Font>,
) {
    spawn_collapsible_section_ext(
        p,
        font,
        icon_font,
        "POLYCOUNT BUDGET",
        false,
        OptimizationSectionMarker,
        |content| {
            // Target Tris
            content.spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    margin: UiRect::bottom(Val::Px(10.0)),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                },
                ..default()
            }).with_children(|row| {
                row.spawn(TextBundle::from_section(
                    "Target Budget:",
                    TextStyle { font: font.clone(), font_size: 13.0, color: Color::srgb(0.8, 0.8, 0.8) },
                ));
                
                let input_id = spawn_text_input(row, font, "15000", "15000", Val::Px(80.0));
                row.spawn(OptimizationTargetInputMarker).set_parent(input_id);
            });

            // Buttons Row
            content.spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    column_gap: Val::Px(5.0),
                    ..default()
                },
                ..default()
            }).with_children(|row| {
                // Optimize Button
                row.spawn((
                    ButtonBundle {
                        style: Style {
                            flex_grow: 1.0,
                            height: Val::Px(30.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        background_color: Color::srgba(0.2, 0.5, 0.2, 0.6).into(),
                        border_radius: BorderRadius::all(Val::Px(4.0)),
                        ..default()
                    },
                    OptimizeMeshButton,
                    crate::actor_editor::widgets::Tooltip("Perform mesh simplification".to_string()),
                )).with_children(|b| {
                    b.spawn(TextBundle::from_section(
                        "OPTIMIZE MESH",
                        TextStyle { font: font.clone(), font_size: 11.0, color: Color::WHITE, ..default() },
                    ));
                });

                // Original Toggle (A/B)
                row.spawn((
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(40.0),
                            height: Val::Px(30.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        background_color: Color::srgba(1.0, 1.0, 1.0, 0.05).into(),
                        border_radius: BorderRadius::all(Val::Px(4.0)),
                        ..default()
                    },
                    OptimizationOriginalToggle,
                    crate::actor_editor::widgets::Tooltip("Show Original (Un-optimized) Mesh".to_string()),
                )).with_children(|b| {
                    b.spawn(TextBundle::from_section(
                        "\u{f01e}", // refresh/swap icon
                        TextStyle { font: icon_font.clone(), font_size: 14.0, color: Color::WHITE },
                    ));
                });

                // Wireframe Toggle
                row.spawn((
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(40.0),
                            height: Val::Px(30.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        background_color: Color::srgba(1.0, 1.0, 1.0, 0.05).into(),
                        border_radius: BorderRadius::all(Val::Px(4.0)),
                        ..default()
                    },
                    OptimizationWireframeToggle,
                    crate::actor_editor::widgets::Tooltip("Toggle Wireframe Overlay (W)".to_string()),
                )).with_children(|b| {
                    b.spawn(TextBundle::from_section(
                        "\u{f00a}", // grid icon
                        TextStyle { font: icon_font.clone(), font_size: 14.0, color: Color::WHITE },
                    ));
                });

            });

            content.spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    margin: UiRect::top(Val::Px(10.0)),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                },
                ..default()
            }).with_children(|row| {
                row.spawn(TextBundle::from_section(
                    "Fill/Rim:",
                    TextStyle { font: font.clone(), font_size: 13.0, color: Color::srgb(0.8, 0.8, 0.8) },
                ));
                
                row.spawn(NodeBundle {
                    style: Style {
                        width: Val::Px(120.0),
                        ..default()
                    },
                    ..default()
                }).with_children(|slider_p| {
                    crate::actor_editor::widgets::spawn_slider_ext(
                        slider_p,
                        0.0,
                        1.0,
                        0.0,
                        (
                            OptimizationRimSlider,
                            crate::actor_editor::widgets::Tooltip("Left: Solid, Right: Hollow, Center: Rim Thickness".to_string()),
                        ),
                    );
                });
            });
        },
        |_header| {}
    );
}

// Helper removed

pub fn mesh_optimization_system(
    mut commands: Commands,
    mut opt_settings: ResMut<OptimizationSettings>,
    mut opt_task: ResMut<crate::actor_editor::systems::optimization::OptimizationTask>,
    mut slicing_settings: ResMut<SlicingSettings>,
    mut status: ResMut<EditorStatus>,
    mut action_stack: ResMut<ActionStack>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut toast_events: EventWriter<ToastEvent>,
    
    btn_query: Query<&Interaction, (With<OptimizeMeshButton>, Changed<Interaction>)>,
    toggle_query: Query<&Interaction, (With<OptimizationOriginalToggle>, Changed<Interaction>)>,
    wire_query: Query<&Interaction, (With<OptimizationWireframeToggle>, Changed<Interaction>)>,
    caps_query: Query<&Interaction, (With<OptimizationCapsToggle>, Changed<Interaction>)>,
    rim_slider_query: Query<&crate::actor_editor::widgets::Slider, (With<OptimizationRimSlider>, Changed<crate::actor_editor::widgets::Slider>)>,
    input_query: Query<&TextInput>,
    marker_query: Query<&Parent, With<OptimizationTargetInputMarker>>,
    mesh_query: Query<(Entity, &OriginalMeshComponent)>,
) {
    // 1. Check if a task is already running
    if let Some(ref mut task) = opt_task.0 {
        if let Some(result) = bevy::tasks::block_on(bevy::tasks::poll_once(task)) {
            // Apply result
            if let Some(optimized) = result.new_mesh {
                let new_handle = meshes.add(optimized);
                
                action_stack.push(Box::new(OptimizeMeshCommand {
                    entity: result.entity,
                    old_mesh: result.original_mesh_handle.clone(),
                    new_mesh: new_handle.clone(),
                    target_tris: result.target_tris,
                }));
                
                // Apply immediately
                commands.entity(result.entity).insert(OptimizedMeshComponent(new_handle.clone()));
                opt_settings.is_optimized = true;
                slicing_settings.trigger_slice = true;
                
                toast_events.send(ToastEvent {
                    message: "Mesh optimized successfully!".to_string(),
                    toast_type: ToastType::Success,
                });
            } else {
                toast_events.send(ToastEvent {
                    message: "Optimization failed or already at budget.".to_string(),
                    toast_type: ToastType::Info,
                });
            }
            
            opt_task.0 = None;
            *status = EditorStatus::Ready;
        }
        return; // Don't start a new task while one is running
    }

    // 2. Sync target triangles from input
    for parent in marker_query.iter() {
        if let Ok(input) = input_query.get(parent.get()) {
            if let Ok(val) = input.value.parse::<usize>() {
                opt_settings.target_triangles = val;
            }
        }
    }

    // 3. Handle Optimize Button
    for interaction in btn_query.iter() {
        if *interaction == Interaction::Pressed && *status == EditorStatus::Ready {
            if let Ok((entity, original)) = mesh_query.get_single() {
                if let Some(source_mesh) = meshes.get(&original.0) {
                    *status = EditorStatus::Processing;
                    
                    let mesh_copy = source_mesh.clone();
                    let target_tris = opt_settings.target_triangles;
                    let original_handle = original.0.clone();
                    
                    let thread_pool = bevy::tasks::AsyncComputeTaskPool::get();
                    let task = thread_pool.spawn(async move {
                        let optimized = perform_mesh_optimization(&mesh_copy, target_tris);
                        crate::actor_editor::systems::optimization::OptimizationResult {
                            entity,
                            original_mesh_handle: original_handle,
                            new_mesh: optimized,
                            target_tris,
                        }
                    });
                    
                    opt_task.0 = Some(task);
                    info!("Started background mesh optimization task...");
                }
            }
        }
    }

    // 4. Handle A/B Toggle
    for interaction in toggle_query.iter() {
        if *interaction == Interaction::Pressed {
            opt_settings.is_optimized = !opt_settings.is_optimized;
            slicing_settings.trigger_slice = true; // Re-slice to show/hide optimization
        }
    }

    // 5. Handle Wireframe Toggle
    for interaction in wire_query.iter() {
        if *interaction == Interaction::Pressed {
            opt_settings.wireframe = !opt_settings.wireframe;
            info!("Wireframe toggled: {}", opt_settings.wireframe);
        }
    }

    // 6. Handle Fill Caps Toggle
    for interaction in caps_query.iter() {
        if *interaction == Interaction::Pressed {
            slicing_settings.show_caps = !slicing_settings.show_caps;
            slicing_settings.trigger_slice = true;
            info!("Fill Caps toggled: {}", slicing_settings.show_caps);
        }
    }

    // 7. Handle Unified Fill/Rim Slider
    for slider in rim_slider_query.iter() {
        let (new_rim, new_caps) = if slider.value < 0.01 {
            (0.0, true) // Solid
        } else if slider.value > 0.99 {
            (0.0, false) // Hollow
        } else {
            // Map 0.01..0.99 to thick..thin rim
            // We'll use 15cm as max rim thickness
            let t = (1.0 - slider.value) * 0.15;
            (t.max(0.001), true)
        };

        if (slicing_settings.rim_thickness - new_rim).abs() > 0.0001 || slicing_settings.show_caps != new_caps {
            slicing_settings.rim_thickness = new_rim;
            slicing_settings.show_caps = new_caps;
            slicing_settings.trigger_slice = true;
        }
    }
}

// I need to update the system signature to include the button background queries.
pub fn mesh_optimization_visuals_system(
    opt_settings: Res<OptimizationSettings>,
    slicing_settings: Res<SlicingSettings>,
    mut btn_query: Query<(&mut BackgroundColor, &Interaction, Option<&OptimizationOriginalToggle>, Option<&OptimizationWireframeToggle>, Option<&OptimizationCapsToggle>)>,
) {
    for (mut bg, interaction, original_opt, wire_opt, caps_opt) in btn_query.iter_mut() {
        let active = if original_opt.is_some() {
            !opt_settings.is_optimized
        } else if wire_opt.is_some() {
            opt_settings.wireframe
        } else if caps_opt.is_some() {
            slicing_settings.show_caps
        } else {
            continue;
        };

        let hovered = *interaction == Interaction::Hovered;
        
        if active {
            *bg = Color::srgba(0.3, 0.6, 1.0, 0.6).into(); // Blue active
        } else if hovered {
            *bg = Color::srgba(1.0, 1.0, 1.0, 0.1).into();
        } else {
            *bg = Color::srgba(1.0, 1.0, 1.0, 0.05).into();
        }
    }
}

// Helper removed
