use crate::terrain_generation::{lod::node_key::NodeKey, terrain_core::cube_to_sphere_dir};
use bevy::math::Vec3;
use bevy_rapier3d::na::RealField;

#[derive(Clone, Copy)]
pub struct NodeBounds {
    pub center: Vec3,
    pub radius: f32,
}

pub fn compute_bounds(key: NodeKey, planet_radius: f32, amp: f32) -> NodeBounds {
    // conservative sphere from 4 corners inflated by amplitude
    let (u0, v0, du, dv) = key.uv_rect();
    let corners = [(u0, v0), (u0 + du, v0), (u0, v0 + dv), (u0 + du, v0 + dv)];

    let mut ps = [Vec3::ZERO; 4];
    for (i, (u, v)) in corners.into_iter().enumerate() {
        let dir = cube_to_sphere_dir(key.face, u, v);
        // push out by worst-case height
        ps[i] = dir * (planet_radius + amp);
    }
    let center = (ps[0] + ps[1] + ps[2] + ps[3]) * 0.25;
    let mut r = 0.0;
    for p in ps {
        r = r.max(center.distance(p));
    }
    NodeBounds { center, radius: r }
}
