use bevy::prelude::*;

pub mod types;
pub mod materials;
pub mod sockets;
pub mod parts;
pub mod systems;

pub use types::*;
pub use systems::*;

pub fn setup_inspector(
    parent: &mut ChildBuilder,
    font: &Handle<Font>,
    icon_font: &Handle<Font>,
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
        sockets::spawn_sockets_section(p, font, icon_font);
        parts::spawn_parts_section(p, font, icon_font);
    });
}
