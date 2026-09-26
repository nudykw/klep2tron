//! Scene introspection (`/scene_tree`, `/entity`, `/mesh`, `/material`) and
//! target-addressed screenshots (`/screenshot?view=`).

use bevy::camera::RenderTarget;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::render::view::screenshot::Screenshot;
use bevy::window::WindowRef;
use std::collections::HashMap;
use std::sync::mpsc::Sender;

use super::http::Response;

/// Components reported per entity. Kept at 11 entries (plus `Entity`) to stay
/// within Bevy's query-tuple limit; `RenderTarget` is queried separately.
pub(super) type SceneItem = (
    Entity,
    Option<&'static Name>,
    Option<&'static ChildOf>,
    Option<&'static Transform>,
    Option<&'static Mesh3d>,
    Option<&'static MeshMaterial3d<StandardMaterial>>,
    Option<&'static Camera3d>,
    Option<&'static Camera2d>,
    Option<&'static DirectionalLight>,
    Option<&'static Node>,
    Option<&'static Text>,
);

#[derive(SystemParam)]
pub(super) struct ControlScene<'w, 's> {
    scene: Query<'w, 's, SceneItem>,
    render_targets: Query<'w, 's, (Entity, &'static RenderTarget)>,
    meshes: Res<'w, Assets<Mesh>>,
    materials: Res<'w, Assets<StandardMaterial>>,
}

fn kind_of(
    cam3: bool,
    cam2: bool,
    mesh: bool,
    node: bool,
    text: bool,
    light: bool,
) -> &'static str {
    if cam3 || cam2 {
        "camera"
    } else if mesh {
        "mesh"
    } else if node {
        "ui_node"
    } else if text {
        "ui_text"
    } else if light {
        "light"
    } else {
        "entity"
    }
}

impl ControlScene<'_, '_> {
    /// Spawn a screenshot of the requested target and forward the PNG on `resp`.
    ///
    /// `view` is `primary` (default), `camera:<index>`, `rtt:<index>` or a bare
    /// entity index. Image-render-target cameras work offscreen, so they can be
    /// captured even when the window is occluded.
    pub(super) fn capture(&self, commands: &mut Commands, view: &Option<String>, resp: Sender<Response>) {
        let shot = match self.resolve(view) {
            Ok(shot) => shot,
            Err(msg) => {
                let _ = resp.send(Response::text(400, msg));
                return;
            }
        };
        commands.spawn(shot).observe(
            move |captured: On<bevy::render::view::screenshot::ScreenshotCaptured>| {
                let image = captured.image.clone();
                let body = match image.try_into_dynamic() {
                    Ok(dyn_img) => {
                        let mut buf = std::io::Cursor::new(Vec::new());
                        match dyn_img.write_to(&mut buf, image::ImageFormat::Png) {
                            Ok(()) => Response::png(buf.into_inner()),
                            Err(e) => Response::text(500, format!("png encode failed: {e}")),
                        }
                    }
                    Err(e) => Response::text(500, format!("image convert failed: {e}")),
                };
                let _ = resp.send(body);
            },
        );
    }

    fn resolve(&self, view: &Option<String>) -> Result<Screenshot, String> {
        let raw = view.as_deref().unwrap_or("primary").trim();
        if raw.is_empty() || raw.eq_ignore_ascii_case("primary") || raw.eq_ignore_ascii_case("window")
        {
            return Ok(Screenshot::primary_window());
        }
        let id_str = raw
            .strip_prefix("camera:")
            .or_else(|| raw.strip_prefix("rtt:"))
            .unwrap_or(raw);
        let id: u32 = id_str.parse().map_err(|_| format!("bad view: {raw}"))?;
        for (entity, target) in self.render_targets.iter() {
            if entity.index().index() != id {
                continue;
            }
            return match target {
                RenderTarget::Image(img) => Ok(Screenshot::image(img.handle.clone())),
                RenderTarget::Window(WindowRef::Primary) => Ok(Screenshot::primary_window()),
                RenderTarget::Window(WindowRef::Entity(window)) => Ok(Screenshot::window(*window)),
                _ => Err(format!("camera {id} has no captureable target")),
            };
        }
        Err(format!("no camera with entity index {id}"))
    }

