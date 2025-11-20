use crate::fragment::Fragment;
use crate::vertex::Vertex;
use crate::light::Light;
use raylib::prelude::Vector3;

/// Optimized barycentric coordinates with early exit
#[inline(always)]
fn barycentric_coordinates(p_x: f32, p_y: f32, a: &Vertex, b: &Vertex, c: &Vertex) -> Option<(f32, f32, f32)> {
    let a_x = a.transformed_position.x;
    let a_y = a.transformed_position.y;
    let b_x = b.transformed_position.x;
    let b_y = b.transformed_position.y;
    let c_x = c.transformed_position.x;
    let c_y = c.transformed_position.y;

    let denom = (b_y - c_y) * (a_x - c_x) + (c_x - b_x) * (a_y - c_y);

    if denom.abs() < 1e-10 {
        return None;
    }

    let w1 = ((b_y - c_y) * (p_x - c_x) + (c_x - b_x) * (p_y - c_y)) / denom;
    
    // Early exit if outside
    if w1 < 0.0 || w1 > 1.0 {
        return None;
    }

    let w2 = ((c_y - a_y) * (p_x - c_x) + (a_x - c_x) * (p_y - c_y)) / denom;
    
    if w2 < 0.0 || w2 > 1.0 {
        return None;
    }

    let w3 = 1.0 - w1 - w2;
    
    if w3 < 0.0 {
        return None;
    }

    Some((w1, w2, w3))
}

/// Scanline rasterization - MUCH faster than pixel-by-pixel
pub fn triangle(v1: &Vertex, v2: &Vertex, v3: &Vertex, light: &Light) -> Vec<Fragment> {
    let mut fragments = Vec::with_capacity(100); // Pre-allocate

    // Sort vertices by Y coordinate
    let mut verts = [v1, v2, v3];
    verts.sort_by(|a, b| a.transformed_position.y.partial_cmp(&b.transformed_position.y).unwrap());
    
    let (top, mid, bottom) = (verts[0], verts[1], verts[2]);

    // Quick backface culling check
    let edge1_x = mid.transformed_position.x - top.transformed_position.x;
    let edge1_y = mid.transformed_position.y - top.transformed_position.y;
    let edge2_x = bottom.transformed_position.x - top.transformed_position.x;
    let edge2_y = bottom.transformed_position.y - top.transformed_position.y;
    
    let cross = edge1_x * edge2_y - edge1_y * edge2_x;
    if cross <= 0.0 {
        return fragments; // Backface culled
    }

    let base_color = Vector3::new(0.5, 0.5, 0.5);

    // Get bounds
    let min_y = top.transformed_position.y.floor() as i32;
    let max_y = bottom.transformed_position.y.ceil() as i32;

    // Scanline algorithm
    for y in min_y..=max_y {
        let y_f = y as f32 + 0.5;

        // Find X intersections for this scanline
        let mut x_intersections = Vec::with_capacity(2);

        // Check each edge
        for i in 0..3 {
            let v_a = verts[i];
            let v_b = verts[(i + 1) % 3];

            let y1 = v_a.transformed_position.y;
            let y2 = v_b.transformed_position.y;

            // Skip horizontal edges
            if (y2 - y1).abs() < 0.01 {
                continue;
            }

            // Check if scanline intersects this edge
            if (y_f >= y1 && y_f < y2) || (y_f >= y2 && y_f < y1) {
                let t = (y_f - y1) / (y2 - y1);
                let x = v_a.transformed_position.x + t * (v_b.transformed_position.x - v_a.transformed_position.x);
                x_intersections.push(x);
            }
        }

        if x_intersections.len() < 2 {
            continue;
        }

        let x_min = x_intersections[0].min(x_intersections[1]).floor() as i32;
        let x_max = x_intersections[0].max(x_intersections[1]).ceil() as i32;

        // Rasterize this scanline
        for x in x_min..=x_max {
            let p_x = x as f32 + 0.5;

            if let Some((w1, w2, w3)) = barycentric_coordinates(p_x, y_f, v1, v2, v3) {
                // Interpolate normal
                let interpolated_normal = Vector3::new(
                    w1 * v1.normal.x + w2 * v2.normal.x + w3 * v3.normal.x,
                    w1 * v1.normal.y + w2 * v2.normal.y + w3 * v3.normal.y,
                    w1 * v1.normal.z + w2 * v2.normal.z + w3 * v3.normal.z,
                );

                let normal_length = (interpolated_normal.x * interpolated_normal.x
                                   + interpolated_normal.y * interpolated_normal.y
                                   + interpolated_normal.z * interpolated_normal.z).sqrt();
                
                let normalized_normal = if normal_length > 0.0 {
                    Vector3::new(
                        interpolated_normal.x / normal_length,
                        interpolated_normal.y / normal_length,
                        interpolated_normal.z / normal_length,
                    )
                } else {
                    interpolated_normal
                };

                // Interpolate world position
                let world_pos = Vector3::new(
                    w1 * v1.position.x + w2 * v2.position.x + w3 * v3.position.x,
                    w1 * v1.position.y + w2 * v2.position.y + w3 * v3.position.y,
                    w1 * v1.position.z + w2 * v2.position.z + w3 * v3.position.z,
                );

                // Light calculation
                let light_dir_x = light.position.x - world_pos.x;
                let light_dir_y = light.position.y - world_pos.y;
                let light_dir_z = light.position.z - world_pos.z;
                
                let light_length = (light_dir_x * light_dir_x + light_dir_y * light_dir_y + light_dir_z * light_dir_z).sqrt();
                
                let (light_dir_norm_x, light_dir_norm_y, light_dir_norm_z) = if light_length > 0.0 {
                    (light_dir_x / light_length, light_dir_y / light_length, light_dir_z / light_length)
                } else {
                    (0.0, 0.0, 0.0)
                };

                let intensity = (normalized_normal.x * light_dir_norm_x
                               + normalized_normal.y * light_dir_norm_y
                               + normalized_normal.z * light_dir_norm_z).max(0.0);

                let shaded_color = Vector3::new(
                    base_color.x * intensity,
                    base_color.y * intensity,
                    base_color.z * intensity,
                );

                let depth = w1 * v1.transformed_position.z
                          + w2 * v2.transformed_position.z
                          + w3 * v3.transformed_position.z;

                fragments.push(Fragment::new_with_world_pos(p_x, y_f, shaded_color, depth, world_pos));
            }
        }
    }

    fragments
}