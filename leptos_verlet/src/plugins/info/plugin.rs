use bevy::prelude::*;

use crate::plugins::schedule::plugin::SimulationCycle;

#[derive(Event, Clone)]
pub struct PointInfo {
    pub position: Vec3,
    pub velocity: Vec3,
}

pub struct InfoPlugin;
impl Plugin for InfoPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (do_something).chain().in_set(SimulationCycle::Preparation),
        );
    }
}

fn do_something() {}
