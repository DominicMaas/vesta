use crate::{
    chunk::{material::ChunkMaterial, ChunkId, CHUNK_XZ, CHUNK_Y},
    Player,
};
use bevy::prelude::*;

use super::{ChunkLoadQueue, RENDER_DISTANCE};

/// Ensures that the chunk material is loaded
pub fn setup(mut world: ResMut<crate::world::World>, mut materials: ResMut<Assets<ChunkMaterial>>) {
    world.chunk_material = materials.add(ChunkMaterial {});
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

    let chunk_x = ((transform.translation.x / CHUNK_XZ as f32).floor() * CHUNK_XZ as f32) as isize;
    let chunk_z = ((transform.translation.z / CHUNK_XZ as f32).floor() * CHUNK_XZ as f32) as isize;

    for x in (chunk_x - render_distance..chunk_x + render_distance).step_by(CHUNK_XZ) {
        for z in (chunk_z - render_distance..chunk_z + render_distance).step_by(CHUNK_XZ) {
            // Determine the chunk id
            let chunk_id = ChunkId::new(x, z);

            // If this chunk doesn't exist, create it
            if !world.chunks.contains_key(&chunk_id) {
                // Only 1000 per frame
                if queue.0.len() > 1000 {
                    return;
                }

                // Insert an empty chunk into the world. This is just to allocate the position in the map
                // we will fill it with voxel data later
                world.chunks.insert(chunk_id, None);

                queue.0.push_back(chunk_id);
            }
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

        gizmos.line(pos, pos + Vec3::new(0.0, CHUNK_Y as f32, 0.0), Color::RED);
        gizmos.line(
            pos + Vec3::new(CHUNK_XZ as f32, 0.0, 0.0),
            pos + Vec3::new(CHUNK_XZ as f32, CHUNK_Y as f32, 0.0),
            Color::RED,
        );
        gizmos.line(
            pos + Vec3::new(0.0, 0.0, CHUNK_XZ as f32),
            pos + Vec3::new(0.0, CHUNK_Y as f32, CHUNK_XZ as f32),
            Color::RED,
        );
        gizmos.line(
            pos + Vec3::new(CHUNK_XZ as f32, 0.0, CHUNK_XZ as f32),
            pos + Vec3::new(CHUNK_XZ as f32, CHUNK_Y as f32, CHUNK_XZ as f32),
            Color::RED,
        );
    }
}
