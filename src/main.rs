use bevy::{
    DefaultPlugins,
    app::{App, Startup, Update},
    pbr::MaterialPlugin,
    utils::default,
};
use bevy_rapier3d::{
    plugin::{NoUserData, RapierPhysicsPlugin},
    render::RapierDebugRenderPlugin,
};

use crate::{
    root::{flycam, world},
    terrain_generation::{
        lod::{TerrainLodPlugin, systems::PlanetRadius},
        terrain_core::NoiseParams,
        world_normal_material::WorldNormalMaterial,
    },
};

mod root;
mod terrain_generation;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin {
            enabled: true,
            ..default()
        })
        .add_plugins(MaterialPlugin::<WorldNormalMaterial>::default())
        .insert_resource(NoiseParams::default())
        .insert_resource(PlanetRadius(50.0))
        .add_plugins(TerrainLodPlugin)
        .add_systems(Startup, world::setup_world)
        .add_systems(
            Update,
            (
                world::spin,
                flycam::flycam_move,
                flycam::flycam_mouse_look,
                flycam::toggle_capture,
            ),
        )
        .run();
}
