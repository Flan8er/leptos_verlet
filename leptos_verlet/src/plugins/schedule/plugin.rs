use bevy::prelude::*;

use crate::plugins::play_state::plugin::SimulationPlayState;

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum SimulationCycle {
    Preparation1,
    Compute,
    Preparation2,
    Converge,
    Preparation3,
    Render,
}

pub struct SchedulePlugin;
impl Plugin for SchedulePlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            (
                SimulationCycle::Preparation1,
                SimulationCycle::Compute,
                SimulationCycle::Preparation2,
                SimulationCycle::Converge,
                SimulationCycle::Preparation3,
                SimulationCycle::Render,
            )
                .chain(),
        )
        .configure_sets(
            Update,
            (
                SimulationCycle::Compute,
                SimulationCycle::Converge,
                SimulationCycle::Render,
            )
                .chain()
                .run_if(in_state(SimulationPlayState::Running)),
        );
    }
}
