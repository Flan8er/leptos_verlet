[![crates.io](https://img.shields.io/crates/v/leptos_verlet.svg)](https://crates.io/crates/leptos_verlet)
[![docs.rs](https://docs.rs/leptos_verlet/badge.svg)](https://docs.rs/leptos_verlet)

# Leptos Verlet

```rust
use leptos_verlet::prelude::*;

#[component]
pub fn App() -> impl IntoView {
    let simulation_container = NodeRef::<Div>::new();

    let active_modifier: RwSignal<ModificationTarget> = RwSignal::new(ModificationTarget::None);

    view! {
        <VerletConfigProvider/>

        <main class="w-screen h-screen flex items-center justify-center overflow-hidden relative">
            <ElementPane active_modifier/>
            <InfoModal active_modifier/>

            <div
                node_ref=simulation_container
                class="w-full h-full relative"
            >
                <VerletCanvas parent_element=simulation_container/>

                <MouseMonitor active_modifier/>
            </div>

            <ControlPane active_modifier/>
        </main>
    }
}
```

## Compatibility

| Crate version | Compatible Leptos version | Compatible Bevy version |
| ------------- | ------------------------- | ----------------------- |
| 1.0           | 0.7                       | 0.15                    |
