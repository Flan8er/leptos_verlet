use bevy::prelude::*;
use leptos::prelude::*;
use leptos_bevy_canvas::prelude::*;
use leptos_use::{UseElementSizeReturn, use_element_size};

use crate::{
    core::{
        render::RenderView, schedule::SchedulePlugin, setup::StartSimulation,
        simulation::RunSimulation, spawner::SpawnRequest,
    },
    interaction::play_state::StatePlugin,
    prelude::{LeptosResize, ModificationTarget, ModifyEventType, SimulationPlayStateRequest},
};

#[component]
pub fn VerletConfigProvider() -> impl IntoView {
    let (state_sender, bevy_state_receiver) = event_l2b::<SimulationPlayStateRequest>();
    let (target_sender, bevy_target_receiver) = event_l2b::<ModificationTarget>();
    let (event_sender, bevy_event_receiver) = event_l2b::<ModifyEventType>();
    let (element_size_sender, bevy_element_size_receiver) = event_l2b::<LeptosResize>();
    let (spawn_sender, bevy_spawn_receiver) = event_l2b::<SpawnRequest>();

    provide_context(state_sender);
    provide_context(target_sender);
    provide_context(event_sender);
    provide_context(element_size_sender);
    provide_context(spawn_sender);

    provide_context(bevy_state_receiver);
    provide_context(bevy_target_receiver);
    provide_context(bevy_event_receiver);
    provide_context(bevy_element_size_receiver);
    provide_context(bevy_spawn_receiver);

    view! {
        <></>
    }
}

#[component]
pub fn VerletCanvas(parent_element: NodeRef<leptos::html::Div>) -> impl IntoView {
    // let state_sender = expect_context::<LeptosEventSender<SimulationPlayStateRequest>>();
    // let target_sender = expect_context::<LeptosEventSender<ModificationTarget>>();
    // let event_sender = expect_context::<LeptosEventSender<ModifyEventType>>();
    let element_size_sender = expect_context::<LeptosEventSender<LeptosResize>>();
    // let spawn_sender = expect_context::<LeptosEventSender<SpawnRequest>>();

    let bevy_state_receiver = expect_context::<BevyEventReceiver<SimulationPlayStateRequest>>();
    let bevy_target_receiver = expect_context::<BevyEventReceiver<ModificationTarget>>();
    let bevy_event_receiver = expect_context::<BevyEventReceiver<ModifyEventType>>();
    let bevy_element_size_receiver = expect_context::<BevyEventReceiver<LeptosResize>>();
    let bevy_spawn_receiver = expect_context::<BevyEventReceiver<SpawnRequest>>();

    let UseElementSizeReturn { width, height } = use_element_size(parent_element);
    Effect::new(move |_| {
        let width = width.get() as f32;
        let height = height.get() as f32;
        element_size_sender.send(LeptosResize { width, height })
    });

    view! {
        <BevyCanvas
            init=move || {
                init_bevy_app(bevy_state_receiver, bevy_target_receiver, bevy_event_receiver, bevy_element_size_receiver, bevy_spawn_receiver)
            }
        />
    }
}

fn init_bevy_app(
    state_receiver: BevyEventReceiver<SimulationPlayStateRequest>,
    target_receiver: BevyEventReceiver<ModificationTarget>,
    event_receiver: BevyEventReceiver<ModifyEventType>,
    window_resize_receiver: BevyEventReceiver<LeptosResize>,
    spawn_receiver: BevyEventReceiver<SpawnRequest>,
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
    .import_event_from_leptos(spawn_receiver)
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