    pub(super) fn tree(&self, root: Option<u32>, depth: usize) -> String {
        use serde_json::{json, Value};

        let max_depth = depth.clamp(1, 12);
        let mut names: HashMap<u32, Option<String>> = HashMap::new();
        let mut kinds: HashMap<u32, &'static str> = HashMap::new();
        let mut children: HashMap<u32, Vec<u32>> = HashMap::new();
        let mut parent_of: HashMap<u32, u32> = HashMap::new();

        for (entity, name, parent, _t, mesh, _mat, cam3, cam2, light, node, text) in
            self.scene.iter()
        {
            let id = entity.index().index();
            names.insert(id, name.map(|n| n.as_str().to_string()));
            kinds.insert(
                id,
                kind_of(cam3.is_some(), cam2.is_some(), mesh.is_some(), node.is_some(), text.is_some(), light.is_some()),
            );
            if let Some(parent) = parent {
                let pid = parent.0.index().index();
                children.entry(pid).or_default().push(id);
                parent_of.insert(id, pid);
            }
        }

        let roots: Vec<u32> = match root {
            Some(id) => vec![id],
            None => names.keys().copied().filter(|id| !parent_of.contains_key(id)).collect(),
        };

        let mut budget = 1024usize;
        let roots: Vec<Value> = roots
            .iter()
            .filter_map(|id| {
                if budget == 0 {
                    None
                } else {
                    Some(node_json(*id, 1, max_depth, &children, &names, &kinds, &mut budget))
                }
            })
            .collect();

        json!({
            "roots": roots,
            "depth": max_depth,
            "entity_count": names.len(),
            "truncated": budget == 0,
        })
        .to_string()
    }

    pub(super) fn entity(&self, id: u32) -> Result<String, String> {
        use serde_json::json;

        for (entity, name, parent, transform, mesh, mat, cam3, cam2, light, node, text) in
            self.scene.iter()
        {
            if entity.index().index() != id {
                continue;
            }
            let mut components = Vec::new();
            if transform.is_some() {
                components.push("Transform");
            }
            if mesh.is_some() {
                components.push("Mesh3d");
            }
            if mat.is_some() {
                components.push("MeshMaterial3d<StandardMaterial>");
            }
            if cam3.is_some() {
                components.push("Camera3d");
            }
            if cam2.is_some() {
                components.push("Camera2d");
            }
            if light.is_some() {
                components.push("DirectionalLight");
            }
            if node.is_some() {
                components.push("Node");
            }
            if text.is_some() {
                components.push("Text");
            }
            if parent.is_some() {
                components.push("ChildOf");
            }

            return Ok(json!({
                "entity": id,
                "name": name.map(|n| n.as_str().to_string()),
                "parent": parent.map(|p| p.0.index().index()),
                "kind": kind_of(cam3.is_some(), cam2.is_some(), mesh.is_some(), node.is_some(), text.is_some(), light.is_some()),
                "components": components,
                "transform": transform.map(|t| json!({
                    "translation": [t.translation.x, t.translation.y, t.translation.z],
                    "rotation": [t.rotation.x, t.rotation.y, t.rotation.z, t.rotation.w],
                    "scale": [t.scale.x, t.scale.y, t.scale.z],
                })),
                "mesh": mesh.map(|m| format!("{:?}", m.0.id())),
                "material": mat.map(|m| format!("{:?}", m.0.id())),
                "text": text.map(|t| t.0.clone()),
                "render_target": self.render_target_json(id),
            })
            .to_string());
        }
        Err(format!("no entity with index {id}"))
    }

