use leptos::{html::Div, prelude::*};
use leptos_use::{
    UseDraggableOptions, UseDraggableReturn, core::Position, use_draggable_with_options,
};

use leptos_verlet::prelude::*;

#[component]
pub fn InfoModal(active_modifier: RwSignal<ModificationTarget>) -> impl IntoView {
    let point = expect_context::<PointInfoReceiver>();
    let point_sender = expect_context::<PointInfoSender>();
    let el = NodeRef::<Div>::new();

    // `style` is a helper string "left: {x}px; top: {y}px;"
    #[allow(unused_variables)]
    let UseDraggableReturn { x, y, style, .. } = use_draggable_with_options(
        el,
        UseDraggableOptions::default().initial_value(Position { x: 40.0, y: 40.0 }),
    );

    view! {
        <Show
            when=move || active_modifier.get() == ModificationTarget::PointInfo
            fallback=|| view! {<></>}
        >
            <div
                node_ref=el
                style=move || format!("position: fixed; {}", style.get())
                class="bg-card p-4 z-[100] flex flex-col item-start justify-start cursor-default rounded-lg"
            >
                <h3>"Position"</h3>
                <div class="flex w-full space-between gap-2">
                    { // X
                        view! {
                            <div class="flex items-center gap-2">
                                <h3>"X"</h3>
                                <p
                                    class="w-16 px-1 py-0.5 text-sm bg-card-active border rounded text-primary-text focus:outline-none focus:ring-2 focus:ring-accent"
                                >{move || match point.get() {
                                    Some(point_info) => point_info.position[0],
                                    None => 0.
                                }}</p>
                            </div>
                        }
                    }
                    { // Y
                        view! {
                            <div class="flex items-center gap-2">
                                <h3>"Y"</h3>
                                <p
                                    class="w-16 px-1 py-0.5 text-sm bg-card-active border rounded text-primary-text focus:outline-none focus:ring-2 focus:ring-accent"
                                >{move || match point.get() {
                                    Some(point_info) => point_info.position[1],
                                    None => 0.
                                }}</p>
                            </div>
                        }
                    }
                    { // Z
                        view! {
                            <div class="flex items-center gap-2">
                                <h3>"Z"</h3>
                                <p
                                    class="w-16 px-1 py-0.5 text-sm bg-card-active border rounded text-primary-text focus:outline-none focus:ring-2 focus:ring-accent"
                                >{move || match point.get() {
                                    Some(point_info) => point_info.position[2],
                                    None => 0.
                                }}</p>
                            </div>
                        }
                    }
                </div>
                <h3>"Velocity"</h3>
                <div class="flex w-full space-between gap-2">
                    { // VX
                        view! {
                            <div class="flex items-center gap-2">
                                <h3>"X"</h3>
                                <input
                                    class="w-16 px-1 py-0.5 text-sm bg-card-active border rounded text-primary-text focus:outline-none focus:ring-2 focus:ring-accent"
                                    type="number"
                                    value=move || match point.get() {
                                        Some(point_info) => point_info.velocity[0],
                                        None => 0.
                                    }
                                    on:input=
                                    {
                                        let point_sender = point_sender.clone();
                                        move |ev| {
                                            if let Ok(val) = event_target_value(&ev).parse::<f32>() {
                                                match point.get() {
                                                    Some(mut current_state) => {
                                                        current_state.velocity[0] = val;
                                                        point_sender.send(SetPointInfo {
                                                            position: current_state.position,
                                                            velocity: current_state.velocity
                                                        }).ok();

                                                    },
                                                    None => ()
                                                };
                                            }
                                        }
                                    }
                                />
                            </div>
                        }
                    }
                    { // VY
                        view! {
                            <div class="flex items-center gap-2">
                                <h3>"Y"</h3>
                                <input
                                    class="w-16 px-1 py-0.5 text-sm bg-card-active border rounded text-primary-text focus:outline-none focus:ring-2 focus:ring-accent"
                                    type="number"
                                    value=move || match point.get() {
                                        Some(point_info) => point_info.velocity[1],
                                        None => 0.
                                    }
                                    on:input=
                                    {
                                        let point_sender = point_sender.clone();
                                        move |ev| {
                                            if let Ok(val) = event_target_value(&ev).parse::<f32>() {
                                                match point.get() {
                                                    Some(mut current_state) => {
                                                        current_state.velocity[1] = val;
                                                        point_sender.send(SetPointInfo {
                                                            position: current_state.position,
                                                            velocity: current_state.velocity
                                                        }).ok();

                                                    },
                                                    None => ()
                                                };
                                            }
                                        }
                                    }
                                />
                            </div>
                        }
                    }
                    { // VZ
                        view! {
                            <div class="flex items-center gap-2">
                                <h3>"Z"</h3>
                                <input
                                    class="w-16 px-1 py-0.5 text-sm bg-card-active border rounded text-primary-text focus:outline-none focus:ring-2 focus:ring-accent"
                                    type="number"
                                    value=move || match point.get() {
                                        Some(point_info) => point_info.velocity[2],
                                        None => 0.
                                    }
                                    on:input=
                                    {
                                        let point_sender = point_sender.clone();
                                        move |ev| {
                                            if let Ok(val) = event_target_value(&ev).parse::<f32>() {
                                                match point.get() {
                                                    Some(mut current_state) => {
                                                        current_state.velocity[2] = val;
                                                        point_sender.send(SetPointInfo {
                                                            position: current_state.position,
                                                            velocity: current_state.velocity
                                                        }).ok();

                                                    },
                                                    None => ()
                                                };
                                            }
                                        }
                                    }
                                />
                            </div>
                        }
                    }
                </div>
            </div>
        </Show>
    }
}
