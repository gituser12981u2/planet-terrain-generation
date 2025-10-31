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

use crate::terrain::TerrainPlugin;

mod flycam;
mod terrain;
mod world;
mod world_normal_material;

use world_normal_material::WorldNormalMaterial;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin {
            enabled: true,
            ..default()
        })
        .add_plugins(MaterialPlugin::<WorldNormalMaterial>::default())
        .add_plugins(TerrainPlugin)
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