    pub(super) fn mesh(&self, id: u32) -> Result<String, String> {
        use serde_json::json;

        let handle = self
            .scene
            .iter()
            .find(|item| item.0.index().index() == id)
            .and_then(|item| item.4.as_ref().map(|m| m.0.clone()))
            .ok_or_else(|| format!("entity {id} has no Mesh3d"))?;
        let mesh = self
            .meshes
            .get(&handle)
            .ok_or_else(|| format!("mesh asset for entity {id} is not loaded"))?;

        let mut attributes = Vec::new();
        let mut vertex_count: Option<usize> = None;
        if let Ok(iter) = mesh.try_attributes() {
            for (attribute, values) in iter {
                let count = values.len();
                vertex_count = Some(vertex_count.map_or(count, |c: usize| c.min(count)));
                attributes.push(json!({ "name": attribute.name, "count": count }));
            }
        }

        Ok(json!({
            "entity": id,
            "handle": format!("{:?}", handle.id()),
            "attributes_available": vertex_count.is_some(),
            "vertex_count": vertex_count,
            "index_count": mesh.indices().map(|i| i.len()),
            "topology": format!("{:?}", mesh.primitive_topology()),
            "attributes": attributes,
        })
        .to_string())
    }

    pub(super) fn material(&self, id: u32) -> Result<String, String> {
        use serde_json::json;

        let handle = self
            .scene
            .iter()
            .find(|item| item.0.index().index() == id)
            .and_then(|item| item.5.as_ref().map(|m| m.0.clone()))
            .ok_or_else(|| format!("entity {id} has no MeshMaterial3d<StandardMaterial>"))?;
        let material = self
            .materials
            .get(&handle)
            .ok_or_else(|| format!("material asset for entity {id} is not loaded"))?;

        let base = material.base_color.to_srgba();
        let emissive = material.emissive;
        Ok(json!({
            "entity": id,
            "handle": format!("{:?}", handle.id()),
            "base_color": [base.red, base.green, base.blue, base.alpha],
            "emissive": [emissive.red, emissive.green, emissive.blue, emissive.alpha],
            "perceptual_roughness": material.perceptual_roughness,
            "metallic": material.metallic,
            "reflectance": material.reflectance,
            "unlit": material.unlit,
            "double_sided": material.double_sided,
            "cull_mode": format!("{:?}", material.cull_mode),
            "alpha_mode": format!("{:?}", material.alpha_mode),
            "base_color_texture": material.base_color_texture.as_ref().map(|t| format!("{:?}", t.id())),
            "emissive_texture": material.emissive_texture.as_ref().map(|t| format!("{:?}", t.id())),
        })
        .to_string())
    }

    fn render_target_json(&self, id: u32) -> serde_json::Value {
        use serde_json::json;

        for (entity, target) in self.render_targets.iter() {
            if entity.index().index() != id {
                continue;
            }
            return match target {
                RenderTarget::Image(img) => {
                    json!({ "kind": "image", "handle": format!("{:?}", img.handle.id()) })
                }
                RenderTarget::Window(WindowRef::Primary) => json!({ "kind": "primary_window" }),
                RenderTarget::Window(WindowRef::Entity(window)) => {
                    json!({ "kind": "window", "entity": window.index().index() })
                }
                RenderTarget::TextureView(handle) => {
                    json!({ "kind": "texture_view", "handle": format!("{handle:?}") })
                }
                RenderTarget::None { size } => json!({ "kind": "none", "size": [size.x, size.y] }),
            };
        }
        serde_json::Value::Null
    }
}

#[allow(clippy::too_many_arguments)]
fn node_json(
    id: u32,
    level: usize,
    max_depth: usize,
    children: &HashMap<u32, Vec<u32>>,
    names: &HashMap<u32, Option<String>>,
    kinds: &HashMap<u32, &'static str>,
    budget: &mut usize,
) -> serde_json::Value {
    use serde_json::{json, Value};

    *budget = budget.saturating_sub(1);
    let mut node = json!({
        "entity": id,
        "name": names.get(&id).cloned().flatten(),
        "kind": kinds.get(&id).copied().unwrap_or("entity"),
    });
    let Some(kids) = children.get(&id) else {
        return node;
    };
    if level >= max_depth {
        if !kids.is_empty() {
            node["children_truncated"] = json!(kids.len());
        }
        return node;
    }
    let built: Vec<Value> = kids
        .iter()
        .filter_map(|child| {
            if *budget == 0 {
                None
            } else {
                Some(node_json(*child, level + 1, max_depth, children, names, kinds, budget))
            }
        })
        .collect();
    if !built.is_empty() {
        node["children"] = Value::Array(built);
    }
    node
}
