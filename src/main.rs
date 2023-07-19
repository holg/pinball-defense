use assets::PinballDefenseAssets;
use ball::BallPlugin;
use ball_camera::BallCameraPlugin;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
//#[cfg(debug_assertions)]
//use bevy_debug_grid::*;
#[cfg(debug_assertions)]
use bevy_inspector_egui::quick::WorldInspectorPlugin;
//use bevy_window_title_diagnostics::WindowTitleLoggerDiagnosticsPlugin;
use collision_handler::CollisionHandlerPlugin;
use controls::ControlsPlugin;
use fps_camera::FirstPersonCameraPlugin;
use prelude::*;
use tower::TowerPlugin;
use world::WorldPlugin;

mod assets;
mod ball;
mod ball_camera;
mod ball_starter;
mod collision_handler;
mod controls;
mod enemy;
mod flipper;
mod fps_camera;
mod prelude;
mod road;
mod tower;
mod world;

#[derive(States, PartialEq, Eq, Clone, Copy, Debug, Hash, Default)]
pub enum GameState {
    #[default]
    Loading,
    Ingame,
}

#[derive(States, Debug, Clone, Eq, PartialEq, Hash, Default)]
pub enum CameraState {
    #[default]
    None,
    BallCamera,
    FpsCamera,
}

fn main() {
    let mut app = App::new();

    app.add_state::<GameState>()
        .add_state::<CameraState>()
        .add_loading_state(
            LoadingState::new(GameState::Loading).continue_to_state(GameState::Ingame),
        )
        .add_collection_to_loading_state::<_, PinballDefenseAssets>(GameState::Loading)
        .add_plugins(DefaultPlugins);

    // Only show debug data in debug mode
    #[cfg(debug_assertions)]
    add_debug_plugins(&mut app);

    app.add_plugins((
        FrameTimeDiagnosticsPlugin,
        //.add_plugin(WindowTitleLoggerDiagnosticsPlugin::default())
        FirstPersonCameraPlugin,
        WorldPlugin,
        BallPlugin,
        BallCameraPlugin,
        TowerPlugin,
        ControlsPlugin,
        CollisionHandlerPlugin,
    ));

    add_rapier(&mut app);
    app.add_systems(Startup, setup_graphics).run();
}

fn add_rapier(app: &mut App) {
    let rapier_cfg = RapierConfiguration {
        //timestep_mode: TimestepMode::Variable {
        //max_dt: 1. / 128.,
        //time_scale: 1.,
        //substeps: 1,
        //},
        // timestep_mode: TimestepMode::Fixed {
        //dt: 1. / 64.,
        //substeps: 2,
        //},
        ..default()
    };
    app.insert_resource(rapier_cfg)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default());
}

#[cfg(debug_assertions)]
fn add_debug_plugins(app: &mut App) {
    app.add_plugins((RapierDebugRenderPlugin::default(), WorldInspectorPlugin::default()))
        //.add_plugin(DebugGridPlugin::with_floor_grid())
        ;
}

#[derive(Component)]
struct Camera;

fn setup_graphics(mut cmds: Commands) {
    cmds.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.2,
    });
}
