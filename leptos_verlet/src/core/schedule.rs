use bevy::prelude::*;

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum SimulationCycle {
    Spawn,
    Compute,
    Converge,
    Render,
}

pub struct SchedulePlugin;
impl Plugin for SchedulePlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            (
                SimulationCycle::Spawn,
                SimulationCycle::Compute,
                SimulationCycle::Converge,
                SimulationCycle::Render,
            )
                .chain(),
        );
    }
}
