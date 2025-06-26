mod core;
mod objects;
mod plugins;

pub mod prelude {
    pub use crate::{
        core::{
            container_bounds::LeptosResize,
            core::{VerletCanvas, VerletConfigProvider},
            parameters::Point,
            spawner::{MaterialType, MeshType, SpawnNode, SpawnRequest},
        },
        plugins::{
            asset_loader::plugin::model_loader,
            info::plugin::{PointInfo, SetPointInfo},
            modification::plugin::{ModificationTarget, ModifyEventType, RelativeWindowPosition},
            play_state::plugin::SimulationPlayStateRequest,
        },
    };
    pub use bevy::math::Vec3;
    pub use bevy::prelude::default;
    pub use leptos_bevy_canvas::prelude::{
        LeptosChannelEventSender, LeptosEventReceiver, LeptosEventSender,
    };
}
