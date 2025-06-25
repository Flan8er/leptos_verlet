use bevy::prelude::*;
use rand::{seq::SliceRandom, thread_rng};

use crate::{
    core::{
        parameters::{BOUNCE_LOSS, CAMERA_DISTANCE, MIN_RENDER_DELTA, Point, Stick},
        render::FrameComparison,
        schedule::SimulationCycle,
        spawner::{SpawnBuffer, SpawnRequest, spawner},
    },
    interaction::{container_bounds::SimulationBounds, play_state::SimulationPlayState},
};

pub struct RunSimulation;
impl Plugin for RunSimulation {
    // Verlet based on: https://www.youtube.com/watch?v=3HjO_RGIjCU
    fn build(&self, app: &mut App) {
        app.insert_resource(SpawnBuffer::default())
            .configure_sets(
                Update,
                (
                    SimulationCycle::Spawn,
                    SimulationCycle::Compute,
                    SimulationCycle::Converge,
                )
                    .chain()
                    .run_if(in_state(SimulationPlayState::Running)),
            )
            .add_systems(
                Update,
                (handle_spawn_requests, spawn_buffer)
                    .chain()
                    .in_set(SimulationCycle::Spawn),
            )
            .add_systems(Update, (update_points).in_set(SimulationCycle::Compute))
            .add_systems(Update, (converge).in_set(SimulationCycle::Converge));
    }
}

/// Listens for any spawn requst sent from Leptos and inserts the mesh_network into
/// the spawn buffer for the next spawn cycle.
fn handle_spawn_requests(
    mut event_reader: EventReader<SpawnRequest>,
    mut buffer: ResMut<SpawnBuffer>,
) {
    for event in event_reader.read() {
        buffer.buffer.push(event.clone())
    }
}
/// Spawn any mesh networks in the spawn buffer and clear out the buffer.
fn spawn_buffer(
    mut buffer: ResMut<SpawnBuffer>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for mesh_network in buffer.buffer.drain(..) {
        spawner(
            mesh_network.mesh_network,
            &mut commands,
            &mut meshes,
            &mut materials,
        );
    }
}

fn update_points(
    mut query: Query<&mut Point>,
    time: Res<Time>,
    mut state: ResMut<FrameComparison>,
) {
    // Randomize the order in which points are updated
    let mut points: Vec<_> = query.iter_mut().collect();
    points.shuffle(&mut thread_rng());

    // Create a value to serve as the maximum distance change of all points.
    // This will be used to see if rendering needs to take place.
    let mut max_delta: f32 = 0.0;

    for point in points.iter_mut() {
        // Locked points should remain stationary
        if point.locked {
            point.prev_position = point.position;
            continue;
        }
        // Store the current position to compare state change after update.
        let previous_position = point.position;

        point.update_properties(&time);

        // Compare the distances before and after updated to see how it compares to the max_delta seen so far.
        let updated_position = point.position;
        let delta = updated_position.distance(previous_position).abs();
        if delta > max_delta {
            // Update max_delta if this value is greater.
            max_delta = delta;
        }
    }

    if max_delta > MIN_RENDER_DELTA {
        // Reflect the in the state that this frame needs to be rendered.
        state.frames_since = 0;
        state.changed = true;
    } else if state.frames_since > state.max_unchanged {
        // Check if the frame should be rendered based on reaching it's frame limit.
        state.frames_since = 0;
        state.changed = true;
    } else {
        // If no changes happened to dictate a rerender, make it known.
        state.frames_since += 1;
        state.changed = false;
    }
}

fn converge(
    mut point_query: Query<&mut Point>,
    stick_query: Query<&Stick>,
    mut state: ResMut<FrameComparison>,
    window_res: Res<SimulationBounds>,
) {
    const SUBSTEPS: usize = 15;

    for _ in 0..SUBSTEPS {
        // first mutate all points
        constrain_points(&mut point_query, &mut state, &window_res);
        // then adjust sticks
        restore_stick_constraints(&mut point_query, &stick_query, &mut state);
    }
}

