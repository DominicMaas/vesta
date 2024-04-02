mod systems;

use bevy::{tasks::Task, utils::HashMap};

use self::systems::{
    apply_chunk_load_tasks, prepare_chunk_load_tasks, process_chunk_state_on_camera, setup,
};
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

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(World::default())
            .insert_resource(ChunkLoadQueue {
                0: VecDeque::with_capacity(10),
            })
            .add_systems(OnEnter(AppState::InGame), setup)
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
    //pub chunk_material: Handle<ChunkMaterial>,
    pub chunk_material: Handle<StandardMaterial>,
}
