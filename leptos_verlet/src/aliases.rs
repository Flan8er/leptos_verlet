use leptos_bevy_canvas::prelude::{LeptosEventReceiver, LeptosEventSender};

use crate::{
    plugins::{
        asset_loader::plugin::LoadModelEvent,
        info::plugin::{PointInfo, SetPointInfo},
        modification::plugin::{ModificationTarget, ModifyEventType},
        play_state::plugin::SimulationPlayStateRequest,
    },
    prelude::{LeptosResize, SpawnRequest},
};

pub type ModificationTargetSender = LeptosEventSender<ModificationTarget>;
pub type ModificationEventSender = LeptosEventSender<ModifyEventType>;
pub type PointInfoReceiver = LeptosEventReceiver<PointInfo>;
pub type PointInfoSender = LeptosEventSender<SetPointInfo>;
pub type PlayStateSender = LeptosEventSender<SimulationPlayStateRequest>;
pub type SpawnSender = LeptosEventSender<SpawnRequest>;
pub type ContainerSizeSender = LeptosEventSender<LeptosResize>;
pub type AssetSender = LeptosEventSender<LoadModelEvent>;
