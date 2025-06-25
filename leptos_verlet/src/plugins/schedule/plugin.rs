use bevy::prelude::*;

use crate::plugins::play_state::plugin::SimulationPlayState;

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum SimulationCycle {
    Preparation,
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
                SimulationCycle::Preparation,
                SimulationCycle::Compute,
                SimulationCycle::Converge,
                SimulationCycle::Render,
            )
                .chain(),
        )
        .configure_sets(
            Update,
            (
                SimulationCycle::Compute,
                SimulationCycle::Converge,
                // SimulationCycle::Render,
            )
                .chain()
                .run_if(in_state(SimulationPlayState::Running)),
        );
    }
}
