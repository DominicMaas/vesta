#![allow(dead_code)]

pub mod material;
pub mod mesher;
pub mod tile_map;

use crate::terrain::Terrain;
use bevy::prelude::*;
use ndshape::{ConstShape, ConstShape3usize};

use self::material::ChunkMaterial;

#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkId {
    pos: IVec2,
}

impl std::fmt::Display for ChunkId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}, {}]", self.pos.x, self.pos.y)
    }
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

    pub fn world_position_to_local(&self, position: Vec3) -> [usize; 3] {
        let local_position = position - self.world_position();
        let x = f32::abs(local_position.x) as usize;
        let y = f32::abs(local_position.y) as usize;
        let z = f32::abs(local_position.z) as usize;

        [x, y, z]
    }
}

// Chunk constants

pub const CHUNK_XZ: usize = 16;
pub const CHUNK_Y: usize = 256;
pub const CHUNK_SZ: usize = CHUNK_XZ * CHUNK_XZ * CHUNK_Y;

pub type ChunkShape = ConstShape3usize<CHUNK_XZ, CHUNK_Y, CHUNK_XZ>;

pub const WORLD_XZ: isize = 24;
pub const WORLD_Y: isize = 1;

pub const WORLD_HEIGHT: usize = WORLD_Y as usize * CHUNK_Y;

#[derive(Default, Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum VoxelId {
    #[default]
    Stone = 1,
    Sand = 2,
    Grass = 3,
    Snow = 4,
    Water = 5,
}

impl VoxelId {
    pub fn from_u8(value: u8) -> Option<VoxelId> {
        match value {
            1 => Some(VoxelId::Stone),
            2 => Some(VoxelId::Sand),
            3 => Some(VoxelId::Grass),
            4 => Some(VoxelId::Snow),
            5 => Some(VoxelId::Water),
            _ => None,
        }
    }
}

#[derive(Default, Copy, Clone, Debug, PartialEq)]
pub enum VoxelType {
    #[default]
    Air,
    Solid {
        id: VoxelId,
    },
    Partial {
        id: VoxelId,
        density: u8,
    },
}

impl VoxelType {
    /// Get the numerical index of this voxel type
    pub fn index(&self) -> u8 {
        match self {
            VoxelType::Air => 0u8,
            VoxelType::Solid { id } => *id as u8,
            VoxelType::Partial { id, density: _ } => *id as u8,
        }
    }

    /// Get the density of this block mapped as a float
    pub fn density_as_float(&self) -> f32 {
        match self {
            VoxelType::Air => 0.0,
            VoxelType::Partial { id: _, density } => (*density as f32) / 100.0,
            VoxelType::Solid { id: _ } => 1.0,
        }
    }
}

// 1,250 (before)

/// Represents a single chunk in the world
#[derive(Component, Debug)]
pub struct Chunk {
    /// 1D Array of all blocks in this chunk
    blocks: Vec<u16>,
}

impl Chunk {
    /// Create a new chunk with the correct internal voxel size (all air)
    pub fn new() -> Self {
        Self {
            blocks: vec![0; ChunkShape::SIZE],
        }
    }

    /// Set the block type at the provided position
    pub fn set_block(&mut self, x: usize, y: usize, z: usize, voxel_type: VoxelType) {
        self.blocks[ChunkShape::linearize([x, y, z])] = Self::voxel_to_internal(voxel_type);
    }

    /// Get the block type at the provided position
    pub fn get_block(&self, x: usize, y: usize, z: usize) -> VoxelType {
        // If outside this chunk
        if (x < 0) || (y < 0) || (z < 0) || (x >= CHUNK_XZ) || (y >= CHUNK_Y) || (z >= CHUNK_XZ) {
            return VoxelType::Air;
        }

        Self::internal_to_voxel(self.blocks[ChunkShape::linearize([x, y, z])])
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
        self.get_block(
            position.x as usize,
            position.y as usize,
            position.z as usize,
        )
    }

    /// Returns if the specified block is transparent (air, water, etc.)
    /// Used for block culling
    pub fn is_transparent(&self, world_position: Vec3, position: Vec3, terrain: &Terrain) -> bool {
        self.get_t_block(world_position, position, terrain) == VoxelType::Air
    }

    /// Turns the internal representation of a voxel to a friendly
    /// external format
    fn internal_to_voxel(internal: u16) -> VoxelType {
        let mut buf = [0; 2];
        buf.copy_from_slice(&internal.to_le_bytes());

        if buf[0] == 100 {
            VoxelType::Solid {
                id: VoxelId::from_u8(buf[1]).unwrap(),
            }
        } else if buf[0] == 0 {
            VoxelType::Air
        } else {
            VoxelType::Partial {
                id: VoxelId::from_u8(buf[1]).unwrap(),
                density: buf[0],
            }
        }
    }

    /// Turns a friendly representation of a voxel to the internal
    /// packed representation
    fn voxel_to_internal(voxel: VoxelType) -> u16 {
        let density = match voxel {
            VoxelType::Air => 0,
            VoxelType::Solid { id: _ } => 100,
            VoxelType::Partial { id: _, density } => density,
        };

        let id = match voxel {
            VoxelType::Air => 0u8,
            VoxelType::Solid { id } => id as u8,
            VoxelType::Partial { id, density: _ } => id as u8,
        };

        u16::from_le_bytes([density, id].try_into().unwrap())
    }
}

// TODO: Replace with (ChunkId, MaterialMeshBundle<ChunkMaterial>)
#[derive(Default, Bundle, Clone)]
pub struct ChunkBundle {
    /// The id of this chunk, used to link up to the world
    pub chunk_id: ChunkId,

    // Bundle containing stuff needed for the mesh
    pub mesh: MaterialMeshBundle<ChunkMaterial>,
}
