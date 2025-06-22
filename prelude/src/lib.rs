use bevy::prelude::*;
use leptos::prelude::*;
use leptos_bevy_canvas::prelude::*;
use leptos_use::{UseElementSizeReturn, use_element_size};

mod core;
mod interaction;
mod objects;

// Define the prelude exports
pub use interaction::{
    modification::{ModificationTarget, ModifyEventType, RelativeWindowPosition},
    state::SimulationPlayStateRequest,
    window_bounds::LeptosResize,
};
pub use leptos_bevy_canvas::prelude::{LeptosChannelEventSender, LeptosEventSender};

use crate::{
    core::{
        render::RenderView, schedule::SchedulePlugin, setup::StartSimulation,
        simulation::RunSimulation,
    },
    interaction::state::StatePlugin,
};

#[component]
pub fn VerletConfigProvider() -> impl IntoView {
    let (state_sender, bevy_state_receiver) = event_l2b::<SimulationPlayStateRequest>();
    let (target_sender, bevy_target_receiver) = event_l2b::<ModificationTarget>();
    let (event_sender, bevy_event_receiver) = event_l2b::<ModifyEventType>();
    let (window_sender, bevy_window_receiver) = event_l2b::<LeptosResize>();

    provide_context(state_sender);
    provide_context(target_sender);
    provide_context(event_sender);
    provide_context(window_sender);

    provide_context(bevy_state_receiver);
    provide_context(bevy_target_receiver);
    provide_context(bevy_event_receiver);
    provide_context(bevy_window_receiver);

    view! {
        <></>
    }
}

#[component]
pub fn VerletCanvas(parent_element: NodeRef<leptos::html::Div>) -> impl IntoView {
    let state_sender = expect_context::<LeptosEventSender<SimulationPlayStateRequest>>();
    let target_sender = expect_context::<LeptosEventSender<ModificationTarget>>();
    let event_sender = expect_context::<LeptosEventSender<ModifyEventType>>();
    let window_sender = expect_context::<LeptosEventSender<LeptosResize>>();

    let bevy_state_receiver = expect_context::<BevyEventReceiver<SimulationPlayStateRequest>>();
    let bevy_target_receiver = expect_context::<BevyEventReceiver<ModificationTarget>>();
    let bevy_event_receiver = expect_context::<BevyEventReceiver<ModifyEventType>>();
    let bevy_window_receiver = expect_context::<BevyEventReceiver<LeptosResize>>();

    let UseElementSizeReturn { width, height } = use_element_size(parent_element);
    Effect::new(move |_| {
        let width = width.get() as f32;
        let height = height.get() as f32;
        window_sender.send(LeptosResize { width, height })
    });

    view! {
        <BevyCanvas
            init=move || {
                init_bevy_app(bevy_state_receiver, bevy_target_receiver, bevy_event_receiver, bevy_window_receiver)
            }
        />
    }
}

fn init_bevy_app(
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
