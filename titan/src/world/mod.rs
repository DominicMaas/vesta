mod systems;

use bevy::{tasks::Task, utils::HashMap};
use ndshape::ConstShape;

use self::systems::{
    apply_chunk_load_tasks, chunk_gizmos, prepare_chunk_load_tasks, process_chunk_state_on_camera,
    setup,
};
use crate::chunk::{ChunkShape, VoxelType, CHUNK_XZ};
use crate::ChunkMaterial;
use crate::{
    chunk::{Chunk, ChunkId},
    AppState,
};
use bevy::prelude::*;
use std::collections::VecDeque;

/// How many chunks away from the player to render (horizontally)
pub const RENDER_DISTANCE: usize = 28;

// A simple queue that keeps track of what chunks currently
// need to be loaded into the world. This is done based on the id of the chunk
#[derive(Default, Resource)]
pub struct ChunkLoadQueue(pub VecDeque<ChunkId>);

#[derive(Component)]
pub struct ChunkLoadTask(Task<(ChunkId, Chunk, Mesh)>);

pub struct WorldPlugin;

// TODO: Split meshing into a separate task

// When we add a chunk id to the load queue, we are telling it to either load
// a chunk from disk or generating a new one. For now we are always generating

// When we add a chunk id to the unload queue, we are telling it to save the chunk to disk

// When we add a chunk id to the rebuild queue, we are telling it to rebuild the mesh

// LoadQueue -> [0,0] -> ChunkLoadTask
//

// LoadQueue
// RebuildQueue
// UnloadQueue

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(World::default())
            .insert_resource(ChunkLoadQueue {
                0: VecDeque::with_capacity(10),
            })
            .add_systems(OnEnter(AppState::InGame), setup)
            //.add_systems(Update, chunk_gizmos.run_if(in_state(AppState::InGame)))
            .add_systems(
                Update,
                process_chunk_state_on_camera.run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                Update,
                prepare_chunk_load_tasks.run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                Update,
                apply_chunk_load_tasks.run_if(in_state(AppState::InGame)),
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
