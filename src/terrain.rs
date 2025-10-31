use bevy::{
    app::{App, Plugin, Startup},
    asset::{AssetServer, Assets, RenderAssetUsages},
    ecs::{
        component::Component,
        resource::Resource,
        system::{Commands, Res, ResMut},
    },
    math::Vec3,
    pbr::MeshMaterial3d,
    render::mesh::{Indices, Mesh, Mesh3d, PrimitiveTopology},
};
use fastnoise_lite::{FastNoiseLite, NoiseType};
use image::RgbImage;
use rayon::{
    iter::{IndexedParallelIterator, ParallelIterator},
    slice::ParallelSliceMut,
};

use crate::world_normal_material::WorldNormalMaterial;

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
            face_size: 513,
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
        // f.set_fractal_type(None);
        Self { f }
    }
}

fn spawn_planet(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut world_mats: ResMut<Assets<WorldNormalMaterial>>,
    asset_server: Res<AssetServer>,
    noise: Res<TerrainNoise>,
    np: Res<NoiseParams>,
    planet: Res<PlanetParams>,
) {
    let res = 4096;
    export_normal_map(
        Face::PosX,
        res,
        planet.radius,
        &np,
        &noise,
        "assets/normals_posx.png",
    );
    export_normal_map(
        Face::NegX,
        res,
        planet.radius,
        &np,
        &noise,
        "assets/normals_negx.png",
    );
    export_normal_map(
        Face::PosY,
        res,
        planet.radius,
        &np,
        &noise,
        "assets/normals_posy.png",
    );
    export_normal_map(
        Face::NegY,
        res,
        planet.radius,
        &np,
        &noise,
        "assets/normals_negy.png",
    );
    export_normal_map(
        Face::PosZ,
        res,
        planet.radius,
        &np,
        &noise,
        "assets/normals_posz.png",
    );
    export_normal_map(
        Face::NegZ,
        res,
        planet.radius,
        &np,
        &noise,
        "assets/normals_negz.png",
    );

    for face in Face::ALL {
        let mesh = build_cube_sphere_face(face, planet.face_size, planet.radius, &np, &noise);
        let handle = meshes.add(mesh);
        let normal_map = asset_server.load(match face {
            Face::PosX => "normals_posx.png",
            Face::NegX => "normals_negx.png",
            Face::PosY => "normals_posy.png",
            Face::NegY => "normals_negy.png",
            Face::PosZ => "normals_posz.png",
            Face::NegZ => "normals_negz.png",
        });

        let mat_handle = world_mats.add(WorldNormalMaterial {
            params: Default::default(),
            normal_map,
        });

        commands.spawn((
            Mesh3d(handle),
            MeshMaterial3d::<WorldNormalMaterial>(mat_handle),
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

/// Write out a normal map for one face at high resolution
/// Each pixel encodes the world space normal as RGB in [0,1]
fn export_normal_map(
    face: Face,
    res: u32,
    radius: f32,
    np: &NoiseParams,
    _tn: &TerrainNoise,
    out_path: &str,
) {
    let w = res as usize;
    let h = res as usize;

    // positions one fBm per pixel
    let mut pos = vec![Vec3::ZERO; w * h];

    pos.par_chunks_mut(w).enumerate().for_each(|(y, row)| {
        let mut f = FastNoiseLite::with_seed(1337);
        f.set_noise_type(Some(NoiseType::OpenSimplex2));
        f.set_frequency(Some(1.0));

        let v = 2.0 * (y as f32 / (h - 1) as f32) - 1.0;
        for x in 0..w {
            let u = 2.0 * (x as f32 / (w - 1) as f32) - 1.0;
            let dir = cube_to_sphere_dir(face, u, v);
            let world_on_surface = dir * radius;
            let hgt = fbm(&f, world_on_surface, np) * np.amplitude;
            row[x] = dir * (radius + hgt);
        }
    });

    // Normals
    let mut buf = vec![0u8; w * h * 3];

    buf.par_chunks_mut(w * 3)
        .enumerate()
        .for_each(|(y, row_bytes)| {
            for x in 0..w {
                let xm = if x > 0 { x - 1 } else { x };
                let xp = if x + 1 < w { x + 1 } else { x };
                let ym = if y > 0 { y - 1 } else { y };
                let yp = if y + 1 < h { y + 1 } else { y };

                let p_l = pos[y * w + xm];
                let p_r = pos[y * w + xp];
                let p_d = pos[ym * w + x];
                let p_u = pos[yp * w + x];

                let su = p_r - p_l;
                let sv = p_u - p_d;

                let mut n = su.cross(sv).normalize_or_zero();
                let p = pos[y * w + x];
                if n.dot(p) < 0.0 {
                    n = -n;
                }

                let i = x * 3;
                row_bytes[i + 0] = ((n.x * 0.5 + 0.5) * 255.0).clamp(0.0, 255.0) as u8;
                row_bytes[i + 1] = ((n.y * 0.5 + 0.5) * 255.0).clamp(0.0, 255.0) as u8;
                row_bytes[i + 2] = ((n.z * 0.5 + 0.5) * 255.0).clamp(0.0, 255.0) as u8;
            }
        });

    let img = RgbImage::from_raw(res, res, buf).expect("rgb buffer size");
    img.save(out_path).unwrap();
}

#[inline]
fn idx(i: usize, j: usize, nx: usize) -> usize {
    j * nx + i
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

    // Derivative norms
    let mut normals = Vec::with_capacity(nx * ny);

    for j in 0..ny {
        for i in 0..nx {
            // neighbor indices with clamping at the border
            let il = if i > 0 { i - 1 } else { i };
            let ir = if i + 1 < nx { i + 1 } else { i };
            let jd = if j > 0 { j - 1 } else { j };
            let ju = if j + 1 < ny { j + 1 } else { j };

            let p_l = Vec3::from(positions[idx(il, j, nx)]);
            let p_r = Vec3::from(positions[idx(ir, j, nx)]);
            let p_d = Vec3::from(positions[idx(i, jd, nx)]);
            let p_u = Vec3::from(positions[idx(i, ju, nx)]);

            // central (or one-sided at edges) differences in parametric u,v
            let su = p_r - p_l; // ∂S/∂u (scale unneeded for direction)
            let sv = p_u - p_d; // ∂S/∂v

            let mut n = su.cross(sv).normalize_or_zero();

            // ensure outward orientation
            let p = Vec3::from(positions[idx(i, j, nx)]);
            if n.dot(p) < 0.0 {
                n = -n;
            }

            normals.push(n.to_array());
        }
    }

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
