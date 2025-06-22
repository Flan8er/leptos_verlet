use bevy::prelude::*;

use crate::core::parameters::HALF_CAMERA_HEIGHT;

#[derive(Event, Clone)]
pub struct LeptosResize {
    pub width: f32,
    pub height: f32,
}

#[derive(Resource)]
pub struct SimulationBounds {
    pub width: f32,
    height: f32,
}
impl Default for SimulationBounds {
    fn default() -> Self {
        Self {
            width: *HALF_CAMERA_HEIGHT * 2.,
            height: *HALF_CAMERA_HEIGHT * 2.,
        }
    }
}

pub fn window_listener(
    mut window_event: EventReader<LeptosResize>,
    mut window_res: ResMut<SimulationBounds>,
) {
    for event in window_event.read() {
        let width = event.width;
        let height = event.height;

        let sim_width = (window_res.height * width) / height;

        window_res.width = sim_width;
    }
}
