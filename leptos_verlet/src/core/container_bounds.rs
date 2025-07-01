use bevy::prelude::*;

use crate::core::parameters::SimulationSettings;

#[derive(Event, Clone)]
pub struct LeptosResize {
    pub width: f32,
    pub height: f32,
}

pub fn window_listener(
    mut window_event: EventReader<LeptosResize>,
    mut sim_settings: ResMut<SimulationSettings>,
) {
    for event in window_event.read() {
        let width = event.width;
        let height = event.height;

        let sim_width = (sim_settings.simulation_bounds.y.1 * width) / height;

        sim_settings.simulation_bounds.x.1 = sim_width;
    }
}
