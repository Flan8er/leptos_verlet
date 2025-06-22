use bevy::prelude::*;
use leptos_bevy_canvas::prelude::*;

mod core;
mod interaction;
mod objects;

// Define the prelude exports
pub use interaction::{
    modification::{ModificationTarget, ModifyEventType, RelativeWindowPosition},
    state::SimulationPlayStateRequest,
    window_bounds::LeptosResize,
};

use crate::{
    core::{
        render::RenderView, schedule::SchedulePlugin, setup::StartSimulation,
        simulation::RunSimulation,
    },
    interaction::state::StatePlugin,
};

pub fn init_bevy_app(
    state_receiver: BevyEventReceiver<SimulationPlayStateRequest>,
    target_receiver: BevyEventReceiver<ModificationTarget>,
    event_receiver: BevyEventReceiver<ModifyEventType>,
    window_resize_receiver: BevyEventReceiver<LeptosResize>,
) -> App {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            canvas: Some("#bevy_canvas".into()),
            transparent: true,
            decorations: false,
            fit_canvas_to_parent: true,
            ..default()
        }),
        ..default()
    }))
    .import_event_from_leptos(state_receiver)
    .import_event_from_leptos(target_receiver)
    .import_event_from_leptos(event_receiver)
    .import_event_from_leptos(window_resize_receiver)
    .insert_resource(ClearColor(Color::NONE))
    // Initialize the schedule the logic runs on
    .add_plugins(SchedulePlugin)
    // Initialize simulation states
    .add_plugins(StatePlugin)
    // Create the UI and spawn the particles
    .add_plugins(StartSimulation)
    // Calculate new frame
    .add_plugins(RunSimulation)
    // Render new frame
    .add_plugins(RenderView);
    app
}

// pub fn add(left: u64, right: u64) -> u64 {
//     left + right
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn it_works() {
//         let result = add(2, 2);
//         assert_eq!(result, 4);
//     }
// }
