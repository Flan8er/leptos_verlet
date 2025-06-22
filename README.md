[![crates.io](https://img.shields.io/crates/v/leptos.svg)](https://crates.io/crates/leptos_verlet)
[![docs.rs](https://docs.rs/leptos/badge.svg)](https://docs.rs/leptos_verlet)

# Leptos Verlet

```rust
use leptos_verlet::prelude::*;

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
```
