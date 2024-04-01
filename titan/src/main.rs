mod atlas;
mod chunk;
mod table;
mod terrain;
mod world;

use bevy::{
    core_pipeline::experimental::taa::TemporalAntiAliasPlugin,
    pbr::ScreenSpaceAmbientOcclusionBundle, prelude::*,
};
use bevy_asset_loader::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;
use chunk::{material::ChunkMaterial, tile_map::TileAssets};
use smooth_bevy_cameras::{
    controllers::fps::{FpsCameraBundle, FpsCameraController, FpsCameraPlugin},
    LookTransformPlugin,
};
use terrain::Terrain;
use world::WorldPlugin;

#[derive(Component)]
pub struct Player;

#[derive(AssetCollection, Resource)]
pub struct GeneralAssets {
    #[asset(path = "environment/pisa_diffuse_rgb9e5_zstd.ktx2")]
    pub diffuse_map: Handle<Image>,

    #[asset(path = "environment/pisa_specular_rgb9e5_zstd.ktx2")]
    pub specular_map: Handle<Image>,
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum AppState {
    #[default]
    Loading,
    InGame,
}

fn main() {
    App::new()
        .init_state::<AppState>()
        //.insert_resource(AtmosphereModel::default())
        .insert_resource(ClearColor(Color::rgb(0.5294, 0.8078, 0.9216)))
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 0.15,
        })
        .insert_resource(Terrain::new(rand::random::<u64>()))
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    mode: AssetMode::Unprocessed,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Voxel Game - Dominic Maas".to_string(),
                        resolution: (1920.0, 1080.0).into(),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(TemporalAntiAliasPlugin)
        .add_plugins(WorldPlugin)
        .add_plugins(EguiPlugin)
        //.add_plugins(AtmospherePlugin)
        .add_plugins(LookTransformPlugin)
        .add_plugins(FpsCameraPlugin::default())
        //.add_plugins(WorldInspectorPlugin::new())
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(MaterialPlugin::<ChunkMaterial>::default())
        //.add_plugin(RapierDebugRenderPlugin::default())
        .add_loading_state(
            LoadingState::new(AppState::Loading)
                .load_collection::<GeneralAssets>()
                .load_collection::<TileAssets>()
                .continue_to_state(AppState::InGame),
        )
        .add_systems(OnEnter(AppState::InGame), setup)
        .add_systems(Update, process_ui.run_if(in_state(AppState::InGame)))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Sun
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::rgb(0.98, 0.95, 0.82),
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 0.0, 0.0)
            .looking_at(Vec3::new(-0.15, -0.05, 0.25), Vec3::Y),
        ..default()
    });
    // Sky
    /*commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::default())),
            material: materials.add(StandardMaterial {
                base_color: Color::hex("888888").unwrap(),
                unlit: true,
                cull_mode: None,
                ..default()
            }),
            transform: Transform::from_scale(Vec3::splat(1000.0)),
            ..default()
        },
        NotShadowCaster,
    ))*/

    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(Cuboid::from_size((1.0, 1.0, 1.0).into()))),
            material: materials.add(StandardMaterial::from(Color::rgb(0.8, 0.2, 0.2))),
            ..default()
        })
        .insert(RigidBody::Dynamic)
        .insert(Collider::cuboid(0.5, 0.5, 0.5))
        .insert(Restitution::coefficient(0.7))
        .insert(TransformBundle::from(Transform::from_xyz(0.0, 64.0, 0.0)));

    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(Cuboid::from_size((1.0, 1.0, 1.0).into()))),
            material: materials.add(StandardMaterial::from(Color::rgb(0.2, 0.2, 0.8))),
            ..default()
        })
        .insert(RigidBody::Dynamic)
        .insert(Collider::cuboid(0.5, 0.5, 0.5))
        .insert(Restitution::coefficient(0.7))
        .insert(TransformBundle::from(Transform::from_xyz(0.0, 64.0, 1.0)));

    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(Cuboid::from_size((2.0, 2.0, 2.0).into()))),
            material: materials.add(StandardMaterial::from(Color::rgb(1.0, 1.0, 1.0))),
            ..default()
        })
        .insert(RigidBody::Dynamic)
        .insert(Collider::cuboid(1.0, 1.0, 1.0))
        .insert(Restitution::coefficient(0.7))
        .insert(TransformBundle::from(Transform::from_xyz(1.0, 64.0, 0.0)));

    // Camera
    commands
        .spawn((
            Player,
            Camera3dBundle {
                camera: Camera {
                    hdr: true,
                    ..default()
                },
                ..default()
            },
            /*FogSettings {
                color: Color::rgba(0.2, 0.2, 0.2, 1.0),
                directional_light_color: Color::rgba(1.0, 0.95, 0.75, 0.5),
                directional_light_exponent: 5.0,
                falloff: FogFalloff::from_visibility_colors(
                    100.0, // distance in world units up to which objects retain visibility (>= 5% contrast)
                    Color::rgb(0.35, 0.5, 0.33), // atmospheric extinction color (after light is lost due to absorption by atmospheric particles)
                    Color::rgb(0.8, 0.8, 0.4), // atmospheric inscattering color (light gained due to scattering from the sun)
                ),
            },*/
        ))
        .insert(ScreenSpaceAmbientOcclusionBundle::default())
        .insert(FpsCameraBundle::new(
            FpsCameraController {
                translate_sensitivity: 40.0,
                ..default()
            },
            Vec3::new(0.0, 32.0, 5.0),
            Vec3::new(0., 32.0, 0.),
            Vec3::Y,
        ))
        .insert(PbrBundle {
            mesh: meshes.add(Mesh::from(Capsule3d::new(0.5, 0.5))),
            material: materials.add(StandardMaterial::from(Color::rgb(0.0, 0.0, 0.0))),
            ..default()
        });
    //.insert(TemporalAntiAliasBundle::default())
    //.insert(Collider::capsule_y(1.0, 1.0))

    //.insert(RigidBody::KinematicPositionBased)
    //.insert(Collider::capsule_y(1.0, 1.0))
    //.insert(LockedAxes::ROTATION_LOCKED)
    //.insert(Ccd::enabled())
    //.insert(AtmosphereCamera(None));
}

fn process_ui(mut contexts: EguiContexts) {
    //, mut atmosphere: AtmosphereMut<Nishita>) {
    egui::Window::new("Voxel Demo").show(contexts.ctx_mut(), |ui| {
        ui.label("Created by Dominic Maas");
        ui.separator();

        //ui.label("Sun Position: ");
        // ui.add(egui::Slider::new(&mut atmosphere.sun_position.x, 0.0..=1.0));
        // ui.add(egui::Slider::new(&mut atmosphere.sun_position.y, 0.0..=1.0));
        //ui.add(egui::Slider::new(&mut atmosphere.sun_position.z, 0.0..=1.0));
    });
}
