use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology, texture::ImageSampler},
};
use bevy_rapier3d::prelude::*;

use crate::{
    block_map::{
        add_uvs, texture_offset_from_block, vertex_offset, FACE_BACK, FACE_BOTTOM, FACE_FRONT,
        FACE_LEFT, FACE_RIGHT, FACE_TOP, INDEX_MAP, NORMAL_MAP, TEXTURE_MAP, VERTEX_MAP,
    },
    table::{CORNERS, EDGES, EDGE_CROSSING_MASK, TRIANGLES},
    terrain::Terrain,
};

// Chunk constants

pub const CHUNK_XZ: usize = 16;
pub const CHUNK_Y: usize = 16;
pub const CHUNK_SZ: usize = CHUNK_XZ * CHUNK_XZ * CHUNK_Y;

pub const WORLD_XZ: isize = 14;
pub const WORLD_Y: isize = 2;

#[derive(Default, Clone, Copy, PartialEq)]
pub struct TerrainVoxel {
    pub density: f32,
}

#[derive(Default, Clone, Copy, PartialEq)]
pub enum VoxelType {
    #[default]
    Air,
    Dirt(TerrainVoxel),
    Grass(TerrainVoxel),
    Leaf,
    Log,
    Stone(TerrainVoxel),
    Sand(TerrainVoxel),
    Glass,
    Brick,
    Water,
}

/// Represents a single chunk in the world
#[derive(Component)]
pub struct Chunk {
    /// 1D Array of all blocks in this chunk
    pub blocks: Vec<VoxelType>,

    /// Where in the world is this chunk
    pub world_position: Vec3,
}

#[derive(Default, Bundle)]
pub struct ChunkBundle {
    /// Chunk data
    pub chunk: Chunk,
    /// The chunk material (this is standard)
    pub material: Handle<StandardMaterial>,
    /// Where the chunk is located in the world
    pub transform: Transform,
    /// Global world transform
    pub global_transform: GlobalTransform,
    /// User indication of whether an entity is visible
    pub visibility: Visibility,
    /// Algorithmically-computed indication of whether an entity is visible and should be extracted for rendering
    pub computed_visibility: ComputedVisibility,
}

impl Default for Chunk {
    fn default() -> Self {
        let mut blocks = Vec::with_capacity(CHUNK_SZ);
        blocks.resize(CHUNK_SZ, VoxelType::Air);
        Self {
            blocks,
            world_position: Vec3::default(),
        }
    }
}

impl Chunk {
    fn new(world_position: Vec3) -> Self {
        Self {
            world_position,
            ..Default::default()
        }
    }

    /// Set the block type at the provided position
    pub fn set_block(&mut self, x: usize, y: usize, z: usize, voxel_type: VoxelType) {
        self.blocks[(z * CHUNK_XZ * CHUNK_Y + y * CHUNK_XZ + x) as usize] = voxel_type;
    }

    /// Get the block type at the provided position
    pub fn get_block(&self, x: usize, y: usize, z: usize) -> VoxelType {
        self.blocks[(z * CHUNK_XZ * CHUNK_Y + y * CHUNK_XZ + x) as usize]
    }

    /// Get the block type at the provided position
    fn get_t_block(&self, position: Vec3, t: &Res<Terrain>) -> VoxelType {
        // If outside this chunk
        if (position.x < 0.0)
            || (position.y < 0.0)
            || (position.z < 0.0)
            || (position.x >= CHUNK_XZ as f32)
            || (position.y >= CHUNK_Y as f32)
            || (position.z >= CHUNK_XZ as f32)
        {
            return t.get_block_type(self.world_position + position);
        }

        // If inside the chunk
        self.blocks[(position.z as usize * CHUNK_XZ * CHUNK_Y
            + position.y as usize * CHUNK_XZ
            + position.x as usize) as usize]
    }

    /// Returns if the specified block is transparent (air, water, etc.)
    /// Used for block culling
    fn is_transparent(&self, position: Vec3, t: &Res<Terrain>) -> bool {
        self.get_t_block(position, t) == VoxelType::Air
    }

    pub fn create_mesh(&self, t: &Res<Terrain>) -> Option<Mesh> {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

        let mut vertices: Vec<[f32; 3]> = Vec::new();
        let mut normals: Vec<[f32; 3]> = Vec::new();
        let mut uvs: Vec<[f32; 2]> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();

        for x in 0..(CHUNK_XZ + 1) {
            for y in 0..(CHUNK_Y + 1) {
                for z in 0..(CHUNK_XZ + 1) {
                    let position = Vec3::new(x as f32, y as f32, z as f32);
                    let voxel_type = self.get_t_block(position, t);

                    // Calculate the cube index by looking at all 8 corners of the current
                    // voxel
                    let mut cube_index = 0;
                    for i in 0..8 {
                        if self.is_transparent((position - 1.0) + CORNERS[i], t) {
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

                        vertices.push(v.to_array());
                        uvs.push([0.0, 0.0]);

                        indices.push(vertices.len() as u32 - 1);
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
        mesh.set_indices(Some(Indices::U32(indices)));

        if index_count > 0 {
            return Some(mesh);
        }

        return None;
    }
}

pub fn chunk_setup(
    mut commands: Commands,
    terrain: Res<Terrain>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut textures: ResMut<Assets<Image>>,
    asset_server: Res<AssetServer>,
) {
    // Load in the block map texture used for the chunks
    let block_map_texture_handle = asset_server.load("block_map.png");

    // Change sampling for block map
    if let Some(mut texture) = textures.get_mut(&block_map_texture_handle.clone()) {
        texture.sampler_descriptor = ImageSampler::nearest();
    }

    // Create the chunk material
    let chunk_mat = materials.add(StandardMaterial {
        base_color_texture: Some(block_map_texture_handle.clone()),
        reflectance: 0.2,
        ..Default::default()
    });

    for y in -WORLD_Y..WORLD_Y {
        for x in -WORLD_XZ..WORLD_XZ {
            for z in -WORLD_XZ..WORLD_XZ {
                // Where this chunk is in the world
                let world_position = Vec3::new(
                    (x * CHUNK_XZ as isize) as f32,
                    (y * CHUNK_Y as isize) as f32,
                    (z * CHUNK_XZ as isize) as f32,
                );

                // Create a default chunk
                let mut chunk = Chunk::new(world_position);

                // Load in some initial terrain
                for cx in 0..CHUNK_XZ {
                    for cy in 0..CHUNK_Y {
                        for cz in 0..CHUNK_XZ {
                            let c_pos = Vec3::new(cx as f32, cy as f32, cz as f32) + world_position;
                            let block_type = terrain.get_block_type(c_pos);

                            chunk.set_block(cx, cy, cz, block_type);
                        }
                    }
                }

                if let Some(m) = chunk.create_mesh(&terrain) {
                    let chunk_mesh_handle = meshes.add(m);
                    //let chunk_mesh = &meshes.get(&chunk_mesh_handle);

                    commands
                        .spawn_bundle(ChunkBundle {
                            chunk,
                            material: chunk_mat.clone(),
                            transform: Transform::from_translation(world_position),
                            ..Default::default()
                        })
                        .insert(chunk_mesh_handle);
                    //.insert(RigidBody::Fixed)
                    //.insert(
                    //    Collider::from_bevy_mesh(
                    //        chunk_mesh.unwrap(),
                    //        &ComputedColliderShape::TriMesh,
                    //    )
                    //    .unwrap(),
                    //);
                }
            }
        }
    }
}
