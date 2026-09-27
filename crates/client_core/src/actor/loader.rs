//! Asset loaders for actor data: `.k2m` meshes and `actor.ron` manifests.
//!
//! Registering these with the [`AssetServer`](bevy::asset::AssetServer) lets the
//! game client load baked actors as regular assets (async, path-safe, WASM-ready)
//! instead of poking at the filesystem directly.

use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, LoadContext};
use bevy::prelude::*;
use bevy::reflect::TypePath;
use shared::npc::ActorProject;

/// A parsed `actor.ron` manifest, wrapped as a Bevy [`Asset`].
#[derive(Asset, TypePath, Debug, Clone)]
pub struct ActorManifest(pub ActorProject);

/// Loads `.k2m` baked meshes into [`Mesh`] assets.
#[derive(Default, TypePath)]
pub struct K2mMeshLoader;

impl AssetLoader for K2mMeshLoader {
    type Asset = Mesh;
    type Settings = ();
    type Error = shared::k2m::K2mError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        shared::k2m::mesh_from_k2m_bytes(&bytes)
    }

    fn extensions(&self) -> &[&str] {
        &["k2m"]
    }
}

/// Loads `actor.ron` actor manifests into [`ActorManifest`] assets.
///
/// Registered for the compound extension `actor.ron`, so it does not clash with
/// other `.ron` files (e.g. `vfx/presets.ron`).
#[derive(Default, TypePath)]
pub struct ActorManifestLoader;

impl AssetLoader for ActorManifestLoader {
    type Asset = ActorManifest;
    type Settings = ();
    type Error = ActorManifestError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let project = ron::de::from_bytes::<ActorProject>(&bytes)?;
        Ok(ActorManifest(project))
    }

    fn extensions(&self) -> &[&str] {
        &["actor.ron"]
    }
}

/// Errors produced while loading an `actor.ron` manifest.
#[derive(Debug)]
#[non_exhaustive]
pub enum ActorManifestError {
    /// Underlying I/O failure while reading the file.
    Io(std::io::Error),
    /// The RON content could not be parsed into [`ActorProject`].
    Ron(ron::error::SpannedError),
}

impl std::fmt::Display for ActorManifestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActorManifestError::Io(err) => write!(f, "i/o error: {err}"),
            ActorManifestError::Ron(err) => write!(f, "invalid actor manifest: {err}"),
        }
    }
}

impl std::error::Error for ActorManifestError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ActorManifestError::Io(err) => Some(err),
            ActorManifestError::Ron(err) => Some(err),
        }
    }
}

impl From<std::io::Error> for ActorManifestError {
    fn from(err: std::io::Error) -> Self {
        ActorManifestError::Io(err)
    }
}

impl From<ron::error::SpannedError> for ActorManifestError {
    fn from(err: ron::error::SpannedError) -> Self {
        ActorManifestError::Ron(err)
    }
}

