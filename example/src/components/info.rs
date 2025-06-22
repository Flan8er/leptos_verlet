use leptos::{html::Div, prelude::*};
use leptos_use::{
    UseDraggableOptions, UseDraggableReturn, core::Position, use_draggable_with_options,
};

use leptos_verlet::prelude::*;

#[component]
pub fn InfoModal(active_modifier: RwSignal<ModificationTarget>) -> impl IntoView {
    let el = NodeRef::<Div>::new();

    // `style` is a helper string "left: {x}px; top: {y}px;"
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
                        // let x = position.get().x;
                        let x = -1.2;
                        view! {
                            <div class="flex items-center gap-2">
                                <h3>"X"</h3>
                                <input
                                    class="w-16 px-1 py-0.5 text-sm bg-card-active border rounded text-primary-text focus:outline-none focus:ring-2 focus:ring-accent"
                                    type="number"
                                    value=move || x.to_string()
                                    on:input=move |ev| {
                                        if let Ok(val) = event_target_value(&ev).parse::<f32>() {
                                            // let mut p = position.get();
                                            // p.x = val;
                                            // set_position(p);
                                        }
                                    }
                                />
                            </div>
                        }
                    }
                    { // Y
                        // let y = position.get().y;
                        let y = 1.3;
                        view! {
                            <div class="flex items-center gap-2">
                                <h3>"Y"</h3>
                                <input
                                    class="w-16 px-1 py-0.5 text-sm bg-card-active border rounded text-primary-text focus:outline-none focus:ring-2 focus:ring-accent"
                                    type="number"
                                    value=move || y.to_string()
                                    on:input=move |ev| {
                                        if let Ok(val) = event_target_value(&ev).parse::<f32>() {
                                            // let mut p = position.get();
                                            // p.y = val;
                                            // set_position(p);
                                        }
                                    }
                                />
                            </div>
                        }
                    }
                    { // Z
                        // let z = position.get().z;
                        let z = 0.1;
                        view! {
                            <div class="flex items-center gap-2">
                                <h3>"Z"</h3>
                                <input
                                    class="w-16 px-1 py-0.5 text-sm bg-card-active border rounded text-primary-text focus:outline-none focus:ring-2 focus:ring-accent"
                                    type="number"
                                    value=move || z.to_string()
                                    on:input=move |ev| {
                                        if let Ok(val) = event_target_value(&ev).parse::<f32>() {
                                            // let mut p = position.get();
                                            // p.z = val;
                                            // set_position(p);
                                        }
                                    }
                                />
                            </div>
                        }
                    }
                </div>
                <h3>"Velocity"</h3>
                <div class="flex w-full space-between gap-2">
                    { // VX
                        // let vx = velocity.get().x;
                        let vx = 0.6;
                        view! {
                            <div class="flex items-center gap-2">
                                <h3>"X"</h3>
                                <input
                                    class="w-16 px-1 py-0.5 text-sm bg-card-active border rounded text-primary-text focus:outline-none focus:ring-2 focus:ring-accent"
                                    type="number"
                                    value=move || vx.to_string()
                                    on:input=move |ev| {
                                        if let Ok(val) = event_target_value(&ev).parse::<f32>() {
                                            // let mut v = velocity.get();
                                            // v.x = val;
                                            // set_velocity(v);
                                        }
                                    }
                                />
                            </div>
                        }
                    }
                    { // VY
                        // let vy = velocity.get().y;
                        let vy = 1.8;
                        view! {
                            <div class="flex items-center gap-2">
                                <h3>"Y"</h3>
                                <input
                                    class="w-16 px-1 py-0.5 text-sm bg-card-active border rounded text-primary-text focus:outline-none focus:ring-2 focus:ring-accent"
                                    type="number"
                                    value=move || vy.to_string()
                                    on:input=move |ev| {
                                        if let Ok(val) = event_target_value(&ev).parse::<f32>() {
                                            // let mut v = velocity.get();
                                            // v.y = val;
                                            // set_velocity(v);
                                        }
                                    }
                                />
                            </div>
                        }
                    }
                    { // VZ
                        // let vz = velocity.get().z;
                        let vz = 0.1;
                        view! {
                            <div class="flex items-center gap-2">
                                <h3>"Z"</h3>
                                <input
                                    class="w-16 px-1 py-0.5 text-sm bg-card-active border rounded text-primary-text focus:outline-none focus:ring-2 focus:ring-accent"
                                    type="number"
                                    value=move || vz.to_string()
                                    on:input=move |ev| {
                                        if let Ok(val) = event_target_value(&ev).parse::<f32>() {
                                            // let mut v = velocity.get();
                                            // v.z = val;
                                            // set_velocity(v);
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
