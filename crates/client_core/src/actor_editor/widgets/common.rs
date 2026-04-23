use bevy::prelude::*;
use super::super::{ActorEditorEntity, ViewportSettings};
use super::sliders::spawn_vertical_range_slider;

#[derive(Component)]
pub struct Tooltip(pub String);

#[derive(Component)]
pub struct TooltipRoot;

pub fn spawn_tooltip_root(commands: &mut Commands, font: &Handle<Font>, target_camera: Option<Entity>) {
    let mut cmd = commands.spawn((NodeBundle { style: Style { position_type: PositionType::Absolute, padding: UiRect::all(Val::Px(10.0)), display: Display::None, width: Val::Auto, height: Val::Auto, ..default() }, background_color: Color::srgba(0.05, 0.05, 0.05, 0.95).into(), border_radius: BorderRadius::all(Val::Px(6.0)), z_index: ZIndex::Global(100), ..default() }, TooltipRoot, ActorEditorEntity, ));
    if let Some(camera) = target_camera { cmd.insert(bevy::ui::TargetCamera(camera)); }
    cmd.with_children(|p| { p.spawn(TextBundle::from_section("", TextStyle { font: font.clone(), font_size: 14.0, color: Color::WHITE }, )); });
}

pub fn tooltip_system(window_query: Query<&Window, With<bevy::window::PrimaryWindow>>, interaction_query: Query<(&Interaction, &Tooltip)>, mut tooltip_query: Query<(&mut Style, &mut Visibility, &Children), With<TooltipRoot>>, mut text_query: Query<&mut Text>, ) {
    let Ok(window) = window_query.get_single() else { return; };
    let Some(cursor_position) = window.cursor_position() else { return; };
    let Ok((mut style, mut visibility, children)) = tooltip_query.get_single_mut() else { return; };
    let mut hovered_text = None;
    for (interaction, tooltip) in interaction_query.iter() { if *interaction == Interaction::Hovered { hovered_text = Some(tooltip.0.clone()); break; } }
    if let Some(text_content) = hovered_text {
        if let Ok(mut text) = text_query.get_mut(children[0]) { text.sections[0].value = text_content; }
        *visibility = Visibility::Visible; style.display = Display::Flex;
        let x = (cursor_position.x + 15.0).min(window.width() - 150.0);
        let y = (cursor_position.y + 15.0).min(window.height() - 40.0);
        style.left = Val::Px(x); style.top = Val::Px(y);
    } else { *visibility = Visibility::Hidden; style.display = Display::None; }
}

#[derive(Component)]
pub struct StatusText;
#[derive(Component)]
pub struct PolycountText;
#[derive(Component)]
pub struct KeyHintText;

pub fn spawn_status_bar(parent: &mut ChildBuilder, font: &Handle<Font>, icon_font: &Handle<Font>) {
    parent.spawn(NodeBundle { style: Style { width: Val::Percent(100.0), height: Val::Px(28.0), border: UiRect::top(Val::Px(1.0)), padding: UiRect::horizontal(Val::Px(15.0)), align_items: AlignItems::Center, justify_content: JustifyContent::SpaceBetween, ..default() }, background_color: Color::srgba(0.05, 0.05, 0.05, 0.9).into(), border_color: Color::srgba(1.0, 1.0, 1.0, 0.1).into(), ..default() }).with_children(|p| {
        p.spawn((NodeBundle { style: Style { flex_direction: FlexDirection::Row, align_items: AlignItems::Center, ..default() }, ..default() }, Interaction::default(), Tooltip("Current Editor State".to_string()), )).with_children(|left| {
            left.spawn(TextBundle::from_section("\u{f05a} ", TextStyle { font: icon_font.clone(), font_size: 14.0, color: Color::srgb(0.3, 0.6, 1.0) }));
            left.spawn((TextBundle::from_section("READY", TextStyle { font: font.clone(), font_size: 12.0, color: Color::srgb(0.7, 0.7, 0.7) }), StatusText));
        });
        p.spawn((NodeBundle { style: Style { flex_direction: FlexDirection::Row, align_items: AlignItems::Center, ..default() }, ..default() }, Interaction::default(), Tooltip("Keyboard Shortcuts & Gizmo Legend".to_string()), )).with_children(|mid| {
            mid.spawn((TextBundle::from_sections(vec![
                TextSection::new("TAB: Mode | G: Grid | R: Reset | ", TextStyle { font: font.clone(), font_size: 12.0, color: Color::srgb(0.5, 0.5, 0.5) }),
                TextSection::new("X", TextStyle { font: font.clone(), font_size: 12.0, color: Color::srgb(1.0, 0.3, 0.3) }),
                TextSection::new(":R ", TextStyle { font: font.clone(), font_size: 12.0, color: Color::srgb(0.5, 0.5, 0.5) }),
                TextSection::new("Y", TextStyle { font: font.clone(), font_size: 12.0, color: Color::srgb(0.3, 1.0, 0.3) }),
                TextSection::new(":G ", TextStyle { font: font.clone(), font_size: 12.0, color: Color::srgb(0.5, 0.5, 0.5) }),
                TextSection::new("Z", TextStyle { font: font.clone(), font_size: 12.0, color: Color::srgb(0.4, 0.4, 1.0) }),
                TextSection::new(":B", TextStyle { font: font.clone(), font_size: 12.0, color: Color::srgb(0.5, 0.5, 0.5) }),
            ]), KeyHintText));
        });
        p.spawn((NodeBundle { style: Style { flex_direction: FlexDirection::Row, align_items: AlignItems::Center, ..default() }, ..default() }, Interaction::default(), Tooltip("Total Scene Complexity".to_string()), )).with_children(|right| {
            right.spawn(TextBundle::from_section("\u{f1b2} ", TextStyle { font: icon_font.clone(), font_size: 14.0, color: Color::srgb(0.7, 0.7, 0.7) }));
            right.spawn((TextBundle::from_section("POLYS: 0", TextStyle { font: font.clone(), font_size: 12.0, color: Color::srgb(0.7, 0.7, 0.7) }), PolycountText));
        });
    });
}

