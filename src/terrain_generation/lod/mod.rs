use bevy::app::{App, Plugin, Startup};

use crate::terrain_generation::{
    lod::systems::{LodBootstrap, setup_roots},
    terrain_core::TerrainNoise,
};

pub mod bounds;
pub mod node_key;
pub mod patch_builder;
pub mod state;
pub mod systems;

pub struct TerrainLodPlugin;

impl Plugin for TerrainLodPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TerrainNoise>()
            .insert_resource(LodBootstrap {
                patch_res: 65,
                skirt: false,
                skirt_depth: 0.0,
            })
            .init_resource::<state::NodeIndex>()
            .add_systems(Startup, setup_roots);
    }
}

/*
    1.	6 roots render.
    2.	Frustum + horizon culling.
    3.	SSE split/merge with parent fallback.
    4.	Async builds.
    5.	Skirts only where neighbor is coarser.
    6.	LRU eviction.
    7.	Optional stitched edges + geomorph.

    •	Switch material back to your WorldNormalMaterial or implement analytic normals in WGSL.
*/
