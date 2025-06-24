use leptos::{logging::log, prelude::*};
use leptos_use::{UseWindowSizeReturn, use_window_size};
use web_sys::wasm_bindgen::JsCast;

use leptos_verlet::prelude::*;

#[component]
pub fn MouseMonitor(active_modifier: RwSignal<ModificationTarget>) -> impl IntoView {
    let event_sender = expect_context::<LeptosEventSender<ModifyEventType>>();
    let UseWindowSizeReturn { width, height } = use_window_size();

    let left_click_action = {
        let sender = event_sender.clone();
        move |x: f64, y: f64| {
            let container_width = width.get_untracked();
            let container_height = height.get_untracked();
            let _ = sender.send(ModifyEventType::Left(RelativeWindowPosition {
                event_x: x as f32,
                event_y: y as f32,
                container_h: container_height as f32,
                container_w: container_width as f32,
            }));
        }
    };
    let middle_click_action = {
        let sender = event_sender.clone();
        move |x: f64, y: f64| {
            let container_width = width.get_untracked();
            let container_height = height.get_untracked();
            let _ = sender.send(ModifyEventType::Middle(RelativeWindowPosition {
                event_x: x as f32,
                event_y: y as f32,
                container_h: container_height as f32,
                container_w: container_width as f32,
            }));
        }
    };
    let right_click_action = {
        let sender = event_sender.clone();
        move |x: f64, y: f64| {
            let container_width = width.get_untracked();
            let container_height = height.get_untracked();
            let _ = sender.send(ModifyEventType::Right(RelativeWindowPosition {
                event_x: x as f32,
                event_y: y as f32,
                container_h: container_height as f32,
                container_w: container_width as f32,
            }));
        }
    };
    let move_action = {
        let sender = event_sender.clone();
        move |x: f64, y: f64| {
            let container_width = width.get_untracked();
            let container_height = height.get_untracked();
            let _ = sender.send(ModifyEventType::Move(RelativeWindowPosition {
                event_x: x as f32,
                event_y: y as f32,
                container_h: container_height as f32,
                container_w: container_width as f32,
            }));
        }
    };
    let release_action = {
        let sender = event_sender.clone();
        move |x: f64, y: f64| {
            let container_width = width.get_untracked();
            let container_height = height.get_untracked();
            let _ = sender.send(ModifyEventType::Release(RelativeWindowPosition {
                event_x: x as f32,
                event_y: y as f32,
                container_h: container_height as f32,
                container_w: container_width as f32,
            }));
        }
    };

    view! {
        <div
            class=move || {
                format!("absolute inset-0 z-[10] {}",
                    if active_modifier.get() == ModificationTarget::None {
                        "cursor-default"
                    } else {
                        "cursor-crosshair"
                    }
                )
            }
            on:mousedown=move |ev| {
                ev.prevent_default();
                if let Some((x, y)) = target_mouse_position(&ev) {
                    log!("Clicked event: {}", ev.button());
                    match ev.button() {
                        0 => left_click_action(x, y),
                        1 => middle_click_action(x, y),
                        2 => right_click_action(x, y),
                        _ => return
                    }
                }
            }
            on:mouseup=move |ev| {
                ev.prevent_default();
                if let Some((x, y)) = target_mouse_position(&ev) {
                    release_action(x, y);
                }
            }
            on:contextmenu=move |ev| {
                ev.prevent_default();
            }
            on:mousemove=move |ev| {
                ev.prevent_default();
                if let Some((x, y)) = target_mouse_position(&ev) {
                    move_action(x, y);
                }
            }
        ></div>
    }
}

fn target_mouse_position(ev: &web_sys::MouseEvent) -> Option<(f64, f64)> {
    let x: f64;
    let y: f64;

    if let Some(target) = ev
        .target()
        .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
    {
        let rect = target.get_bounding_client_rect();
        x = ev.client_x() as f64 - rect.left();
        y = ev.client_y() as f64 - rect.top();
    } else {
        return None;
    }

    leptos::logging::log!("Click position: {},{}", x, y);
    Some((x, y))
}
