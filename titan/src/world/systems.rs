use crate::{
    atlas::TileAtlasBuilder,
    chunk::{
        material::ChunkMaterial, mesher::{ChunkMesher, CubeChunkMesher, MarchingChunkMesher}, tile_map::TileAssets, Chunk, ChunkBundle,
        ChunkId, CHUNK_XZ, CHUNK_Y,
    },
    terrain::Terrain,
    Player,
};
use bevy::{prelude::*, render::texture::ImageSampler, tasks::AsyncComputeTaskPool};
use bevy_rapier3d::prelude::*;
use futures_lite::future;

use super::{ChunkLoadQueue, ChunkLoadTask, RENDER_DISTANCE};

/// Ensures that the chunk material is loaded
pub fn setup(
    mut world: ResMut<crate::world::World>,
    mut materials: ResMut<Assets<ChunkMaterial>>,
    mut textures: ResMut<Assets<Image>>,
    tile_assets: Res<TileAssets>,
) {
    let mut builder = TileAtlasBuilder::new(Vec2::new(16.0, 16.0));

    // Add our textures
    for handle in tile_assets.tiles.iter() {
        let texture = textures.get(handle).unwrap();
        builder.add_texture(handle.clone(), texture).unwrap();
    }

    // Vertically stacked
    builder.max_columns(Some(1));

    // Build our atlas
    let atlas = builder.finish(&mut textures).unwrap();

    // Reinterpret our image as a stacked 2d array, and use near sampling
    // (our textures are pixel art)
    if let Some(atlas_image) = textures.get_mut(&atlas.texture) {
        atlas_image.reinterpret_stacked_2d_as_array(atlas.len() as u32);
        atlas_image.sampler_descriptor = ImageSampler::nearest();
    }

    world.chunk_material = materials.add(ChunkMaterial {
        texture: atlas.texture,
    });
}

/// Starts the process of managing chunks based on the
///  users view position
pub fn process_chunk_state_on_camera(
    query: Query<&Transform, With<Player>>,
    mut world: ResMut<crate::world::World>,
    mut queue: ResMut<ChunkLoadQueue>,
) {
    let transform = query.single();

    let render_distance = (RENDER_DISTANCE * CHUNK_XZ) as isize;

    let chunk_x = ((transform.translation.x / CHUNK_XZ as f32).floor() as isize
        * CHUNK_XZ as isize)
        - CHUNK_XZ as isize;

    let chunk_z = ((transform.translation.z / CHUNK_XZ as f32).floor() as isize
        * CHUNK_XZ as isize)
        - CHUNK_XZ as isize;

    for x in (chunk_x - render_distance..chunk_x + render_distance).step_by(CHUNK_XZ) {
        for z in (chunk_z - render_distance..chunk_z + render_distance).step_by(CHUNK_XZ) {
            // Determine the chunk id
            let chunk_id = ChunkId::new(x, z);

            // If this chunk doesn't exist, create it
            if !world.chunks.contains_key(&chunk_id) {
                // Insert an empty chunk into the world. This is just to allocate the position in the map
                // we will fill it with voxel data later
                world.chunks.insert(chunk_id, Chunk::empty());

                queue.0.push_back(chunk_id);
            }
        }
    }
}

pub fn prepare_chunk_load_tasks(
    mut commands: Commands,
    mut queue: ResMut<ChunkLoadQueue>,
    terrain_res: Res<Terrain>,
    mut world: ResMut<crate::world::World>,
) {
    let thread_pool = AsyncComputeTaskPool::get();

    let s = terrain_res.noise_func.get_seed();
    
    while let Some(chunk_id) = queue.0.pop_front() {
        if let Some(_) = world.chunks.get_mut(&chunk_id) {
            let task = thread_pool.spawn(async move {
                let terrain = Terrain::new(s);

                let chunk = terrain.generate2(chunk_id.world_position());
                let mesh = MarchingChunkMesher::build(&chunk, chunk_id.world_position(), &terrain).unwrap();

                (chunk_id, chunk, mesh)
            });

            commands.spawn(ChunkLoadTask(task));
        }
    }
}

pub fn apply_chunk_load_tasks(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut world: ResMut<crate::world::World>,

    mut tasks: Query<(Entity, &mut ChunkLoadTask)>,
) {
    for (entity, mut task) in &mut tasks {
        if let Some(chunk_data) = future::block_on(future::poll_once(&mut task.0)) {
            // Add this mesh to our world
            let chunk_mesh_handle = meshes.add(chunk_data.2);
            let collider_mesh = meshes.get(&chunk_mesh_handle.clone()).unwrap();

            // Ensure our world has the new chunk data
            world.chunks.insert(chunk_data.0, chunk_data.1);

            // Replace the chunk load task with our bundle
            commands
                .entity(entity)
                .remove::<ChunkLoadTask>()
                .insert(ChunkBundle {
                    chunk_id: chunk_data.0,
                    material: world.chunk_material.clone(),
                    transform: Transform::from_translation(chunk_data.0.world_position()),
                    ..Default::default()
                })
                .insert(chunk_mesh_handle)
                .insert(RigidBody::Fixed)
                .insert(Name::new(format!(
                    "Chunk: {}",
                    chunk_data.0.world_position()
                )))
                .insert(
                    Collider::from_bevy_mesh(&collider_mesh, &ComputedColliderShape::TriMesh)
                        .unwrap(),
                );
        }
    }
}

pub fn chunk_gizmos(mut gizmos: Gizmos, world: Res<crate::world::World>) {
    for (chunk_id, _) in world.chunks.iter() {
        let pos = chunk_id.world_position();

        //gizmos.cuboid(
        //     Transform::from_translation(chunk_id.world_position()).with_scale(Vec3::new(
        //        CHUNK_XZ as f32,
        //        CHUNK_Y as f32,
        //       CHUNK_XZ as f32,
        //   )),
        //   Color::BLACK,
        // );

        gizmos.line(pos, pos + Vec3::new(0.0, CHUNK_Y as f32, 0.0), Color::BLACK);
        gizmos.line(
            pos + Vec3::new(CHUNK_XZ as f32, 0.0, 0.0),
            pos + Vec3::new(CHUNK_XZ as f32, CHUNK_Y as f32, 0.0),
            Color::BLACK,
        );
        gizmos.line(
            pos + Vec3::new(0.0, 0.0, CHUNK_XZ as f32),
            pos + Vec3::new(0.0, CHUNK_Y as f32, CHUNK_XZ as f32),
            Color::BLACK,
        );
        gizmos.line(
            pos + Vec3::new(CHUNK_XZ as f32, 0.0, CHUNK_XZ as f32),
            pos + Vec3::new(CHUNK_XZ as f32, CHUNK_Y as f32, CHUNK_XZ as f32),
            Color::BLACK,
        );
    }
}
