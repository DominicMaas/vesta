use bevy::{math::bounding::RayCast3d, prelude::*};

use crate::{chunk::{ChunkId, VoxelType}, GeneralAssets, Player};

#[derive(Component)]
pub struct DebugMenu;

pub fn setup(
    mut commands: Commands,
    assets: Res<GeneralAssets>,
    mut config_store: ResMut<GizmoConfigStore>,
) {
    // Set depth for gizmos
    for (_, config, _) in config_store.iter_mut() {
        config.line_width = 5.0;
        config.depth_bias = -1.0;
    }

    // Player debug text

    let style = TextStyle {
        font: assets.ui_font.clone(),
        font_size: 20.0,
        ..default()
    };

    commands
        .spawn(
            TextBundle::from_sections(vec![
                TextSection::new("Position: -\n", style.clone()),
                TextSection::new("Facing: -\n", style.clone()),
                TextSection::new("Chunk: -\n\n", style.clone()),
                TextSection::new("Selector Chunk: -\n", style.clone()),
                TextSection::new("Selector Block: -\n", style.clone()),
                TextSection::new("\nLoaded Chunks: -\n", style.clone()),
            ])
            .with_style(Style {
                position_type: PositionType::Absolute,
                top: Val::Px(12.0),
                left: Val::Px(12.0),
                ..default()
            }),
        )
        .insert(DebugMenu);
}

pub fn draw_cursor(
    camera_query: Query<(&Camera, &GlobalTransform), With<Player>>,
    windows: Query<&Window>,
    world: Res<crate::world::World>,
    mut text: Query<&mut Text, With<DebugMenu>>,
    mut gizmos: Gizmos,
) {
    let (camera, camera_transform) = camera_query.single();
    let window = windows.single();

    text.single_mut().sections[0].value =
        format!("Position: {:?}\n", camera_transform.translation().floor());

    text.single_mut().sections[1].value =
        format!("Facing: {:?}\n", camera_transform.forward().floor());

    text.single_mut().sections[2].value = format!(
        "Chunk: {}\n\n",
        crate::world::World::get_id_for_position(camera_transform.translation()),
    );

    text.single_mut().sections[3].value = format!("Selector Chunk: -\n");
    text.single_mut().sections[4].value = format!("Selector Block: -\n");
    text.single_mut().sections[5].value = format!("\nLoaded Chunks: {}\n", world.chunks.len());

    let w = window.width();
    let h = window.height();

    
    
    // Calculate a ray pointing from the camera into the world from the center of the screen
    if let Some(ray) = camera.viewport_to_world(camera_transform, (w / 2., h / 2.).into()) {
        let _ray_cast = RayCast3d::from_ray(ray, 200.0);

        //  for (hit_box, id) in chunks_query.iter()  {

        //    if let Some(_) = ray_cast.aabb_intersection_at(&Aabb3d::new(hit_box.center.into(), hit_box.half_extents.into())) {
        //        break;
        //    }
        // }

        let r = ray.get_point(10.0);
        text.single_mut().sections[3].value = format!(
            "Selector Chunk: {}\n",
            crate::world::World::get_id_for_position(r)
        );

        if let Some(b) = world.get_block(r) {
            if b != VoxelType::Air {
                text.single_mut().sections[4].value = format!("Selector Block: {:?}\n", b);

                gizmos.primitive_3d(
                    Cuboid {
                        half_size: (0.5, 0.5, 0.5).into(),
                    },
                    Vec3::new(f32::floor(r.x), f32::floor(r.y), f32::floor(r.z))
                        + Vec3::new(0.5, 0.5, 0.5),
                    Quat::from_rotation_x(0.0),
                    Color::GREEN,
                );
            }
        }
    };
}
