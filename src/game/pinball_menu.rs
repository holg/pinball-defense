use super::ball::{CollisionWithBallEvent, PinBall};
use super::events::collision::COLLIDE_ONLY_WITH_BALL;
use super::events::tween_completed::{ACTIVATE_PINBALL_MENU_EVENT_ID, DESPAWN_ENTITY_EVENT_ID};
use super::level::LevelUpEvent;
use super::progress_bar::ProgressBarFullEvent;
use super::tower::{SpawnTowerEvent, TowerType, TowerUpgrade};
use super::world::QueryWorld;
use super::GameState;
use crate::prelude::*;
use crate::settings::GraphicsSettings;
use bevy::utils::hashbrown::HashSet;
use bevy_rapier3d::rapier::prelude::CollisionEventFlags;
use bevy_tweening::{lens::TransformRotateYLens, Animator, Delay, EaseFunction, Sequence, Tween};
use std::time::Duration;

pub struct PinballMenuPlugin;

impl Plugin for PinballMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PinballMenuEvent>()
            .add_event::<TowerMenuExecuteEvent>()
            .add_event::<UpgradeMenuExecuteEvent>()
            .add_event::<PinballMenuOnSetSelectedEvent>()
            .init_resource::<UnlockedTowers>()
            .init_resource::<UnlockedUpgrades>()
            .add_systems(
                Update,
                (
                    menu_event_system,
                    spawn_system,
                    execute_system,
                    de_activate_system,
                    ready_system,
                    selected_system,
                    unlock_system,
                )
                    .run_if(in_state(GameState::Ingame)),
            );
    }
}

// --- Public Area ---

#[derive(Event, Debug, Clone, Copy)]
pub enum PinballMenuEvent {
    Disable,
    SetReady,
    Activate,
    Deactivate,
}

#[derive(Component, Debug, Clone, Copy)]
pub enum PinballMenuTrigger {
    Tower,
    Upgrade,
}

#[derive(Event, Clone, Copy)]
pub struct TowerMenuExecuteEvent {
    pub foundation_id: Entity,
    pub tower_type: TowerType,
}

impl TowerMenuExecuteEvent {
    pub fn new(foundation_id: Entity, tower_type: TowerType) -> Self {
        Self {
            foundation_id,
            tower_type,
        }
    }
}

#[derive(Event, Clone, Copy)]
pub struct PinballMenuOnSetSelectedEvent(pub Entity);

#[derive(Event, Clone, Copy)]
pub struct UpgradeMenuExecuteEvent {
    pub tower_id: Entity,
    pub upgrade: TowerUpgrade,
}

impl UpgradeMenuExecuteEvent {
    pub fn new(tower_id: Entity, upgrade: TowerUpgrade) -> Self {
        Self { tower_id, upgrade }
    }
}

// --- Private Area ---

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
                (Deactivate, Activated) => Some(deactivate(cmds, q_lights, q_pbm_el)),
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
    assets: Res<PinballDefenseAssets>,
    q_pbw: QueryWorld,
    q_pb_menu: Query<&PinballMenu>,
    g_sett: Res<GraphicsSettings>,
    q_selected: Query<&PinballMenuTrigger, With<PinballMenuSelected>>,
    unlocked_towers: Res<UnlockedTowers>,
    unlocked_tower_upgrades: Res<UnlockedUpgrades>,
) {
    if q_pb_menu.is_empty() {
        if let Ok(trigger) = q_selected.get_single() {
            log!("🐢 Spawn {trigger:?} menu");
            cmds.entity(q_pbw.single())
                .with_children(|p| match *trigger {
                    PinballMenuTrigger::Tower => {
                        spawn_tower_menu(p, &assets, &g_sett, &unlocked_towers, MENU_POS)
                    }
                    PinballMenuTrigger::Upgrade => {
                        spawn_upgrade_menu(p, &assets, &g_sett, &unlocked_tower_upgrades, MENU_POS)
                    }
                });
        }
    }
}

#[derive(Component)]
enum PinballMenu {
    Tower,
    Upgrade,
}

fn spawn_tower_menu(
    parent: &mut ChildBuilder,
    assets: &PinballDefenseAssets,
    g_sett: &GraphicsSettings,
    unlocked_towers: &UnlockedTowers,
    pos: Vec3,
) {
    parent
        .spawn((
            spatial_from_pos(pos),
            PinballMenu::Tower,
            PinballMenuStatus::Disabled,
            Name::new("Pinball Tower Menu"),
        ))
        .with_children(|p| {
            let mut angle = 0.39;
            let angle_add = angle * 2. / unlocked_towers.0.len() as f32;
            for tower in unlocked_towers.0.iter() {
                spawn_menu_element(*tower, p, assets, g_sett, angle, 0.1);
                angle -= angle_add;
            }
        });
}

