mod core;
mod interaction;
mod objects;

pub mod prelude {
    pub use crate::{
        core::core::{VerletCanvas, VerletConfigProvider},
        interaction::{
            modification::{ModificationTarget, ModifyEventType, RelativeWindowPosition},
            state::SimulationPlayStateRequest,
            window_bounds::LeptosResize,
        },
    };
    pub use leptos_bevy_canvas::prelude::{LeptosChannelEventSender, LeptosEventSender};
}
