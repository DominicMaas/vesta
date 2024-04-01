use bevy::{
    prelude::*,
    render::{mesh::Indices, render_asset::RenderAssetUsages, render_resource::PrimitiveTopology},
};

use crate::{
    table::{
        VoxelFace, CORNERS, EDGES, FACE_BACK, FACE_BOTTOM, FACE_FRONT, FACE_LEFT, FACE_RIGHT,
        FACE_TOP, INDEX_MAP, NORMAL_MAP, TEXTURE_MAP, TRIANGLES, VERTEX_MAP,
    },
    terrain::Terrain,
};

use super::{
    material::{ATTRIBUTE_BASE_TEXTURE_INDEX, ATTRIBUTE_BASE_VOXEL_INDEX},
    Chunk, VoxelType, CHUNK_XZ, CHUNK_Y,
};

pub trait ChunkMesher {
    fn build(chunk: &Chunk, world_position: Vec3, terrain: &Terrain) -> Option<Mesh>;
}

pub struct CubeChunkMesher {}
pub struct MarchingChunkMesher {}

impl CubeChunkMesher {
    fn build_face(
        chunk: &Chunk,
        face: VoxelFace,
        world_position: Vec3,
        position: Vec3,
        voxel_type: VoxelType,
        terrain: &Terrain,
        index: &mut u32,
        vertices: &mut Vec<[f32; 3]>,
        normals: &mut Vec<[f32; 3]>,
        uvs: &mut Vec<[f32; 2]>,
        voxel_indices: &mut Vec<u32>,
        texture_indices: &mut Vec<u32>,
        indices: &mut Vec<u32>,
    ) {
        let pos_offset = match face {
            FACE_TOP => Vec3::new(0.0, 1.0, 0.0),
            FACE_BOTTOM => Vec3::new(0.0, -1.0, 0.0),
            FACE_LEFT => Vec3::new(-1.0, 0.0, 0.0),
            FACE_RIGHT => Vec3::new(1.0, 0.0, 0.0),
            FACE_FRONT => Vec3::new(0.0, 0.0, 1.0),
            FACE_BACK => Vec3::new(0.0, 0.0, -1.0),
            _ => Vec3::default(),
        };

        if chunk.is_transparent(world_position, position + pos_offset, terrain) {
            for i in 0..4 {
                let v = position + VERTEX_MAP[face][i];

                vertices.push(v.into());
                normals.push(NORMAL_MAP[face][i]);
                uvs.push(TEXTURE_MAP[face][i]);

                texture_indices.push(voxel_type.texture_index(face));

                voxel_indices.push(voxel_type.index());
            }

            for i in 0..6 {
                indices.push(*index + INDEX_MAP[face][i])
            }

            *index = *index + 4;
        }
    }
}

impl ChunkMesher for CubeChunkMesher {
    fn build(chunk: &Chunk, world_position: Vec3, terrain: &Terrain) -> Option<Mesh> {
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );

        let mut vertices: Vec<[f32; 3]> = Vec::new();
        let mut normals: Vec<[f32; 3]> = Vec::new();
        let mut uvs: Vec<[f32; 2]> = Vec::new();
        let mut voxel_indices: Vec<u32> = Vec::new();
        let mut texture_indices: Vec<u32> = Vec::new();

        let mut indices: Vec<u32> = Vec::new();

        let mut index = 0;

        for x in 0..(CHUNK_XZ) {
            for y in 0..(CHUNK_Y) {
                for z in 0..(CHUNK_XZ) {
                    let position = Vec3::new(x as f32, y as f32, z as f32);
                    let voxel_type = chunk.get_t_block(world_position, position, terrain);

                    // Don't build for air
                    if voxel_type == VoxelType::Air {
                        continue;
                    }

                    // Build the 6 faces
                    for face in 0..6 {
                        Self::build_face(
                            chunk,
                            face,
                            world_position,
                            position,
                            voxel_type,
                            terrain,
                            &mut index,
                            &mut vertices,
                            &mut normals,
                            &mut uvs,
                            &mut voxel_indices,
                            &mut texture_indices,
                            &mut indices,
                        );
                    }
                }
            }
        }

        let index_count = indices.len();

        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.insert_attribute(ATTRIBUTE_BASE_VOXEL_INDEX, voxel_indices);
        mesh.insert_attribute(ATTRIBUTE_BASE_TEXTURE_INDEX, texture_indices);
        mesh.insert_indices(Indices::U32(indices));

        if index_count > 0 {
            return Some(mesh);
        }

        return None;
    }
}

impl ChunkMesher for MarchingChunkMesher {
    fn build(chunk: &Chunk, world_position: Vec3, terrain: &Terrain) -> Option<Mesh> {
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );

        let mut vertices: Vec<[f32; 3]> = Vec::new();
        let mut normals: Vec<[f32; 3]> = Vec::new();
        let mut uvs: Vec<[f32; 2]> = Vec::new();
        let mut voxel_indices: Vec<u32> = Vec::new();
        let mut texture_indices: Vec<u32> = Vec::new();

        let mut indices: Vec<u32> = Vec::new();

        let mut index = 0;

        for x in 0..(CHUNK_XZ) {
            for y in 0..(CHUNK_Y) {
                for z in 0..(CHUNK_XZ) {
                    let position = Vec3::new(x as f32, y as f32, z as f32);
                    let voxel_type = chunk.get_t_block(world_position, position, terrain);

                    // Calculate the cube index by looking at all 8 corners of the current
                    // voxel
                    let mut cube_index = 0;
                    for i in 0..8 {
                        if chunk.is_transparent(world_position, position + CORNERS[i], terrain) {
                            cube_index |= 1 << i;
                        }
                    }

                    // Look up the triangulation for this index
                    let triangles = TRIANGLES[cube_index];
                    for edge_index in triangles {
                        if edge_index == -1 {
                            break;
                        }

                        let index_a = EDGES[edge_index as usize][0];
                        let index_b = EDGES[edge_index as usize][1];

                        let v = position + ((CORNERS[index_a] + CORNERS[index_b]) / 2.0);

                        vertices.push(v.into());
                        // Normals are calculated in a separate pass
                        uvs.push((0.0, 0.0).into());
                        voxel_indices.push(voxel_type.index());
                        texture_indices.push(VoxelType::Grass.texture_index(FACE_TOP));
                        indices.push(index);

                        index = index + 1;
                    }
                }
            }
        }

        // Calculate normals for all vertices
        for vertex in vertices.chunks(3) {
            let v1 = Vec3::from(vertex[0]);
            let v2 = Vec3::from(vertex[1]);
            let v3 = Vec3::from(vertex[2]);

            let normal = (v2 - v1).cross(v3 - v1).normalize();

            normals.push(normal.to_array());
            normals.push(normal.to_array());
            normals.push(normal.to_array());
        }

        let index_count = indices.len();

        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.insert_attribute(ATTRIBUTE_BASE_VOXEL_INDEX, voxel_indices);
        mesh.insert_attribute(ATTRIBUTE_BASE_TEXTURE_INDEX, texture_indices);
        mesh.insert_indices(Indices::U32(indices));

        if index_count > 0 {
            return Some(mesh);
        }

        return None;
    }
}
