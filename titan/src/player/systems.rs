use bevy::{prelude::*, window::CursorGrabMode};

use crate::{
    chunk::{ChunkId, VoxelId, VoxelType},
    world::NeedsRemesh,
    GeneralAssets, Player,
};

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
    mut commands: Commands,
    camera_query: Query<(&Camera, &GlobalTransform), With<Player>>,
    mut windows: Query<&mut Window>,
    mut world: ResMut<crate::world::World>,
    mut text: Query<&mut Text, With<DebugMenu>>,
    mut gizmos: Gizmos,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    button_input: Res<ButtonInput<KeyCode>>,
    chunk_entities: Query<(Entity, &ChunkId)>,
) {
    if button_input.just_pressed(KeyCode::Escape) {
        let mut window = windows.single_mut();

        window.cursor.visible = true;
        window.cursor.grab_mode = CursorGrabMode::None;
    }

    if mouse_button_input.just_pressed(MouseButton::Left) {
        let mut window = windows.single_mut();

        window.cursor.visible = false;
        window.cursor.grab_mode = CursorGrabMode::Locked;
    }

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
        let mut prev_position: Option<Vec3> = None;
        let mut block_position: Option<(Vec3, VoxelType)> = None;

        for i in 0..20 {
            let r = ray.get_point(i as f32);

            if let Some(b) = world.get_block(r) {
                if b != VoxelType::Air {
                    block_position = Some((r, b));
                    break;
                }
            }

            prev_position = Some(r);
        }

        if let Some((position, voxel_type)) = block_position {
            let chunk_id = crate::world::World::get_id_for_position(position);

            text.single_mut().sections[3].value = format!("Selector Chunk: {}\n", chunk_id);
            text.single_mut().sections[4].value = format!("Selector Block: {:?}\n", voxel_type);

            if let Some(p) = prev_position {
                gizmos.primitive_3d(
                    Sphere { radius: 0.5 },
                    Vec3::new(f32::floor(p.x), f32::floor(p.y), f32::floor(p.z)),
                    Quat::from_rotation_x(0.0),
                    Color::GREEN,
                );
            }

            gizmos.primitive_3d(
                Sphere { radius: 0.5 },
                Vec3::new(
                    f32::floor(position.x),
                    f32::floor(position.y),
                    f32::floor(position.z),
                ),
                Quat::from_rotation_x(0.0),
                Color::RED,
            );

            if mouse_button_input.just_pressed(MouseButton::Left) {
                let place_pos = match prev_position {
                    Some(p) => p,
                    None => position,
                };

                world.set_block(place_pos, VoxelType::Solid { id: VoxelId::Stone });

                // mark dirty
                chunk_entities
                    .iter()
                    .filter(|(_entity, &id)| id == chunk_id)
                    .for_each(|(entity, _id)| {
                        commands.entity(entity).try_insert(NeedsRemesh);
                    });
            }

            if mouse_button_input.just_pressed(MouseButton::Right) {
                world.set_block(position, VoxelType::Air);

                // mark dirty
                chunk_entities
                    .iter()
                    .filter(|(_entity, &id)| id == chunk_id)
                    .for_each(|(entity, _id)| {
                        commands.entity(entity).try_insert(NeedsRemesh);
                    });
            }
        }
    };
}
