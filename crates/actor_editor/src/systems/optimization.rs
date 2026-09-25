use bevy::prelude::*;
use bevy::render::mesh::{Indices, VertexAttributeValues};
use super::super::{SlicingSettings};
use super::undo_redo::Command;

#[derive(Resource)]
pub struct OptimizationSettings {
    pub target_triangles: usize,
    pub is_optimized: bool,
    pub wireframe: bool,
}

impl Default for OptimizationSettings {
    fn default() -> Self {
        Self {
            target_triangles: 15000,
            is_optimized: false,
            wireframe: false,
        }
    }
}

#[derive(Component)]
pub struct OptimizedMeshComponent(pub Handle<Mesh>);

#[derive(Resource, Default)]
pub struct OptimizationTask(pub Option<bevy::tasks::Task<OptimizationResult>>);

pub struct OptimizationResult {
    pub entity: Entity,
    pub original_mesh_handle: Handle<Mesh>,
    pub new_mesh: Option<Mesh>,
    pub target_tris: usize,
}

pub struct OptimizeMeshCommand {
    pub entity: Entity,
    pub old_mesh: Handle<Mesh>,
    pub new_mesh: Handle<Mesh>,
    pub target_tris: usize,
}

impl Command for OptimizeMeshCommand {
    fn name(&self) -> String { format!("Optimize Mesh ({} tris)", self.target_tris) }
    
    fn execute(&self, world: &mut World) {
        if let Some(mut entity) = world.get_entity_mut(self.entity) {
            entity.insert(OptimizedMeshComponent(self.new_mesh.clone()));
        }
        if let Some(mut opt_settings) = world.get_resource_mut::<OptimizationSettings>() {
            opt_settings.is_optimized = true;
        }
        if let Some(mut slicing) = world.get_resource_mut::<SlicingSettings>() {
            slicing.trigger_slice = true;
        }
    }
    
    fn undo(&self, world: &mut World) {
        if let Some(mut entity) = world.get_entity_mut(self.entity) {
            entity.remove::<OptimizedMeshComponent>();
        }
        if let Some(mut opt_settings) = world.get_resource_mut::<OptimizationSettings>() {
            opt_settings.is_optimized = false;
        }
        if let Some(mut slicing) = world.get_resource_mut::<SlicingSettings>() {
            slicing.trigger_slice = true;
        }
    }
}

pub fn perform_mesh_optimization(
    mesh: &Mesh,
    target_tris: usize,
) -> Option<Mesh> {
    let positions = if let Some(VertexAttributeValues::Float32x3(p)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        p
    } else {
        return None;
    };
    
    let indices = match mesh.indices() {
        Some(Indices::U16(i)) => i.iter().map(|&x| x as u32).collect::<Vec<u32>>(),
        Some(Indices::U32(i)) => i.clone().into_iter().collect::<Vec<u32>>(),
        None => (0..positions.len() as u32).collect::<Vec<u32>>(),
    };

    let target_index_count = (target_tris * 3).min(indices.len());
    let pos_data: Vec<[f32; 3]> = positions.iter().map(|p| *p).collect();
    
    // --- Step 1: Welding (Merging duplicate vertices) ---
    // meshopt works best when vertices are shared. Many OBJ/GLB loaders might produce unwelded meshes.
    let (weld_vertex_count, weld_remap) = meshopt::generate_vertex_remap(&pos_data, Some(&indices));
    let welded_indices = meshopt::remap_index_buffer(Some(&indices), weld_vertex_count, &weld_remap);
    let welded_pos = meshopt::remap_vertex_buffer(&pos_data, weld_vertex_count, &weld_remap);
    
    info!("Optimization: Welded vertices {} -> {}", pos_data.len(), weld_vertex_count);
    
    // --- Step 2: Simplification ---
    let pos_bytes = meshopt::utilities::typed_to_bytes(&welded_pos);
    let adapter = meshopt::VertexDataAdapter::new(pos_bytes, 12, 0).expect("Failed to create adapter");
    
    info!("Optimization: Target tris={}, Current indices={}, Target indices={}", target_tris, welded_indices.len(), target_index_count);
    
    // Use a slightly more relaxed threshold if needed, and try to reach the target
    let mut new_indices = meshopt::simplify(&welded_indices, &adapter, target_index_count, 0.05, meshopt::SimplifyOptions::None, None);
    
    // If regular simplify failed to reduce much, try sloppy simplify (aggressive)
    if new_indices.len() > target_index_count * 2 && new_indices.len() > 100 {
        info!("Optimization: Regular simplify insufficient, trying sloppy simplify...");
        new_indices = meshopt::simplify_sloppy(&welded_indices, &adapter, target_index_count, 0.1, None);
    }

    info!("Optimization: New indices count={}", new_indices.len());
    
    if new_indices.len() >= indices.len() && new_indices.len() > 0 {
        return None; 
    }

    // --- Step 3: Vertex Optimization (Remapping to final buffer) ---
    // We use the welded_pos as the base now
    let (vertex_count, remap) = meshopt::generate_vertex_remap(&welded_pos, Some(&new_indices));
    let opt_indices = meshopt::remap_index_buffer(Some(&new_indices), vertex_count, &remap);
    
    let mut new_mesh = Mesh::new(mesh.primitive_topology(), mesh.asset_usage);
    new_mesh.insert_indices(Indices::U32(opt_indices));
    
    // Remap all attributes from the ORIGINAL mesh, but first we need to weld them too
    for (id, values) in mesh.attributes() {
        let remapped = match values {
            VertexAttributeValues::Float32x3(v) => {
                let welded = meshopt::remap_vertex_buffer(v, weld_vertex_count, &weld_remap);
                let opt = meshopt::remap_vertex_buffer(&welded, vertex_count, &remap);
                VertexAttributeValues::Float32x3(opt)
            }
            VertexAttributeValues::Float32x2(v) => {
                let welded = meshopt::remap_vertex_buffer(v, weld_vertex_count, &weld_remap);
                let opt = meshopt::remap_vertex_buffer(&welded, vertex_count, &remap);
                VertexAttributeValues::Float32x2(opt)
            }
            VertexAttributeValues::Float32x4(v) => {
                let welded = meshopt::remap_vertex_buffer(v, weld_vertex_count, &weld_remap);
                let opt = meshopt::remap_vertex_buffer(&welded, vertex_count, &remap);
                VertexAttributeValues::Float32x4(opt)
            }
            _ => continue,
        };

        // Match common IDs to their corresponding MeshVertexAttribute
        let attr = if id == Mesh::ATTRIBUTE_POSITION.id {
            Mesh::ATTRIBUTE_POSITION
        } else if id == Mesh::ATTRIBUTE_NORMAL.id {
            Mesh::ATTRIBUTE_NORMAL
        } else if id == Mesh::ATTRIBUTE_UV_0.id {
            Mesh::ATTRIBUTE_UV_0
        } else if id == Mesh::ATTRIBUTE_COLOR.id {
            Mesh::ATTRIBUTE_COLOR
        } else {
            continue;
        };

        new_mesh.insert_attribute(attr, remapped);
    }
    
    Some(new_mesh)
}
