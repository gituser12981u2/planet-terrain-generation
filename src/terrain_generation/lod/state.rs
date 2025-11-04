use bevy::{
    ecs::{component::Component, entity::Entity, resource::Resource},
    platform::collections::HashMap,
};
use bevy_asset::Handle;
use bevy_render::mesh::Mesh;

use crate::terrain_generation::lod::{bounds::NodeBounds, node_key::NodeKey};

#[derive(Component)]
pub struct QuadNode {
    pub key: NodeKey,
    pub bounds: NodeBounds,
    pub mesh: Handle<Mesh>,
}

#[derive(Resource, Default)]
pub struct NodeIndex(pub HashMap<NodeKey, Entity>);

#[derive(Resource)]
pub struct PlanetLod {
    pub roots: [Entity; 6],
    pub radius: f32,
    pub patch_res: u32,
}
