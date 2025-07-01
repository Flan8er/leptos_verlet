use bevy::prelude::*;

use crate::{plugins::schedule::plugin::SimulationCycle, prelude::Point};

#[derive(Event, Clone, Copy)]
pub struct PointInfo {
    pub position: Vec3,
    pub velocity: Vec3,
}

#[derive(Event, Clone, Copy)]
pub struct SetPointInfo {
    pub position: Vec3,
    pub velocity: Vec3,
}

impl Default for PointInfo {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            velocity: Vec3::ZERO,
        }
    }
}
impl PointInfo {
    pub fn new(point: &Point) -> Self {
        Self {
            position: point.position,
            velocity: point.calculate_velocity(),
        }
    }
}

#[derive(Component)]
pub struct ActiveInfoTarget;

pub struct InfoPlugin;
impl Plugin for InfoPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (receive_leptos_update, send_leptos_update)
                .chain()
                .in_set(SimulationCycle::Preparation1),
        );
    }
}

/// Whenever the target point changes send an update with the Point's properties
fn send_leptos_update(
    target_query: Query<&Point, With<ActiveInfoTarget>>,
    mut writer: EventWriter<PointInfo>,
) {
    match target_query.get_single() {
        Ok(target_point) => {
            let _ = writer.send(PointInfo::new(target_point));
        }
        Err(_) => return,
    }
}

fn receive_leptos_update(
    mut reader: EventReader<SetPointInfo>,
    mut target_query: Query<&mut Point, With<ActiveInfoTarget>>,
) {
    for event in reader.read() {
        match target_query.get_single_mut() {
            Ok(mut target_point) => {
                target_point.position = event.position;
                target_point.prev_position = target_point.position - event.velocity;
            }
            Err(_) => return,
        }
    }
}
