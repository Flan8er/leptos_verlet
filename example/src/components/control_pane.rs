use leptos::prelude::*;
use leptos_bevy_canvas::prelude::*;
use leptos_icons::Icon;

use prelude::*;

#[component]
pub fn ControlPane(active_modifier: RwSignal<ModificationTarget>) -> impl IntoView {
    let state_sender = expect_context::<LeptosEventSender<SimulationPlayStateRequest>>();
    let target_sender = expect_context::<LeptosEventSender<ModificationTarget>>();

    let play_sim = {
        let state_sender = state_sender.clone();
        let target_sender = target_sender.clone();
        move |_| {
            // When play is selected, send the play request and ensure we escape from any modification logic
            state_sender.send(SimulationPlayStateRequest::Play).ok();
            target_sender.send(ModificationTarget::None).ok();
            // Update the modification state so the element pane is reflective of Bevy's state.
            active_modifier.set(ModificationTarget::None)
        }
    };
    let pause_sim = {
        let sender = state_sender.clone();
        move |_| {
            sender.send(SimulationPlayStateRequest::Pause).ok();
        }
    };
    let restart_sim = {
        let sender = state_sender.clone();
        move |_| {
            sender.send(SimulationPlayStateRequest::Reset).ok();
        }
    };

    view! {
        <div class="h-full flex flex-col p-4 gap-4 z-[20] absolute right-0 top-0">
            <ControlPaneItem icon=icondata::BsPlayFill on_click=play_sim/>
            <ControlPaneItem icon=icondata::RiPauseMiniMediaLine on_click=pause_sim/>
            <ControlPaneItem icon=icondata::MdiReload on_click=restart_sim/>
        </div>
    }
}

#[component]
fn ControlPaneItem<F>(icon: icondata::Icon, on_click: F) -> impl IntoView
where
    F: Fn(web_sys::MouseEvent) + Clone + 'static,
{
    view! {
        <div
            class="w-[35px] h-[35px] p-[4px] rounded-md bg-card hover:bg-card-active flex items-center justify-center duration-200"
            on:click=on_click
        >
            <Icon icon width="100%" height="100%"/>
        </div>
    }
}
