use bevy::prelude::*;

use crate::{
    core::parameters::{Point, Stick},
    interaction::{
        modification::{ModificationTarget, handle_modification_event, handle_target_change},
        window_bounds::{SimulationBounds, window_listener},
    },
};

#[derive(Event, Clone)]
pub enum SimulationPlayStateRequest {
    Pause,
    Play,
    Reset,
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum SimulationPlayState {
    #[default]
    Running,
    Paused,
}

pub struct StatePlugin;
impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<SimulationPlayState>()
            .init_state::<ModificationTarget>()
            .insert_resource(SimulationBounds::default())
            .add_systems(
                Update,
                (
                    window_listener,
                    handle_target_change,
                    handle_play_state_request,
                    handle_modification_event,
                )
                    .chain(),
            );
    }
}

fn handle_play_state_request(
    mut commands: Commands,
    mut next_state: ResMut<NextState<SimulationPlayState>>,
    state: Res<State<SimulationPlayState>>,
    mut event_reader: EventReader<SimulationPlayStateRequest>,
    point_query: Query<Entity, With<Point>>,
    stick_query: Query<Entity, With<Stick>>,
) {
    for event in event_reader.read() {
        match event {
            SimulationPlayStateRequest::Pause => {
                if state.get() == &SimulationPlayState::Running {
                    next_state.set(SimulationPlayState::Paused)
                }
            }
            SimulationPlayStateRequest::Play => {
                if state.get() == &SimulationPlayState::Paused {
                    next_state.set(SimulationPlayState::Running)
                }
            }
            SimulationPlayStateRequest::Reset => {
                for entity in stick_query.iter() {
                    commands.entity(entity).despawn_recursive();
                }

                for entity in point_query.iter() {
                    commands.entity(entity).despawn_recursive();
                }

                next_state.set(SimulationPlayState::Running)
            }
        }
    }
}
