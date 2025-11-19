use raylib::prelude::*;
use crate::vertex::Vertex;
use crate::fragment::Fragment;
use crate::Uniforms;
use nalgebra_glm::{self as glm, length};

// =============================================================
// === CONVERSIÓN ENTRE nalgebra_glm Y raylib ==================
// =============================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanetShaderType {
    Terra,       // Planeta tipo Tierra (océanos, nubes, vegetación)
    Vulcan,      // Planeta volcánico / rocoso
    Solarius,    // Estrella (plasma, fuego, manchas solares)
    Nepturion,   // Planeta gaseoso tipo Neptuno
    Mossar,      // Planeta orgánico o musgoso
 
}

/// Convierte una `glm::Mat4` a una `raylib::Matrix`
fn glm_to_raylib(mat: &glm::Mat4) -> Matrix {
    let m = mat.as_slice();
    Matrix {
        m0: m[0],  m1: m[1],  m2: m[2],  m3: m[3],
        m4: m[4],  m5: m[5],  m6: m[6],  m7: m[7],
        m8: m[8],  m9: m[9],  m10: m[10], m11: m[11],
        m12: m[12], m13: m[13], m14: m[14], m15: m[15],
    }
}

// =============================================================
// === FUNCIONES BASE DE SHADER ================================
// =============================================================

// Multiplica una matriz 4x4 (raylib::Matrix) con un vector 4D (Vector4)
fn multiply_matrix_vector4(matrix: &Matrix, vector: &Vector4) -> Vector4 {
    Vector4::new(
        matrix.m0 * vector.x + matrix.m4 * vector.y + matrix.m8 * vector.z + matrix.m12 * vector.w,
        matrix.m1 * vector.x + matrix.m5 * vector.y + matrix.m9 * vector.z + matrix.m13 * vector.w,
        matrix.m2 * vector.x + matrix.m6 * vector.y + matrix.m10 * vector.z + matrix.m14 * vector.w,
        matrix.m3 * vector.x + matrix.m7 * vector.y + matrix.m11 * vector.z + matrix.m15 * vector.w,
    )
}

// =============================================================
// === VERTEX SHADER ===========================================
// =============================================================
pub fn vertex_shader(vertex: &Vertex, uniforms: &Uniforms) -> Vertex {
    // Convertimos las matrices de nalgebra_glm a raylib::Matrix
    let model_mat = glm_to_raylib(&uniforms.model_matrix);
    let view_mat = glm_to_raylib(&uniforms.view_matrix);
    let proj_mat = glm_to_raylib(&uniforms.projection_matrix);
    let viewport_mat = glm_to_raylib(&uniforms.viewport_matrix);

    // Posición homogénea del vértice
    let position_vec4 = Vector4::new(
        vertex.position.x,
        vertex.position.y,
        vertex.position.z,
        1.0,
    );

    // Transformaciones
    let world_position = multiply_matrix_vector4(&model_mat, &position_vec4);
    let view_position = multiply_matrix_vector4(&view_mat, &world_position);
    let clip_position = multiply_matrix_vector4(&proj_mat, &view_position);

    // División de perspectiva (NDC)
    let ndc = if clip_position.w != 0.0 {
        Vector3::new(
            clip_position.x / clip_position.w,
            clip_position.y / clip_position.w,
            clip_position.z / clip_position.w,
        )
    } else {
        Vector3::new(clip_position.x, clip_position.y, clip_position.z)
    };

    // Aplicamos Viewport transform
    let ndc_vec4 = Vector4::new(ndc.x, ndc.y, ndc.z, 1.0);
    let screen_position = multiply_matrix_vector4(&viewport_mat, &ndc_vec4);

    let transformed_position = Vector3::new(
        screen_position.x,
        screen_position.y,
        screen_position.z,
    );

    // Retornamos el vértice transformado
    Vertex {
        position: vertex.position,
        normal: vertex.normal,
        tex_coords: vertex.tex_coords,
        color: vertex.color,
        transformed_position,
        transformed_normal: vertex.normal, // TODO: normal matrix
    }
}

