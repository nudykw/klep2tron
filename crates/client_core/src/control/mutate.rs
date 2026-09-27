//! Mutation actions (fixtures without clicking): `SetTransform`,
//! `DespawnEntity`, `SpawnEntity`.
//!
//! Spawned fixtures are plain entities (not `MapEntity`), so they survive map
//! rebuilds and can be despawned by their entity index. The last spawned id is
//! published as `last_spawned` in `GET /state`.

use bevy::prelude::*;
use serde_json::Value;

use crate::ClientAssets;

use super::state::{ControlAction, ControlExtras};

fn vec3(value: &Value) -> Option<Vec3> {
    let array = value.as_array()?;
    if array.len() < 3 {
        return None;
    }
    Some(Vec3::new(
        array[0].as_f64()? as f32,
        array[1].as_f64()? as f32,
        array[2].as_f64()? as f32,
    ))
}

fn vec4(value: &Value) -> Option<Vec4> {
    let array = value.as_array()?;
    if array.len() < 4 {
        return None;
    }
    Some(Vec4::new(
        array[0].as_f64()? as f32,
        array[1].as_f64()? as f32,
        array[2].as_f64()? as f32,
        array[3].as_f64()? as f32,
    ))
}

fn quat(value: &Value) -> Option<Quat> {
    let array = value.as_array()?;
    if array.len() < 4 {
        return None;
    }
    Some(Quat::from_xyzw(
        array[0].as_f64()? as f32,
        array[1].as_f64()? as f32,
        array[2].as_f64()? as f32,
        array[3].as_f64()? as f32,
    ))
}

fn euler_deg(value: &Value) -> Option<Quat> {
    let deg = vec3(value)?;
    Some(Quat::from_euler(
        EulerRot::XYZ,
        deg.x.to_radians(),
        deg.y.to_radians(),
        deg.z.to_radians(),
    ))
}

fn entity_id(args: &Value) -> Option<u32> {
    args.get("id").and_then(|v| v.as_u64()).map(|v| v as u32)
}

pub(super) fn handle_mutations(
    mut actions: MessageReader<ControlAction>,
    mut commands: Commands,
    mut transforms: Query<(Entity, &mut Transform)>,
    assets: Res<ClientAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut extras: Option<ResMut<ControlExtras>>,
) {
    for action in actions.read() {
        match action.name.as_str() {
            "SetTransform" => {
                let Some(id) = entity_id(&action.args) else { continue };
                let relative = action
                    .args
                    .get("relative")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if let Some((_, mut transform)) =
                    transforms.iter_mut().find(|(entity, _)| entity.index().index() == id)
                {
                    if let Some(translation) = action.args.get("translation").and_then(vec3) {
                        transform.translation = if relative {
                            transform.translation + translation
                        } else {
                            translation
                        };
                    }
                    if let Some(scale) = action.args.get("scale").and_then(vec3) {
                        transform.scale = if relative {
                            transform.scale + scale
                        } else {
                            scale
                        };
                    }
                    if let Some(rotation) = action.args.get("rotation").and_then(quat) {
                        transform.rotation = rotation;
                    } else if let Some(rotation) = action.args.get("rotation_euler_deg").and_then(euler_deg) {
                        transform.rotation = rotation;
                    }
                }
            }
            "DespawnEntity" => {
                let Some(id) = entity_id(&action.args) else { continue };
                if let Some(entity) = transforms
                    .iter()
                    .find(|(entity, _)| entity.index().index() == id)
                    .map(|(entity, _)| entity)
                {
                    commands.entity(entity).despawn();
                }
            }
            "SpawnEntity" => {
                let mesh_name = action
                    .args
                    .get("mesh")
                    .and_then(|v| v.as_str())
                    .unwrap_or("cube")
                    .to_ascii_lowercase();
                let mesh = if matches!(mesh_name.as_str(), "wedge" | "wedge_n" | "wedgen") {
                    assets.wedge_mesh.clone()
                } else {
                    assets.cube_mesh.clone()
                };

                let material = if action.args.get("material").and_then(|v| v.as_str()) == Some("highlight")
                {
                    assets.highlight_material.clone()
                } else {
                    let color = action
                        .args
                        .get("color")
                        .and_then(vec4)
                        .map(|c| Color::srgba(c.x, c.y, c.z, c.w))
                        .unwrap_or(Color::srgb(1.0, 0.0, 1.0));
                    materials.add(StandardMaterial { base_color: color, ..default() })
                };

                let mut transform = Transform::from_translation(
                    action.args.get("translation").and_then(vec3).unwrap_or(Vec3::ZERO),
                );
                if let Some(scale) = action.args.get("scale").and_then(vec3) {
                    transform.scale = scale;
                }
                if let Some(rotation) = action.args.get("rotation_euler_deg").and_then(euler_deg) {
                    transform.rotation = rotation;
                }

                let mut entity = commands.spawn((Mesh3d(mesh), MeshMaterial3d(material), transform));
                if let Some(name) = action.args.get("name").and_then(|v| v.as_str()) {
                    entity.insert(Name::new(name.to_string()));
                }
                let id = entity.id().index().index();
                if let Some(extras) = extras.as_mut() {
                    extras.0.insert("last_spawned".into(), serde_json::json!(id));
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_vectors_and_quaternions() {
        let v = serde_json::json!([1.0, 2.0, 3.0]);
        assert_eq!(vec3(&v), Some(Vec3::new(1.0, 2.0, 3.0)));
        assert_eq!(vec3(&serde_json::json!([1.0, 2.0])), None);
        assert_eq!(vec4(&serde_json::json!([1.0, 2.0, 3.0, 4.0])), Some(Vec4::new(1.0, 2.0, 3.0, 4.0)));
        assert!(quat(&serde_json::json!([0.0, 0.0, 0.0, 1.0])).is_some());
        assert!(euler_deg(&serde_json::json!([0.0, 90.0, 0.0])).is_some());
        assert_eq!(entity_id(&serde_json::json!({"id": 42})), Some(42));
        assert_eq!(entity_id(&serde_json::json!({})), None);
    }
}
