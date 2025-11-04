// use crate::terrain_generation::terrain_core::{
//     Face, NoiseParams, TerrainNoise, cube_to_sphere_dir, fbm,
// };
// use crate::terrain_generation::world_normal_material::WorldNormalMaterial;
// use bevy::{
//     app::{App, Plugin, Startup},
//     asset::{AssetServer, Assets},
//     ecs::{
//         component::Component,
//         resource::Resource,
//         system::{Commands, Res, ResMut},
//     },
//     math::Vec3,
//     render::mesh::Mesh,
// };
// use fastnoise_lite::{FastNoiseLite, NoiseType};
// use image::RgbImage;
// use rayon::{
//     iter::{IndexedParallelIterator, ParallelIterator},
//     slice::ParallelSliceMut,
// };

// #[derive(Component)]
// struct PlanetFace;

// #[derive(Resource)]
// pub struct PlanetParams {
//     pub radius: f32,
//     pub face_size: u32,
// }
// impl Default for PlanetParams {
//     fn default() -> Self {
//         Self {
//             radius: 50.0,
//             face_size: 513,
//         }
//     }
// }

// fn spawn_planet(
//     mut commands: Commands,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut world_mats: ResMut<Assets<WorldNormalMaterial>>,
//     asset_server: Res<AssetServer>,
//     noise: Res<TerrainNoise>,
//     np: Res<NoiseParams>,
//     planet: Res<PlanetParams>,
// ) {
//     let res = 4096;
//     export_normal_map(
//         Face::PosX,
//         res,
//         planet.radius,
//         &np,
//         &noise,
//         "assets/normals_posx.png",
//     );
//     export_normal_map(
//         Face::NegX,
//         res,
//         planet.radius,
//         &np,
//         &noise,
//         "assets/normals_negx.png",
//     );
//     export_normal_map(
//         Face::PosY,
//         res,
//         planet.radius,
//         &np,
//         &noise,
//         "assets/normals_posy.png",
//     );
//     export_normal_map(
//         Face::NegY,
//         res,
//         planet.radius,
//         &np,
//         &noise,
//         "assets/normals_negy.png",
//     );
//     export_normal_map(
//         Face::PosZ,
//         res,
//         planet.radius,
//         &np,
//         &noise,
//         "assets/normals_posz.png",
//     );
//     export_normal_map(
//         Face::NegZ,
//         res,
//         planet.radius,
//         &np,
//         &noise,
//         "assets/normals_negz.png",
//     );

//     for face in Face::ALL {
//         let mesh = build_patch_mesh(face, planet.face_size, planet.radius, &np, &noise);
//         let handle = meshes.add(mesh);
//         let normal_map = asset_server.load(match face {
//             Face::PosX => "normals_posx.png",
//             Face::NegX => "normals_negx.png",
//             Face::PosY => "normals_posy.png",
//             Face::NegY => "normals_negy.png",
//             Face::PosZ => "normals_posz.png",
//             Face::NegZ => "normals_negz.png",
//         });

//         let mat_handle = world_mats.add(WorldNormalMaterial {
//             params: Default::default(),
//             normal_map,
//         });

//         commands.spawn((
//             Mesh3d(handle),
//             MeshMaterial3d::<WorldNormalMaterial>(mat_handle),
//             PlanetFace,
//         ));
//     }
// }

// /// Write out a normal map for one face at high resolution
// /// Each pixel encodes the world space normal as RGB in [0,1]
// fn export_normal_map(
//     face: Face,
//     res: u32,
//     radius: f32,
//     np: &NoiseParams,
//     _tn: &TerrainNoise,
//     out_path: &str,
// ) {
//     let w = res as usize;
//     let h = res as usize;

//     // positions one fBm per pixel
//     let mut pos = vec![Vec3::ZERO; w * h];

//     pos.par_chunks_mut(w).enumerate().for_each(|(y, row)| {
//         let mut f = FastNoiseLite::with_seed(1337);
//         f.set_noise_type(Some(NoiseType::OpenSimplex2));
//         f.set_frequency(Some(1.0));

//         let v = 2.0 * (y as f32 / (h - 1) as f32) - 1.0;
//         row.iter_mut().enumerate().for_each(|(x, cell)| {
//             let u = 2.0 * (x as f32 / (w - 1) as f32) - 1.0;
//             let dir = cube_to_sphere_dir(face, u, v);
//             let world_on_surface = dir * radius;
//             let hgt = fbm(&f, world_on_surface, np) * np.amplitude;
//             *cell = dir * (radius + hgt);
//         });
//     });

//     // Normals
//     let mut buf = vec![0u8; w * h * 3];

//     buf.par_chunks_mut(w * 3)
//         .enumerate()
//         .for_each(|(y, row_bytes)| {
//             for x in 0..w {
//                 let xm = if x > 0 { x - 1 } else { x };
//                 let xp = if x + 1 < w { x + 1 } else { x };
//                 let ym = if y > 0 { y - 1 } else { y };
//                 let yp = if y + 1 < h { y + 1 } else { y };

//                 let p_l = pos[y * w + xm];
//                 let p_r = pos[y * w + xp];
//                 let p_d = pos[ym * w + x];
//                 let p_u = pos[yp * w + x];

//                 let su = p_r - p_l;
//                 let sv = p_u - p_d;

//                 let mut n = su.cross(sv).normalize_or_zero();
//                 let p = pos[y * w + x];
//                 if n.dot(p) < 0.0 {
//                     n = -n;
//                 }

//                 let i = x * 3;
//                 row_bytes[i] = ((n.x * 0.5 + 0.5) * 255.0).clamp(0.0, 255.0) as u8;
//                 row_bytes[i + 1] = ((n.y * 0.5 + 0.5) * 255.0).clamp(0.0, 255.0) as u8;
//                 row_bytes[i + 2] = ((n.z * 0.5 + 0.5) * 255.0).clamp(0.0, 255.0) as u8;
//             }
//         });

//     let img = RgbImage::from_raw(res, res, buf).expect("rgb buffer size");
//     img.save(out_path).unwrap();
// }
