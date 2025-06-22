use leptos::prelude::*;
mod app;
use app::App;

pub mod components;

pub fn main() {
    console_error_panic_hook::set_once();

    mount_to_body(|| {
        view! { <App/> }
    });
}