// =============================================================
// === FRAGMENT SHADERS DE EJEMPLO =============================
// =============================================================
#[allow(dead_code)]
fn shader_terra(fragment: &Fragment, time: f32) -> Vector3 {
    let p = fragment.world_position;
    let base_color = fragment.color;

    // Simula océanos con sinusoides lentas
    let ocean = ((p.x * 0.8 + p.y * 1.2 + time * 0.5).sin() * 0.5 + 0.5).powf(1.8);

    // Continentes verdes usando patrones de interferencia
    let land = ((p.x * 2.1 + p.z * 1.4 - time * 0.2).cos() * (p.y * 1.5).sin()).abs();

    // Nubes dinámicas
    let clouds = ((p.x * 5.0 + p.y * 5.0 + time * 2.0).sin() * 0.5 + 0.5).powf(6.0);

    let color_ocean = Vector3::new(0.0, 0.25, 0.8);
    let color_land = Vector3::new(0.1, 0.6, 0.2);
    let color_clouds = Vector3::new(1.0, 1.0, 1.0);

    let mix_earth = color_ocean * (1.0 - land) + color_land * land;
    let final_color = mix_earth * (1.0 - clouds * 0.3) + color_clouds * clouds * 0.5;

    Vector3::new(
        base_color.x * final_color.x,
        base_color.y * final_color.y,
        base_color.z * final_color.z,
    )
}

#[allow(dead_code)]
fn shader_vulcan(fragment: &Fragment, time: f32) -> Vector3 {
    let p = fragment.world_position;
    let base_color = fragment.color;

    let crack_pattern = ((p.x * 8.0).sin() * (p.y * 8.0).cos() * (p.z * 6.0).sin()).abs();
    let heat_wave = ((p.x * 3.0 + p.y * 2.0 + time * 5.0).sin() * 0.5 + 0.5).powf(8.0);

    let rock_color = Vector3::new(0.3, 0.2, 0.15);
    let lava_color = Vector3::new(1.0, 0.4, 0.05);

    let lava_mix = crack_pattern.powf(3.0) * heat_wave;
    let color = rock_color * (1.0 - lava_mix) + lava_color * lava_mix;

    // Brillo dinámico (simula calor)
    let glow = (time * 10.0).sin() * 0.1 + 0.9;
    color * glow * base_color
}

#[allow(dead_code)]
pub fn shader_solarius(fragment: &Fragment, time: f32) -> Vector3 {
    let p = fragment.world_position;
    let base_color = fragment.color;

    // Movimiento tipo flujo solar
    let plasma = ((p.x * 4.0 + time * 3.0).sin() + (p.y * 5.0 - time * 2.0).cos()).abs();
    let turbulence = ((p.z * 6.0 + time * 4.0).sin() * 0.5 + 0.5).powf(3.0);

    // Manchas solares oscuras
    let sunspots = (plasma * turbulence).powf(2.0);
    let spot_factor = (1.0 - sunspots * 0.5).max(0.0);

    // Paleta de colores (desde el núcleo al borde)
    let color_core = Vector3::new(1.0, 0.9, 0.3);   // centro brillante
    let color_flame = Vector3::new(1.0, 0.5, 0.0);  // medio ardiente
    let color_outer = Vector3::new(1.0, 0.15, 0.0); // borde rojo oscuro

    // Mezcla entre colores
    let mix1 = color_core * plasma + color_flame * (1.0 - plasma);
    let mix2 = mix1 * spot_factor + color_outer * (1.0 - spot_factor);

    // Pulso radiante (animación de brillo)
    let pulse = (time * 3.0).sin() * 0.25 + 0.9;

    // ✅ Emisión propia: intensidad y brillo amplificados
    let emission_intensity = 2.5; // controla cuánta “luz” emite
    let color_emission = mix2 * emission_intensity * pulse;

    // Combinar con color base del fragmento (si tu modelo tiene color)
    (base_color * 0.3) + color_emission
}


