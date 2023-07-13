mod atlas;
mod chunk;
mod table;
mod terrain;
mod world;

use std::time::Duration;

use bevy::{
    asset::ChangeWatcher,
    core_pipeline::{
        experimental::taa::{TemporalAntiAliasBundle, TemporalAntiAliasPlugin},
        Skybox,
    },
    pbr::{CascadeShadowConfigBuilder, ScreenSpaceAmbientOcclusionBundle},
    prelude::*,
};
use bevy_asset_loader::prelude::*;
use bevy_atmosphere::prelude::*;
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
        .add_state::<AppState>()
        .insert_resource(Msaa::Sample8)
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
                    watch_for_changes: ChangeWatcher::with_delay(Duration::from_millis(200)),
                    ..Default::default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Voxel Game - Dominic Maas".to_string(),
                        resolution: (1920.0, 1080.0).into(),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
        )
        .add_plugins(TemporalAntiAliasPlugin)
        .add_plugins(WorldPlugin)
        .add_plugins(EguiPlugin)
        //.add_plugins(AtmospherePlugin)
        .add_plugins(LookTransformPlugin)
        .add_plugins(FpsCameraPlugin::default())
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(MaterialPlugin::<ChunkMaterial>::default())
        //.add_plugin(RapierDebugRenderPlugin::default())
        .add_loading_state(LoadingState::new(AppState::Loading).continue_to_state(AppState::InGame))
        .add_collection_to_loading_state::<_, TileAssets>(AppState::Loading)
        .add_collection_to_loading_state::<_, GeneralAssets>(AppState::Loading)
        .add_systems(OnEnter(AppState::InGame), setup)
        .add_systems(Update, process_ui.run_if(in_state(AppState::InGame)))
        .run();
}

fn setup(
    //mut atmosphere: ResMut<Atmosphere>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    assets: Res<GeneralAssets>,
) {
    // Sun
    let sun_val: f32 = 2.7;
    let sun_pos = Vec3::new(0.0, sun_val.sin(), sun_val.cos());

    //atmosphere.sun_position = sun_pos;

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform {
            rotation: Quat::from_rotation_x(-sun_pos.y.atan2(sun_pos.z)),
            ..default()
        },
        cascade_shadow_config: CascadeShadowConfigBuilder {
            first_cascade_far_bound: 4.0,
            num_cascades: 5,
            ..Default::default()
        }
        .into(),
        ..Default::default()
    });

    /*commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(StandardMaterial::from(Color::rgb(0.8, 0.2, 0.2))),
            ..Default::default()
        })
        .insert(RigidBody::Dynamic)
        .insert(Collider::cuboid(0.5, 0.5, 0.5))
        .insert(Restitution::coefficient(0.7))
        .insert_bundle(TransformBundle::from(Transform::from_xyz(0.0, 32.0, 0.0)));

    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(StandardMaterial::from(Color::rgb(0.2, 0.2, 0.8))),
            ..Default::default()
        })
        .insert(RigidBody::Dynamic)
        .insert(Collider::cuboid(0.5, 0.5, 0.5))
        .insert(Restitution::coefficient(0.7))
        .insert_bundle(TransformBundle::from(Transform::from_xyz(0.0, 34.0, 1.0)));

    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 2.0 })),
            material: materials.add(StandardMaterial::from(Color::rgb(1.0, 1.0, 1.0))),
            ..Default::default()
        })
        .insert(RigidBody::Dynamic)
        .insert(Collider::cuboid(1.0, 1.0, 1.0))
        .insert(Restitution::coefficient(0.7))
        .insert_bundle(TransformBundle::from(Transform::from_xyz(1.0, 36.0, 0.0)));*/

    // Camera
    commands
        .spawn(Camera3dBundle {
            camera: Camera {
                hdr: true,
                ..Default::default()
            },
            ..Default::default()
        })
        // .insert(EnvironmentMapLight {
        //     diffuse_map: assets.diffuse_map.clone(),
        //     specular_map: assets.specular_map.clone(),
        //})
        //.insert(Skybox(assets.diffuse_map.clone()))
        .insert(FpsCameraBundle::new(
            FpsCameraController {
                translate_sensitivity: 20.0,
                ..Default::default()
            },
            Vec3::new(0.0, 32.0, 5.0),
            Vec3::new(0., 32.0, 0.),
            Vec3::Y,
        ))
        .insert(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Capsule {
                radius: 1.0,
                rings: 0,
                depth: 2.0,
                latitudes: 16,
                longitudes: 32,
                uv_profile: shape::CapsuleUvProfile::Aspect,
            })),
            material: materials.add(StandardMaterial::from(Color::rgb(0.0, 0.0, 0.0))),
            ..Default::default()
        })
        .insert(TemporalAntiAliasBundle::default())
        .insert(AtmosphereCamera::default())
        .insert(Player {});

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
