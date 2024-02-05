use bevy::{prelude::*, asset::HandleId};

use crate::{BlockType, CHUNK_HEIGHT, CHUNK_WIDTH};

use super::generator::Generator;

#[derive(Copy, Clone, Debug)]
pub enum ChunkState {
    /// The chunk has been created but there is no information associated with it
    Created,

    /// The chunk is currently loading (building terrain)
    Loading,

    /// The chunk is loaded and rendering as usual
    Loaded,

    /// The chunk is currently dirty and needs to be rebuilt
    Dirty,
}

#[derive(Bundle)]
pub struct ChunkBundle {
    //pub chunk: Chunk,
    //pub mesh: Handle<Mesh>,
    //pub material: Handle<StandardMaterial>,
    pub mesh: Handle<Mesh>,
    pub transform: Transform,
    pub material: Handle<StandardMaterial>,
}

impl ChunkBundle {
    pub fn new(mesh: Handle<Mesh>, chunk_material_id: HandleId) -> ChunkBundle {
    
        ChunkBundle {
            
            material: materials.get(handle),
            mesh,
            transform: Transform::from_xyz(0.0, 10.0, 10.0),
        }
    }
}

#[derive(Component)]
pub struct Chunk {
    /// What state the chunks is in, this determines how this chunk is treated in the world
    state: ChunkState,

    /// 1D Array of all blocks in this chunk
    blocks: Vec<BlockType>,
}

impl Chunk {
    /// Create a new chunk, this only performs the bare minimum in order to maximise
    /// parallel processing later on
    pub fn new() -> Self {
        Self {
            state: ChunkState::Created,
            blocks: vec![BlockType::Air; (CHUNK_WIDTH * CHUNK_WIDTH * CHUNK_HEIGHT) as usize],
        }
    }

    // ----- Block Array Helpers ----- //

    /// Set the block type at the provided position and mark the chunk as dirty
    pub fn set_block(&mut self, x: u32, y: u32, z: u32, block_type: BlockType) {
        match self.state {
            ChunkState::Dirty | ChunkState::Loaded | ChunkState::Loading => {
                self.blocks[(z * CHUNK_WIDTH * CHUNK_HEIGHT + y * CHUNK_WIDTH + x) as usize] =
                    block_type;
                self.state = ChunkState::Dirty;
            }
            _ => {
                println!(
                    "Cannot set block at position ({},{},{}), the chunk is not in a loaded state!",
                    x, y, z
                );
            }
        }
    }

    /// Get the block type of the block at the specified location
    pub fn get_block(&self, x: u32, y: u32, z: u32) -> BlockType {
        match self.state {
            ChunkState::Dirty | ChunkState::Loaded => {
                self.blocks[(z * CHUNK_WIDTH * CHUNK_HEIGHT + y * CHUNK_WIDTH + x) as usize]
            }
            _ => {
                println!(
                    "Cannot get block at position ({},{},{}), the chunk is not in a loaded state!",
                    x, y, z
                );
                BlockType::Air
            }
        }
    }

    fn get_block_type(&self, x: i32, y: i32, z: i32, generator: &Generator) -> BlockType {
        // Above the max possible chunk
        if y >= CHUNK_HEIGHT as i32 {
            return BlockType::Air;
        }

        // Outside of this chunk
        if (x < 0) || (z < 0) || (x >= CHUNK_WIDTH as i32) || (z >= CHUNK_WIDTH as i32) {
            // TODO: Check for existing chunks

            // This chunk is not loaded / does not exist, get the theoretical block type
            let mut world_pos = Vec3::new(x as f32, y as f32, z as f32);
            //world_pos += self.position;

            return generator.get_theoretical_block_type(world_pos);
        }

        // Get the block type within the chunk
        return self.get_block(x as u32, y as u32, z as u32);
    }

    pub fn get_state(&self) -> ChunkState {
        self.state
    }

    // Gets if the block at the specified position is transparent. Takes into account
    // water and air (air blocks are always transparent, water blocks are transparent to each other)
    fn is_transparent(&self, x: i32, y: i32, z: i32, generator: &Generator) -> bool {
        // Never render the bottom face of the world
        if y < 0 {
            return false;
        }

        return self.get_block_type(x, y, z, generator) == BlockType::Air
            || self.get_block_type(x, y, z, generator) == BlockType::Water { flowing: true }
            || self.get_block_type(x, y, z, generator) == BlockType::Water { flowing: false };
    }
}

pub fn create_mesh() -> Mesh {
    for x in 0..CHUNK_WIDTH {
        for y in 0..CHUNK_HEIGHT {
            for z in 0..CHUNK_WIDTH {}
        }
    }

    Mesh::from(shape::Cube { size: 1.0 })
}
