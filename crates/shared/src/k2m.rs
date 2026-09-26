//! Klep2tron Mesh (`.k2m`) — high-performance binary mesh format.
//!
//! This module is the single source of truth for the format: both the actor
//! editor and the game client build/parse `.k2m` files through it. See
//! `docs/Actor_Storage_Format.md` for the specification.
//!
//! Layout (little endian):
//! `[u32 magic] [u32 version] [u32 vertex_count] [u32 index_count]`
//! `[positions: f32x3 * N] [normals: f32x3 * N] [uvs: f32x2 * N] [indices: u32 * M]`

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};
use std::io::{Cursor, Read};
#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;

/// Magic number: `"K2M!"` in little endian.
pub const K2M_MAGIC: u32 = 0x4B324D21;
/// Current `.k2m` format version.
pub const K2M_VERSION: u32 = 1;

/// Errors produced while parsing or serializing a `.k2m` mesh.
#[derive(Debug)]
#[non_exhaustive]
pub enum K2mError {
    /// The file did not start with [`K2M_MAGIC`].
    InvalidMagic([u8; 4]),
    /// The file version is not [`K2M_VERSION`].
    UnsupportedVersion(u32),
    /// Reading past the end of the buffer or an underlying I/O failure.
    Io(std::io::Error),
    /// A mesh was missing an attribute required by the format.
    MissingAttribute(&'static str),
}

impl std::fmt::Display for K2mError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            K2mError::InvalidMagic(bytes) => write!(f, "invalid .k2m magic: {bytes:?}"),
            K2mError::UnsupportedVersion(version) => {
                write!(f, "unsupported .k2m version: {version}")
            }
            K2mError::Io(err) => write!(f, "i/o error: {err}"),
            K2mError::MissingAttribute(name) => write!(f, "mesh is missing `{name}`"),
        }
    }
}

impl std::error::Error for K2mError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            K2mError::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for K2mError {
    fn from(err: std::io::Error) -> Self {
        K2mError::Io(err)
    }
}

/// Parse a `.k2m` byte buffer into a Bevy [`Mesh`].
pub fn mesh_from_k2m_bytes(bytes: &[u8]) -> Result<Mesh, K2mError> {
    let mut cursor = Cursor::new(bytes);

    let magic = read_u32(&mut cursor)?;
    if magic != K2M_MAGIC {
        return Err(K2mError::InvalidMagic(magic.to_le_bytes()));
    }

    let version = read_u32(&mut cursor)?;
    if version != K2M_VERSION {
        return Err(K2mError::UnsupportedVersion(version));
    }

    let vertex_count = read_u32(&mut cursor)? as usize;
    let index_count = read_u32(&mut cursor)? as usize;

    let mut positions = Vec::with_capacity(vertex_count);
    for _ in 0..vertex_count {
        positions.push([read_f32(&mut cursor)?, read_f32(&mut cursor)?, read_f32(&mut cursor)?]);
    }

    let mut normals = Vec::with_capacity(vertex_count);
    for _ in 0..vertex_count {
        normals.push([read_f32(&mut cursor)?, read_f32(&mut cursor)?, read_f32(&mut cursor)?]);
    }

    let mut uvs = Vec::with_capacity(vertex_count);
    for _ in 0..vertex_count {
        uvs.push([read_f32(&mut cursor)?, read_f32(&mut cursor)?]);
    }

    let mut indices = Vec::with_capacity(index_count);
    for _ in 0..index_count {
        indices.push(read_u32(&mut cursor)?);
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));

    Ok(mesh)
}

/// Serialize a Bevy [`Mesh`] into the `.k2m` byte layout.
pub fn mesh_to_k2m_bytes(mesh: &Mesh) -> Result<Vec<u8>, K2mError> {
    let positions = match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        Some(VertexAttributeValues::Float32x3(values)) => values,
        _ => return Err(K2mError::MissingAttribute("POSITION")),
    };
    let normals = match mesh.attribute(Mesh::ATTRIBUTE_NORMAL) {
        Some(VertexAttributeValues::Float32x3(values)) => values,
        _ => return Err(K2mError::MissingAttribute("NORMAL")),
    };
    let uvs = match mesh.attribute(Mesh::ATTRIBUTE_UV_0) {
        Some(VertexAttributeValues::Float32x2(values)) => values,
        _ => return Err(K2mError::MissingAttribute("UV_0")),
    };

    let indices: Vec<u32> = match mesh.indices() {
        Some(Indices::U32(idx)) => idx.clone(),
        Some(Indices::U16(idx)) => idx.iter().map(|&i| i as u32).collect(),
        None => (0..positions.len() as u32).collect(),
    };

    let mut out = Vec::with_capacity(16 + positions.len() * 32 + indices.len() * 4);
    out.extend_from_slice(&K2M_MAGIC.to_le_bytes());
    out.extend_from_slice(&K2M_VERSION.to_le_bytes());
    out.extend_from_slice(&(positions.len() as u32).to_le_bytes());
    out.extend_from_slice(&(indices.len() as u32).to_le_bytes());

    for position in positions {
        for value in position {
            out.extend_from_slice(&value.to_le_bytes());
        }
    }
    for normal in normals {
        for value in normal {
            out.extend_from_slice(&value.to_le_bytes());
        }
    }
    for uv in uvs {
        for value in uv {
            out.extend_from_slice(&value.to_le_bytes());
        }
    }
    for index in indices {
        out.extend_from_slice(&index.to_le_bytes());
    }

    Ok(out)
}

/// Read a `.k2m` file from disk into a [`Mesh`] (native only).
#[cfg(not(target_arch = "wasm32"))]
pub fn import_mesh_from_k2m(path: impl AsRef<Path>) -> std::io::Result<Mesh> {
    let bytes = std::fs::read(path)?;
    mesh_from_k2m_bytes(&bytes).map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))
}

/// Write a [`Mesh`] to disk in the `.k2m` format (native only).
#[cfg(not(target_arch = "wasm32"))]
pub fn export_mesh_to_k2m(mesh: &Mesh, path: impl AsRef<Path>) -> std::io::Result<()> {
    let bytes = mesh_to_k2m_bytes(mesh)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))?;
    std::fs::write(path, bytes)
}

fn read_u32(reader: &mut impl Read) -> Result<u32, K2mError> {
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf))
}

fn read_f32(reader: &mut impl Read) -> Result<f32, K2mError> {
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    Ok(f32::from_le_bytes(buf))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn triangle() -> Mesh {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 0.0, 1.0]; 3]);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]]);
        mesh.insert_indices(Indices::U32(vec![0, 1, 2]));
        mesh
    }

    #[test]
    fn round_trip_preserves_geometry() {
        let original = triangle();
        let bytes = mesh_to_k2m_bytes(&original).expect("serialize");
        let restored = mesh_from_k2m_bytes(&bytes).expect("parse");

        assert_eq!(
            restored.attribute(Mesh::ATTRIBUTE_POSITION),
            original.attribute(Mesh::ATTRIBUTE_POSITION)
        );
        assert_eq!(
            restored.attribute(Mesh::ATTRIBUTE_UV_0),
            original.attribute(Mesh::ATTRIBUTE_UV_0)
        );
        assert_eq!(restored.indices(), original.indices());
    }

    #[test]
    fn rejects_bad_magic() {
        let bytes = mesh_to_k2m_bytes(&triangle()).expect("serialize");
        let mut corrupted = bytes.clone();
        corrupted[0] ^= 0xFF;
        assert!(matches!(
            mesh_from_k2m_bytes(&corrupted),
            Err(K2mError::InvalidMagic(_))
        ));
    }
}

