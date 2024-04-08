mod atlas;
mod chunk;
mod player;
mod table;
mod terrain;
mod world;

use bevy::{
    core_pipeline::{
        bloom::BloomSettings,
        experimental::taa::{TemporalAntiAliasBundle, TemporalAntiAliasPlugin},
    },
    pbr::{light_consts::lux, ScreenSpaceAmbientOcclusionBundle},
    prelude::*,
};
use bevy_asset_loader::prelude::*;
use bevy_atmosphere::{
    collection::nishita::Nishita,
    model::AtmosphereModel,
    plugin::{AtmosphereCamera, AtmospherePlugin},
    system_param::AtmosphereMut,
};
use bevy_rapier3d::prelude::*;
use chunk::{material::ChunkMaterial, tile_map::TileAssets};
use iyes_perf_ui::{PerfUiCompleteBundle, PerfUiPlugin};
use player::PlayerPlugin;
use smooth_bevy_cameras::{
    controllers::fps::{FpsCameraBundle, FpsCameraController, FpsCameraPlugin},
    LookTransformPlugin,
};
use terrain::Terrain;
use world::WorldPlugin;

#[derive(Component)]
pub struct Player;

// Marker for updating the position of the light, not needed unless we have multiple lights
#[derive(Component)]
struct Sun;

// Timer for updating the daylight cycle (updating the atmosphere every frame is slow, so it's better to do incremental changes)
#[derive(Resource)]
struct CycleTimer(Timer);

#[derive(AssetCollection, Resource)]
pub struct GeneralAssets {
    #[asset(path = "environment/pisa_diffuse_rgb9e5_zstd.ktx2")]
    pub diffuse_map: Handle<Image>,

    #[asset(path = "environment/pisa_specular_rgb9e5_zstd.ktx2")]
    pub specular_map: Handle<Image>,

    #[asset(path = "fonts/BerkeleyMono-Regular.ttf")]
    pub ui_font: Handle<Font>,
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
        .insert_resource(AtmosphereModel::default())
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 130.,
        })
        .insert_resource(Terrain::new(rand::random::<u64>()))
        .insert_resource(CycleTimer(Timer::new(
            bevy::utils::Duration::from_millis(50),
            TimerMode::Repeating,
        )))
        .add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
        .add_plugins(bevy::diagnostic::EntityCountDiagnosticsPlugin)
        .add_plugins(bevy::diagnostic::SystemInformationDiagnosticsPlugin)
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
        .add_plugins(PlayerPlugin)
        .add_plugins(WorldPlugin)
        .add_plugins(PerfUiPlugin)
        .add_plugins(AtmospherePlugin)
        .add_plugins(LookTransformPlugin)
        .add_plugins(FpsCameraPlugin::default())
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(MaterialPlugin::<ChunkMaterial>::default())
        .add_loading_state(
            LoadingState::new(AppState::Loading)
                .load_collection::<GeneralAssets>()
                .load_collection::<TileAssets>()
                .continue_to_state(AppState::InGame),
        )
        .add_systems(OnEnter(AppState::InGame), setup)
        .add_systems(Update, daylight_cycle.run_if(in_state(AppState::InGame)))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Sun
    commands.spawn((
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                shadows_enabled: true,
                illuminance: lux::AMBIENT_DAYLIGHT,
                ..default()
            },

            ..default()
        },
        Sun,
    ));

    // Camera
    commands
        .spawn((
            Player,
            Camera3dBundle {
                camera: Camera {
                    hdr: true,
                    ..default()
                },
                projection: Projection::Perspective(PerspectiveProjection {
                    far: 2000.0,
                    ..default()
                }),
                ..default()
            },
            BloomSettings::NATURAL,
            AtmosphereCamera::default(),
            FogSettings {
                color: Color::rgba(0.2, 0.2, 0.2, 1.0),
                directional_light_color: Color::rgba(1.0, 0.95, 0.75, 0.5),
                directional_light_exponent: 6.0,
                falloff: FogFalloff::from_visibility_color(600.0, Color::rgb(1.0, 0.95, 0.75)),
            },
        ))
        .insert(ScreenSpaceAmbientOcclusionBundle::default())
        .insert(TemporalAntiAliasBundle::default())
        .insert(FpsCameraBundle::new(
            FpsCameraController {
                translate_sensitivity: 50.0,
                ..default()
            },
            Vec3::new(0.0, 32.0, 5.0),
            Vec3::new(0., 32.0, 0.),
            Vec3::Y,
        ))
        .insert(PbrBundle {
            mesh: meshes.add(Mesh::from(Capsule3d::new(0.5, 1.0))),
            material: materials.add(StandardMaterial::from(Color::rgb(0.0, 0.0, 0.0))),
            ..default()
        })
        .insert(Collider::capsule_y(0.5, 0.5));

    commands.spawn(PerfUiCompleteBundle::default());
    //

    //.insert(RigidBody::KinematicPositionBased)
    //.insert(Collider::capsule_y(1.0, 1.0))
    //.insert(LockedAxes::ROTATION_LOCKED)
    //.insert(Ccd::enabled())
    //.insert(AtmosphereCamera(None));
}

// We can edit the Atmosphere resource and it will be updated automatically
fn daylight_cycle(
    mut atmosphere: AtmosphereMut<Nishita>,
    mut query: Query<(&mut Transform, &mut DirectionalLight), With<Sun>>,
    mut timer: ResMut<CycleTimer>,
    time: Res<Time>,
) {
    timer.0.tick(time.delta());

    if timer.0.finished() {
        let start_offset = 0.45;

        let t = start_offset + (time.elapsed_seconds_wrapped() / 2000.0);
        atmosphere.sun_position = Vec3::new(0., t.sin(), t.cos());

        if let Some((mut light_trans, mut directional)) = query.single_mut().into() {
            light_trans.rotation = Quat::from_rotation_x(-t);
            directional.illuminance = t.sin().max(0.0).powf(2.0) * lux::AMBIENT_DAYLIGHT;
        }
    }
}
