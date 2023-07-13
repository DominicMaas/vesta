use bevy::{
    prelude::*,
    reflect::{TypePath, TypeUuid},
    render::{
        mesh::MeshVertexAttribute,
        render_resource::{AsBindGroup, ShaderRef, VertexFormat},
    },
};

pub const ATTRIBUTE_BASE_VOXEL_INDEX: MeshVertexAttribute =
    MeshVertexAttribute::new("VoxelIndex", 687404547, VertexFormat::Uint32);

pub const ATTRIBUTE_BASE_TEXTURE_INDEX: MeshVertexAttribute =
    MeshVertexAttribute::new("TextureIndex", 708080084, VertexFormat::Uint32);

// A material that describes a chunk
#[derive(AsBindGroup, TypeUuid, TypePath, Debug, Clone)]
#[uuid = "e3442aaf-990b-4950-88ca-03d1990224fa"]
pub struct ChunkMaterial {
    #[texture(0, dimension = "2d_array")]
    #[sampler(1)]
    pub texture: Handle<Image>,
}

impl Material for ChunkMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/chunk.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "shaders/chunk.wgsl".into()
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
            Mesh::ATTRIBUTE_UV_0.at_shader_location(2),
            ATTRIBUTE_BASE_VOXEL_INDEX.at_shader_location(3),
            ATTRIBUTE_BASE_TEXTURE_INDEX.at_shader_location(4),
        ])?;
        descriptor.vertex.buffers = vec![vertex_layout];

        if let Some(label) = &mut descriptor.label {
            *label = format!("chunk_{}", *label).into();
        }

        Ok(())
    }
}
