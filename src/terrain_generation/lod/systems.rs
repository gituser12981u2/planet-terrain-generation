use bevy::{
    color::Color,
    ecs::{
        entity::Entity,
        resource::Resource,
        system::{Commands, Res, ResMut},
    },
    pbr::{MeshMaterial3d, StandardMaterial},
};
use bevy_asset::Assets;
use bevy_render::mesh::{Mesh, Mesh3d};

use crate::terrain_generation::{
    lod::{
        bounds::compute_bounds,
        node_key::NodeKey,
        patch_builder::build_patch_mesh,
        state::{NodeIndex, PlanetLod, QuadNode},
    },
    terrain_core::{Face, NoiseParams, TerrainNoise},
};

#[derive(Resource, Clone)]
pub struct LodBootstrap {
    pub patch_res: u32,
    pub skirt: bool,
    pub skirt_depth: f32,
}

pub fn setup_roots(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut index: ResMut<NodeIndex>,
    tn: Res<TerrainNoise>,
    np: Res<NoiseParams>,
    params: Res<LodBootstrap>,
    planet_radius: Res<PlanetRadius>,
) {
    let mut roots: [Entity; 6] = [Entity::PLACEHOLDER; 6];

    for (k, face) in Face::ALL.iter().enumerate() {
        let key = NodeKey::root(*face);
        let bounds = compute_bounds(key, planet_radius.0, np.amplitude);
        let mesh = build_patch_mesh(
            key,
            params.patch_res,
            planet_radius.0,
            &np,
            &tn,
            params.skirt,
            params.skirt_depth,
        );
        let h_mesh = meshes.add(mesh);

        let e = commands
            .spawn((
                QuadNode {
                    key,
                    bounds,
                    mesh: h_mesh.clone(),
                },
                Mesh3d(h_mesh),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(0.54, 0.44, 0.33),
                    perceptual_roughness: 0.9,
                    metallic: 0.0,
                    ..Default::default()
                })),
            ))
            .id();

        index.0.insert(key, e);
        roots[k] = e
    }

    commands.insert_resource(PlanetLod {
        roots,
        radius: planet_radius.0,
        patch_res: params.patch_res,
    });
}

#[derive(Resource)]
pub struct PlanetRadius(pub f32);
