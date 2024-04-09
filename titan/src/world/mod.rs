mod generation;
mod meshing;
mod systems;

use bevy::{tasks::Task, utils::HashMap};

use self::generation::{process_generation_tasks, queue_generation_tasks};
use self::meshing::{process_meshing_tasks, queue_meshing_tasks};
use self::systems::{process_chunk_state_on_camera, setup};
use crate::chunk::{VoxelType, CHUNK_XZ};
use crate::ChunkMaterial;
use crate::{
    chunk::{Chunk, ChunkId},
    AppState,
};
use bevy::prelude::*;
use std::collections::VecDeque;

/// How many chunks away from the player to render (horizontally)
pub const RENDER_DISTANCE: usize = 20;

// The general idea:
// First, check around the player to determine which chunks need to be loaded. These chunks
// can either be loaded from disk, or generated (doesn't matter, we determine later). This is done
// by checking if a chunk exists within the chunk data structure

// [generation/queue_generation_tasks] Next, we loop through the queue, and spawn an ChunkId component and a ChunkGenerationTask
// (todo: Do we also want to spawn everything else we need for a mesh here as well?)

// [generation/process_generation_tasks] Chunk Generation loop takes in tasks, when they are complete, the generated world is added to the
// chunk data structure, the task is removed, and NeedsReMesh is added to the entity

// [meshing/queue_meshing_tasks] Meshing loop looks for chunks that have NeedRemesh, and adds a ChunkMeshingTask to the entity

// [meshing/process_meshing_tasks] Meshing complete loop looks for chunks that have ChunkMeshingTask, and once complete, removes this task
// and adds the mesh (or replaces the mesh if one exists)

// A simple queue that keeps track of what chunks currently
// need to be loaded into the world. This is done based on the id of the chunk
#[derive(Default, Resource)]
pub struct ChunkLoadQueue(pub VecDeque<ChunkId>);

// A simple queue that keeps track of what chunks currently
// need to be unloaded from the world. This is done based on the id of the chunk
#[derive(Default, Resource)]
pub struct ChunkUnloadQueue(pub VecDeque<ChunkId>);

/// Represents a task that takes in a chunk with data, and generates a mesh
#[derive(Component)]
pub struct ChunkMeshingTask(Task<Mesh>);

/// Represents a task that takes in a chunk position, and generates required data for it
#[derive(Component)]
pub struct ChunkGenerationTask(Task<Chunk>);

// When this component is added to chunk, it needs a mesh generated (or replaced),
// occurs after initial chunk generation and after chunk modification
#[derive(Component)]
pub struct NeedsRemesh;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(World::default())
            .insert_resource(ChunkLoadQueue {
                0: VecDeque::with_capacity(10),
            })
            .add_systems(OnEnter(AppState::InGame), setup)
            .add_systems(
                Update,
                (
                    process_chunk_state_on_camera,
                    queue_generation_tasks,
                    process_generation_tasks,
                    queue_meshing_tasks,
                    process_meshing_tasks,
                )
                    .chain()
                    .run_if(in_state(AppState::InGame)),
            );
    }
}

/// Represents a world
#[derive(Resource, Default)]
pub struct World {
    pub chunks: HashMap<ChunkId, Option<Chunk>>,
    pub chunk_material: Handle<ChunkMaterial>,
}

impl World {
    pub fn get_chunk(&self, id: ChunkId) -> Option<&Chunk> {
        if self.chunks.contains_key(&id) {
            match &self.chunks[&id] {
                Some(c) => Some(c),
                None => None,
            }
        } else {
            None
        }
    }

    pub fn get_block(&self, position: Vec3) -> Option<VoxelType> {
        let id = Self::get_id_for_position(position);
        let [x, y, z] = id.world_position_to_local(position);

        if self.chunks.contains_key(&id) {
            match &self.chunks[&id] {
                Some(chunk) => Some(chunk.get_block(x, y, z)),
                None => None,
            }
        } else {
            None
        }
    }

    /// Given world coordinates, determines the chunk id which contains these
    /// coordinates
    pub fn get_id_for_position(position: Vec3) -> ChunkId {
        let x = ((position.x / CHUNK_XZ as f32).floor() * CHUNK_XZ as f32) as isize;
        let z = ((position.z / CHUNK_XZ as f32).floor() * CHUNK_XZ as f32) as isize;

        ChunkId::new(x, z)
    }
}
