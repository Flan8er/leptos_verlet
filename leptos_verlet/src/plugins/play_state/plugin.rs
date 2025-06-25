use bevy::prelude::*;

use crate::core::parameters::{Point, Stick};

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

pub struct PlayStatePlugin;
impl Plugin for PlayStatePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<SimulationPlayState>()
            .add_systems(Update, (handle_play_state_request,).chain());
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
