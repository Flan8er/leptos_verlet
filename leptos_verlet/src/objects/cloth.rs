use bevy::prelude::*;

use crate::{
    core::{
        parameters::{Point, SimulationSettings},
        spawner::{SpawnNode, spawner},
    },
    prelude::{MaterialType, MeshType},
};

const GRID_GAP: f32 = 0.1; // m
const CONTROL_BAR_HEIGHT: f32 = 0.075;
const FLOOR_OFFSET: f32 = 0.25; // m

pub fn spawn_cloth(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    point_material: MaterialType,
    stick_material: MaterialType,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    sim_settings: &Res<SimulationSettings>, // bounds: &Res<SimulationBounds>,
) {
    let point_mesh = MeshType::Sphere;
    let stick_mesh = MeshType::Cuboid;

    // simulation dimensions
    let window_width = sim_settings.simulation_bounds.x.1;
    let window_height = sim_settings.simulation_bounds.y.1;

    // how many points in each direction
    let cols = (window_width / GRID_GAP).floor() as usize;
    let rows = ((window_height - FLOOR_OFFSET - CONTROL_BAR_HEIGHT) / GRID_GAP).floor() as usize;

    // same “weird but working” cloth‐placement math
    let initial_x = (window_width - (GRID_GAP * cols as f32)) / 2.0 - GRID_GAP - window_width / 2.0;
    let initial_y = GRID_GAP * rows as f32 + FLOOR_OFFSET - CONTROL_BAR_HEIGHT;

    // collect all point positions (flattened row-major)
    let mut positions = Vec::with_capacity(rows * cols);
    for row in 0..rows {
        let y = initial_y - GRID_GAP * row as f32;
        for col in 0..cols {
            let x = initial_x + GRID_GAP * (col + 1) as f32;
            positions.push(Vec3::new(x, y, 0.0));
        }
    }

    // build a bidirectional adjacency list for horiz + vert neighbors
    let mut adj = vec![Vec::new(); positions.len()];
    for row in 0..rows {
        for col in 0..cols {
            let idx = row * cols + col;
            // right neighbor
            if col + 1 < cols {
                let r = idx + 1;
                adj[idx].push(r);
                adj[r].push(idx);
            }
            // below neighbor
            if row + 1 < rows {
                let d = idx + cols;
                adj[idx].push(d);
                adj[d].push(idx);
            }
        }
    }

    // turn positions + adj into Vec<SpawnNode>
    let mesh_network: Vec<SpawnNode> = positions
        .iter()
        .enumerate()
        .map(|(i, &pos)| {
            let neighbors = &adj[i];
            SpawnNode {
                point: Point::new(pos, pos, false),
                connection: Some(neighbors.iter().map(|&j| positions[j]).collect()),
                point_material: point_material.clone(),
                point_mesh: point_mesh.clone(),
                point_size: sim_settings.default_geometry_point_size,
                connection_mesh: Some(vec![stick_mesh.clone(); neighbors.len()]),
                connection_material: Some(vec![stick_material.clone(); neighbors.len()]),
                connection_size: Some(vec![
                    sim_settings.default_geometry_stick_size;
                    neighbors.len()
                ]),
                connection_scale: Some(vec![Vec3::ONE; neighbors.len()]),
                ..default()
            }
        })
        .collect();

    spawner(mesh_network, commands, meshes, materials);
}
