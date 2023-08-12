use super::ball::CollisionWithBallEvent;
use super::events::collision::COLLIDE_ONLY_WITH_BALL;
use super::events::tween_completed::{ACTIVATE_PINBALL_MENU_EVENT_ID, DESPAWN_ENTITY_EVENT_ID};
use super::tower::foundation::{DespawnFoundationEvent, QuerySelected, SelectedTowerFoundation};
use super::tower::{SpawnTowerEvent, TowerType};
use super::world::QueryWorld;
use super::GameState;
use crate::prelude::*;
use crate::settings::GraphicsSettings;
use bevy_rapier3d::rapier::prelude::CollisionEventFlags;
use bevy_tweening::{
    lens::{TransformPositionLens, TransformRotateYLens},
    Animator, Delay, EaseFunction, Sequence, Tween,
};
use std::time::Duration;

pub struct PinballMenuPlugin;

impl Plugin for PinballMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PinballMenuEvent>().add_systems(
            Update,
            (menu_event_system, spawn_system, execute_system).run_if(in_state(GameState::Ingame)),
        );
    }
}

#[derive(Event, Debug, Clone, Copy)]
pub enum PinballMenuEvent {
    Disable,
    SetReady,
    Activate,
    Deactivate,
}

#[derive(Component, Debug, Clone, Copy, Default)]
enum PinballMenuStatus {
    #[default]
    Disabled,
    Ready,
    Activated,
}

fn menu_event_system(
    mut evr: EventReader<PinballMenuEvent>,
    mut q_pb_menu: Query<(Entity, &mut PinballMenuStatus), With<PinballMenu>>,
    cmds: Commands,
    q_pbm_el: QueryPinballMenuElements,
    q_lights: Query<&mut Visibility, With<PinballMenuElementLight>>,
    meshes: Res<Assets<Mesh>>,
    assets: Res<PinballDefenseAssets>,
) {
    if let Some(ev) = evr.iter().next() {
        if let Ok((menu_entity, mut status)) = q_pb_menu.get_single_mut() {
            use PinballMenuEvent::*;
            use PinballMenuStatus::*;
            if let Some(new_status) = match (ev, *status) {
                (Disable, Activated) => Some(despawn(cmds, q_lights, q_pbm_el, menu_entity)),
                (SetReady, Disabled) => Some(Ready),
                (Deactivate, Activated) => Some(deactivate(cmds, q_lights, q_pbm_el)), // TODO Status setzen
                (Activate, Ready) => Some(activate(cmds, q_lights, q_pbm_el, meshes, assets)),
                _ => None,
            } {
                *status = new_status;
            }
        }
    }
}

