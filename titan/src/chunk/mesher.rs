use bevy::{
    prelude::*,
    render::{mesh::Indices, render_asset::RenderAssetUsages, render_resource::PrimitiveTopology},
    utils::hashbrown::hash_set::Difference,
};
use fast_surface_nets::{
    ndshape::{ConstShape, ConstShape3u32},
    surface_nets, SurfaceNetsBuffer,
};

use crate::{
    table::{
        VoxelFace, CORNERS, EDGES, EDGE_CROSSING_MASK, EDGE_DIRECTION, FACE_BACK, FACE_BOTTOM,
        FACE_FRONT, FACE_LEFT, FACE_RIGHT, FACE_TOP, INDEX_MAP, NORMAL_MAP, TEXTURE_MAP, TRIANGLES,
        VERTEX_MAP,
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
pub struct SurfaceNetsChunkMesher {}

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

const SURFACE: f32 = 0.5;

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

                    // An array of sampled points
                    let mut cube = [0.0; 8];

                    // Calculate the cube index by looking at all 8 corners of the current
                    // voxel
                    let mut cube_index = 0;
                    for i in 0..8 {
                        cube[i] =
                            match chunk.get_t_block(world_position, position + CORNERS[i], terrain)
                            {
                                VoxelType::Air => 0.0,
                                VoxelType::Dirt(v) => v,
                                VoxelType::Grass(v) => v,
                                VoxelType::Stone(v) => v,
                            };

                        cube[i] = Terrain::map_range(
                            (u8::MIN as f32, u8::MAX as f32),
                            (0.0, 1.0),
                            cube[i],
                        );

                        if cube[i] <= SURFACE {
                            cube_index |= 1 << i;
                        }
                    }

                    let smooth = true;

                    if smooth {
                        // Helper functions
                        fn get_offset(v1: f32, v2: f32) -> f32 {
                            let delta = v2 - v1;
                            if delta == 0.0 {
                                SURFACE
                            } else {
                                (SURFACE - v1) / delta
                            }
                        }

                        // Find which edges are intersected by the surface
                        let edge_flags = EDGE_CROSSING_MASK[cube_index];

                        // The cube is entirely inside or outside the surface
                        if edge_flags == 0 {
                            continue;
                        }

                        let mut edge_verts = vec![Vec3::new(0.0, 0.0, 0.0); 12];

                        // Find the point of intersection of the surface with each edge
                        for i in 0..12 {
                            // if there is an intersection on this edge
                            if (edge_flags & (1 << i)) != 0 {
                                let offset = get_offset(cube[EDGES[i][0]], cube[EDGES[i][1]]);

                                let v =
                                    position + CORNERS[EDGES[i][0]] + offset * EDGE_DIRECTION[i];
                                edge_verts[i] = v;
                            }
                        }

                        // Look up the triangulation for this index
                        let triangles = TRIANGLES[cube_index];
                        for edge_index in triangles {
                            if edge_index == -1 {
                                break;
                            }

                            let v = edge_verts[edge_index as usize];
                            vertices.push(v.into());

                            uvs.push((0.0, 0.0).into());
                            voxel_indices.push(voxel_type.index());
                            texture_indices.push(VoxelType::Grass(1.0).texture_index(FACE_TOP));
                            indices.push(index);

                            index = index + 1;
                        }
                    } else {
                        // Look up the triangulation for this index
                        let triangles = TRIANGLES[cube_index]; // triTable

                        // edges is triangle

                        for edge_index in triangles {
                            if edge_index == -1 {
                                break;
                            }

                            let i1 = EDGES[edge_index as usize][0];
                            let i2 = EDGES[edge_index as usize][1];

                            let v1 = position + CORNERS[i1];
                            let v2 = position + CORNERS[i2];

                            if smooth {

                                // fn interp(vertex1: Vec3, value1: f32, vertex2: Vec3, value2: f32, iso: f32) -> Vec3 {
                                //     return vertex1 + (iso - value1) * (vertex2 - vertex1) / (value2 - value1);

                                //    return (edgeVertex1 + (isoLevel - valueAtVertex1) * (edgeVertex2 - edgeVertex1)  / (valueAtVertex2 - valueAtVertex1));
                                // }

                                // let v = interp(CORNERS[i1], cube[i1], CORNERS[i2], cube[i2], d) + position;
                                // vertices.push(v.into());

                                // Sample at both edges of the terrain
                                /*  let v1_sample = cube[i1];
                                let v2_sample = cube[i2];

                                // Calculate the difference between the two values
                                let mut difference = v2_sample - v1_sample;

                                if difference == 0.5 {
                                    difference = 0.5;
                                } else {
                                    difference = (0.5 - v1_sample) / difference;
                                }

                                // Calculate the point along the edge that the terrain passe through
                                vertices.push((v1 + ((v2 - v1) * difference)).into());*/
                            } else {
                                let v = v1 + (v2 - v1) * 0.5;

                                vertices.push((v).into());

                                // vertices.push(((v1 + v2) / 2.0).into());
                            }

                            // Normals are calculated in a separate pass
                            uvs.push((0.0, 0.0).into());
                            voxel_indices.push(voxel_type.index());
                            texture_indices.push(VoxelType::Grass(1.0).texture_index(FACE_TOP));
                            indices.push(index);

                            index = index + 1;
                        }
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

        let _ = mesh.generate_tangents();

        if index_count > 0 {
            return Some(mesh);
        }

        return Some(Mesh::from(Cuboid::new(1.0, 1.0, 1.0)));
    }
}

impl ChunkMesher for SurfaceNetsChunkMesher {
    fn build(chunk: &Chunk, world_position: Vec3, terrain: &Terrain) -> Option<Mesh> {
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );

        type ChunkShape = ConstShape3u32<16, 64, 16>;

        let mut sdf = [1.0; ChunkShape::USIZE];
        for i in 0u32..ChunkShape::SIZE {
            let [x, y, z] = ChunkShape::delinearize(i);

            let position = Vec3::new(x as f32, y as f32, z as f32);
            let voxel_type = chunk.get_t_block(world_position, position, terrain);

            sdf[i as usize] = Terrain::map_range(
                (i8::MIN as f32, i8::MAX as f32),
                (-1.0, 1.0),
                match voxel_type {
                    VoxelType::Air => 1.0,
                    VoxelType::Dirt(v) => v as f32,
                    VoxelType::Grass(v) => v as f32,
                    VoxelType::Stone(v) => v as f32,
                },
            );
        }

        let mut buffer = SurfaceNetsBuffer::default();
        surface_nets(&sdf, &ChunkShape {}, [0, 0, 0], [15, 63, 15], &mut buffer);

        if buffer.indices.is_empty() {
            return Some(Mesh::from(Cuboid::new(1.0, 1.0, 1.0)));
        }

        let mut uvs: Vec<[f32; 2]> = Vec::new();
        uvs.resize(buffer.positions.len(), (0., 0.).into());

        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, buffer.positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, buffer.normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        //mesh.insert_attribute(ATTRIBUTE_BASE_VOXEL_INDEX, voxel_indices);
        //mesh.insert_attribute(ATTRIBUTE_BASE_TEXTURE_INDEX, texture_indices);
        mesh.insert_indices(Indices::U32(buffer.indices));

        let _ = mesh.generate_tangents();

        return Some(mesh);
    }
}
