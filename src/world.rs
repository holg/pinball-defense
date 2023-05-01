use crate::assets::PinballDefenseAssets;
use crate::ball::BallSpawn;
use crate::ball_starter::BallStarterPlugin;
use crate::flipper::FlipperPlugin;
use crate::prelude::*;
use crate::road::{add_road_path, animate_cube, spawn_road};
use crate::tower::{spawn_tower_machine_gun, spawn_tower_microwave, spawn_tower_tesla};
use crate::GameState;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(FlipperPlugin)
            .add_plugin(BallStarterPlugin)
            .add_system(setup_world.in_schedule(OnEnter(GameState::Ingame)))
            .add_system(animate_cube.in_set(OnUpdate(GameState::Ingame)));
    }
}

#[derive(Component)]
struct World;

#[derive(Component)]
struct Ground;

fn setup_world(
    mut cmds: Commands,
    mut ball_spawn: ResMut<BallSpawn>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut assets: ResMut<PinballDefenseAssets>,
) {
    cmds.spawn(SpatialBundle {
        transform: Transform::from_rotation(Quat::from_rotation_z(-0.25)),
        ..default()
    })
    .with_children(|parent| {
        parent
            .spawn((
                PbrBundle {
                    mesh: assets.world_1_mesh.clone(),
                    material: materials.add(StandardMaterial {
                        base_color: Color::BLUE,
                        perceptual_roughness: 0.5,
                        metallic: 0.5,
                        reflectance: 0.5,
                        ..default()
                    }),
                    ..default()
                },
                //Ccd::enabled(),
                ColliderDebugColor(Color::GOLD),
                Collider::from_bevy_mesh(
                    meshes
                        .get(&assets.world_1_mesh)
                        .expect("Failed to find mesh"),
                    &ComputedColliderShape::TriMesh,
                )
                .unwrap(),
            ))
            .insert(Ground);

        // Top glass
        let (x, y, z) = (2.60, 0.02, 1.40);
        parent
            .spawn((
                SpatialBundle {
                    transform: Transform::from_translation(Vec3::new(0., 0.06, 0.)),
                    ..default()
                },
                ColliderDebugColor(Color::GOLD),
                Collider::cuboid(x / 2., y / 2., z / 2.),
            ))
            .insert(Name::new("Pinball Glass"));
        parent.spawn(PointLightBundle {
            transform: Transform::from_xyz(1., 1., 0.5).looking_at(Vec3::ZERO, Vec3::Y),
            point_light: PointLight {
                intensity: 78.,
                color: Color::WHITE,
                shadows_enabled: true,
                radius: 0.1,
                range: 4.,
                ..default()
            },
            ..default()
        });

        // Ball starter
        let bs_pos = Vec3::new(1.175, -0.018, -0.657);
        crate::ball_starter::spawn(parent, bs_pos, &mut meshes, &mut materials);

        // Flipper left
        let fl_pos = Transform::from_xyz(0.83, -0.043, 0.32);
        crate::flipper::spawn_left(fl_pos, parent, &mut meshes, &mut materials, &mut assets);

        // Flipper right
        let fr_pos = Transform::from_xyz(0.83, -0.043, -0.246);
        crate::flipper::spawn_right(fr_pos, parent, &mut meshes, &mut materials, &mut assets);

        test_tower(parent, &mut materials, &assets);
        spawn_road(parent, &mut materials, &assets);
        add_road_path(parent, &assets, &mut meshes, &mut materials);
    })
    .insert(World)
    .insert(Name::new("Pinball World"));
    ball_spawn.0 = Vec3::new(0.96, -0.26, -0.6);
}

fn test_tower(
    parent: &mut ChildBuilder,
    materials: &mut Assets<StandardMaterial>,
    assets: &PinballDefenseAssets,
) {
    spawn_tower_microwave(parent, materials, assets, Vec3::new(0., -0.025, -0.2));
    spawn_tower_machine_gun(parent, materials, assets, Vec3::new(0., -0.025, 0.2));
    spawn_tower_tesla(parent, materials, assets, Vec3::new(0., -0.025, 0.));
}