type QueryPinballMenuElements<'w, 's, 'a> =
    Query<'w, 's, (Entity, &'a Transform), With<PinballMenuElement>>;

fn spawn_system(
    mut cmds: Commands,
    mut mats: ResMut<Assets<StandardMaterial>>,
    assets: Res<PinballDefenseAssets>,
    q_pbw: QueryWorld,
    q_pb_menu: Query<&PinballMenu>,
    g_sett: Res<GraphicsSettings>,
    q_selected: Query<Entity, With<SelectedTowerFoundation>>,
) {
    if !q_selected.is_empty() && q_pb_menu.is_empty() {
        log!("🐢 Spawn tower menu for: {:?}", q_selected);
        cmds.entity(q_pbw.single()).with_children(|p| {
            let pos = Vec3::new(1.2, 0., 0.05);
            spawn(p, &mut mats, &assets, &g_sett, pos);
        });
    }
}

#[derive(Component)]
struct PinballMenu;

fn spawn(
    parent: &mut ChildBuilder,
    mats: &mut Assets<StandardMaterial>,
    assets: &PinballDefenseAssets,
    g_sett: &GraphicsSettings,
    pos: Vec3,
) {
    use TowerType::*;
    parent
        .spawn((
            spatial_from_pos(pos),
            PinballMenu,
            PinballMenuStatus::Disabled,
            Name::new("Tower Menu"),
        ))
        .with_children(|p| {
            spawn_menu_element(Gun, p, mats, assets, g_sett, -0.25, 0.1);
            spawn_menu_element(Microwave, p, mats, assets, g_sett, 0., 1.);
            spawn_menu_element(Tesla, p, mats, assets, g_sett, 0.25, 0.1);
        });
}

// Only pub(crate)for collision events
#[derive(Component)]
struct PinballMenuElement;

#[derive(Component)]
struct PinballMenuElementLight;

fn spawn_menu_element(
    tower_type: TowerType,
    parent: &mut ChildBuilder,
    mats: &mut Assets<StandardMaterial>,
    assets: &PinballDefenseAssets,
    g_sett: &GraphicsSettings,
    angle: f32,
    delay_secs: f32,
) {
    parent
        .spawn((
            PbrBundle {
                mesh: assets.menu_element.clone(),
                material: mats.add(StandardMaterial {
                    base_color: Color::MIDNIGHT_BLUE,
                    perceptual_roughness: 0.6,
                    metallic: 0.2,
                    reflectance: 0.8,
                    ..default()
                }),
                transform: Transform::from_translation(elem_start_pos()),
                ..default()
            },
            // Game components
            PinballMenuElement,
            Name::new("Tower Menu element"),
            tower_type,
            // Spawn animation
            Animator::new(spawn_animation(angle, delay_secs)),
        ))
        .with_children(|parent| {
            // Active status light
            parent.spawn((
                PointLightBundle {
                    visibility: Visibility::Hidden,
                    point_light: PointLight {
                        color: Color::GREEN,
                        intensity: 0.2,
                        shadows_enabled: g_sett.is_shadows,
                        ..default()
                    },
                    transform: Transform::from_translation(Vec3::new(-0.8, 0., 0.)),
                    ..default()
                },
                PinballMenuElementLight,
            ));
        });
}
fn despawn(
    mut cmds: Commands,
    q_lights: Query<&mut Visibility, With<PinballMenuElementLight>>,
    q_pbm_el: QueryPinballMenuElements,
    menu_entity: Entity,
) -> PinballMenuStatus {
    // Despawn menu
    let delay: Delay<Transform> =
        Delay::new(Duration::from_secs(2)).with_completed_event(DESPAWN_ENTITY_EVENT_ID);
    cmds.entity(menu_entity).insert(Animator::new(delay));
    // Despawn animation
    q_pbm_el.for_each(|(entity, trans)| {
        cmds.entity(entity)
            .insert(Animator::new(despawn_animation(trans.rotation.y)));
    });
    deactivate(cmds, q_lights, q_pbm_el);
    PinballMenuStatus::Disabled
}

fn activate(
    mut cmds: Commands,
    mut q_lights: Query<&mut Visibility, With<PinballMenuElementLight>>,
    q_pbm_el: QueryPinballMenuElements,
    meshes: Res<Assets<Mesh>>,
    assets: Res<PinballDefenseAssets>,
) -> PinballMenuStatus {
    q_pbm_el.for_each(|(entity, _)| {
        cmds.entity(entity)
            .insert((
                // Active status collider
                ColliderDebugColor(Color::GREEN),
                Sensor,
                ActiveEvents::COLLISION_EVENTS,
                Collider::from_bevy_mesh(
                    meshes
                        .get(&assets.menu_element.clone())
                        .expect("Failed to find mesh"),
                    &ComputedColliderShape::TriMesh,
                )
                .unwrap(),
                COLLIDE_ONLY_WITH_BALL,
            ))
            .remove::<ColliderDisabled>();
    });
    q_lights.for_each_mut(|mut visi| *visi = Visibility::Inherited);
    PinballMenuStatus::Activated
}

fn deactivate(
    mut cmds: Commands,
    mut q_lights: Query<&mut Visibility, With<PinballMenuElementLight>>,
    q_pbm_el: QueryPinballMenuElements,
) -> PinballMenuStatus {
    q_pbm_el.for_each(|(entity, _)| {
        cmds.entity(entity).remove::<Collider>();
    });
    q_lights.for_each_mut(|mut visi| *visi = Visibility::Hidden);
    PinballMenuStatus::Ready
}

fn elem_start_pos() -> Vec3 {
    Vec3::new(0., -0.1, 0.)
}

fn despawn_animation(angle: f32) -> Sequence<Transform> {
    let rotate = Tween::new(
        EaseFunction::QuadraticOut,
        Duration::from_secs_f32(1.),
        TransformRotateYLens {
            start: angle,
            end: 0.,
        },
    );
    let slide_down = Tween::new(
        EaseFunction::QuadraticIn,
        Duration::from_secs_f32(0.5),
        TransformPositionLens {
            start: Vec3::default(),
            end: elem_start_pos(),
        },
    );

    rotate.then(slide_down)
}

fn spawn_animation(angle: f32, delay_secs: f32) -> Sequence<Transform> {
    let wait = Delay::new(Duration::from_secs_f32(delay_secs));
    let slide_up = Tween::new(
        EaseFunction::QuadraticIn,
        Duration::from_secs(1),
        TransformPositionLens {
            start: elem_start_pos(),
            end: Vec3::default(),
        },
    );
    let rotate = Tween::new(
        EaseFunction::QuadraticIn,
        Duration::from_secs(1),
        TransformRotateYLens {
            start: 0.,
            end: angle,
        },
    );

    wait.then(slide_up)
        .then(rotate.with_completed_event(ACTIVATE_PINBALL_MENU_EVENT_ID))
}

fn execute_system(
    mut ball_coll_ev: EventReader<CollisionWithBallEvent>,
    mut despawn_foundation_ev: EventWriter<DespawnFoundationEvent>,
    mut pb_menu_ev: EventWriter<PinballMenuEvent>,
    mut spawn_tower_ev: EventWriter<SpawnTowerEvent>,
    q_menu_els: Query<(Entity, &TowerType), With<PinballMenuElement>>,
    q_selected: QuerySelected,
) {
    for CollisionWithBallEvent(id, flag) in ball_coll_ev.iter() {
        if *flag == CollisionEventFlags::SENSOR {
            if let Some((_, tower_type)) = q_menu_els.iter().find(|(el_id, _)| *el_id == *id) {
                if let Ok((_, sel_trans)) = q_selected.get_single() {
                    despawn_foundation_ev.send(DespawnFoundationEvent);

                    // Despawn menu
                    pb_menu_ev.send(PinballMenuEvent::Disable);

                    // Spawn new tower
                    let pos = sel_trans.translation;
                    spawn_tower_ev.send(SpawnTowerEvent(
                        *tower_type,
                        Vec3::new(pos.x, -0.025, pos.z),
                    ));

                    return;
                }
            }
        }
    }
}