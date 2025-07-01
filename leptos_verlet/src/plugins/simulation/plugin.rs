use bevy::{prelude::*, utils::HashSet};
use rand::{seq::SliceRandom, thread_rng};

use crate::{
    core::{
        parameters::{Point, SimulationSettings, Stick},
        spawner::{SpawnBuffer, SpawnRequest, spawner},
    },
    plugins::{render::plugin::FrameComparison, schedule::plugin::SimulationCycle},
};

pub struct SimulationPlugin;
impl Plugin for SimulationPlugin {
    // Verlet based on: https://www.youtube.com/watch?v=3HjO_RGIjCU
    fn build(&self, app: &mut App) {
        app.insert_resource(SpawnBuffer::default())
            .add_systems(
                Update,
                (handle_spawn_requests, spawn_buffer)
                    .chain()
                    .in_set(SimulationCycle::Preparation1),
            )
            .add_systems(
                Update,
                (despawn_overflows, update_points).in_set(SimulationCycle::Compute),
            )
            .add_systems(
                Update,
                (converge, filter).chain().in_set(SimulationCycle::Converge),
            );
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

fn despawn_overflows(
    stick_query: Query<(Entity, &Stick)>,
    point_query: Query<(Entity, &Point)>,
    mut commands: Commands,
) {
    const ACTUAL_SIMULATION_BOUNDS: f32 = 5_000.;
    let mut despawned_points = HashSet::new();

    // First check all the sticks for points that could overflow
    for (stick_entity, stick) in &stick_query {
        if let Ok([p1, p2]) = point_query.get_many([stick.point1, stick.point2]) {
            let p1_out = p1.1.position.length() >= ACTUAL_SIMULATION_BOUNDS;
            let p2_out = p2.1.position.length() >= ACTUAL_SIMULATION_BOUNDS;

            if p1_out || p2_out {
                if !despawned_points.contains(&p1.0) {
                    commands.entity(p1.0).despawn();
                    despawned_points.insert(p1.0);
                }
                if !despawned_points.contains(&p2.0) {
                    commands.entity(p2.0).despawn();
                    despawned_points.insert(p2.0);
                }
            }
        } else {
            commands.entity(stick_entity).despawn();
        }
    }

    // Check individual points that might not be
    for (entity, point) in &point_query {
        if point.position.length() >= ACTUAL_SIMULATION_BOUNDS
            && !despawned_points.contains(&entity)
        {
            commands.entity(entity).despawn();
        }
    }
}

fn update_points(
    mut query: Query<&mut Point>,
    time: Res<Time>,
    mut state: ResMut<FrameComparison>,
    sim_settings: Res<SimulationSettings>,
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

        point.update_properties(&time, &sim_settings);

        // Compare the distances before and after updated to see how it compares to the max_delta seen so far.
        let updated_position = point.position;
        let delta = updated_position.distance(previous_position).abs();
        if delta > max_delta {
            // Update max_delta if this value is greater.
            max_delta = delta;
        }
    }

    if max_delta > sim_settings.min_render_delta {
        // Reflect the in the state that this frame needs to be rendered.
        state.frames_since = 0;
        state.changed = true;
    } else if state.frames_since > sim_settings.max_unchanged_frames {
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
    sim_settings: Res<SimulationSettings>,
) {
    for _ in 0..sim_settings.converge_iterations {
        // first mutate all points
        constrain_points(&mut point_query, &mut state, &sim_settings);
        // then adjust sticks
        restore_stick_constraints(&mut point_query, &stick_query, &mut state, &sim_settings);
    }
}

fn constrain_points(
    query: &mut Query<&mut Point>,
    state: &mut ResMut<FrameComparison>,
    sim_settings: &Res<SimulationSettings>,
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

        let half_width = sim_settings.simulation_bounds.x.1 * 0.5;
        let half_depth = sim_settings.simulation_bounds.z.1 * 0.5;
        let x_bounds_enabled = sim_settings.simulation_bounds.x.0;
        let y_bounds_enabled = sim_settings.simulation_bounds.y.0;
        let z_bounds_enabled = sim_settings.simulation_bounds.z.0;

        let coef_restitution = sim_settings.coeff_restitution;

        // Calculate bounce when the point hits the floor
        if pt.position.y <= 0. && y_bounds_enabled {
            // Bound the point to the floor
            pt.position.y = 0.;
            pt.prev_position.y = pt.position.y + velocity.y * coef_restitution;
        }
        // Calculate bounce when the point hits the left wall
        if pt.position.x <= -half_width && x_bounds_enabled {
            // Bound the point to the wall
            pt.position.x = -half_width;
            pt.prev_position.x = pt.position.x + velocity.x * coef_restitution;
        }
        // Calculate bounce when the point hits the right wall
        else if pt.position.x >= half_width && x_bounds_enabled {
            // Bound the point to the wall
            pt.position.x = half_width;
            pt.prev_position.x = pt.position.x + velocity.x * coef_restitution;
        }
        // Flip the Z travel of going beyond some bound
        if pt.position.z <= -half_depth && z_bounds_enabled {
            pt.position.z = -half_depth;
            pt.prev_position.z = pt.position.z + velocity.z * coef_restitution;
        } else if pt.position.z > half_depth && z_bounds_enabled {
            pt.position.z = half_depth;
            pt.prev_position.z = pt.position.z + velocity.z * coef_restitution;
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
    if max_delta > sim_settings.min_render_delta {
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
    sim_settings: &Res<SimulationSettings>,
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
    if max_delta > sim_settings.min_render_delta {
        // Reflect the in the state that this frame needs to be rendered.
        state.frames_since = 0;
        state.changed = true;
    } else {
        // If no changes happened to dictate a rerender, make it known.
        state.changed = false;
    }
}

fn filter(mut q: Query<&mut Point>, sim_settings: Res<SimulationSettings>) {
    // how aggressively to shave off the discrete acceleration “spike”
    let jerk_damp: f32 = sim_settings.jerk_damping;

    for mut pt in q.iter_mut() {
        if pt.locked {
            continue;
        }

        // t–1 -> the previous position
        let p_nm1 = pt.previously_rendered_position;
        // t -> the current position
        let p_n = pt.prev_position;
        // t+1 -> The next position
        let p_np1 = pt.position;

        // compute the second finite difference:
        let delta2 = p_np1 - 2.0 * p_n + p_nm1;
        // subtract off a fraction of it:
        let damped = p_np1 - delta2 * jerk_damp;

        pt.position = damped;
    }
}
