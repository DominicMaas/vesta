#![allow(dead_code)]

pub mod material;
pub mod mesher;
pub mod tile_map;

use crate::{
    table::{VoxelFace, FACE_BOTTOM, FACE_TOP},
    terrain::Terrain,
};
use bevy::prelude::*;

use self::material::ChunkMaterial;

#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkId {
    pos: IVec2,
}

impl ChunkId {
    pub fn new(x: isize, z: isize) -> Self {
        Self {
            pos: IVec2::new(x as i32, z as i32),
        }
    }

    pub fn world_position(&self) -> Vec3 {
        Vec3::new(self.pos.x as f32, 0.0, self.pos.y as f32)
    }
}

// Chunk constants

pub const CHUNK_XZ: usize = 16;
pub const CHUNK_Y: usize = 64;
pub const CHUNK_SZ: usize = CHUNK_XZ * CHUNK_XZ * CHUNK_Y;

pub const WORLD_XZ: isize = 18;
pub const WORLD_Y: isize = 1;

pub const WORLD_HEIGHT: usize = WORLD_Y as usize * CHUNK_Y;

#[derive(Default, Copy, Clone, Debug, PartialEq)]
#[repr(u32)]
pub enum VoxelType {
    #[default]
    Air,
    Dirt,
    Grass,
    Leaf,
    Log,
    Stone,
    Sand,
    Glass,
    Brick,
    Water,
}

impl VoxelType {
    /// Get the texture index of rhis voxel type
    pub fn texture_index(&self, face: VoxelFace) -> u32 {
        match self {
            VoxelType::Dirt => 0,
            VoxelType::Grass => match face {
                FACE_TOP => 1,
                FACE_BOTTOM => 0,
                _ => 2,
            },
            VoxelType::Stone => 3,
            _ => 0,
        }
    }

    /// Get the numerical index of this voxel type
    pub fn index(&self) -> u32 {
        *self as u32
    }
}

/// Represents a single chunk in the world
#[derive(Component, Debug)]
pub struct Chunk {
    /// 1D Array of all blocks in this chunk
    pub blocks: Vec<VoxelType>,
}

// TODO: Replace with (ChunkId, MaterialMeshBundle<ChunkMaterial>)
#[derive(Default, Bundle, Clone)]
pub struct ChunkBundle {
    /// The id of this chunk, used to link up to the world
    pub chunk_id: ChunkId,
    
    // 
    pub mesh: MaterialMeshBundle<StandardMaterial>,
}

impl Default for Chunk {
    fn default() -> Self {
        Chunk::empty()
    }
}

impl Chunk {
    /// Create a new chunk with the correct internal voxel size (all air)
    pub fn new() -> Self {
        let mut blocks = Vec::with_capacity(CHUNK_SZ);
        blocks.resize(CHUNK_SZ, VoxelType::Air);
        Self { blocks }
    }

    /// Create an empty chunk with no voxel information
    pub fn empty() -> Self {
        Self { blocks: Vec::new() }
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
    fn get_t_block(&self, world_position: Vec3, position: Vec3, terrain: &Terrain) -> VoxelType {
        // If outside this chunk
        if (position.x < 0.0)
            || (position.y < 0.0)
            || (position.z < 0.0)
            || (position.x >= CHUNK_XZ as f32)
            || (position.y >= CHUNK_Y as f32)
            || (position.z >= CHUNK_XZ as f32)
        {
            return terrain.get_block_type(world_position + position);
        }

        // If inside the chunk
        self.blocks[(position.z as usize * CHUNK_XZ * CHUNK_Y
            + position.y as usize * CHUNK_XZ
            + position.x as usize) as usize]
    }

    /// Returns if the specified block is transparent (air, water, etc.)
    /// Used for block culling
    fn is_transparent(&self, world_position: Vec3, position: Vec3, terrain: &Terrain) -> bool {
        self.get_t_block(world_position, position, terrain) == VoxelType::Air
    }
}
