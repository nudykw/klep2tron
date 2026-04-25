use bevy::prelude::*;

pub mod types;
pub mod materials;
pub mod sockets;
pub mod parts;
pub mod vfx;
pub mod optimization;
pub mod systems;

pub use types::*;
pub use systems::*;

pub fn setup_inspector(
    parent: &mut ChildBuilder,
    font: &Handle<Font>,
    icon_font: &Handle<Font>,
    vfx_presets: &crate::actor_editor::vfx_assets::VfxPresets,
    vfx_registry: &crate::actor_editor::vfx_assets::VfxRegistry,
) {
    parent.spawn((
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                flex_shrink: 0.0,
                ..default()
            },
            focus_policy: bevy::ui::FocusPolicy::Block,
            ..default()
        },
        InspectorPanel,
        Interaction::default(),
    )).with_children(|p| {
        materials::spawn_materials_section(p, font, icon_font);
        optimization::spawn_optimization_section_v2(p, font, icon_font);
        sockets::spawn_sockets_section(p, font, icon_font, vfx_presets, vfx_registry);
        parts::spawn_parts_section(p, font, icon_font);
    });
}
