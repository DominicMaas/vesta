use crate::{
    chunk::{mesher, ChunkId},
    terrain::Terrain,
};

use super::{ChunkGenerationTask, ChunkLoadQueue, NeedsRemesh};

use bevy::{
    prelude::*,
    tasks::{futures_lite::future, AsyncComputeTaskPool},
};

pub fn queue_generation_tasks(
    mut commands: Commands,
    mut load_queue: ResMut<ChunkLoadQueue>,
    mut meshes: ResMut<Assets<Mesh>>,
    world: Res<crate::world::World>,
    terrain_res: Res<Terrain>,
) {
    let thread_pool = AsyncComputeTaskPool::get();

    while let Some(chunk_id) = load_queue.0.pop_front() {
        let seed = terrain_res.noise_func.get_seed();

        let task = thread_pool.spawn(async move {
            let terrain = Terrain::new(seed); // TODO: Share this globally
            terrain.generate2(chunk_id.world_position())
        });

        commands
            .spawn(chunk_id)
            .insert(MaterialMeshBundle {
                material: world.chunk_material.clone(),
                transform: Transform::from_translation(chunk_id.world_position()),
                mesh: meshes.add(mesher::empty_mesh()),
                ..default()
            })
            .insert(Name::new(format!("Chunk: {}", chunk_id)))
            .insert(ChunkGenerationTask(task));
    }
}

pub fn process_generation_tasks(
    mut commands: Commands,
    mut world: ResMut<crate::world::World>,
    mut generation_tasks: Query<(Entity, &ChunkId, &mut ChunkGenerationTask)>,
) {
    for (entity, chunk_id, mut task) in generation_tasks.iter_mut() {
        if let Some(chunk_data) = future::block_on(future::poll_once(&mut task.0)) {
            // Ensure our world has the new chunk data
            world.chunks.insert(*chunk_id, Some(chunk_data));

            // We now need a mesh!
            commands
                .entity(entity)
                .remove::<ChunkGenerationTask>()
                .try_insert(NeedsRemesh);
        }
    }
}
