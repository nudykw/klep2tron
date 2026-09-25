use bevy::prelude::*;
use bevy::render::mesh::{Indices, VertexAttributeValues};
use std::fs::File;
use std::io::{Write, Read};
use std::path::Path;

/// Simple binary format for Klep2tron Mesh (.k2m)
/// Header: [u32: magic] [u32: version]
/// Counts: [u32: vertex_count] [u32: index_count]
/// Data:   [Positions] [Normals] [UVs] [Indices]
const K2M_MAGIC: u32 = 0x4B324D21; // "K2M!"
const K2M_VERSION: u32 = 1;

pub fn export_mesh_to_k2m(mesh: &Mesh, path: impl AsRef<Path>) -> std::io::Result<()> {
    let positions = if let Some(VertexAttributeValues::Float32x3(p)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        p
    } else {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "No positions"));
    };

    let normals = if let Some(VertexAttributeValues::Float32x3(n)) = mesh.attribute(Mesh::ATTRIBUTE_NORMAL) {
        n
    } else {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "No normals"));
    };

    let uvs = if let Some(VertexAttributeValues::Float32x2(u)) = mesh.attribute(Mesh::ATTRIBUTE_UV_0) {
        u
    } else {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "No UVs"));
    };

    let indices = match mesh.indices() {
        Some(Indices::U32(idx)) => idx.clone(),
        Some(Indices::U16(idx)) => idx.iter().map(|&i| i as u32).collect(),
        None => (0..positions.len() as u32).collect(),
    };

    let mut file = File::create(path)?;
    
    // Header
    file.write_all(&K2M_MAGIC.to_le_bytes())?;
    file.write_all(&K2M_VERSION.to_le_bytes())?;
    
    // Counts
    file.write_all(&(positions.len() as u32).to_le_bytes())?;
    file.write_all(&(indices.len() as u32).to_le_bytes())?;
    
    // Data - Positions
    for p in positions {
        for val in p { file.write_all(&val.to_le_bytes())?; }
    }
    
    // Data - Normals
    for n in normals {
        for val in n { file.write_all(&val.to_le_bytes())?; }
    }
    
    // Data - UVs
    for u in uvs {
        for val in u { file.write_all(&val.to_le_bytes())?; }
    }
    
    // Data - Indices
    for i in indices {
        file.write_all(&i.to_le_bytes())?;
    }
    
    Ok(())
}

pub fn import_mesh_from_k2m(path: impl AsRef<Path>) -> std::io::Result<Mesh> {
    let mut file = File::open(path)?;
    let mut buf = [0u8; 4];
    
    // Magic
    file.read_exact(&mut buf)?;
    if u32::from_le_bytes(buf) != K2M_MAGIC {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid magic"));
    }
    
    // Version
    file.read_exact(&mut buf)?;
    
    // Counts
    file.read_exact(&mut buf)?;
    let vertex_count = u32::from_le_bytes(buf) as usize;
    file.read_exact(&mut buf)?;
    let index_count = u32::from_le_bytes(buf) as usize;
    
    // Positions
    let mut positions = Vec::with_capacity(vertex_count);
    for _ in 0..vertex_count {
        let mut p = [0.0f32; 3];
        for i in 0..3 {
            file.read_exact(&mut buf)?;
            p[i] = f32::from_le_bytes(buf);
        }
        positions.push(p);
    }
    
    // Normals
    let mut normals = Vec::with_capacity(vertex_count);
    for _ in 0..vertex_count {
        let mut n = [0.0f32; 3];
        for i in 0..3 {
            file.read_exact(&mut buf)?;
            n[i] = f32::from_le_bytes(buf);
        }
        normals.push(n);
    }
    
    // UVs
    let mut uvs = Vec::with_capacity(vertex_count);
    for _ in 0..vertex_count {
        let mut u = [0.0f32; 2];
        for i in 0..2 {
            file.read_exact(&mut buf)?;
            u[i] = f32::from_le_bytes(buf);
        }
        uvs.push(u);
    }
    
    // Indices
    let mut indices = Vec::with_capacity(index_count);
    for _ in 0..index_count {
        file.read_exact(&mut buf)?;
        indices.push(u32::from_le_bytes(buf));
    }
    
    let mut mesh = Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList, bevy::render::render_asset::RenderAssetUsages::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    
    Ok(mesh)
}
