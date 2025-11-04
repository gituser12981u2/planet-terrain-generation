use bevy::math::Vec3;
use bevy_asset::RenderAssetUsages;
use bevy_render::mesh::{Indices, Mesh, PrimitiveTopology};

use crate::terrain_generation::{
    lod::node_key::NodeKey,
    terrain_core::{NoiseParams, TerrainNoise, cube_to_sphere_dir, fbm},
};

pub fn build_patch_mesh(
    key: NodeKey,
    res: u32,
    planet_radius: f32,
    np: &NoiseParams,
    tn: &TerrainNoise,
    add_skirt: bool,
    skirt_depth: f32,
) -> Mesh {
    let nx = res as usize;
    let ny = res as usize;
    let (u0, v0, du, dv) = key.uv_rect();

    let mut positions = Vec::with_capacity(nx * ny);
    let mut normals = Vec::with_capacity(nx * ny);
    let mut uvs = Vec::with_capacity(nx * ny);

    // vertices
    for j in 0..ny {
        let v = v0 + (j as f32 / (ny - 1) as f32) * dv;
        for i in 0..nx {
            let u = u0 + (i as f32 / (nx - 1) as f32) * du;
            let dir = cube_to_sphere_dir(key.face, u, v);
            let world_on_surface = dir * planet_radius;
            let h = fbm(&tn.f, world_on_surface, np) * np.amplitude;
            let pos = dir * (planet_radius + h);
            positions.push(pos.to_array());
            uvs.push([i as f32 / (nx - 1) as f32, j as f32 / (ny - 1) as f32]);
        }
    }

    // indices
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

    // vertex normals via central differences
    let idx = |i: usize, j: usize| j * nx + i;
    for j in 0..ny {
        for i in 0..nx {
            // neighbor indices with clamping at the border
            let il = if i > 0 { i - 1 } else { i };
            let ir = if i + 1 < nx { i + 1 } else { i };
            let jd = if j > 0 { j - 1 } else { j };
            let ju = if j + 1 < ny { j + 1 } else { j };

            let p_l = Vec3::from(positions[idx(il, j)]);
            let p_r = Vec3::from(positions[idx(ir, j)]);
            let p_d = Vec3::from(positions[idx(i, jd)]);
            let p_u = Vec3::from(positions[idx(i, ju)]);
            let mut n = (p_r - -p_l).cross(p_u - p_d).normalize_or_zero();

            // ensure outward orientation
            let p = Vec3::from(positions[idx(i, j)]);
            if n.dot(p) < 0.0 {
                n = -n;
            }
            normals.push(n.to_array());
        }
    }

    // optional skirts
    if add_skirt {
        // TODO: append skirt vertices + indices on 4 borders, offset along -normalize(pos) by skirt_depth
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
