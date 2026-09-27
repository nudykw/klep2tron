//! `.k2m` mesh export/import — thin wrappers over [`shared::k2m`].
//!
//! The format itself lives in `shared` so the game client and the editor share
//! a single implementation (`docs/Actor_Storage_Format.md`).

use bevy::prelude::*;
use std::path::Path;

pub use shared::k2m::{K2M_MAGIC, K2M_VERSION};

/// Write a [`Mesh`] to disk in the `.k2m` format.
pub fn export_mesh_to_k2m(mesh: &Mesh, path: impl AsRef<Path>) -> std::io::Result<()> {
    shared::k2m::export_mesh_to_k2m(mesh, path)
}

/// Read a `.k2m` file from disk into a [`Mesh`].
pub fn import_mesh_from_k2m(path: impl AsRef<Path>) -> std::io::Result<Mesh> {
    shared::k2m::import_mesh_from_k2m(path)
}
