use crate::prelude::*;
use crate::tower::foundation::ReadyToBuild;
use crate::{
    progress_bar::ProgressBarFullEvent,
    tower::{base::TowerBase, foundation::TowerFoundation},
};
use bevy::prelude::EventReader;

pub(super) fn on_progress_bar_full_system(
    mut cmds: Commands,
    mut evr: EventReader<ProgressBarFullEvent>,
    q_foundation: Query<With<TowerFoundation>>,
    q_base: Query<With<TowerBase>>,
) {
    for ev in evr.iter() {
        let rel_id = ev.0;
        if q_foundation.contains(rel_id) {
            cmds.entity(rel_id).insert(ReadyToBuild);
        } else if q_base.contains(rel_id) {
            // TODO
        }
    }
}