fn constrain_points(
    query: &mut Query<&mut Point>,
    state: &mut ResMut<FrameComparison>,
    window_res: &Res<SimulationBounds>,
) {
    // Randomize the order in which points are updated
    let mut points: Vec<_> = query.iter_mut().collect();
    points.shuffle(&mut thread_rng());

    // Create a value to serve as the maximum distance change of all points.
    // This will be used to see if rendering needs to take place.
    let mut max_delta: f32 = 0.0;

    for mut pt in points {
        // Locked points should remain stationary
        if pt.locked {
            pt.prev_position = pt.position;
            continue;
        }
        // Store the current position to compare state change after update.
        let previous_position = pt.position;

        let velocity = pt.calculate_velocity();

        // Calculate bounce when the point hits the floor
        if pt.position[1] <= 0. {
            // Bound the point to the floor
            pt.position[1] = 0.;
            pt.prev_position[1] = pt.position[1] + velocity[1] * BOUNCE_LOSS;
        }
        // Calculate bounce when the point hits the left wall
        if pt.position[0] <= -(window_res.width / 2.) {
            // Bound the point to the wall
            pt.position[0] = -(window_res.width / 2.);
            pt.prev_position[0] = pt.position[0] + velocity[0] * BOUNCE_LOSS;
        }
        // Calculate bounce when the point hits the right wall
        else if pt.position[0] >= window_res.width / 2. {
            // Bound the point to the wall
            pt.position[0] = window_res.width / 2.;
            pt.prev_position[0] = pt.position[0] + velocity[0] * BOUNCE_LOSS;
        }
        // Flip the Z travel of going beyond some bound
        if pt.position[2] <= -CAMERA_DISTANCE {
            pt.position[2] = -CAMERA_DISTANCE;
            pt.prev_position[2] = pt.position[2] + velocity[0] * BOUNCE_LOSS;
        } else if pt.position[2] > CAMERA_DISTANCE {
            pt.position[2] = CAMERA_DISTANCE;
            pt.prev_position[2] = pt.position[2] + velocity[0] * BOUNCE_LOSS;
        }

        // Compare the distances before and after updated to see how it compares to the max_delta seen so far.
        let updated_position = pt.position;
        let delta = updated_position.distance(previous_position).abs();
        if delta > max_delta && !state.changed {
            // Update max_delta if this value is greater.
            max_delta = delta;
        }
    }

    // No further checks required if the frame is already going to change.
    if state.changed {
        return;
    }
    if max_delta < MIN_RENDER_DELTA {
        // Reflect the in the state that this frame needs to be rendered.
        state.frames_since = 0;
        state.changed = true;
    } else {
        // If no changes happened to dictate a rerender, make it known.
        state.changed = false;
    }
}

fn restore_stick_constraints(
    point_query: &mut Query<&mut Point>,
    stick_query: &Query<&Stick>,
    state: &mut ResMut<FrameComparison>,
) {
    // Create a value to serve as the maximum distance change of all points.
    // This will be used to see if rendering needs to take place.
    let mut max_delta: f32 = 0.0;

    for stick in &mut stick_query.iter() {
        if let Ok([mut p1, mut p2]) = point_query.get_many_mut([stick.point1, stick.point2]) {
            // If both points are locked, skip computation
            if p1.locked && p2.locked {
                continue;
            }

            // Store the current position to compare state change after update.
            let p1_previous_position = p1.position;
            let p2_previous_position = p2.position;

            let delta = p2.position - p1.position;
            let current_len = delta.length();

            let offset = delta * (current_len - stick.length) / current_len / 2.0;

            // Locked points should remain stationary
            // If one point is locked, apply double the offset to the other point
            if p1.locked {
                p2.position -= 2. * offset;
            } else if p2.locked {
                p1.position += 2. * offset;
            } else {
                // If neither point is locked, apply the offset evenly
                p1.position += offset;
                p2.position -= offset;
            }

            // Compare the distances before and after updated to see how it compares to the max_delta seen so far.
            let p1_updated_position = p1.position;
            let p2_updated_position = p2.position;
            let delta1 = p1_updated_position.distance(p1_previous_position).abs();
            let delta2 = p2_updated_position.distance(p2_previous_position).abs();
            if delta1 > max_delta {
                // Update max_delta if this value is greater.
                max_delta = delta1;
            }
            if delta2 > max_delta {
                // Update max_delta if this value is greater.
                max_delta = delta2;
            }
        }
    }

    // No further checks required if the frame is already going to change.
    if state.changed {
        return;
    }
    if max_delta < MIN_RENDER_DELTA {
        // Reflect the in the state that this frame needs to be rendered.
        state.frames_since = 0;
        state.changed = true;
    } else {
        // If no changes happened to dictate a rerender, make it known.
        state.changed = false;
    }
}
