mod core;
mod interaction;
mod objects;

pub mod prelude {
    pub use crate::{
        core::{
            core::{VerletCanvas, VerletConfigProvider},
            parameters::Point,
            spawner::{MaterialType, MeshType, SpawnNode, SpawnRequest},
        },
        interaction::{
            container_bounds::LeptosResize,
            modification::{ModificationTarget, ModifyEventType, RelativeWindowPosition},
            play_state::SimulationPlayStateRequest,
        },
    };
    pub use bevy::math::Vec3;
    pub use leptos_bevy_canvas::prelude::{LeptosChannelEventSender, LeptosEventSender};
}
