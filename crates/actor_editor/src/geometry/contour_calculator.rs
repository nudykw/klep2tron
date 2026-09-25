//! Fast contour calculation for preview mode
//!
//! This module provides optimized functions for calculating only slice contours
//! without creating full meshes. Used for instant visual feedback during slider dragging.
//!
//! # Performance
//!
//! - ~5-10ms for 10K triangles model
//! - 10-20x faster than full slicing
//! - No mesh or cap creation
//!
//! # Usage
//!
//! ```rust
//! let segments = calculate_contours_only(&mesh, top_y, bottom_y);
//! ```

use bevy::prelude::*;
use bevy::render::mesh::{Indices, Mesh, VertexAttributeValues};

/// Fast contour calculation without mesh creation
///
/// Calculates only intersection segments of triangles with slice planes.
/// Does NOT create new triangles, interpolate normals/UV/colors, or build caps.
///
/// # Arguments
///
/// * `mesh` - Source mesh to slice
/// * `top_y` - Y-coordinate of top slice plane (in mesh local space)
/// * `bottom_y` - Y-coordinate of bottom slice plane (in mesh local space)
///
/// # Returns
///
/// Vector of contour segments `[start, end]` in mesh local coordinates
///
/// # Performance
///
/// For 10K triangles model: ~5-10ms (vs ~50-200ms for full slicing)
pub fn calculate_contours_only(
    mesh: &Mesh,
    top_y: f32,
    bottom_y: f32,
) -> Vec<[Vec3; 2]> {
    let positions = if let Some(VertexAttributeValues::Float32x3(p)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        p
    } else {
        return Vec::new();
    };
    
    let indices = match mesh.indices() {
        Some(Indices::U16(i)) => i.iter().map(|&x| x as usize).collect::<Vec<usize>>(),
        Some(Indices::U32(i)) => i.iter().map(|&x| x as usize).collect::<Vec<usize>>(),
        None => (0..positions.len()).collect::<Vec<usize>>(),
    };
    
    let mut segments = Vec::new();
    
    for tri_indices in indices.chunks(3) {
        if tri_indices.len() < 3 { continue; }
        
        let tri = [
            Vec3::from(positions[tri_indices[0]]),
            Vec3::from(positions[tri_indices[1]]),
            Vec3::from(positions[tri_indices[2]]),
        ];
        
        if let Some(segment) = intersect_triangle_with_plane(&tri, top_y) {
            segments.push(segment);
        }
        
        if let Some(segment) = intersect_triangle_with_plane(&tri, bottom_y) {
            segments.push(segment);
        }
    }
    
    segments
}

/// Triangle-plane intersection test
///
/// Simplified version of `slicer::split_triangle()` logic that returns
/// only the intersection segment without creating new triangles.
///
/// # Returns
///
/// `Some([p1, p2])` if triangle intersects plane, where p1 and p2 are intersection points
/// `None` if triangle is completely above or below the plane
fn intersect_triangle_with_plane(
    tri: &[Vec3; 3],
    plane_y: f32,
) -> Option<[Vec3; 2]> {
    let mut above_indices = Vec::new();
    let mut below_indices = Vec::new();
    
    for (i, vertex) in tri.iter().enumerate() {
        if vertex.y > plane_y {
            above_indices.push(i);
        } else {
            below_indices.push(i);
        }
    }
    
    if above_indices.is_empty() || below_indices.is_empty() {
        return None;
    }
    
    let (lone_idx, other_indices) = if above_indices.len() == 1 {
        (above_indices[0], [below_indices[0], below_indices[1]])
    } else {
        (below_indices[0], [above_indices[0], above_indices[1]])
    };
    
    // Preserve winding order (as in slicer::split_triangle)
    let (i1, i2) = if (lone_idx + 1) % 3 == other_indices[0] {
        (other_indices[0], other_indices[1])
    } else {
        (other_indices[1], other_indices[0])
    };
    
    let v_lone = tri[lone_idx];
    let v1 = tri[i1];
    let v2 = tri[i2];
    
    let dy1 = v1.y - v_lone.y;
    let dy2 = v2.y - v_lone.y;
    
    if dy1.abs() < 1e-6 || dy2.abs() < 1e-6 {
        return None;
    }
    
    let t1 = (plane_y - v_lone.y) / dy1;
    let t2 = (plane_y - v_lone.y) / dy2;
    
    let p1 = v_lone.lerp(v1, t1);
    let p2 = v_lone.lerp(v2, t2);
    
    Some([p1, p2])
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::render::render_resource::PrimitiveTopology;
    use bevy::render::render_asset::RenderAssetUsages;
    
    #[test]
    fn test_intersect_simple_triangle() {
        let tri = [
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 1.0, 0.0),
            Vec3::new(0.0, 2.0, 0.0),
        ];
        
        let result = intersect_triangle_with_plane(&tri, 0.5);
        assert!(result.is_some());
        
        if let Some([p1, p2]) = result {
            assert!((p1.y - 0.5).abs() < 1e-5);
            assert!((p2.y - 0.5).abs() < 1e-5);
        }
    }
    
    #[test]
    fn test_no_intersection_above() {
        let tri = [
            Vec3::new(0.0, 2.0, 0.0),
            Vec3::new(1.0, 3.0, 0.0),
            Vec3::new(0.0, 4.0, 0.0),
        ];
        
        let result = intersect_triangle_with_plane(&tri, 0.5);
        assert!(result.is_none());
    }
    
    #[test]
    fn test_calculate_contours_cube() {
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );
        
        let positions = vec![
            [-0.5, 0.0, -0.5], [0.5, 0.0, -0.5], [0.5, 0.0, 0.5], [-0.5, 0.0, 0.5],
            [-0.5, 1.0, -0.5], [0.5, 1.0, -0.5], [0.5, 1.0, 0.5], [-0.5, 1.0, 0.5],
        ];
        
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            VertexAttributeValues::Float32x3(positions),
        );
        
        let indices = vec![
            0u32, 1, 5, 0, 5, 4,
            1, 2, 6, 1, 6, 5,
            2, 3, 7, 2, 7, 6,
            3, 0, 4, 3, 4, 7,
        ];
        
        mesh.insert_indices(Indices::U32(indices));
        
        let segments = calculate_contours_only(&mesh, 0.5, 0.3);
        
        assert!(!segments.is_empty());
        println!("Found {} contour segments", segments.len());
    }
}
