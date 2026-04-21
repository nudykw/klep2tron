import sys
import os

file_path = "crates/client_core/src/actor_editor/systems_logic.rs"
content = open(file_path).read()

old = """        if relative_path.ends_with(".obj") {
            pending.mesh_handle = Some(asset_server.load(relative_path));
            pending.handle = None;
        } else {
            pending.handle = Some(asset_server.load(relative_path));
            pending.mesh_handle = None;
        }"""

new = """        if relative_path.ends_with(".obj") {
            pending.mesh_handle = Some(asset_server.load(relative_path));
            pending.handle = None;
        } else {
            // For GLTF/GLB we need to specify a label to load it as a Scene
            let scene_path = format!("{}#Scene0", relative_path);
            pending.handle = Some(asset_server.load(scene_path));
            pending.mesh_handle = None;
        }"""

if old in content:
    with open(file_path, "w") as f:
        f.write(content.replace(old, new))
    print("Success")
else:
    print("Failed to find old content")