#[allow(dead_code)]
pub fn shader_nepturion(fragment: &Fragment, time: f32) -> Vector3 {
    let p = fragment.world_position;
    let base_color = fragment.color;

    // --- Superficie gaseosa animada ---
    let band = ((p.y * 4.0 + time * 0.8).sin() * 0.5 + 0.5).powf(2.0);
    let turbulence = ((p.x * 6.0 + p.z * 4.0 + time * 2.0).cos() * 0.5 + 0.5).powf(3.0);

    let band_color1 = Vector3::new(0.05, 0.2, 0.7);
    let band_color2 = Vector3::new(0.2, 0.4, 0.9);
    let highlight = Vector3::new(0.5, 0.8, 1.0);

    let gas_mix = band_color1 * band + band_color2 * (1.0 - band);
    let final_color = gas_mix * (1.0 - turbulence * 0.3) + highlight * turbulence * 0.4;

    // --- Brillo atmosférico leve ---
    let glow = ((p.y + time * 0.2).sin() * 0.5 + 0.5) * 0.2 + 0.8;
    let mut color = final_color * glow * base_color;

    // --- 🌌 Anillos orbitales ---
    // Calculamos distancia desde el eje Y (plano de los anillos)
    let r = length(&glm::vec3(p.x, 0.0, p.z));

    // Definimos región donde hay anillos
    let ring_inner = 1.2;
    let ring_outer = 2.5;

    if r > ring_inner && r < ring_outer {
        // Ondulación sutil y rotación del patrón
        let rotation = (time * 0.5).sin() * 0.3;
        let ring_pattern = (((r * 30.0) + rotation).sin() * 0.5 + 0.5).powf(6.0);

        // Color de los anillos
        let ring_color = Vector3::new(0.7, 0.9, 1.0) * 1.5;

        // Gradiente de opacidad (más fuerte cerca del centro de los anillos)
        let fade = (1.0 - ((r - ring_inner) / (ring_outer - ring_inner)).powf(1.5)).clamp(0.0, 1.0);

        // Factor de inclinación del plano de los anillos
        let tilt = (p.y * 2.0).abs().max(0.1);
        let transparency = (1.0 - tilt).clamp(0.0, 1.0) * 0.6;

        // Color combinado
        let ring_contrib = ring_color * ring_pattern * fade * transparency;
        color += ring_contrib;
    }

    color
}


#[allow(dead_code)]
fn shader_mossar(fragment: &Fragment, time: f32) -> Vector3 {
    let p = fragment.world_position;
    let base_color = fragment.color;

    let moss = ((p.x * 3.0 + p.y * 2.5).cos() * (p.z * 3.5).sin() * 0.5 + 0.5).powf(2.5);
    let bio_glow = ((p.x + p.y + time * 1.5).sin() * 0.5 + 0.5).powf(10.0);

    let color_moss = Vector3::new(0.1, 0.6, 0.2);
    let color_dark = Vector3::new(0.05, 0.25, 0.05);
    let color_glow = Vector3::new(0.4, 1.0, 0.6);

    let blend = color_moss * moss + color_dark * (1.0 - moss);
    let final_color = blend * (1.0 - bio_glow * 0.3) + color_glow * bio_glow * 0.5;

    final_color * base_color
}




pub fn fragment_shader(fragment: &Fragment, uniforms: &Uniforms, planet_type: PlanetShaderType) -> Vector3 {
    let time = uniforms.time;
    match planet_type {
        PlanetShaderType::Terra => shader_terra(fragment, time),
        PlanetShaderType::Vulcan => shader_vulcan(fragment, time),
        PlanetShaderType::Solarius => shader_solarius(fragment, time),
        PlanetShaderType::Nepturion => shader_nepturion(fragment, time),
        PlanetShaderType::Mossar => shader_mossar(fragment, time),
    
    }
}
