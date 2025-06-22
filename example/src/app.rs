use leptos::prelude::*;
use leptos_bevy_canvas::prelude::*;
use leptos_use::{UseWindowSizeReturn, use_window_size};

use crate::components::{
    control_pane::ControlPane, element_pane::ElementPane, info::InfoModal,
    mouse_monitor::MouseMonitor,
};

use prelude::*;

// Create sticks between two points

#[component]
pub fn App() -> impl IntoView {
    let (state_sender, bevy_state_receiver) = event_l2b::<SimulationPlayStateRequest>();
    let (target_sender, bevy_target_receiver) = event_l2b::<ModificationTarget>();
    let (event_sender, bevy_event_receiver) = event_l2b::<ModifyEventType>();
    let (window_sender, bevy_window_receiver) = event_l2b::<LeptosResize>();

    provide_context(state_sender);
    provide_context(target_sender);
    provide_context(event_sender);

    let UseWindowSizeReturn { width, height } = use_window_size();
    Effect::new(move |_| {
        let width = width.get() as f32;
        let height = height.get() as f32;
        window_sender.send(LeptosResize { width, height })
    });

    // IoCutOutline, MdiTransitConnectionHorizontal (rotate 45deg), BsLink45deg
    let active_modifier: RwSignal<ModificationTarget> = RwSignal::new(ModificationTarget::None);

    view! {
        <main class="w-screen h-screen flex items-center justify-center overflow-hidden relative">
            <ElementPane active_modifier/>
            <InfoModal active_modifier/>

            <div class="w-full h-full relative">
                <BevyCanvas
                    init=move || {
                        init_bevy_app(bevy_state_receiver, bevy_target_receiver, bevy_event_receiver, bevy_window_receiver)
                    }
                />

                <MouseMonitor active_modifier/>
            </div>

            <ControlPane active_modifier/>
        </main>
    }
}
