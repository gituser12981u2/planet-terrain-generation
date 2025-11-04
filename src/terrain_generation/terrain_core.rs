use bevy::{ecs::resource::Resource, math::Vec3};
use fastnoise_lite::{FastNoiseLite, NoiseType};

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub enum Face {
    PosX,
    NegX,
    PosY,
    NegY,
    PosZ,
    NegZ,
}
impl Face {
    pub const ALL: [Face; 6] = [
        Self::PosX,
        Self::NegX,
        Self::PosY,
        Self::NegY,
        Self::PosZ,
        Self::NegZ,
    ];
}

/// Bases chosen so a fixed CCW winding is outward for every face.
#[inline]
pub fn face_basis(face: Face) -> (Vec3 /*n*/, Vec3 /*u*/, Vec3 /*v*/) {
    match face {
        Face::PosX => (Vec3::X, Vec3::Z, Vec3::Y),
        Face::NegX => (-Vec3::X, -Vec3::Z, Vec3::Y),
        Face::PosY => (Vec3::Y, Vec3::X, Vec3::Z),
        Face::NegY => (-Vec3::Y, Vec3::X, -Vec3::Z),
        Face::PosZ => (Vec3::Z, Vec3::X, Vec3::Y),
        Face::NegZ => (-Vec3::Z, -Vec3::X, Vec3::Y),
    }
}

/// Low-distortion spherified cube mapping.
#[inline]
pub fn cube_to_sphere_dir(face: Face, u: f32, v: f32) -> Vec3 {
    let (n, ax, ay) = face_basis(face);
    let p = (n + u * ax + v * ay).normalize();

    // spherified cube correction
    let (x, y, z) = (p.x, p.y, p.z);
    let (x2, y2, z2) = (x * x, y * y, z * z);
    let sx = x * (1.0 - (y2 + z2) / 6.0 + (y2 * z2) / 24.0);
    let sy = y * (1.0 - (z2 + x2) / 6.0 + (z2 * x2) / 24.0);
    let sz = z * (1.0 - (x2 + y2) / 6.0 + (x2 * y2) / 24.0);
    Vec3::new(sx, sy, sz).normalize()
}

#[derive(Resource)]
pub struct NoiseParams {
    pub amplitude: f32, // meters of relief
    pub base_freq: f32, // cycles per world unit
    pub octaves: u32,
    pub lacunarity: f32,
    pub gain: f32,
}
impl Default for NoiseParams {
    fn default() -> Self {
        Self {
            amplitude: 7.0,
            base_freq: 0.012,
            octaves: 10,
            lacunarity: 2.0,
            gain: 0.5,
        }
    }
}

#[derive(Resource)]
pub struct TerrainNoise {
    pub f: FastNoiseLite,
}
impl Default for TerrainNoise {
    fn default() -> Self {
        let mut f = FastNoiseLite::with_seed(1337);
        f.set_noise_type(Some(NoiseType::OpenSimplex2));
        f.set_frequency(Some(1.0));
        Self { f }
    }
}

/// Seamless 3D Perlin fBm sampled in world-space direction.
/// fBm in WORLD space. Input in meters.
#[inline]
pub fn fbm(noiser: &FastNoiseLite, world_p: Vec3, np: &NoiseParams) -> f32 {
    let mut a = 1.0;
    let mut f = np.base_freq.max(1e-6); // cycles per meter
    let mut sum = 0.0;
    let mut amp_sum = 0.0;

    for _ in 0..np.octaves {
        let p = world_p * f; // scale by frequency [1/m]
        let n = noiser.get_noise_3d(p.x, p.y, p.z); // [-1,1]
        sum += n * a;
        amp_sum += a;
        f *= np.lacunarity;
        a *= np.gain;
    }
    if amp_sum > 0.0 { sum / amp_sum } else { 0.0 }
}
