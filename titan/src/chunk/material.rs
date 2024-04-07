use bevy::{
    prelude::*,
    reflect::TypePath,
    render::{
        mesh::MeshVertexAttribute,
        render_resource::{AsBindGroup, ShaderRef, VertexFormat},
    },
};

pub const ATTRIBUTE_BASE_VOXEL_INDEX: MeshVertexAttribute =
    MeshVertexAttribute::new("VoxelIndex", 687404547, VertexFormat::Uint32);

// A material that describes a chunk
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct ChunkMaterial {}

impl Material for ChunkMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/chunk.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "shaders/chunk.wgsl".into()
    }

    fn prepass_vertex_shader() -> ShaderRef {
        "shaders/chunk_prepass.wgsl".into()
    }

    fn prepass_fragment_shader() -> ShaderRef {
        "shaders/chunk_prepass.wgsl".into()
    }
    
    fn specialize(
        _pipeline: &bevy::pbr::MaterialPipeline<Self>,
        descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        layout: &bevy::render::mesh::MeshVertexBufferLayout,
        _key: bevy::pbr::MaterialPipelineKey<Self>,
    ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
        let vertex_layout = layout.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            Mesh::ATTRIBUTE_NORMAL.at_shader_location(1),
            ATTRIBUTE_BASE_VOXEL_INDEX.at_shader_location(2),
        ])?;
        descriptor.vertex.buffers = vec![vertex_layout];

        if let Some(label) = &mut descriptor.label {
            *label = format!("chunk_{}", *label).into();
        }

        Ok(())
    }
}