#[derive(Component, Clone, Copy)]
pub enum ViewportToggleType { Grid, Slices, Sockets, Gizmos, Reset, }

#[derive(Component)]
pub struct ViewportToggleButton(pub ViewportToggleType);

pub fn viewport_button_system(
    mut interaction_query: Query<(&Interaction, &ViewportToggleButton), Changed<Interaction>>,
    mut viewport_settings: ResMut<ViewportSettings>,
    mut all_buttons: Query<(&ViewportToggleButton, &mut BackgroundColor)>,
    mut reset_events: EventWriter<super::super::ResetCameraEvent>,
) {
    for (interaction, toggle) in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            match toggle.0 {
                ViewportToggleType::Grid => viewport_settings.grid = !viewport_settings.grid,
                ViewportToggleType::Slices => viewport_settings.slices = !viewport_settings.slices,
                ViewportToggleType::Sockets => viewport_settings.sockets = !viewport_settings.sockets,
                ViewportToggleType::Gizmos => viewport_settings.gizmos = !viewport_settings.gizmos,
                ViewportToggleType::Reset => {
                    reset_events.send(super::super::ResetCameraEvent);
                }
            }
        }
    }

    for (toggle, mut bg) in all_buttons.iter_mut() {
        let active = match toggle.0 {
            ViewportToggleType::Grid => viewport_settings.grid,
            ViewportToggleType::Slices => viewport_settings.slices,
            ViewportToggleType::Sockets => viewport_settings.sockets,
            ViewportToggleType::Gizmos => viewport_settings.gizmos,
            ViewportToggleType::Reset => false,
        };
        if active { *bg = Color::srgba(0.3, 0.6, 1.0, 0.8).into(); } else { *bg = Color::srgba(0.2, 0.2, 0.2, 0.9).into(); }
    }
}


#[derive(Component)]
pub struct SlicerLockButton;
#[derive(Component)]
pub struct SlicerContainer;

pub fn spawn_viewport_slicer(parent: &mut ChildBuilder, icon_font: &Handle<Font>, initial_min: f32, initial_max: f32) {
    parent.spawn((NodeBundle { 
        style: Style { 
            position_type: PositionType::Absolute, 
            left: Val::Px(20.0), 
            top: Val::Px(150.0), 
            height: Val::Px(400.0), 
            width: Val::Px(40.0), 
            flex_direction: FlexDirection::Column, 
            align_items: AlignItems::Center, 
            padding: UiRect::vertical(Val::Px(10.0)), 
            ..default() 
        }, 
        background_color: Color::srgba(0.1, 0.1, 0.1, 0.6).into(),
        border_radius: BorderRadius::all(Val::Px(8.0)),
        ..default() 
    }, SlicerContainer, )).with_children(|p| {
        p.spawn((ButtonBundle { 
            style: Style { 
                width: Val::Px(30.0), 
                height: Val::Px(30.0), 
                justify_content: JustifyContent::Center, 
                align_items: AlignItems::Center, 
                margin: UiRect::bottom(Val::Px(10.0)), 
                ..default() 
            }, 
            background_color: Color::srgba(0.2, 0.2, 0.2, 0.9).into(), 
            border_radius: BorderRadius::all(Val::Px(6.0)), 
            ..default() 
        }, SlicerLockButton, Tooltip("Lock/Unlock Slicer (L)".to_string()), )).with_children(|btn| {
            btn.spawn(TextBundle::from_section("\u{f023}", TextStyle { font: icon_font.clone(), font_size: 16.0, color: Color::WHITE }, ));
        });
        spawn_vertical_range_slider(p, icon_font, initial_min, initial_max);
    });
}
