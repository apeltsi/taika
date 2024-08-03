pub mod random;

/// A 4x4 identity matrix
pub const IDENTITY_MATRIX: [[f32; 4]; 4] = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

/// A linear interpolation between two values (f32)
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// A linear interpolation between two values (f64)
pub fn lerp64(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}
