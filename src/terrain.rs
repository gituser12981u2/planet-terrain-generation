use bevy::{
    app::{App, Plugin, Startup},
    asset::{Assets, RenderAssetUsages},
    color::Color,
    ecs::{
        component::Component,
        resource::Resource,
        system::{Commands, Res, ResMut},
    },
    math::Vec3,
    pbr::{MeshMaterial3d, StandardMaterial},
    render::mesh::{Indices, Mesh, Mesh3d, PrimitiveTopology},
    utils::default,
};
use fastnoise_lite::{FastNoiseLite, NoiseType};

#[derive(Component)]
struct PlanetFace;

pub struct TerrainPlugin;
impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlanetParams>()
            .init_resource::<NoiseParams>()
            .init_resource::<TerrainNoise>()
            .add_systems(Startup, spawn_planet);
    }
}

#[derive(Resource)]
pub struct PlanetParams {
    pub radius: f32,
    pub face_size: u32,
}
impl Default for PlanetParams {
    fn default() -> Self {
        Self {
            radius: 50.0,
            face_size: 129,
        }
    }
}

#[derive(Resource, Clone)]
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
            amplitude: 7.5,
            base_freq: 0.06,
            octaves: 8,
            lacunarity: 2.2,
            gain: 0.46,
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
        f.set_noise_type(Some(NoiseType::Perlin));
        f.set_frequency(Some(1.0));
        // f.set_fractal_type(None);
        Self { f }
    }
}

fn spawn_planet(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    noise: Res<TerrainNoise>,
    np: Res<NoiseParams>,
    planet: Res<PlanetParams>,
) {
    for face in Face::ALL {
        let mesh = build_cube_sphere_face(face, planet.face_size, planet.radius, &np, &noise);
        let handle = meshes.add(mesh);
        commands.spawn((
            Mesh3d(handle),
            MeshMaterial3d(mats.add(StandardMaterial {
                base_color: Color::srgb(0.54, 0.44, 0.33),
                perceptual_roughness: 0.95,
                // alpha_mode: AlphaMode::Opaque,
                cull_mode: Some(bevy::render::render_resource::Face::Back),
                ..default()
            })),
            PlanetFace,
        ));
    }
}

#[derive(Clone, Copy)]
enum Face {
    PosX,
    NegX,
    PosY,
    NegY,
    PosZ,
    NegZ,
}
impl Face {
    const ALL: [Face; 6] = [
        Self::PosX,
        Self::NegX,
        Self::PosY,
        Self::NegY,
        Self::PosZ,
        Self::NegZ,
    ];
}

/// Bases chosen so a fixed CCW winding is outward for every face.
fn face_basis(face: Face) -> (Vec3 /*n*/, Vec3 /*u*/, Vec3 /*v*/) {
    match face {
        Face::PosX => (Vec3::X, Vec3::Z, Vec3::Y),
        Face::NegX => (-Vec3::X, -Vec3::Z, Vec3::Y),
        Face::PosY => (Vec3::Y, Vec3::X, Vec3::Z),
        Face::NegY => (-Vec3::Y, Vec3::X, -Vec3::Z),
        Face::PosZ => (Vec3::Z, Vec3::X, Vec3::Y),
        Face::NegZ => (-Vec3::Z, -Vec3::X, Vec3::Y),
    }
}

/// Low-distortion “spherified cube” mapping.
fn cube_to_sphere_dir(face: Face, u: f32, v: f32) -> Vec3 {
    let (n, ax, ay) = face_basis(face);
    let p = (n + u * ax + v * ay).normalize();
    let (x, y, z) = (p.x, p.y, p.z);
    let (x2, y2, z2) = (x * x, y * y, z * z);
    let sx = x * (1.0 - (y2 + z2) / 6.0 + (y2 * z2) / 24.0);
    let sy = y * (1.0 - (z2 + x2) / 6.0 + (z2 * x2) / 24.0);
    let sz = z * (1.0 - (x2 + y2) / 6.0 + (x2 * y2) / 24.0);
    Vec3::new(sx, sy, sz).normalize()
}

/// Seamless 3D Perlin fBm sampled in world-space direction.
/// fBm in WORLD space. Input in meters.
fn fbm(noiser: &FastNoiseLite, world_p: Vec3, np: &NoiseParams) -> f32 {
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

fn build_cube_sphere_face(
    face: Face,
    size: u32,
    radius: f32,
    np: &NoiseParams,
    tn: &TerrainNoise,
) -> Mesh {
    let nx = size as usize;
    let ny = size as usize;

    // vertices
    let mut positions = Vec::with_capacity(nx * ny);
    let mut uvs = Vec::with_capacity(nx * ny);

    for j in 0..ny {
        let v = 2.0 * (j as f32 / (ny - 1) as f32) - 1.0;
        for i in 0..nx {
            let u = 2.0 * (i as f32 / (nx - 1) as f32) - 1.0;

            let dir = cube_to_sphere_dir(face, u, v); // unit
            let world_on_surface = dir * radius; // meters
            let h = fbm(&tn.f, world_on_surface, np) * np.amplitude;
            let pos = dir * (radius + h);

            positions.push(pos.to_array());
            uvs.push([i as f32 / (nx - 1) as f32, j as f32 / (ny - 1) as f32]);
        }
    }

    // indices with per-triangle outward correction
    let mut indices = Vec::<u32>::with_capacity((nx - 1) * (ny - 1) * 6);
    let pos = |k: u32| -> Vec3 { Vec3::from(positions[k as usize]) };

    for j in 0..(ny - 1) {
        for i in 0..(nx - 1) {
            let i0 = (j * nx + i) as u32;
            let i1 = i0 + 1;
            let i2 = i0 + nx as u32;
            let i3 = i2 + 1;

            // tri A: i0, i2, i1
            {
                let a = pos(i0);
                let b = pos(i2);
                let c = pos(i1);
                let n = (b - a).cross(c - a);
                let centroid = (a + b + c) / 3.0;
                // ensure outward (dot > 0)
                if n.dot(centroid) >= 0.0 {
                    indices.extend_from_slice(&[i0, i2, i1]);
                } else {
                    indices.extend_from_slice(&[i0, i1, i2]);
                }
            }
            // tri B: i1, i2, i3
            {
                let a = pos(i1);
                let b = pos(i2);
                let c = pos(i3);
                let n = (b - a).cross(c - a);
                let centroid = (a + b + c) / 3.0;
                if n.dot(centroid) >= 0.0 {
                    indices.extend_from_slice(&[i1, i2, i3]);
                } else {
                    indices.extend_from_slice(&[i1, i3, i2]);
                }
            }
        }
    }

    // radial normals from displaced positions
    let normals: Vec<[f32; 3]> = positions
        .iter()
        .map(|p| Vec3::from(*p).normalize_or_zero().to_array())
        .collect();

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}
