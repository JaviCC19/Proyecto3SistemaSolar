use std::fmt;
use std::ops::{Add, Mul};
use nalgebra_glm::Vec3;

#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    /// Constructor con valores de 0 a 255
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Color { r, g, b }
    }

    /// Color negro (por defecto)
    pub fn black() -> Self {
        Color { r: 0, g: 0, b: 0 }
    }

    /// Constructor desde valores float (0.0 a 1.0)
    pub fn from_float(r: f32, g: f32, b: f32) -> Self {
        Color {
            r: (r.clamp(0.0, 1.0) * 255.0) as u8,
            g: (g.clamp(0.0, 1.0) * 255.0) as u8,
            b: (b.clamp(0.0, 1.0) * 255.0) as u8,
        }
    }

    /// Crea un color desde un valor hexadecimal (0xRRGGBB)
    pub fn from_hex(hex: u32) -> Self {
        let r = ((hex >> 16) & 0xFF) as u8;
        let g = ((hex >> 8) & 0xFF) as u8;
        let b = (hex & 0xFF) as u8;
        Color { r, g, b }
    }

    /// Convierte el color actual a formato hexadecimal (0xRRGGBB)
    pub fn to_hex(&self) -> u32 {
        ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }

    /// ✅ Nuevo: crea un color desde un `Vec3` (valores entre 0.0 y 1.0)
    pub fn from_vec3(v: Vec3) -> Self {
        Color {
            r: (v.x.clamp(0.0, 1.0) * 255.0) as u8,
            g: (v.y.clamp(0.0, 1.0) * 255.0) as u8,
            b: (v.z.clamp(0.0, 1.0) * 255.0) as u8,
        }
    }

    /// ✅ Nuevo: convierte el color actual a un `Vec3` normalizado (0.0 a 1.0)
    pub fn to_vec3(&self) -> Vec3 {
        Vec3::new(
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
        )
    }
}

impl Add for Color {
    type Output = Color;

    fn add(self, other: Color) -> Color {
        Color {
            r: self.r.saturating_add(other.r),
            g: self.g.saturating_add(other.g),
            b: self.b.saturating_add(other.b),
        }
    }
}

impl Mul<f32> for Color {
    type Output = Color;

    fn mul(self, scalar: f32) -> Color {
        Color {
            r: (self.r as f32 * scalar).clamp(0.0, 255.0) as u8,
            g: (self.g as f32 * scalar).clamp(0.0, 255.0) as u8,
            b: (self.b as f32 * scalar).clamp(0.0, 255.0) as u8,
        }
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Color(r: {}, g: {}, b: {})", self.r, self.g, self.b)
    }
}
