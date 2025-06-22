use leptos::{html::Div, prelude::*};

use crate::components::{
    control_pane::ControlPane, element_pane::ElementPane, info::InfoModal,
    mouse_monitor::MouseMonitor,
};

use prelude::*;

#[component]
pub fn App() -> impl IntoView {
    let verlet_container = NodeRef::<Div>::new();

    let active_modifier: RwSignal<ModificationTarget> = RwSignal::new(ModificationTarget::None);

    view! {
        <VerletConfigProvider/>

        <main class="w-screen h-screen flex items-center justify-center overflow-hidden relative">
            <ElementPane active_modifier/>
            <InfoModal active_modifier/>

            <div
                node_ref=verlet_container
                class="w-full h-full relative"
            >
                <VerletCanvas parent_element=verlet_container/>

                <MouseMonitor active_modifier/>
            </div>

            <ControlPane active_modifier/>
        </main>
    }
}
