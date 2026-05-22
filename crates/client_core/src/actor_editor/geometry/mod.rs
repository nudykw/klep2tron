pub mod slicer;
pub mod capper;
pub mod raycast;
pub mod contour_calculator;
pub mod mesh_transfer;
pub use mesh_transfer::transfer_triangles;

use bevy::prelude::*;

#[derive(Debug, Clone)]
pub struct SlicedParts {
    pub head: Option<Mesh>,
    pub body: Option<Mesh>,
    pub legs: Option<Mesh>,
    pub contours: Vec<[Vec3; 2]>, // Segments for engraving and capping
    // Кол-во оригинальных треугольников до добавления крышек (для CapTriangleRange)
    pub head_orig_tris: usize,
    pub body_orig_tris: usize,
    pub legs_orig_tris: usize,
}
