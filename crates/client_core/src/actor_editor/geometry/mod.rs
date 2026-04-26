pub mod slicer;
pub mod capper;
pub mod raycast;
pub mod contour_calculator;

use bevy::prelude::*;

#[derive(Debug, Clone)]
pub struct SlicedParts {
    pub head: Option<Mesh>,
    pub body: Option<Mesh>,
    pub legs: Option<Mesh>,
    pub contours: Vec<[Vec3; 2]>, // Segments for engraving and capping
}
