use bevy::{
    prelude::*,
    tasks::{futures_lite::future, AsyncComputeTaskPool},
};

use crate::{
    chunk::{
        mesher::{ChunkMesher, MarchingChunkMesher},
        ChunkId,
    },
    terrain::Terrain,
};

use super::{ChunkMeshingTask, NeedsRemesh};

pub fn queue_meshing_tasks(
    mut commands: Commands,
    terrain_res: Res<Terrain>,
    world: Res<crate::world::World>,
    dirty_chunks: Query<(Entity, &ChunkId), With<NeedsRemesh>>,
) {
    let thread_pool = AsyncComputeTaskPool::get();

    for (entity, &chunk_id) in dirty_chunks.iter() {
        // Only applicable if there is chunk data within the chunks resource
        if let Some(chunk_data) = world.get_chunk(chunk_id) {
            let chunk_data_local = chunk_data.clone(); // TODO: We don't want to do this!!!
            let seed = terrain_res.noise_func.get_seed();

            let task = thread_pool.spawn(async move {
                let terrain = Terrain::new(seed); // TODO: Share this globally
                MarchingChunkMesher::build(&chunk_data_local, chunk_id.world_position(), &terrain)
                    .unwrap()
            });

            commands
                .entity(entity)
                .try_insert(ChunkMeshingTask(task))
                .remove::<NeedsRemesh>();
        }
    }
}

pub fn process_meshing_tasks(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut meshing_tasks: Query<(Entity, &Handle<Mesh>, &mut ChunkMeshingTask), With<ChunkId>>,
) {
    for (entity, old_mesh, mut task) in meshing_tasks.iter_mut() {
        if let Some(new_mesh) = future::block_on(future::poll_once(&mut task.0)) {
            // Replace the mesh
            *meshes.get_mut(old_mesh).unwrap() = new_mesh;

            // have to remove AABB to force it to re-calculate
            commands
                .entity(entity)
                .remove::<ChunkMeshingTask>()
                .remove::<bevy::render::primitives::Aabb>();
        }
    }
}
