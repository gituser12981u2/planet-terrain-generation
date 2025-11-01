use bevy::pbr::{Material, MaterialPipeline, MaterialPipelineKey};
use bevy::prelude::*;
use bevy::render::mesh::MeshVertexBufferLayoutRef;
use bevy::render::render_resource::*;

#[allow(dead_code)]
#[derive(ShaderType, Clone, Copy, Default, Debug)]
pub struct WorldNormalParams {
    pub _pad: Vec4, // makes binding(0) a valid 16-byte uniform
}

#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
pub struct WorldNormalMaterial {
    #[uniform(0)]
    pub params: WorldNormalParams, // group(1), binding(0)

    #[texture(1)]
    #[sampler(2)]
    pub normal_map: Handle<Image>, // group(1), bindings(1),(2)
}

impl Material for WorldNormalMaterial {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Path("shaders/world_normal_material.wgsl".into())
    }

    fn vertex_shader() -> ShaderRef {
        ShaderRef::Default
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = Some(Face::Back);
        Ok(())
    }
}