fn spawn_upgrade_menu(
    parent: &mut ChildBuilder,
    assets: &PinballDefenseAssets,
    g_sett: &GraphicsSettings,
    unlocked_tower_upgrades: &UnlockedUpgrades,
    pos: Vec3,
) {
    parent
        .spawn((
            spatial_from_pos(pos),
            PinballMenu::Upgrade,
            PinballMenuStatus::Disabled,
            Name::new("Pinball Upgrade Menu"),
        ))
        .with_children(|p| {
            let mut angle = 0.39;
            let angle_add = angle * 2. / unlocked_tower_upgrades.0.len() as f32;
            for tower_upgrade in unlocked_tower_upgrades.0.iter() {
                spawn_menu_element(*tower_upgrade, p, assets, g_sett, angle, 0.1);
                angle -= angle_add;
            }
        });
}

// Only pub(crate)for collision events
#[derive(Component)]
struct PinballMenuElement;

#[derive(Component)]
struct PinballMenuElementLight;

fn spawn_menu_element(
    menu_el_type: impl Component + GetMaterial,
    parent: &mut ChildBuilder,
    assets: &PinballDefenseAssets,
    g_sett: &GraphicsSettings,
    angle: f32,
    delay_secs: f32,
) {
    parent
        .spawn((
            PbrBundle {
                mesh: assets.pinball_menu_element.clone(),
                material: menu_el_type.get_menu_element_material(assets),
                transform: Transform {
                    rotation: Quat::from_rotation_y(ELEM_START_ANGLE),
                    ..default()
                },
                ..default()
            },
            // Game components
            PinballMenuElement,
            Name::new("Pinball Menu Element"),
            menu_el_type,
            // Spawn animation
            Animator::new(spawn_animation(angle, delay_secs)),
        ))
        .with_children(|parent| {
            // Active status light
            parent.spawn((
                SpotLightBundle {
                    transform: Transform::from_translation(Vec3::new(-0.79, 0., 0.))
                        .looking_at(Vec3::new(-1.0, 0.0, 0.0), Vec3::Z),
                    spot_light: SpotLight {
                        intensity: 28., // lumens - roughly a 100W non-halogen incandescent bulb
                        color: Color::BEIGE,
                        shadows_enabled: g_sett.is_shadows,
                        range: 0.2,
                        inner_angle: 0.2,
                        outer_angle: 0.8,
                        ..default()
                    },
                    visibility: Visibility::Hidden,
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
        let secs = (trans.rotation.y + 0.2) * 2.;
        log!("Durationn for {}: {}", trans.rotation.y, secs);
        cmds.entity(entity).insert(Animator::new(despawn_animation(
            trans.rotation.y,
            Duration::from_secs_f32(secs),
        )));
    });
    deactivate(cmds, q_lights, q_pbm_el);
    PinballMenuStatus::Disabled
}

fn de_activate_system(
    mut pb_menu_ev: EventWriter<PinballMenuEvent>,
    q_ball: Query<&Transform, With<PinBall>>,
    q_pb_menu_status: Query<&PinballMenuStatus>,
) {
    for status in q_pb_menu_status.iter() {
        match *status {
            PinballMenuStatus::Disabled => (),
            PinballMenuStatus::Ready => {
                if is_ball_in_x_zone(&q_ball, 0.6, 0.8) {
                    pb_menu_ev.send(PinballMenuEvent::Activate);
                }
            }
            PinballMenuStatus::Activated => {
                if !is_ball_in_x_zone(&q_ball, 0.28, 1.) {
                    pb_menu_ev.send(PinballMenuEvent::Deactivate);
                }
            }
        }
    }
}

fn is_ball_in_x_zone(q_ball: &Query<&Transform, With<PinBall>>, start: f32, end: f32) -> bool {
    for ball in q_ball.iter() {
        let trans = ball.translation;
        if trans.x >= start && trans.x <= end && trans.z >= -0.42 && trans.z <= 0.46 {
            return true;
        }
    }
    false
}

fn activate(
    mut cmds: Commands,
    mut q_lights: Query<&mut Visibility, With<PinballMenuElementLight>>,
    q_pbm_el: QueryPinballMenuElements,
    meshes: Res<Assets<Mesh>>,
    assets: Res<PinballDefenseAssets>,
) -> PinballMenuStatus {
    q_pbm_el.for_each(|(entity, _)| {
        cmds.entity(entity).insert((
            // Active status collider
            ColliderDebugColor(Color::GREEN),
            Sensor,
            ActiveEvents::COLLISION_EVENTS,
            Collider::from_bevy_mesh(
                meshes
                    .get(&assets.pinball_menu_element_collider.clone())
                    .expect("Failed to find mesh"),
                &ComputedColliderShape::TriMesh,
            )
            .unwrap(),
            COLLIDE_ONLY_WITH_BALL,
        ));
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

const ELEM_START_ANGLE: f32 = -0.58;

fn spawn_animation(angle: f32, delay_secs: f32) -> Sequence<Transform> {
    let wait = Delay::new(Duration::from_secs_f32(delay_secs));
    let rotate = Tween::new(
        EaseFunction::ElasticOut,
        Duration::from_secs_f32(2.),
        TransformRotateYLens {
            start: ELEM_START_ANGLE + 0.2,
            end: angle,
        },
    );

    wait.then(rotate.with_completed_event(ACTIVATE_PINBALL_MENU_EVENT_ID))
}

fn despawn_animation(angle: f32, duration: Duration) -> Sequence<Transform> {
    let wait = Delay::new(duration);
    let rotate = Tween::new(
        EaseFunction::ExponentialInOut,
        duration,
        TransformRotateYLens {
            start: angle,
            end: ELEM_START_ANGLE,
        },
    );
    wait.then(rotate)
}

type QueryUpgradeMenuEls<'w, 's, 'a> =
    Query<'w, 's, (Entity, &'a TowerUpgrade), (With<PinballMenuElement>, Without<TowerType>)>;

fn execute_system(
    mut cmds: Commands,
    mut ball_coll_ev: EventReader<CollisionWithBallEvent>,
    mut on_tower_el_selected: EventWriter<TowerMenuExecuteEvent>,
    mut on_upgrade_el_selected: EventWriter<UpgradeMenuExecuteEvent>,
    mut pb_menu_ev: EventWriter<PinballMenuEvent>,
    mut spawn_tower_ev: EventWriter<SpawnTowerEvent>,
    q_pb_menu: Query<&PinballMenu>,
    q_tower_menu_els: Query<(Entity, &TowerType), With<PinballMenuElement>>,
    q_upgrade_menu_els: QueryUpgradeMenuEls,
    q_selected: Query<(Entity, &Transform), With<PinballMenuSelected>>,
) {
    for CollisionWithBallEvent(id, flag) in ball_coll_ev.iter() {
        if *flag == CollisionEventFlags::SENSOR {
            if let Ok(pb_menu) = q_pb_menu.get_single() {
                match pb_menu {
                    PinballMenu::Tower => {
                        if let Some((_, tower_type)) =
                            q_tower_menu_els.iter().find(|(el_id, _)| *el_id == *id)
                        {
                            if let Ok((foundation_id, sel_trans)) = q_selected.get_single() {
                                // Deselect
                                cmds.entity(foundation_id).remove::<PinballMenuSelected>();

                                on_tower_el_selected
                                    .send(TowerMenuExecuteEvent::new(foundation_id, *tower_type));

                                // Spawn new tower
                                let pos = sel_trans.translation;
                                spawn_tower_ev.send(SpawnTowerEvent(
                                    *tower_type,
                                    Vec3::new(pos.x, -0.025, pos.z),
                                ));
                            }
                        }
                    }
                    PinballMenu::Upgrade => {
                        if let Some((_, upgrade)) =
                            q_upgrade_menu_els.iter().find(|(el_id, _)| *el_id == *id)
                        {
                            if let Ok((tower_id, _)) = q_selected.get_single() {
                                // Deselect
                                cmds.entity(tower_id).remove::<PinballMenuSelected>();

                                on_upgrade_el_selected.send(UpgradeMenuExecuteEvent::new(
                                    q_selected.single().0,
                                    *upgrade,
                                ));
                            }
                        }
                    }
                }

                // Despawn menu
                pb_menu_ev.send(PinballMenuEvent::Disable);

                return;
            }
        }
    }
}

const MENU_POS: Vec3 = Vec3::new(1.3, 0., 0.038);

pub fn spawn_pinball_menu_glass(
    parent: &mut ChildBuilder,
    assets: &PinballDefenseAssets,
    mats: &mut Assets<StandardMaterial>,
) {
    parent.spawn((
        PbrBundle {
            mesh: assets.world_1_menu_glass.clone(),
            material: mats.add(StandardMaterial {
                base_color: Color::ALICE_BLUE,
                perceptual_roughness: 0.,
                metallic: 0.,
                reflectance: 0.6,
                alpha_mode: AlphaMode::Multiply,
                ..default()
            }),
            transform: Transform::from_translation(MENU_POS),
            ..default()
        },
        Name::new("Pinball menu glass"),
    ));
}

#[derive(Component)]
struct PinballMenuReady;

fn ready_system(
    mut cmds: Commands,
    mut on_progress_full_ev: EventReader<ProgressBarFullEvent>,
    q_trigger: Query<&PinballMenuTrigger>,
) {
    for ev in on_progress_full_ev.iter() {
        if q_trigger.contains(ev.0) {
            cmds.entity(ev.0).insert(PinballMenuReady);
        }
    }
}

#[derive(Component)]
struct PinballMenuSelected;

fn selected_system(
    mut cmds: Commands,
    mut on_sel_ev: EventWriter<PinballMenuOnSetSelectedEvent>,
    q_ready: Query<(Entity, &PinballMenuTrigger), With<PinballMenuReady>>,
    q_selected: Query<With<PinballMenuSelected>>,
    unlocked_towers: Res<UnlockedTowers>,
    unlocked_tower_upgrades: Res<UnlockedUpgrades>,
) {
    if q_selected.is_empty() {
        for (ready_id, trigger) in q_ready.iter() {
            if is_unlock_available(*trigger, &unlocked_towers, &unlocked_tower_upgrades) {
                set_selected(&mut cmds, ready_id);
                on_sel_ev.send(PinballMenuOnSetSelectedEvent(ready_id));
                return;
            }
        }
    }
}

fn is_unlock_available(
    trigger: PinballMenuTrigger,
    unlocked_towers: &UnlockedTowers,
    unlocked_tower_upgrades: &UnlockedUpgrades,
) -> bool {
    match trigger {
        PinballMenuTrigger::Tower => !unlocked_towers.0.is_empty(),
        PinballMenuTrigger::Upgrade => !unlocked_tower_upgrades.0.is_empty(),
    }
}

fn set_selected(cmds: &mut Commands, ref_id: Entity) {
    cmds.entity(ref_id)
        .remove::<PinballMenuReady>()
        .insert(PinballMenuSelected);
}

#[derive(Resource, Default)]
struct UnlockedTowers(HashSet<TowerType>);

#[derive(Resource, Default)]
struct UnlockedUpgrades(HashSet<TowerUpgrade>);

fn unlock_system(
    mut lvl_up_ev: EventReader<LevelUpEvent>,
    mut towers: ResMut<UnlockedTowers>,
    mut upgrades: ResMut<UnlockedUpgrades>,
) {
    for ev in lvl_up_ev.iter() {
        if let Some(tower_type) = new_tower_unlock(ev.0) {
            towers.0.insert(tower_type);
        }
        if let Some(tower_upgrade) = new_tower_upgrade_unlock(ev.0) {
            upgrades.0.insert(tower_upgrade);
        }
    }
}

fn new_tower_unlock(level: u16) -> Option<TowerType> {
    match level {
        2 => Some(TowerType::Gun),
        3 => Some(TowerType::Tesla),
        4 => Some(TowerType::Microwave),
        _ => None,
    }
}

fn new_tower_upgrade_unlock(level: u16) -> Option<TowerUpgrade> {
    match level {
        5 => Some(TowerUpgrade::Range),
        6 => Some(TowerUpgrade::Damage),
        _ => None,
    }
}

trait GetMaterial {
    fn get_menu_element_material(&self, assets: &PinballDefenseAssets) -> Handle<StandardMaterial>;
}

impl GetMaterial for TowerType {
    fn get_menu_element_material(&self, assets: &PinballDefenseAssets) -> Handle<StandardMaterial> {
        match *self {
            TowerType::Gun => assets.pinball_menu_element_gun_material.clone(),
            TowerType::Tesla => assets.pinball_menu_element_tesla_material.clone(),
            TowerType::Microwave => assets.pinball_menu_element_microwave_material.clone(),
        }
    }
}

impl GetMaterial for TowerUpgrade {
    fn get_menu_element_material(&self, assets: &PinballDefenseAssets) -> Handle<StandardMaterial> {
        match *self {
            TowerUpgrade::Damage => assets.pinball_menu_element_damage_upgrade_mat.clone(),
            TowerUpgrade::Range => assets.pinball_menu_element_range_upgrade_mat.clone(),
        }
    }
}
